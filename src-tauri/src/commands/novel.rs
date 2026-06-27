use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::novel::{
    ImportNovelRequest, ImportNovelResultDto, MarkNovelChapterEventFailedRequest, NovelChapterDto,
    RetryNovelChapterEventRequest, UpdateNovelChapterEventRequest,
};
use crate::services::novel_service;
use tauri::State;

#[tauri::command]
pub fn import_novel(
    state: State<'_, AppState>,
    request: ImportNovelRequest,
) -> AppResult<ImportNovelResultDto> {
    novel_service::import_novel(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_novel_chapters(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<NovelChapterDto>> {
    novel_service::list_novel_chapters(state.database(), project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn update_novel_chapter_event(
    state: State<'_, AppState>,
    request: UpdateNovelChapterEventRequest,
) -> AppResult<NovelChapterDto> {
    novel_service::update_novel_chapter_event(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn mark_novel_chapter_event_failed(
    state: State<'_, AppState>,
    request: MarkNovelChapterEventFailedRequest,
) -> AppResult<NovelChapterDto> {
    novel_service::mark_novel_chapter_event_failed(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn retry_novel_chapter_event(
    state: State<'_, AppState>,
    request: RetryNovelChapterEventRequest,
) -> AppResult<NovelChapterDto> {
    novel_service::retry_novel_chapter_event(state.database(), request).map_err(AppErrorDto::from)
}
