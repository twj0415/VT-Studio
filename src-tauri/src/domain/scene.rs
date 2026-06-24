use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NarrationDto {
    pub index: u32,
    pub text: String,
    pub locked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageCandidateDto {
    pub image_id: String,
    pub item_id: String,
    pub image_path: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub model: String,
    pub provider_model_id: String,
    pub workflow_preset_id: Option<String>,
    pub status: String,
    pub selected: bool,
    pub created_at: String,
    pub derived_from_image_id: Option<String>,
    pub generation_context_snapshot: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VideoSegmentDto {
    pub segment_id: String,
    pub item_id: String,
    pub input_image_id: String,
    pub video_path: String,
    pub video_prompt: String,
    pub duration_seconds: f64,
    pub model: String,
    pub provider_model_id: String,
    pub workflow_preset_id: Option<String>,
    pub status: String,
    pub selected: bool,
    pub created_at: String,
    pub generation_context_snapshot: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoryboardItemDto {
    pub item_id: String,
    pub project_id: String,
    pub index: u32,
    pub source_text: String,
    pub narration_text: String,
    pub visual_goal: String,
    pub visual_description: String,
    pub characters: Vec<String>,
    pub character_ids: Vec<String>,
    pub location_id: Option<String>,
    pub scene_description: String,
    pub image_prompt: String,
    pub negative_prompt: String,
    pub video_prompt: String,
    pub duration_seconds: f64,
    pub selected_image_id: Option<String>,
    pub selected_video_segment_id: Option<String>,
    pub status: String,
    pub lock_flags_json: Value,
    pub shot_size: Option<String>,
    pub camera_motion: Option<String>,
    pub composition: Option<String>,
    pub pace: Option<String>,
    pub transition_type: Option<String>,
    pub image_status: String,
    pub audio_status: String,
    pub video_status: String,
    pub subtitle_status: String,
    pub render_status: String,
    pub segment_status: String,
    pub image_candidates: Vec<ImageCandidateDto>,
    pub video_segments: Vec<VideoSegmentDto>,
}

pub type SceneDto = StoryboardItemDto;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryboardDto {
    pub storyboard_id: String,
    pub project_id: String,
    pub confirmed_narrations: Vec<NarrationDto>,
    pub items: Vec<StoryboardItemDto>,
    pub review_status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateImagePromptsRequest {
    pub project_id: String,
    pub item_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartImageGenerationRequest {
    pub project_id: String,
    pub item_id: String,
    pub count: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectImageCandidateRequest {
    pub item_id: String,
    pub image_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartVideoGenerationRequest {
    pub project_id: String,
    pub item_id: String,
    pub count: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectVideoSegmentRequest {
    pub item_id: String,
    pub segment_id: String,
}
