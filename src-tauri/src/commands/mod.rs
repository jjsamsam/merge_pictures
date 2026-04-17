use crate::domain::{
    InspectInputsRequest, MergePlan, MergeRequest, MergeResult, PreviewItem, PreviewRequest,
};
use crate::errors::AppError;
use crate::services::merger::MergerService;

#[tauri::command]
pub fn inspect_inputs(request: InspectInputsRequest) -> Result<Vec<PreviewItem>, AppError> {
    MergerService::default().inspect_inputs(request)
}

#[tauri::command]
pub fn preview_merge_plan(request: PreviewRequest) -> Result<MergePlan, AppError> {
    Ok(MergerService::default().preview_plan(request))
}

#[tauri::command]
pub fn merge_to_pdf(request: MergeRequest) -> Result<MergeResult, AppError> {
    MergerService::default().merge(request)
}

#[allow(dead_code)]
fn _keep_preview_item_imported(_item: PreviewItem) {}
