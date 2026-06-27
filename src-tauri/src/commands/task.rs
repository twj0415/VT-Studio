use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::event::{emit_task_progress, ProgressEvent};
use crate::core::result::AppResult;
use crate::domain::task::{
    CompositionTaskDto, ListTasksRequest, RetryTaskStepRequest, StartCompositionRequest,
    TaskDetailDto, TaskProjectRequest, TaskSummaryDto,
};
use crate::services::task_service;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn create_task(state: State<'_, AppState>, project_id: String) -> AppResult<TaskDetailDto> {
    task_service::create_task(state.database(), project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn get_task_detail(state: State<'_, AppState>, project_id: String) -> AppResult<TaskDetailDto> {
    task_service::get_task_detail(state.database(), project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn approve_task_step(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    step_name: String,
) -> AppResult<TaskDetailDto> {
    let detail = task_service::approve_task_step(state.database(), project_id, step_name)
        .map_err(AppErrorDto::from)?;
    emit_progress(&app_handle, &detail, "Task review step was approved.")?;
    Ok(detail)
}

#[tauri::command]
pub fn start_task(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: TaskProjectRequest,
) -> AppResult<TaskDetailDto> {
    let detail = task_service::start_task(state.database(), request).map_err(AppErrorDto::from)?;
    emit_progress(&app_handle, &detail, "Task was started.")?;
    Ok(detail)
}

#[tauri::command]
pub fn cancel_task(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: TaskProjectRequest,
) -> AppResult<TaskDetailDto> {
    let detail = task_service::cancel_task_with_registry(
        state.database(),
        state.process_handle_registry(),
        request,
    )
    .map_err(AppErrorDto::from)?;
    emit_progress(&app_handle, &detail, "Task was cancelled.")?;
    Ok(detail)
}

#[tauri::command]
pub fn resume_task(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: TaskProjectRequest,
) -> AppResult<TaskDetailDto> {
    let detail = task_service::resume_task(state.database(), request).map_err(AppErrorDto::from)?;
    emit_progress(&app_handle, &detail, "Task was resumed.")?;
    Ok(detail)
}

#[tauri::command]
pub fn retry_task_step(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: RetryTaskStepRequest,
) -> AppResult<TaskDetailDto> {
    let detail =
        task_service::retry_task_step(state.database(), request).map_err(AppErrorDto::from)?;
    emit_progress(&app_handle, &detail, "Task step was scheduled for retry.")?;
    Ok(detail)
}

#[tauri::command]
pub fn list_tasks(
    state: State<'_, AppState>,
    request: ListTasksRequest,
) -> AppResult<Vec<TaskSummaryDto>> {
    task_service::list_tasks(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn start_composition(
    state: State<'_, AppState>,
    request: StartCompositionRequest,
) -> AppResult<CompositionTaskDto> {
    task_service::start_composition(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

fn emit_progress(app_handle: &AppHandle, detail: &TaskDetailDto, message: &str) -> AppResult<()> {
    emit_task_progress(app_handle, ProgressEvent::from_task_detail(detail, message))
        .map_err(AppErrorDto::from)
}
