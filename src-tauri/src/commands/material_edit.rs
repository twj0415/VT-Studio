use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::material_edit::{
    BindStoryboardMaterialRequest, MarkStoryboardNoMaterialRequest, MaterialAnalysisSuggestionDto,
    MaterialAnalysisSuggestionIdRequest, MaterialEditProjectRequest, MaterialEditProjectStateDto,
    SaveMaterialAnalysisSuggestionRequest, StoryboardMaterialCoverageDto,
};
use crate::services::material_edit_service;
use tauri::State;

#[tauri::command]
pub fn get_material_edit_project_state(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<MaterialEditProjectStateDto> {
    material_edit_service::get_material_edit_project_state(state.database(), project_id)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn save_material_analysis_suggestion(
    state: State<'_, AppState>,
    request: SaveMaterialAnalysisSuggestionRequest,
) -> AppResult<MaterialAnalysisSuggestionDto> {
    material_edit_service::save_material_analysis_suggestion(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn approve_material_analysis_suggestion(
    state: State<'_, AppState>,
    request: MaterialAnalysisSuggestionIdRequest,
) -> AppResult<MaterialAnalysisSuggestionDto> {
    material_edit_service::approve_material_analysis_suggestion(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn reject_material_analysis_suggestion(
    state: State<'_, AppState>,
    request: MaterialAnalysisSuggestionIdRequest,
) -> AppResult<MaterialAnalysisSuggestionDto> {
    material_edit_service::reject_material_analysis_suggestion(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn bind_storyboard_material(
    state: State<'_, AppState>,
    request: BindStoryboardMaterialRequest,
) -> AppResult<StoryboardMaterialCoverageDto> {
    material_edit_service::bind_storyboard_material(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn mark_storyboard_no_material_needed(
    state: State<'_, AppState>,
    request: MarkStoryboardNoMaterialRequest,
) -> AppResult<StoryboardMaterialCoverageDto> {
    material_edit_service::mark_storyboard_no_material_needed(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn validate_material_storyboard_coverage(
    state: State<'_, AppState>,
    request: MaterialEditProjectRequest,
) -> AppResult<MaterialEditProjectStateDto> {
    material_edit_service::validate_material_storyboard_coverage(state.database(), request)
        .map_err(AppErrorDto::from)
}
