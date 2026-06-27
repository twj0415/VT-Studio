use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPackDto {
    pub pack_id: String,
    pub source_type: String,
    pub name: String,
    pub description: String,
    pub applicable_input_types: Vec<String>,
    pub content_category: Option<String>,
    pub default_tone: Option<String>,
    pub default_aspect_ratio: String,
    pub default_duration_seconds: u32,
    pub default_scene_count: u32,
    pub rule_refs: Value,
    pub recommended_executable_refs: Value,
    pub asset_refs: Value,
    pub is_enabled: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub project_reference_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVideoPacksRequest {
    pub source_type: Option<String>,
    pub include_disabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPackIdRequest {
    pub pack_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertUserVideoPackRequest {
    pub pack_id: Option<String>,
    pub name: String,
    pub description: String,
    pub applicable_input_types: Vec<String>,
    pub content_category: Option<String>,
    pub default_tone: Option<String>,
    pub default_aspect_ratio: String,
    pub default_duration_seconds: u32,
    pub default_scene_count: u32,
    pub rule_refs: Value,
    pub recommended_executable_refs: Value,
    pub asset_refs: Value,
    pub is_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVideoPackEnabledRequest {
    pub pack_id: String,
    pub is_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectConfigAsVideoPackRequest {
    pub project_id: String,
    pub name: String,
    pub description: String,
}
