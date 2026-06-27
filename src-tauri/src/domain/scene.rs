use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::media::{AssetDto, AssetReferenceDto, MediaProbeDto};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NarrationDto {
    pub index: u32,
    pub text: String,
    pub locked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScriptDraftNarrationDto {
    pub index: u32,
    pub source_text: String,
    pub narration_text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyScriptDraftRequest {
    pub project_id: String,
    pub raw_output: String,
    pub expected_count: Option<usize>,
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
    pub media_probe: Option<MediaProbeDto>,
    pub generation_context_snapshot: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoryboardDownstreamResetRecord {
    pub reset_id: String,
    pub item_id: String,
    pub trigger_field: String,
    pub affected_objects: Vec<String>,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleChunkDto {
    pub chunk_id: String,
    pub text: String,
    pub start_seconds: Option<f64>,
    pub end_seconds: Option<f64>,
    pub estimated: bool,
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
    pub subtitle_chunks: Vec<SubtitleChunkDto>,
    pub audio_path: Option<String>,
    pub audio_duration_seconds: Option<f64>,
    pub audio_probe: Option<MediaProbeDto>,
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
    pub image_last_error_json: Option<Value>,
    pub image_retry_count: u32,
    pub audio_status: String,
    pub audio_last_error_json: Option<Value>,
    pub audio_retry_count: u32,
    pub video_status: String,
    pub subtitle_status: String,
    pub render_status: String,
    pub segment_status: String,
    pub image_candidates: Vec<ImageCandidateDto>,
    pub video_segments: Vec<VideoSegmentDto>,
    pub downstream_reset_records: Option<Vec<StoryboardDownstreamResetRecord>>,
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
    pub image_kind: Option<String>,
    pub asset_kind: Option<String>,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub workflow_params: Option<Value>,
    pub aspect_ratio: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub seed: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCharacterResourcePlanRequest {
    pub project_id: String,
    pub item_id: String,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterResourceRequirementDto {
    pub character_id: String,
    pub character_name: String,
    pub role: String,
    pub requirement: String,
    pub available: bool,
    pub asset_id: Option<String>,
    pub relative_path: Option<String>,
    pub missing_reason: Option<String>,
    pub source_options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterResourcePlanDto {
    pub project_id: String,
    pub item_id: String,
    pub option_id: String,
    pub source_type: String,
    pub source_id: String,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub required_count: usize,
    pub optional_count: usize,
    pub unused_count: usize,
    pub missing_required_count: usize,
    pub items: Vec<CharacterResourceRequirementDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartImageAssetGenerationRequest {
    pub project_id: String,
    pub image_kind: String,
    pub asset_kind: Option<String>,
    pub owner_kind: String,
    pub owner_id: String,
    pub reference_role: String,
    pub item_id: Option<String>,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub count: Option<u32>,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub workflow_params: Option<Value>,
    pub aspect_ratio: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub seed: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedImageAssetDto {
    pub asset: AssetDto,
    pub reference: AssetReferenceDto,
    pub image_kind: String,
    pub asset_kind: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub reference_role: String,
    pub usage_kind: String,
    pub relative_path: String,
    pub generation_context_snapshot: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectImageCandidateRequest {
    pub item_id: String,
    pub image_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearHistoricalImageCandidatesRequest {
    pub item_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartVideoGenerationRequest {
    pub project_id: String,
    pub item_id: String,
    pub count: Option<u32>,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub workflow_params: Option<Value>,
    pub aspect_ratio: Option<String>,
    pub resolution: Option<String>,
    pub fps: Option<u32>,
    pub seed: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTtsGenerationRequest {
    pub project_id: String,
    pub item_id: String,
    pub provider_model_id: Option<String>,
    pub voice_id: Option<String>,
    pub speed: Option<f64>,
    pub pitch: Option<f64>,
    pub volume: Option<f64>,
    pub format: Option<String>,
    pub sample_rate: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceStoryboardAudioRequest {
    pub project_id: String,
    pub item_id: String,
    pub source_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeStoryboardAudioRequest {
    pub project_id: String,
    pub item_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSubtitlesRequest {
    pub project_id: String,
    pub item_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStoryboardSubtitlesRequest {
    pub project_id: String,
    pub item_id: String,
    pub subtitle_chunks: Vec<SubtitleChunkDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleStyleDto {
    pub preset_id: String,
    pub position: String,
    pub font_size: u32,
    pub color: String,
    pub outline_color: String,
    pub outline_width: u32,
    pub highlight_color: String,
    pub mode: String,
    pub safe_top: u32,
    pub safe_bottom: u32,
    pub safe_left: u32,
    pub safe_right: u32,
    pub max_chars_per_line: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleWordTimingDto {
    pub token: String,
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub estimated: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTimelineChunkDto {
    pub item_id: String,
    pub item_index: u32,
    pub chunk_id: String,
    pub text: String,
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub estimated: bool,
    pub word_timings: Vec<SubtitleWordTimingDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubtitlesFileDto {
    pub schema_version: u32,
    pub project_id: String,
    pub generated_at: String,
    pub style: SubtitleStyleDto,
    pub chunks: Vec<SubtitleTimelineChunkDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSubtitlesResultDto {
    pub project_id: String,
    pub subtitle_path: String,
    pub items: Vec<StoryboardItemDto>,
    pub subtitles: SubtitlesFileDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectVideoSegmentRequest {
    pub item_id: String,
    pub segment_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearHistoricalVideoSegmentsRequest {
    pub item_id: String,
}
