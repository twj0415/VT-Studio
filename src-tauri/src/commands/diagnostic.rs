use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::diagnostic::{AppReleaseInfoDto, RuntimeSelfCheckDto};
use crate::services::diagnostic_service;
use tauri::State;

#[tauri::command]
pub fn get_app_release_info() -> AppReleaseInfoDto {
    diagnostic_service::get_app_release_info()
}

#[tauri::command]
pub fn run_runtime_self_check(state: State<'_, AppState>) -> AppResult<RuntimeSelfCheckDto> {
    diagnostic_service::run_runtime_self_check(state.database(), state.workspace_root())
        .map_err(AppErrorDto::from)
}
