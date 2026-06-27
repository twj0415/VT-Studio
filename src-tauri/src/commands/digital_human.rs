use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::digital_human::{DigitalHumanProjectStateDto, StartDigitalHumanVideoRequest};
use crate::services::digital_human_service;
use tauri::State;

#[tauri::command]
pub fn get_digital_human_project_state(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<DigitalHumanProjectStateDto> {
    digital_human_service::get_digital_human_project_state(state.database(), project_id)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn mark_digital_human_tts_succeeded(
    state: State<'_, AppState>,
    project_id: String,
    reference_audio_path: String,
) -> AppResult<DigitalHumanProjectStateDto> {
    digital_human_service::mark_digital_human_tts_succeeded(
        state.database(),
        project_id,
        reference_audio_path,
    )
    .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn mark_digital_human_tts_failed(
    state: State<'_, AppState>,
    project_id: String,
    error_reason: String,
) -> AppResult<DigitalHumanProjectStateDto> {
    digital_human_service::mark_digital_human_tts_failed(state.database(), project_id, error_reason)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn start_digital_human_video(
    state: State<'_, AppState>,
    request: StartDigitalHumanVideoRequest,
) -> AppResult<DigitalHumanProjectStateDto> {
    digital_human_service::start_digital_human_video(state.database(), request)
        .map_err(AppErrorDto::from)
}
