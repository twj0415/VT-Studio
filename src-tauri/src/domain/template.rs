use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTemplateManifestsRequest {
    pub aspect_ratio: Option<String>,
    pub template_type: Option<String>,
    pub source_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateTemplateParamsRequest {
    pub manifest: TemplateManifestDto,
    pub params: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewTemplateRequest {
    pub template_id: String,
    pub aspect_ratio: String,
    pub template_type: String,
    pub params: Value,
    pub data: TemplateRenderDataDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderTemplateRequest {
    pub template_id: String,
    pub aspect_ratio: String,
    pub template_type: String,
    pub params: Value,
    pub data: TemplateRenderDataDto,
    pub output_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateManifestDto {
    pub template_id: String,
    pub template_type: String,
    pub source_type: String,
    pub display_name: String,
    pub display_name_key: String,
    pub version: String,
    pub aspect_ratio: String,
    pub entry_path: String,
    pub viewport: TemplateViewportDto,
    pub params: Vec<TemplateParamSchemaDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateViewportDto {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateParamSchemaDto {
    pub name: String,
    pub param_type: String,
    pub default_value: Value,
    pub required: bool,
    pub dictionary_code: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateParamValidationResultDto {
    pub valid: bool,
    pub normalized_params: Value,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateRenderDataDto {
    pub title: Option<String>,
    pub narration: Option<String>,
    pub subtitle_chunks: Option<Vec<String>>,
    pub image_path: Option<String>,
    pub video_frame_path: Option<String>,
    pub character_names: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewTemplateResponseDto {
    pub preview_path: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderTemplateResponseDto {
    pub rendered_frame_path: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSidecarBinaryStatusDto {
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
pub struct TemplateSidecarStatusDto {
    pub node: TemplateSidecarBinaryStatusDto,
    pub chromium: TemplateSidecarBinaryStatusDto,
    pub playwright_driver: TemplateSidecarBinaryStatusDto,
    pub ready: bool,
    pub checked_at: String,
}
