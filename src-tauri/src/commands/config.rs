use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::config::{
    AppConfigDto, DeleteProviderConfigRequest, ListProviderConfigsRequest, ProviderConfigDto,
    ProviderSecretAliasRequest, ProviderSecretHandleDto, ProviderSecretStatusDto,
    SaveProviderSecretRequest,
};
use crate::domain::config::{
    DeleteProviderModelRequest, ListProviderModelsRequest, ProviderModelDto,
};
use crate::domain::config::{
    DeleteWorkflowPresetRequest, ListWorkflowPresetsRequest, WorkflowPresetDto,
};
use crate::services::config_service;
use tauri::State;

#[tauri::command]
pub fn get_app_config(state: State<'_, AppState>) -> AppResult<AppConfigDto> {
    config_service::get_app_config(state.database()).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn update_app_config(
    state: State<'_, AppState>,
    config: AppConfigDto,
) -> AppResult<AppConfigDto> {
    config_service::update_app_config(state.database(), config).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn save_provider_secret(
    state: State<'_, AppState>,
    request: SaveProviderSecretRequest,
) -> AppResult<ProviderSecretHandleDto> {
    state
        .keyring_service()
        .save_provider_secret(
            &request.provider_id,
            &request.auth_type,
            request.key_alias.as_deref(),
            &request.secret,
        )
        .map(|handle| ProviderSecretHandleDto {
            provider_id: handle.provider_id,
            auth_type: handle.auth_type,
            key_alias: handle.key_alias,
            has_secret: handle.has_secret,
        })
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_provider_secret(
    state: State<'_, AppState>,
    request: ProviderSecretAliasRequest,
) -> AppResult<ProviderSecretStatusDto> {
    state
        .keyring_service()
        .delete_provider_secret(&request.key_alias)
        .map(|_| ProviderSecretStatusDto {
            key_alias: request.key_alias,
            has_secret: false,
        })
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn has_provider_secret(
    state: State<'_, AppState>,
    request: ProviderSecretAliasRequest,
) -> AppResult<ProviderSecretStatusDto> {
    state
        .keyring_service()
        .has_provider_secret(&request.key_alias)
        .map(|has_secret| ProviderSecretStatusDto {
            key_alias: request.key_alias,
            has_secret,
        })
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_provider_configs(
    state: State<'_, AppState>,
    request: ListProviderConfigsRequest,
) -> AppResult<Vec<ProviderConfigDto>> {
    config_service::list_provider_configs(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn upsert_provider_config(
    state: State<'_, AppState>,
    config: ProviderConfigDto,
) -> AppResult<ProviderConfigDto> {
    config_service::upsert_provider_config(state.database(), config).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_provider_config(
    state: State<'_, AppState>,
    request: DeleteProviderConfigRequest,
) -> AppResult<ProviderConfigDto> {
    config_service::delete_provider_config(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_provider_models(
    state: State<'_, AppState>,
    request: ListProviderModelsRequest,
) -> AppResult<Vec<ProviderModelDto>> {
    config_service::list_provider_models(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn upsert_provider_model(
    state: State<'_, AppState>,
    model: ProviderModelDto,
) -> AppResult<ProviderModelDto> {
    config_service::upsert_provider_model(state.database(), model).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_provider_model(
    state: State<'_, AppState>,
    request: DeleteProviderModelRequest,
) -> AppResult<ProviderModelDto> {
    config_service::delete_provider_model(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_workflow_presets(
    state: State<'_, AppState>,
    request: ListWorkflowPresetsRequest,
) -> AppResult<Vec<WorkflowPresetDto>> {
    config_service::list_workflow_presets(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn upsert_workflow_preset(
    state: State<'_, AppState>,
    preset: WorkflowPresetDto,
) -> AppResult<WorkflowPresetDto> {
    config_service::upsert_workflow_preset(state.database(), preset).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_workflow_preset(
    state: State<'_, AppState>,
    request: DeleteWorkflowPresetRequest,
) -> AppResult<WorkflowPresetDto> {
    config_service::delete_workflow_preset(state.database(), request).map_err(AppErrorDto::from)
}
