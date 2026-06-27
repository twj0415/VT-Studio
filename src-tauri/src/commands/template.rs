use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::template::{
    ListTemplateManifestsRequest, PreviewTemplateRequest, PreviewTemplateResponseDto,
    RenderTemplateRequest, RenderTemplateResponseDto, TemplateManifestDto,
    TemplateParamValidationResultDto, TemplateSidecarStatusDto, ValidateTemplateParamsRequest,
};
use crate::services::template_service;
use tauri::State;

#[tauri::command]
pub fn list_template_manifests(
    state: State<'_, AppState>,
    request: ListTemplateManifestsRequest,
) -> AppResult<Vec<TemplateManifestDto>> {
    template_service::list_template_manifests(state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn validate_template_params(
    request: ValidateTemplateParamsRequest,
) -> AppResult<TemplateParamValidationResultDto> {
    template_service::validate_template_params(request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn preview_template(
    state: State<'_, AppState>,
    request: PreviewTemplateRequest,
) -> AppResult<PreviewTemplateResponseDto> {
    template_service::preview_template(state.workspace_root(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn render_template(
    state: State<'_, AppState>,
    request: RenderTemplateRequest,
) -> AppResult<RenderTemplateResponseDto> {
    template_service::render_template(state.workspace_root(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn check_template_sidecars(state: State<'_, AppState>) -> AppResult<TemplateSidecarStatusDto> {
    template_service::check_template_sidecars(state.workspace_root()).map_err(AppErrorDto::from)
}
