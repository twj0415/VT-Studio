use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::style::{
    AnalyzeStyleReferenceRequest, ApplyStylePresetRequest, BindStyleReferenceAssetRequest,
    BindStyleReferenceAssetResponse, BuildImagePromptPreviewRequest, ImagePromptPreviewDto,
    StyleBibleDto, StylePresetDto, StyleReferenceAnalysisDto, UpsertProjectStyleBibleRequest,
};
use crate::services::style_service;
use tauri::State;

#[tauri::command]
pub fn get_project_style_bible(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<StyleBibleDto> {
    style_service::get_project_style_bible(state.database(), project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_style_presets(state: State<'_, AppState>) -> AppResult<Vec<StylePresetDto>> {
    style_service::list_style_presets(state.database()).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn upsert_project_style_bible(
    state: State<'_, AppState>,
    request: UpsertProjectStyleBibleRequest,
) -> AppResult<StyleBibleDto> {
    style_service::upsert_project_style_bible(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn apply_style_preset(
    state: State<'_, AppState>,
    request: ApplyStylePresetRequest,
) -> AppResult<StyleBibleDto> {
    style_service::apply_style_preset(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn bind_style_reference_asset(
    state: State<'_, AppState>,
    request: BindStyleReferenceAssetRequest,
) -> AppResult<BindStyleReferenceAssetResponse> {
    style_service::bind_style_reference_asset(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn analyze_style_reference_image(
    state: State<'_, AppState>,
    request: AnalyzeStyleReferenceRequest,
) -> AppResult<StyleReferenceAnalysisDto> {
    style_service::analyze_style_reference_image(state.database(), state.keyring_service(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn build_image_prompt_preview(
    state: State<'_, AppState>,
    request: BuildImagePromptPreviewRequest,
) -> AppResult<ImagePromptPreviewDto> {
    style_service::build_image_prompt_preview(state.database(), request).map_err(AppErrorDto::from)
}
