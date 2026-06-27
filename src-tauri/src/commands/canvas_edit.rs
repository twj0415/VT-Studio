use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::canvas_edit::{CanvasEditCandidateResultDto, CreateCanvasEditCandidateRequest};
use crate::services::canvas_edit_service;
use tauri::State;

#[tauri::command]
pub fn create_canvas_edit_candidate(
    state: State<'_, AppState>,
    request: CreateCanvasEditCandidateRequest,
) -> AppResult<CanvasEditCandidateResultDto> {
    canvas_edit_service::create_canvas_edit_candidate(
        state.database(),
        state.workspace_root(),
        request,
    )
    .map_err(AppErrorDto::from)
}
