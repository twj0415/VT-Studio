use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DigitalHumanProjectStateDto {
    pub project_id: String,
    pub tts_status: String,
    pub video_status: String,
    pub reference_image_path: Option<String>,
    pub reference_audio_path: Option<String>,
    pub output_video_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartDigitalHumanVideoRequest {
    pub project_id: String,
    pub reference_image_path: Option<String>,
    pub prompt: String,
}
