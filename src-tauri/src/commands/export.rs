use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::export::{
    BackupWorkspaceDto, BackupWorkspaceRequest, ExportDiagnosticPackageDto,
    ExportDiagnosticPackageRequest, ExportFinalVideoRequest, ExportProjectPackageRequest,
    ExportRecordDto, ImportProjectPackageDto, ImportProjectPackageRequest,
    ListExportRecordsRequest, OpenExportDirectoryDto, OpenExportDirectoryRequest,
    RestoreWorkspaceDto, RestoreWorkspaceRequest,
};
use crate::services::{
    export_service,
    storage_service::{FileBucket, StorageService},
};
use std::path::Path;
use std::process::Command;
use tauri::State;

#[tauri::command]
pub fn export_final_video(
    state: State<'_, AppState>,
    request: ExportFinalVideoRequest,
) -> AppResult<ExportRecordDto> {
    export_service::export_final_video(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn list_export_records(
    state: State<'_, AppState>,
    request: ListExportRecordsRequest,
) -> AppResult<Vec<ExportRecordDto>> {
    export_service::list_export_records(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn export_project_package(
    state: State<'_, AppState>,
    request: ExportProjectPackageRequest,
) -> AppResult<ExportRecordDto> {
    export_service::export_project_package(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn import_project_package(
    state: State<'_, AppState>,
    request: ImportProjectPackageRequest,
) -> AppResult<ImportProjectPackageDto> {
    export_service::import_project_package(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn backup_workspace(
    state: State<'_, AppState>,
    request: BackupWorkspaceRequest,
) -> AppResult<BackupWorkspaceDto> {
    export_service::backup_workspace(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn restore_workspace(
    state: State<'_, AppState>,
    request: RestoreWorkspaceRequest,
) -> AppResult<RestoreWorkspaceDto> {
    export_service::restore_workspace(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn export_diagnostic_package(
    state: State<'_, AppState>,
    request: ExportDiagnosticPackageRequest,
) -> AppResult<ExportDiagnosticPackageDto> {
    export_service::export_diagnostic_package(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn open_export_directory(
    state: State<'_, AppState>,
    request: OpenExportDirectoryRequest,
) -> AppResult<OpenExportDirectoryDto> {
    let directory =
        export_service::open_export_directory(state.database(), state.workspace_root(), request)
            .map_err(AppErrorDto::from)?;
    let storage = StorageService::new(state.workspace_root());
    let output_bucket_path = directory
        .directory_relative_path
        .strip_prefix("outputs/")
        .ok_or_else(|| {
            AppErrorDto::from("export.open_failed: directory is outside outputs.".to_string())
        })?;
    let absolute_directory = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Output, output_bucket_path)
        .map_err(|error| AppErrorDto::from(format!("export.open_failed: {error}")))?;
    open_directory(&absolute_directory).map_err(AppErrorDto::from)?;
    Ok(directory)
}

#[cfg(target_os = "windows")]
fn open_directory(directory: &Path) -> Result<(), String> {
    Command::new("explorer")
        .arg(directory)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("export.open_failed: {error}"))
}

#[cfg(target_os = "macos")]
fn open_directory(directory: &Path) -> Result<(), String> {
    Command::new("open")
        .arg(directory)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("export.open_failed: {error}"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_directory(directory: &Path) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(directory)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("export.open_failed: {error}"))
}
