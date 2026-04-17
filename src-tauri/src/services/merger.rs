use crate::domain::{
    ImageFitMode, InspectInputsRequest, MergePlan, MergeRequest, MergeResult, PageSizePreset,
    PreviewItem, PreviewRequest, SupportedKind,
};
use crate::errors::AppError;
use lopdf::{Document, Object, ObjectId, dictionary};
use printpdf::{
    Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Pt, Px, XObjectTransform, image::RawImage,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug, Default)]
pub struct MergerService;

impl MergerService {
    pub fn inspect_inputs(
        &self,
        request: InspectInputsRequest,
    ) -> Result<Vec<PreviewItem>, AppError> {
        let items = request
            .paths
            .into_iter()
            .filter_map(|path| inspect_input_path(&path).transpose())
            .collect::<Result<Vec<_>, _>>()?;

        if items.is_empty() {
            return Err(AppError::NoSupportedInputs);
        }

        Ok(items)
    }

    pub fn preview_plan(&self, request: PreviewRequest) -> MergePlan {
        let total_items = request.items.len();
        let total_bytes = request.items.iter().map(|item| item.size).sum();
        let contains_images = request
            .items
            .iter()
            .any(|item| matches!(item.kind, SupportedKind::Image));
        let contains_pdfs = request
            .items
            .iter()
            .any(|item| matches!(item.kind, SupportedKind::Pdf));

        MergePlan {
            total_items,
            total_bytes,
            contains_images,
            contains_pdfs,
        }
    }

    pub fn merge(&self, request: MergeRequest) -> Result<MergeResult, AppError> {
        validate_request(&request)?;

        let temp_dir = TempDir::new().map_err(|error| AppError::Io(error.to_string()))?;
        let prepared_paths = prepare_input_pdfs(&request, temp_dir.path())?;
        let mut merged = merge_pdf_documents(&prepared_paths)?;

        merged
            .save(&request.output_path)
            .map_err(|error| AppError::Write(error.to_string()))?;

        Ok(MergeResult {
            output_path: request.output_path,
            merged_items: request.items.len(),
            status: "completed".to_string(),
        })
    }
}

fn validate_request(request: &MergeRequest) -> Result<(), AppError> {
    if request.items.is_empty() {
        return Err(AppError::EmptyInput);
    }

    if !request.output_path.to_lowercase().ends_with(".pdf") {
        return Err(AppError::InvalidOutputPath);
    }

    for item in &request.items {
        let path = Path::new(&item.path);
        if !path.exists() {
            return Err(AppError::MissingInputFile(item.path.clone()));
        }

        match item.kind {
            SupportedKind::Image | SupportedKind::Pdf => {}
        }
    }

    Ok(())
}

fn inspect_input_path(path: &str) -> Result<Option<PreviewItem>, AppError> {
    let source = Path::new(path);
    if !source.exists() {
        return Err(AppError::MissingInputFile(path.to_string()));
    }

    let metadata = fs::metadata(source).map_err(|error| AppError::Io(error.to_string()))?;
    if !metadata.is_file() {
        return Ok(None);
    }

    let name = source
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());

    let kind = match detect_supported_kind(&name) {
        Some(kind) => kind,
        None => return Ok(None),
    };

    Ok(Some(PreviewItem {
        path: path.to_string(),
        name,
        kind,
        size: metadata.len(),
        page_count: inspect_page_count(source, &kind)?,
        pixel_width: inspect_pixel_width(source, &kind)?,
        pixel_height: inspect_pixel_height(source, &kind)?,
    }))
}

fn inspect_page_count(path: &Path, kind: &SupportedKind) -> Result<Option<u32>, AppError> {
    if !matches!(kind, SupportedKind::Pdf) {
        return Ok(None);
    }

    let document = Document::load(path).map_err(|error| AppError::Pdf(error.to_string()))?;
    Ok(Some(document.get_pages().len() as u32))
}

