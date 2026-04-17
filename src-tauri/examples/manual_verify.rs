use image::{ImageBuffer, Rgba};
use lopdf::{Dictionary, Document, Object, ObjectId};
use merge_picture_lib::domain::{
    ImageFitMode, MergeRequest, PageSizePreset, PreviewItem, SupportedKind,
};
use merge_picture_lib::services::merger::MergerService;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("manual verification failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let output_dir = project_root()
        .join("artifacts")
        .join("manual-verification");
    fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;

    let image_one_path = output_dir.join("sample-1.png");
    let image_two_path = output_dir.join("sample-2.png");
    let output_pdf_path = output_dir.join("merged-samples.pdf");

    write_sample_png(&image_one_path, 1200, 800, [240, 120, 64, 255])?;
    write_sample_png(&image_two_path, 900, 1200, [64, 140, 240, 255])?;

    let service = MergerService;
    service
        .merge(MergeRequest {
            items: vec![
                preview_item(&image_one_path),
                preview_item(&image_two_path),
            ],
            output_path: output_pdf_path.display().to_string(),
            image_page_size: PageSizePreset::A4,
            image_margin_mm: 12,
            image_fit_mode: ImageFitMode::Contain,
        })
        .map_err(|error| error.to_string())?;

    let merged = Document::load(&output_pdf_path).map_err(|error| error.to_string())?;
    let page_count = merged.get_pages().len();

    if page_count != 2 {
        return Err(format!("expected 2 pages, found {page_count}"));
    }

    if !page_contains_image_xobject(&merged, 1) || !page_contains_image_xobject(&merged, 2) {
        return Err("one or more merged pages do not contain an embedded image".to_string());
    }

    println!("Manual verification succeeded.");
    println!("Sample images:");
    println!("  {}", image_one_path.display());
    println!("  {}", image_two_path.display());
    println!("Merged PDF:");
    println!("  {}", output_pdf_path.display());
    println!("Checks:");
    println!("  - page count: {page_count}");
    println!("  - page 1 contains image: yes");
    println!("  - page 2 contains image: yes");

    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri should have a parent")
        .to_path_buf()
}

fn write_sample_png(path: &Path, width: u32, height: u32, color: [u8; 4]) -> Result<(), String> {
    let image: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(width, height, Rgba(color));
    image.save(path).map_err(|error| error.to_string())
}

fn preview_item(path: &Path) -> PreviewItem {
    PreviewItem {
        path: path.display().to_string(),
        name: path
            .file_name()
            .expect("file name should exist")
            .to_string_lossy()
            .into_owned(),
        kind: SupportedKind::Image,
        size: fs::metadata(path).expect("metadata should exist").len(),
        page_count: None,
        pixel_width: None,
        pixel_height: None,
    }
}

fn page_contains_image_xobject(document: &Document, page_number: u32) -> bool {
    let pages = document.get_pages();
    let Some(page_id) = pages.get(&page_number) else {
        return false;
    };

    let page_dict = get_dict(document, *page_id);
    let Ok(resources_obj) = page_dict.get(b"Resources") else {
        return false;
    };
    let resources = get_dict_from_object(document, resources_obj);
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
