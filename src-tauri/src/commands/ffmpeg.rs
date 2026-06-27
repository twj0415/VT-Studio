use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::media::FfmpegSidecarStatusDto;
use crate::services::ffmpeg_service;
use tauri::State;

#[tauri::command]
pub fn check_ffmpeg_sidecars(state: State<'_, AppState>) -> AppResult<FfmpegSidecarStatusDto> {
    ffmpeg_service::check_ffmpeg_sidecars(state.workspace_root()).map_err(AppErrorDto::from)
}
