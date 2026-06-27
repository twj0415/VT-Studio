use crate::domain::media::AssetReferenceDto;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationBibleDto {
    pub location_bible_id: String,
    pub project_id: String,
    pub location_id: String,
    pub name: String,
    pub space_description: String,
    pub lighting: String,
    pub time_of_day: String,
    pub props: Vec<String>,
    pub visual_prompt: String,
    pub negative_prompt: String,
    pub reference_image_path: Option<String>,
    pub reference_images: Vec<Value>,
    pub variants: Vec<Value>,
    pub data: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertProjectLocationBibleRequest {
    pub project_id: String,
    pub location_id: Option<String>,
    pub name: String,
    pub space_description: String,
    pub lighting: String,
    pub time_of_day: String,
    pub props: Vec<String>,
    pub visual_prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub reference_image_path: Option<String>,
    pub reference_images: Option<Vec<Value>>,
    pub variants: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationBibleIdRequest {
    pub location_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindLocationReferenceAssetRequest {
    pub project_id: String,
    pub location_id: String,
    pub asset_id: String,
    pub reference_role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindLocationReferenceAssetResponse {
    pub location_bible: LocationBibleDto,
    pub reference: AssetReferenceDto,
}
