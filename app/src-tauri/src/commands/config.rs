use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::config::AppConfigDto;
use crate::services::config_service;

#[tauri::command]
pub fn get_app_config() -> AppResult<AppConfigDto> {
    config_service::get_app_config().map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn update_app_config(config: AppConfigDto) -> AppResult<AppConfigDto> {
    config_service::update_app_config(config).map_err(AppErrorDto::from)
}
