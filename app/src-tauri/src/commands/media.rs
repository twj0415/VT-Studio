use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::media::ExecutableMediaOptionDto;
use crate::services::media_service;

#[tauri::command]
pub fn list_executable_media_options() -> AppResult<Vec<ExecutableMediaOptionDto>> {
    media_service::list_executable_media_options().map_err(AppErrorDto::from)
}
