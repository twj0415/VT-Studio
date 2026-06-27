use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::media::{
    AssetDto, AssetPreviewDto, AssetPreviewRequest, AssetReferenceDto, CreateAssetReferenceRequest,
    DeleteAssetReferenceRequest, DeleteAssetRequest, ExecutableMediaOptionDto, ImportAssetRequest,
    ListAssetsRequest, MediaProbeDto, ProbeMediaRequest,
};
use crate::services::{ffmpeg_service, media_service};
use tauri::State;

#[tauri::command]
pub fn list_executable_media_options(
    state: State<'_, AppState>,
) -> AppResult<Vec<ExecutableMediaOptionDto>> {
    media_service::list_executable_media_options(state.database()).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn import_asset(
    state: State<'_, AppState>,
    request: ImportAssetRequest,
) -> AppResult<AssetDto> {
    media_service::import_asset(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_assets(
    state: State<'_, AppState>,
    request: ListAssetsRequest,
) -> AppResult<Vec<AssetDto>> {
    media_service::list_assets(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_asset(
    state: State<'_, AppState>,
    request: DeleteAssetRequest,
) -> AppResult<AssetDto> {
    media_service::delete_asset(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn create_asset_reference(
    state: State<'_, AppState>,
    request: CreateAssetReferenceRequest,
) -> AppResult<AssetReferenceDto> {
    media_service::create_asset_reference(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_asset_references(
    state: State<'_, AppState>,
    asset_id: String,
) -> AppResult<Vec<AssetReferenceDto>> {
    media_service::list_asset_references(state.database(), asset_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_asset_reference(
    state: State<'_, AppState>,
    request: DeleteAssetReferenceRequest,
) -> AppResult<AssetReferenceDto> {
    media_service::delete_asset_reference(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn collect_project_asset_paths(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<String>> {
    media_service::collect_project_asset_paths(state.database(), project_id)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn read_asset_preview(
    state: State<'_, AppState>,
    request: AssetPreviewRequest,
) -> AppResult<AssetPreviewDto> {
    media_service::read_asset_preview(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn probe_media(
    state: State<'_, AppState>,
    request: ProbeMediaRequest,
) -> AppResult<MediaProbeDto> {
    ffmpeg_service::probe_media(
        state.workspace_root(),
        &request.relative_path,
        request.media_kind.as_deref(),
    )
    .map_err(AppErrorDto::from)
}
