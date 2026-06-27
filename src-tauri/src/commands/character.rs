use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::character::{
    BindCharacterReferenceAssetRequest, BindCharacterReferenceAssetResponse, CharacterBibleDto,
    CharacterBibleIdRequest, UpsertProjectCharacterBibleRequest,
};
use crate::services::character_service;
use tauri::State;

#[tauri::command]
pub fn list_project_character_bibles(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<CharacterBibleDto>> {
    character_service::list_project_character_bibles(state.database(), project_id)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn upsert_project_character_bible(
    state: State<'_, AppState>,
    request: UpsertProjectCharacterBibleRequest,
) -> AppResult<CharacterBibleDto> {
    character_service::upsert_project_character_bible(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn delete_project_character_bible(
    state: State<'_, AppState>,
    request: CharacterBibleIdRequest,
) -> AppResult<CharacterBibleDto> {
    character_service::delete_project_character_bible(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn bind_character_reference_asset(
    state: State<'_, AppState>,
    request: BindCharacterReferenceAssetRequest,
) -> AppResult<BindCharacterReferenceAssetResponse> {
    character_service::bind_character_reference_asset(state.database(), request)
        .map_err(AppErrorDto::from)
}
