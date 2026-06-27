use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::local_memory::{
    BuildLocalMemoryContextRequest, CreateLocalMemoryRetrievalRequest,
    ListLocalMemoryEntriesRequest, LocalMemoryCandidateDecisionRequest, LocalMemoryContextDto,
    LocalMemoryEntryDto, LocalMemoryRetrievalCandidateDto, LocalMemoryRetrievalDto,
    UpsertLocalMemoryEntryRequest,
};
use crate::services::local_memory_service;
use tauri::State;

#[tauri::command]
pub fn upsert_local_memory_entry(
    state: State<'_, AppState>,
    request: UpsertLocalMemoryEntryRequest,
) -> AppResult<LocalMemoryEntryDto> {
    local_memory_service::upsert_local_memory_entry(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_local_memory_entries(
    state: State<'_, AppState>,
    request: ListLocalMemoryEntriesRequest,
) -> AppResult<Vec<LocalMemoryEntryDto>> {
    local_memory_service::list_local_memory_entries(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn create_local_memory_retrieval(
    state: State<'_, AppState>,
    request: CreateLocalMemoryRetrievalRequest,
) -> AppResult<LocalMemoryRetrievalDto> {
    local_memory_service::create_local_memory_retrieval(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn approve_local_memory_candidate(
    state: State<'_, AppState>,
    request: LocalMemoryCandidateDecisionRequest,
) -> AppResult<LocalMemoryRetrievalCandidateDto> {
    local_memory_service::approve_local_memory_candidate(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn reject_local_memory_candidate(
    state: State<'_, AppState>,
    request: LocalMemoryCandidateDecisionRequest,
) -> AppResult<LocalMemoryRetrievalCandidateDto> {
    local_memory_service::reject_local_memory_candidate(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn build_local_memory_context(
    state: State<'_, AppState>,
    request: BuildLocalMemoryContextRequest,
) -> AppResult<LocalMemoryContextDto> {
    local_memory_service::build_local_memory_context(state.database(), request)
        .map_err(AppErrorDto::from)
}
