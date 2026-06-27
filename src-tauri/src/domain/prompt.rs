use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreativeRuleDto {
    pub rule_id: String,
    pub key: String,
    pub name: String,
    pub module: String,
    pub rule_type: String,
    pub provider_kind: String,
    pub version: String,
    pub output_schema: Value,
    pub params_schema: Value,
    pub description: String,
    pub source_type: String,
    pub enabled: bool,
    pub body: String,
    pub relative_path: String,
    pub content_hash: String,
    pub schema_hash: String,
    pub reference_counts: CreativeRuleReferenceCountsDto,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreativeRuleReferenceCountsDto {
    pub video_packs: u32,
    pub projects: u32,
    pub task_steps: u32,
    pub generation_contexts: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreativeRuleRefDto {
    pub slot: String,
    pub rule_key: String,
    pub rule_id: String,
    pub source_type: String,
    pub rule_type: String,
    pub module: String,
    pub name: String,
    pub version: String,
    pub content_hash: String,
    pub schema_hash: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCreativeRulesRequest {
    pub source_type: Option<String>,
    pub module: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreativeRuleIdRequest {
    pub rule_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCreativeRuleRequest {
    pub key: String,
    pub name: String,
    pub module: String,
    pub rule_type: String,
    pub provider_kind: String,
    pub version: Option<String>,
    pub output_schema: Value,
    pub params_schema: Option<Value>,
    pub description: String,
    pub enabled: bool,
    pub body: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetCreativeRuleEnabledRequest {
    pub rule_id: String,
    pub enabled: bool,
}
