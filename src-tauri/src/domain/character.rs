use crate::domain::media::AssetReferenceDto;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBibleDto {
    pub character_bible_id: String,
    pub project_id: String,
    pub character_id: String,
    pub name: String,
    pub alias: Vec<String>,
    pub age: String,
    pub gender: String,
    pub appearance: String,
    pub clothing: String,
    pub personality: String,
    pub visual_prompt: String,
    pub negative_prompt: String,
    pub reference_image_path: Option<String>,
    pub reference_images: Vec<Value>,
    pub lock_flags: Value,
    pub data: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertProjectCharacterBibleRequest {
    pub project_id: String,
    pub character_id: Option<String>,
    pub name: String,
    pub alias: Vec<String>,
    pub age: String,
    pub gender: String,
    pub appearance: String,
    pub clothing: String,
    pub personality: String,
    pub visual_prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub reference_image_path: Option<String>,
    pub reference_images: Option<Vec<Value>>,
    pub lock_flags: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBibleIdRequest {
    pub character_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindCharacterReferenceAssetRequest {
    pub project_id: String,
    pub character_id: String,
    pub asset_id: String,
    pub reference_role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindCharacterReferenceAssetResponse {
    pub character_bible: CharacterBibleDto,
    pub reference: AssetReferenceDto,
}
