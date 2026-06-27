use crate::domain::scene::VideoSegmentDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageSlideshowProjectStateDto {
    pub project_id: String,
    pub template_motion_status: String,
    pub segment_composition_status: String,
    pub items: Vec<ImageSlideshowItemStateDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageSlideshowItemStateDto {
    pub item_id: String,
    pub project_id: String,
    pub selected_image_id: Option<String>,
    pub selected_video_segment_id: Option<String>,
    pub template_segments: Vec<VideoSegmentDto>,
    pub ready_for_composition: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterTemplateMotionSegmentRequest {
    pub project_id: String,
    pub item_id: String,
    pub input_image_id: String,
    pub video_path: String,
    pub duration_seconds: f64,
    pub template_id: String,
    pub template_type: String,
    pub workflow_preset_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSlideshowProjectRequest {
    pub project_id: String,
}
