use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::task::{CompositionTaskDto, StartCompositionRequest, TaskDetailDto};
use crate::services::task_service;

#[tauri::command]
pub fn create_task(project_id: String) -> AppResult<TaskDetailDto> {
    task_service::create_task(project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn get_task_detail(project_id: String) -> AppResult<TaskDetailDto> {
    task_service::get_task_detail(project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn approve_task_step(project_id: String, step_name: String) -> AppResult<TaskDetailDto> {
    task_service::approve_task_step(project_id, step_name).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn start_composition(request: StartCompositionRequest) -> AppResult<CompositionTaskDto> {
    task_service::start_composition(request).map_err(AppErrorDto::from)
}
