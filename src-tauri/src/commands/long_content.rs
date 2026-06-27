use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::long_content::{
    ListLongContentPlansRequest, LongContentPlanDto, LongContentPlanIdRequest,
    SaveLongContentPlanRequest,
};
use crate::services::long_content_service;
use tauri::State;

#[tauri::command]
pub fn save_long_content_plan(
    state: State<'_, AppState>,
    request: SaveLongContentPlanRequest,
) -> AppResult<LongContentPlanDto> {
    long_content_service::save_long_content_plan(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_long_content_plans(
    state: State<'_, AppState>,
    request: ListLongContentPlansRequest,
) -> AppResult<Vec<LongContentPlanDto>> {
    long_content_service::list_long_content_plans(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn approve_long_content_plan(
    state: State<'_, AppState>,
    request: LongContentPlanIdRequest,
) -> AppResult<LongContentPlanDto> {
    long_content_service::approve_long_content_plan(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn reject_long_content_plan(
    state: State<'_, AppState>,
    request: LongContentPlanIdRequest,
) -> AppResult<LongContentPlanDto> {
    long_content_service::reject_long_content_plan(state.database(), request)
        .map_err(AppErrorDto::from)
}
