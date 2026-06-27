use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::video_pack::{
    ListVideoPacksRequest, SaveProjectConfigAsVideoPackRequest, SetVideoPackEnabledRequest,
    UpsertUserVideoPackRequest, VideoPackDto, VideoPackIdRequest,
};
use crate::services::video_pack_service;
use tauri::State;

#[tauri::command]
pub fn list_video_packs(
    state: State<'_, AppState>,
    request: ListVideoPacksRequest,
) -> AppResult<Vec<VideoPackDto>> {
    video_pack_service::list_video_packs(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn get_video_pack(
    state: State<'_, AppState>,
    request: VideoPackIdRequest,
) -> AppResult<VideoPackDto> {
    video_pack_service::get_video_pack(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn clone_video_pack_to_user(
    state: State<'_, AppState>,
    request: VideoPackIdRequest,
) -> AppResult<VideoPackDto> {
    video_pack_service::clone_video_pack_to_user(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn upsert_user_video_pack(
    state: State<'_, AppState>,
    request: UpsertUserVideoPackRequest,
) -> AppResult<VideoPackDto> {
    video_pack_service::upsert_user_video_pack(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn set_video_pack_enabled(
    state: State<'_, AppState>,
    request: SetVideoPackEnabledRequest,
) -> AppResult<VideoPackDto> {
    video_pack_service::set_video_pack_enabled(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_user_video_pack(
    state: State<'_, AppState>,
    request: VideoPackIdRequest,
) -> AppResult<VideoPackDto> {
    video_pack_service::delete_user_video_pack(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn save_project_config_as_video_pack(
    state: State<'_, AppState>,
    request: SaveProjectConfigAsVideoPackRequest,
) -> AppResult<VideoPackDto> {
    video_pack_service::save_project_config_as_video_pack(
        state.database(),
        state.workspace_root(),
        request,
    )
    .map_err(AppErrorDto::from)
}
