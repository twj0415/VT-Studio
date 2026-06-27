use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::location::{
    BindLocationReferenceAssetRequest, BindLocationReferenceAssetResponse, LocationBibleDto,
    LocationBibleIdRequest, UpsertProjectLocationBibleRequest,
};
use crate::services::location_service;
use tauri::State;

#[tauri::command]
pub fn list_project_location_bibles(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<LocationBibleDto>> {
    location_service::list_project_location_bibles(state.database(), project_id)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn upsert_project_location_bible(
    state: State<'_, AppState>,
    request: UpsertProjectLocationBibleRequest,
) -> AppResult<LocationBibleDto> {
    location_service::upsert_project_location_bible(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_project_location_bible(
    state: State<'_, AppState>,
    request: LocationBibleIdRequest,
) -> AppResult<LocationBibleDto> {
    location_service::delete_project_location_bible(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn bind_location_reference_asset(
    state: State<'_, AppState>,
    request: BindLocationReferenceAssetRequest,
) -> AppResult<BindLocationReferenceAssetResponse> {
    location_service::bind_location_reference_asset(state.database(), request)
        .map_err(AppErrorDto::from)
}
