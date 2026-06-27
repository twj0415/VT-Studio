use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::provider::{
    ProviderDryRunRequest, ProviderDryRunResponse, ProviderGenerationTestRequest,
    ProviderGenerationTestResponse,
};
use crate::services::provider_service::ProviderManager;
use tauri::State;

#[tauri::command]
pub fn provider_dry_run(
    state: State<'_, AppState>,
    request: ProviderDryRunRequest,
) -> AppResult<ProviderDryRunResponse> {
    ProviderManager::new(state.database(), state.keyring_service())
        .dry_run(request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn provider_generation_test(
    state: State<'_, AppState>,
    request: ProviderGenerationTestRequest,
) -> AppResult<ProviderGenerationTestResponse> {
    ProviderManager::new(state.database(), state.keyring_service())
        .generation_test(request)
        .map_err(AppErrorDto::from)
}