fn inspect_pixel_width(path: &Path, kind: &SupportedKind) -> Result<Option<u32>, AppError> {
    if !matches!(kind, SupportedKind::Image) {
        return Ok(None);
    }

    let (width, _) =
        image::image_dimensions(path).map_err(|error| AppError::Image(error.to_string()))?;
    Ok(Some(width))
}

fn inspect_pixel_height(path: &Path, kind: &SupportedKind) -> Result<Option<u32>, AppError> {
    if !matches!(kind, SupportedKind::Image) {
        return Ok(None);
    }

    let (_, height) =
        image::image_dimensions(path).map_err(|error| AppError::Image(error.to_string()))?;
    Ok(Some(height))
}

fn detect_supported_kind(file_name: &str) -> Option<SupportedKind> {
    let normalized = file_name.to_lowercase();
    if normalized.ends_with(".pdf") {
        return Some(SupportedKind::Pdf);
    }

    if [".jpg", ".jpeg", ".png", ".webp"]
        .iter()
        .any(|extension| normalized.ends_with(extension))
    {
        return Some(SupportedKind::Image);
    }

    None
}

fn prepare_input_pdfs(request: &MergeRequest, temp_dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    request
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| match item.kind {
            SupportedKind::Pdf => Ok(PathBuf::from(&item.path)),
            SupportedKind::Image => image_to_pdf(
                &item.path,
                temp_dir,
                index,
                request.image_page_size,
                request.image_margin_mm,
                request.image_fit_mode,
            ),
        })
        .collect()
}

fn image_to_pdf(
    path: &str,
    temp_dir: &Path,
    index: usize,
    page_size: PageSizePreset,
    margin_mm: u32,
    fit_mode: ImageFitMode,
) -> Result<PathBuf, AppError> {
    let bytes = fs::read(path).map_err(|error| AppError::Io(error.to_string()))?;
    let dynamic_image =
        image::load_from_memory(&bytes).map_err(|error| AppError::Image(error.to_string()))?;
    let width = dynamic_image.width() as usize;
    let height = dynamic_image.height() as usize;

    let mut warnings = Vec::new();
    let raw_image = RawImage::decode_from_bytes(&bytes, &mut warnings)
        .map_err(|error| AppError::Image(error))?;

    let dpi = 300.0_f32;
    let image_width_pt: Pt = Px(width).into_pt(dpi);
    let image_height_pt: Pt = Px(height).into_pt(dpi);
    let margin_pt = Pt::from(Mm(margin_mm as f32));

    let (page_width_pt, page_height_pt, content_width_pt, content_height_pt) =
        resolve_page_layout(image_width_pt, image_height_pt, page_size, margin_pt);
    let (translate_x, translate_y, scale_x, scale_y) = resolve_image_transform(
        image_width_pt,
        image_height_pt,
        content_width_pt,
        content_height_pt,
        margin_pt,
        fit_mode,
    );

    let mut document = PdfDocument::new("Merge Picture Image Input");
    let image_id = document.add_image(&raw_image);
    let page = PdfPage::new(
        Mm::from(page_width_pt),
        Mm::from(page_height_pt),
        vec![Op::UseXobject {
            id: image_id,
            transform: XObjectTransform {
                translate_x: Some(translate_x),
                translate_y: Some(translate_y),
                rotate: None,
                scale_x: Some(scale_x),
                scale_y: Some(scale_y),
                dpi: Some(dpi),
            },
        }],
    );

    document.with_pages(vec![page]);
    let pdf_bytes = document.save(&PdfSaveOptions::default(), &mut warnings);
    let output_path = temp_dir.join(format!("image-input-{index}.pdf"));
    fs::write(&output_path, pdf_bytes).map_err(|error| AppError::Write(error.to_string()))?;
    Ok(output_path)
}

