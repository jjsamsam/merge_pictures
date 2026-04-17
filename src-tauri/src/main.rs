use merge_picture_lib::domain::{
    InspectInputsRequest, MergePlan, MergeRequest, MergeResult, PreviewRequest,
};
use merge_picture_lib::errors::AppError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            inspect_inputs,
            preview_merge_plan,
            merge_to_pdf
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}

#[tauri::command]
fn inspect_inputs(
    request: InspectInputsRequest,
) -> Result<Vec<merge_picture_lib::domain::PreviewItem>, AppError> {
    merge_picture_lib::commands::inspect_inputs(request)
}

#[tauri::command]
fn preview_merge_plan(request: PreviewRequest) -> Result<MergePlan, AppError> {
    merge_picture_lib::commands::preview_merge_plan(request)
}

#[tauri::command]
fn merge_to_pdf(request: MergeRequest) -> Result<MergeResult, AppError> {
    merge_picture_lib::commands::merge_to_pdf(request)
}
