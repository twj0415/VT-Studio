use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLatestTaskDto {
    pub task_id: String,
    pub task_status: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    pub project_id: String,
    pub title: String,
    pub workflow_type: String,
    pub input_type: String,
    pub input_process_mode: String,
    pub input_options: Value,
    pub source_text: Option<String>,
    pub source_text_path: Option<String>,
    pub aspect_ratio: String,
    pub target_scene_count: u32,
    pub segment_duration_seconds: f64,
    pub style_prompt: Option<String>,
    pub tone: Option<String>,
    pub content_language: String,
    pub lifecycle: String,
    pub created_at: String,
    pub updated_at: String,
    pub latest_task: Option<ProjectLatestTaskDto>,
}

pub type ProjectSummaryDto = ProjectDto;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBibleDto {
    pub project_id: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetailDto {
    pub project: ProjectDto,
    pub project_bible: ProjectBibleDto,
    pub style_bible: Option<NamedProjectAssetDto>,
    pub character_bibles: Vec<NamedProjectAssetDto>,
    pub location_bibles: Vec<NamedProjectAssetDto>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedProjectAssetDto {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResult<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProjectsRequest {
    pub page: u32,
    pub page_size: u32,
    pub keyword: Option<String>,
    pub lifecycle: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub title: String,
    pub workflow_type: String,
    pub input_type: String,
    pub topic: Option<String>,
    pub source_text: Option<String>,
    pub source_text_path: Option<String>,
    pub content_language: String,
    pub tone: Option<String>,
    pub aspect_ratio: String,
    pub target_scene_count: u32,
    pub segment_duration_seconds: f64,
    pub style_prompt: Option<String>,
    pub input_process_mode: String,
    pub input_options: Option<Value>,
}
