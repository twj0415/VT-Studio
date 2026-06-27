use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutableMediaOptionDto {
    pub option_id: String,
    pub source_type: String,
    pub source_id: String,
    pub label: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub vendor: String,
    pub kind: String,
    pub capability: String,
    pub capabilities: Vec<String>,
    pub constraints: Value,
    pub input_plan: MediaInputPlanDto,
    pub status: String,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
    pub normalized_params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInputPlanDto {
    pub plan_kind: String,
    pub ability_type: String,
    pub image_kind: Option<String>,
    pub asset_kind: Option<String>,
    pub items: Vec<MediaInputRequirementDto>,
    pub required_count: usize,
    pub optional_count: usize,
    pub unused_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInputRequirementDto {
    pub input_key: String,
    pub input_group: String,
    pub owner_type: Option<String>,
    pub owner_id: Option<String>,
    pub requirement: String,
    pub source_options: Vec<String>,
    pub missing_reason: Option<String>,
    pub ui_schema: Value,
    pub constraints: Value,
    pub normalized_params: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetDto {
    pub asset_id: String,
    pub kind: String,
    pub relative_path: String,
    pub source_kind: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub is_builtin: bool,
    pub lifecycle: String,
    pub metadata: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetReferenceDto {
    pub reference_id: String,
    pub asset_id: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub usage_kind: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAssetRequest {
    pub source_path: String,
    pub kind: String,
    pub display_name: Option<String>,
    pub mime_type: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAssetsRequest {
    pub kind: Option<String>,
    pub include_deleted: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAssetRequest {
    pub asset_id: String,
    pub physical: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAssetReferenceRequest {
    pub reference_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssetReferenceRequest {
    pub asset_id: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub usage_kind: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SidecarBinaryStatusDto {
    pub name: String,
    pub relative_path: String,
    pub exists: bool,
    pub executable: bool,
    pub version: Option<String>,
    pub error_code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegSidecarStatusDto {
    pub ffmpeg: SidecarBinaryStatusDto,
    pub ffprobe: SidecarBinaryStatusDto,
    pub ready: bool,
    pub checked_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeMediaRequest {
    pub relative_path: String,
    pub media_kind: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaProbeDto {
    pub path: String,
    pub media_kind: String,
    pub container: Option<String>,
    pub format_name: Option<String>,
    pub duration_seconds: f64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub video_codec: Option<String>,
    pub pixel_format: Option<String>,
    pub audio_codec: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub bit_rate: Option<u64>,
    pub has_video_stream: bool,
    pub has_audio_stream: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPreviewRequest {
    pub asset_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetPreviewDto {
    pub asset_id: String,
    pub relative_path: String,
    pub media_kind: String,
    pub mime_type: String,
    pub preview_kind: String,
    pub bytes: Vec<u8>,
}
