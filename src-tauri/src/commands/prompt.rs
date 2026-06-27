use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::prompt::{
    CreativeRuleDto, CreativeRuleIdRequest, ListCreativeRulesRequest, SaveCreativeRuleRequest,
    SetCreativeRuleEnabledRequest,
};
use crate::domain::structured_output::{
    StructuredOutputValidationResult, ValidateStructuredOutputRequest,
};
use crate::services::{prompt_service, structured_output_service};
use tauri::State;

#[tauri::command]
pub fn list_creative_rules(
    state: State<'_, AppState>,
    request: ListCreativeRulesRequest,
) -> AppResult<Vec<CreativeRuleDto>> {
    prompt_service::list_creative_rules(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn get_creative_rule(
    state: State<'_, AppState>,
    request: CreativeRuleIdRequest,
) -> AppResult<CreativeRuleDto> {
    prompt_service::get_creative_rule(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn clone_creative_rule_to_user(
    state: State<'_, AppState>,
    request: CreativeRuleIdRequest,
) -> AppResult<CreativeRuleDto> {
    prompt_service::clone_creative_rule_to_user(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn save_user_creative_rule(
    state: State<'_, AppState>,
    request: SaveCreativeRuleRequest,
) -> AppResult<CreativeRuleDto> {
    prompt_service::save_user_creative_rule(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn set_user_creative_rule_enabled(
    state: State<'_, AppState>,
    request: SetCreativeRuleEnabledRequest,
) -> AppResult<CreativeRuleDto> {
    prompt_service::set_user_creative_rule_enabled(
        state.database(),
        state.workspace_root(),
        request,
    )
    .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_user_creative_rule(
    state: State<'_, AppState>,
    request: CreativeRuleIdRequest,
) -> AppResult<CreativeRuleDto> {
    prompt_service::delete_user_creative_rule(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn validate_structured_output(
    request: ValidateStructuredOutputRequest,
) -> AppResult<StructuredOutputValidationResult> {
    structured_output_service::validate_structured_output(request).map_err(AppErrorDto::from)
}