fn resolve_page_layout(
    image_width_pt: Pt,
    image_height_pt: Pt,
    page_size: PageSizePreset,
    margin_pt: Pt,
) -> (Pt, Pt, Pt, Pt) {
    match page_size {
        PageSizePreset::Auto => {
            let page_width = Pt(image_width_pt.0 + margin_pt.0 * 2.0);
            let page_height = Pt(image_height_pt.0 + margin_pt.0 * 2.0);
            (page_width, page_height, image_width_pt, image_height_pt)
        }
        PageSizePreset::A4 => {
            let page_width = Pt::from(Mm(210.0));
            let page_height = Pt::from(Mm(297.0));
            (
                page_width,
                page_height,
                Pt((page_width.0 - margin_pt.0 * 2.0).max(1.0)),
                Pt((page_height.0 - margin_pt.0 * 2.0).max(1.0)),
            )
        }
        PageSizePreset::Letter => {
            let page_width = Pt::from(Mm(215.9));
            let page_height = Pt::from(Mm(279.4));
            (
                page_width,
                page_height,
                Pt((page_width.0 - margin_pt.0 * 2.0).max(1.0)),
                Pt((page_height.0 - margin_pt.0 * 2.0).max(1.0)),
            )
        }
    }
}

fn resolve_image_transform(
    image_width_pt: Pt,
    image_height_pt: Pt,
    content_width_pt: Pt,
    content_height_pt: Pt,
    margin_pt: Pt,
    fit_mode: ImageFitMode,
) -> (Pt, Pt, f32, f32) {
    let width_scale = content_width_pt.0 / image_width_pt.0.max(1.0);
    let height_scale = content_height_pt.0 / image_height_pt.0.max(1.0);

    let (scale_x, scale_y) = match fit_mode {
        ImageFitMode::Contain => {
            let scale = width_scale.min(height_scale);
            (scale, scale)
        }
        ImageFitMode::Cover => {
            let scale = width_scale.max(height_scale);
            (scale, scale)
        }
        ImageFitMode::Fill => (width_scale, height_scale),
    };

    let rendered_width = image_width_pt.0 * scale_x;
    let rendered_height = image_height_pt.0 * scale_y;
    let translate_x = Pt(margin_pt.0 + (content_width_pt.0 - rendered_width) / 2.0);
    let translate_y = Pt(margin_pt.0 + (content_height_pt.0 - rendered_height) / 2.0);

    (translate_x, translate_y, scale_x, scale_y)
}

