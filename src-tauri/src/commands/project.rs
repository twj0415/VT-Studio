use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::project::{
    CreateProjectRequest, GenerateProjectCoverRequest, ListProjectsRequest, PageResult,
    ProjectDetailDto, ProjectSummaryDto, ReplaceProjectCoverImageRequest,
};
use crate::services::project_service;
use tauri::State;

#[tauri::command]
pub fn list_projects(
    state: State<'_, AppState>,
    request: ListProjectsRequest,
) -> AppResult<PageResult<ProjectSummaryDto>> {
    project_service::list_projects(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    request: CreateProjectRequest,
) -> AppResult<ProjectDetailDto> {
    project_service::create_project(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn get_project_detail(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<ProjectDetailDto> {
    project_service::get_project_detail(state.database(), project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn update_project(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<ProjectDetailDto> {
    project_service::update_project(state.database(), project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn generate_project_cover(
    state: State<'_, AppState>,
    request: GenerateProjectCoverRequest,
) -> AppResult<ProjectDetailDto> {
    project_service::generate_project_cover(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn replace_project_cover_image(
    state: State<'_, AppState>,
    request: ReplaceProjectCoverImageRequest,
) -> AppResult<ProjectDetailDto> {
    project_service::replace_project_cover_image(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}
