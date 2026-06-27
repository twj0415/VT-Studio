use crate::domain::scene::ImageCandidateDto;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCanvasEditCandidateRequest {
    pub project_id: String,
    pub source_image_id: String,
    pub edited_image_path: String,
    pub edit_flow_path: String,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub edit_kind: Option<String>,
    pub flow_snapshot: Option<Value>,
    pub select_after_create: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CanvasEditCandidateResultDto {
    pub candidate: ImageCandidateDto,
    pub selected_item_id: Option<String>,
}
