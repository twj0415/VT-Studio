use crate::domain::media::FfmpegSidecarStatusDto;
use crate::domain::template::TemplateSidecarStatusDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppReleaseInfoDto {
    pub app_name: String,
    pub product_name: String,
    pub version: String,
    pub identifier: String,
    pub target_platform: String,
    pub update_channel: String,
    pub auto_update_enabled: bool,
    pub update_feed_url: Option<String>,
    pub update_requires_https: bool,
    pub update_signature_required: bool,
    pub installer_signature_required: bool,
    pub signed_installer_verified: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSelfCheckDto {
    pub ready: bool,
    pub checked_at: String,
    pub workspace: RuntimeCheckItemDto,
    pub sqlite: RuntimeCheckItemDto,
    pub ffmpeg: FfmpegSidecarStatusDto,
    pub template_sidecar: TemplateSidecarStatusDto,
    pub template_manifest: RuntimeCheckItemDto,
    pub template_preview: RuntimeCheckItemDto,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCheckItemDto {
    pub key: String,
    pub ready: bool,
    pub skipped: bool,
    pub error_code: Option<String>,
    pub message: Option<String>,
}
