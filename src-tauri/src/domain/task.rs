use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const IMAGE_TO_VIDEO_TASK_KIND: &str = "image_to_video";
pub const DIGITAL_HUMAN_TASK_KIND: &str = "digital_human";
pub const MATERIAL_EDIT_TASK_KIND: &str = "material_edit";
pub const IMAGE_SLIDESHOW_TASK_KIND: &str = "image_slideshow";
pub const IMAGE_TO_VIDEO_PIPELINE_STEPS: &[&str] = &[
    "project_init",
    "storyboard_generation",
    "storyboard_review",
    "image_prompt_generation",
    "image_generation",
    "image_review",
    "video_prompt_generation",
    "video_generation",
    "video_review",
    "final_composition",
    "export",
    "cleanup",
];
pub const DIGITAL_HUMAN_PIPELINE_STEPS: &[&str] = &[
    "project_init",
    "script_review",
    "digital_human_asset_review",
    "tts_generation",
    "digital_human_generation",
    "subtitle_generation",
    "final_composition",
    "export",
];
pub const MATERIAL_EDIT_PIPELINE_STEPS: &[&str] = &[
    "project_init",
    "material_import",
    "material_analysis",
    "material_grouping",
    "storyboard_generation",
    "storyboard_review",
    "material_matching",
    "segment_composition",
    "final_composition",
    "export",
];
pub const IMAGE_SLIDESHOW_PIPELINE_STEPS: &[&str] = &[
    "project_init",
    "storyboard_generation",
    "storyboard_review",
    "image_prompt_generation",
    "image_generation",
    "image_review",
    "template_motion",
    "segment_composition",
    "final_composition",
    "export",
];

pub fn initial_step_status(step_name: &str) -> &'static str {
    match step_name {
        "project_init" | "storyboard_generation" => "succeeded",
        "storyboard_review"
        | "script_review"
        | "digital_human_asset_review"
        | "material_import"
        | "material_analysis"
        | "material_matching"
        | "template_motion" => "waiting_user",
        _ => "pending",
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionTaskDto {
    pub task_id: String,
    pub project_id: String,
    pub segment_ids: Vec<String>,
    pub output_path: String,
    pub enhancements: Value,
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
    pub include_subtitle: Option<bool>,
    pub subtitle_path: Option<String>,
    pub include_bgm: Option<bool>,
    pub bgm_asset_id: Option<String>,
    pub bgm_volume: Option<f64>,
    pub bgm_loop: Option<bool>,
    pub bgm_fade_in_seconds: Option<f64>,
    pub bgm_fade_out_seconds: Option<f64>,
    pub include_cover_metadata: Option<bool>,
    pub cover_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProjectRequest {
    pub project_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryTaskStepRequest {
    pub project_id: String,
    pub step_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTasksRequest {
    pub project_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStepDto {
    pub step_id: String,
    pub step_name: String,
    pub status: String,
    pub output_json: Option<Value>,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSummaryDto {
    pub task_id: String,
    pub project_id: String,
    pub task_status: String,
    pub current_step: Option<String>,
    pub summary: String,
    pub created_at: String,
    pub updated_at: String,
}