fn merge_pdf_documents(paths: &[PathBuf]) -> Result<Document, AppError> {
    if paths.is_empty() {
        return Err(AppError::EmptyInput);
    }

    let mut documents = Vec::with_capacity(paths.len());
    for path in paths {
        let document = Document::load(path).map_err(|error| AppError::Pdf(error.to_string()))?;
        documents.push(document);
    }

    let mut merged = Document::with_version("1.5");
    let mut max_id = 1;
    let mut page_ids = Vec::<ObjectId>::new();
    let mut objects = BTreeMap::<ObjectId, Object>::new();

    for document in &mut documents {
        document.renumber_objects_with(max_id);
        max_id = document.max_id + 1;

        page_ids.extend(document.get_pages().into_values());
        objects.extend(document.objects.clone());
    }

    let pages_id = merged.new_object_id();
    let catalog_id = merged.new_object_id();

    for (object_id, object) in objects {
        merged.objects.insert(object_id, object);
    }

    let kids = page_ids
        .iter()
        .copied()
        .map(Object::Reference)
        .collect::<Vec<_>>();

    let page_count = kids.len() as u32;

    for object_id in &page_ids {
        if let Ok(object) = merged.get_object_mut(*object_id) {
            if let Ok(dictionary) = object.as_dict_mut() {
                dictionary.set("Parent", pages_id);
            }
        }
    }

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => Object::Array(kids),
        "Count" => page_count,
    };

    let catalog_dict = dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    };

    merged
        .objects
        .insert(pages_id, Object::Dictionary(pages_dict));
    merged
        .objects
        .insert(catalog_id, Object::Dictionary(catalog_dict));
    merged.trailer.set("Root", catalog_id);
    merged.max_id = merged.objects.len() as u32;
    merged.compress();

    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::MergerService;
    use crate::domain::{
        ImageFitMode, InspectInputsRequest, MergeRequest, PageSizePreset, PreviewItem,
        PreviewRequest, SupportedKind,
    };
    use crate::errors::AppError;
    use lopdf::Document;
    use printpdf::{Mm, Op, PdfDocument, PdfPage, PdfSaveOptions};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn sample_item(kind: SupportedKind) -> PreviewItem {
        PreviewItem {
            path: "/tmp/sample".to_string(),
            name: "sample".to_string(),
            kind,
            size: 120,
            page_count: None,
            pixel_width: None,
            pixel_height: None,
        }
    }

    #[test]
    fn preview_marks_mixed_inputs() {
        let service = MergerService;
        let plan = service.preview_plan(PreviewRequest {
            items: vec![
                sample_item(SupportedKind::Image),
                sample_item(SupportedKind::Pdf),
            ],
        });

        assert_eq!(plan.total_items, 2);
        assert_eq!(plan.total_bytes, 240);
        assert!(plan.contains_images);
        assert!(plan.contains_pdfs);
    }

    #[test]
    fn merge_rejects_non_pdf_output() {
        let service = MergerService;
        let result = service.merge(MergeRequest {
            items: vec![sample_item(SupportedKind::Pdf)],
            output_path: "merged.txt".to_string(),
            image_page_size: PageSizePreset::Auto,
            image_margin_mm: 12,
            image_fit_mode: ImageFitMode::Contain,
        });

        assert!(matches!(result, Err(AppError::InvalidOutputPath)));
    }

    #[test]
    fn merge_combines_multiple_pdf_inputs() {
        let temp_dir = tempdir().expect("temp dir");
        let first = temp_dir.path().join("first.pdf");
        let second = temp_dir.path().join("second.pdf");
        let output = temp_dir.path().join("merged.pdf");

        write_blank_pdf(&first);
        write_blank_pdf(&second);

        let service = MergerService;
        let result = service
            .merge(MergeRequest {
                items: vec![
                    PreviewItem {
                        path: first.display().to_string(),
                        name: "first.pdf".to_string(),
                        kind: SupportedKind::Pdf,
                        size: fs::metadata(&first).expect("first metadata").len(),
                        page_count: Some(1),
                        pixel_width: None,
                        pixel_height: None,
                    },
                    PreviewItem {
                        path: second.display().to_string(),
                        name: "second.pdf".to_string(),
                        kind: SupportedKind::Pdf,
                        size: fs::metadata(&second).expect("second metadata").len(),
                        page_count: Some(1),
                        pixel_width: None,
                        pixel_height: None,
                    },
                ],
                output_path: output.display().to_string(),
                image_page_size: PageSizePreset::A4,
                image_margin_mm: 10,
                image_fit_mode: ImageFitMode::Contain,
            })
            .expect("merge should succeed");

        assert_eq!(result.status, "completed");
        assert!(output.exists());

        let merged = Document::load(&output).expect("merged pdf should load");
        assert_eq!(merged.get_pages().len(), 2);
    }

    #[test]
    fn inspect_inputs_filters_unsupported_files() {
        let temp_dir = tempdir().expect("temp dir");
        let supported = temp_dir.path().join("sample.pdf");
        let ignored = temp_dir.path().join("notes.txt");

        write_blank_pdf(&supported);
        fs::write(&ignored, "hello").expect("text fixture");

        let service = MergerService;
        let items = service
            .inspect_inputs(InspectInputsRequest {
                paths: vec![
                    supported.display().to_string(),
                    ignored.display().to_string(),
                ],
            })
            .expect("supported file should be inspected");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, SupportedKind::Pdf);
        assert_eq!(items[0].page_count, Some(1));
    }

    fn write_blank_pdf(path: &Path) {
        let mut document = PdfDocument::new("fixture");
        document.with_pages(vec![PdfPage::new(
            Mm(40.0),
            Mm(60.0),
            vec![Op::Marker {
                id: "fixture".to_string(),
            }],
        )]);
        let bytes = document.save(&PdfSaveOptions::default(), &mut Vec::new());
        fs::write(path, bytes).expect("fixture should be written");
    }
}
