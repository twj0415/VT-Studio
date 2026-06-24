use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionTaskDto {
    pub task_id: String,
    pub project_id: String,
    pub segment_ids: Vec<String>,
    pub output_path: String,
    pub status: String,
    pub progress: u32,
    pub error_json: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartCompositionRequest {
    pub project_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStepDto {
    pub step_id: String,
    pub step_name: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDetailDto {
    pub task_id: String,
    pub project_id: String,
    pub task_status: String,
    pub current_step: Option<String>,
    pub steps: Vec<TaskStepDto>,
    pub composition_task: Option<CompositionTaskDto>,
}
