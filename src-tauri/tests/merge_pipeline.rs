use image::{ImageBuffer, Rgba};
use lopdf::{Dictionary, Document, Object, ObjectId};
use merge_picture_lib::domain::{
    ImageFitMode, MergeRequest, PageSizePreset, PreviewItem, SupportedKind,
};
use merge_picture_lib::services::merger::MergerService;
use printpdf::{Mm, Op, PdfDocument, PdfPage, PdfSaveOptions};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn merges_image_and_pdf_into_single_document() {
    let temp_dir = tempdir().expect("temp dir");
    let image_path = temp_dir.path().join("input.png");
    let pdf_path = temp_dir.path().join("source.pdf");
    let output_path = temp_dir.path().join("merged.pdf");

    write_sample_png(&image_path);
    write_blank_pdf(&pdf_path);

    let service = MergerService;
    let result = service
        .merge(MergeRequest {
            items: vec![
                preview_item(&image_path, SupportedKind::Image),
                preview_item(&pdf_path, SupportedKind::Pdf),
            ],
            output_path: output_path.display().to_string(),
            image_page_size: PageSizePreset::Letter,
            image_margin_mm: 14,
            image_fit_mode: ImageFitMode::Contain,
        })
        .expect("merge should succeed");

    assert_eq!(result.status, "completed");
    assert!(output_path.exists());

    let merged = Document::load(&output_path).expect("merged output should load");
    assert_eq!(merged.get_pages().len(), 2);
    assert!(page_contains_image_xobject(&merged, 1));
}

#[test]
fn rejects_corrupted_pdf_input() {
    let temp_dir = tempdir().expect("temp dir");
    let bad_pdf_path = temp_dir.path().join("broken.pdf");
    let output_path = temp_dir.path().join("merged.pdf");

    fs::write(&bad_pdf_path, b"not a real pdf").expect("corrupted fixture");

    let service = MergerService;
    let result = service.merge(MergeRequest {
        items: vec![preview_item(&bad_pdf_path, SupportedKind::Pdf)],
        output_path: output_path.display().to_string(),
        image_page_size: PageSizePreset::Auto,
        image_margin_mm: 12,
        image_fit_mode: ImageFitMode::Contain,
    });

    assert!(result.is_err());
}

fn preview_item(path: &Path, kind: SupportedKind) -> PreviewItem {
    PreviewItem {
        path: path.display().to_string(),
        name: path
            .file_name()
            .expect("file name")
            .to_string_lossy()
            .into_owned(),
        kind,
        size: fs::metadata(path).expect("metadata").len(),
        page_count: if matches!(kind, SupportedKind::Pdf) {
            Some(1)
        } else {
            None
        },
        pixel_width: if matches!(kind, SupportedKind::Image) {
            Some(24)
        } else {
            None
        },
        pixel_height: if matches!(kind, SupportedKind::Image) {
            Some(16)
        } else {
            None
        },
    }
}

fn write_sample_png(path: &Path) {
    let image: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(24, 16, Rgba([240, 120, 64, 255]));
    image.save(path).expect("png should be written");
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
    fs::write(path, bytes).expect("pdf should be written");
}

fn page_contains_image_xobject(document: &Document, page_number: u32) -> bool {
    let pages = document.get_pages();
    let page_id = *pages.get(&page_number).expect("page should exist");
    let page_dict = get_dict(document, page_id);
    let resources = get_dict_from_object(document, page_dict.get(b"Resources").expect("resources"));

    let Ok(xobject_obj) = resources.get(b"XObject") else {
        return false;
    };

    let xobject_dict = get_dict_from_object(document, xobject_obj);

    xobject_dict.iter().any(|(_, object)| {
        let Object::Reference(object_id) = object else {
            return false;
        };

        let Ok(stream) = document.get_object(*object_id).and_then(Object::as_stream) else {
            return false;
        };

        matches!(
            stream.dict.get(b"Subtype"),
            Ok(Object::Name(name)) if name == b"Image"
        )
    })
}

fn get_dict<'a>(document: &'a Document, object_id: ObjectId) -> &'a Dictionary {
    document
        .get_object(object_id)
        .expect("object should exist")
        .as_dict()
        .expect("object should be a dictionary")
}

fn get_dict_from_object<'a>(document: &'a Document, object: &'a Object) -> &'a Dictionary {
    match object {
        Object::Dictionary(dictionary) => dictionary,
        Object::Reference(object_id) => get_dict(document, *object_id),
        _ => panic!("object should resolve to a dictionary"),
    }
}
