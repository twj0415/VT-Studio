use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecordDto {
    pub export_id: String,
    pub project_id: String,
    pub composition_task_id: Option<String>,
    pub export_kind: String,
    pub source_relative_path: Option<String>,
    pub target_relative_path: Option<String>,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub error_json: Option<Value>,
    pub metadata_json: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFinalVideoRequest {
    pub project_id: String,
    pub overwrite: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProjectPackageRequest {
    pub project_id: String,
    pub overwrite: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProjectPackageRequest {
    pub package_relative_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportProjectPackageDto {
    pub project_id: String,
    pub source_project_id: String,
    pub title: String,
    pub imported_asset_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupWorkspaceRequest {
    pub overwrite: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackupWorkspaceDto {
    pub backup_id: String,
    pub target_relative_path: String,
    pub project_count: usize,
    pub asset_count: usize,
    pub contains_secrets: bool,
    pub requires_secret_reentry: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreWorkspaceRequest {
    pub backup_relative_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RestoredProjectDto {
    pub project_id: String,
    pub source_project_id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RestoreWorkspaceDto {
    pub backup_id: String,
    pub restored_projects: Vec<RestoredProjectDto>,
    pub restored_asset_count: usize,
    pub restored_template_file_count: usize,
    pub requires_secret_reentry: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDiagnosticPackageRequest {
    pub include_media: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportDiagnosticPackageDto {
    pub diagnostic_id: String,
    pub target_relative_path: String,
    pub contains_secrets: bool,
    pub includes_media: bool,
    pub log_file_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListExportRecordsRequest {
    pub project_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenExportDirectoryRequest {
    pub export_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenExportDirectoryDto {
    pub export_id: String,
    pub directory_relative_path: String,
}
