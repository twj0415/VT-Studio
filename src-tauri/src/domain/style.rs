use crate::domain::character::CharacterBibleDto;
use crate::domain::location::LocationBibleDto;
use crate::domain::provider::ProviderMediaInputDto;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleBibleDto {
    pub style_bible_id: String,
    pub project_id: String,
    pub name: String,
    pub style_prompt: String,
    pub color_palette: Vec<String>,
    pub lighting: String,
    pub composition: String,
    pub negative_prompt: String,
    pub reference_image_path: Option<String>,
    pub reference_images: Vec<Value>,
    pub data: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StylePresetDto {
    pub preset_id: String,
    pub source_type: String,
    pub name: String,
    pub style_prompt: String,
    pub color_palette: Vec<String>,
    pub lighting: String,
    pub composition: String,
    pub negative_prompt: String,
    pub reference_image_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertProjectStyleBibleRequest {
    pub project_id: String,
    pub style_bible_id: Option<String>,
    pub name: String,
    pub style_prompt: String,
    pub color_palette: Vec<String>,
    pub lighting: String,
    pub composition: String,
    pub negative_prompt: String,
    pub reference_image_path: Option<String>,
    pub reference_images: Option<Vec<Value>>,
    pub save_as_preset: Option<bool>,
    pub preset_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyStylePresetRequest {
    pub project_id: String,
    pub preset_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindStyleReferenceAssetRequest {
    pub project_id: String,
    pub style_bible_id: Option<String>,
    pub asset_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindStyleReferenceAssetResponse {
    pub style_bible: StyleBibleDto,
    pub reference_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeStyleReferenceRequest {
    pub project_id: String,
    pub style_bible_id: Option<String>,
    pub provider_id: Option<String>,
    pub provider_model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleReferenceAnalysisDto {
    pub project_id: String,
    pub style_bible_id: String,
    pub reference_image_path: String,
    pub style_prompt: String,
    pub color_palette: Vec<String>,
    pub lighting: String,
    pub composition: String,
    pub negative_prompt_suggestion: String,
    pub warnings: Vec<String>,
    pub raw_description: String,
    pub provider_trace_id: Option<String>,
    pub provider_id: Option<String>,
    pub provider_model_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildImagePromptPreviewRequest {
    pub project_id: String,
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptSectionDto {
    pub key: String,
    pub label: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePromptPreviewDto {
    pub project_id: String,
    pub item_id: String,
    pub final_prompt: String,
    pub final_negative_prompt: String,
    pub sections: Vec<PromptSectionDto>,
    pub reference_images: Vec<ProviderMediaInputDto>,
    pub style_bible: Option<StyleBibleDto>,
    pub character_bibles: Vec<CharacterBibleDto>,
    pub location_bible: Option<LocationBibleDto>,
    pub negative_prompt_truncated: bool,
    pub negative_prompt_max_length: usize,
}
