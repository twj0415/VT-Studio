use crate::domain::media::AssetReferenceDto;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaterialAnalysisSuggestionDto {
    pub suggestion_id: String,
    pub project_id: String,
    pub asset_id: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub suggestion: Value,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoryboardMaterialRequirementDto {
    pub item_id: String,
    pub project_id: String,
    pub requirement_status: String,
    pub no_material_reason: Option<String>,
    pub confirmed_by_user: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoryboardMaterialCoverageDto {
    pub item_id: String,
    pub project_id: String,
    pub bound_assets: Vec<AssetReferenceDto>,
    pub requirement: Option<StoryboardMaterialRequirementDto>,
    pub satisfied: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaterialEditProjectStateDto {
    pub project_id: String,
    pub import_status: String,
    pub analysis_status: String,
    pub matching_status: String,
    pub suggestions: Vec<MaterialAnalysisSuggestionDto>,
    pub coverage: Vec<StoryboardMaterialCoverageDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMaterialAnalysisSuggestionRequest {
    pub project_id: String,
    pub asset_id: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub raw_output: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialAnalysisSuggestionIdRequest {
    pub suggestion_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindStoryboardMaterialRequest {
    pub project_id: String,
    pub item_id: String,
    pub asset_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkStoryboardNoMaterialRequest {
    pub project_id: String,
    pub item_id: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialEditProjectRequest {
    pub project_id: String,
}
