use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const LONG_CONTENT_PLAN_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LongContentPlanDto {
    pub plan_id: String,
    pub project_id: String,
    pub plan_kind: String,
    pub parent_plan_id: Option<String>,
    pub chapter_ids: Vec<String>,
    pub content: Value,
    pub status: String,
    pub schema_version: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLongContentPlanRequest {
    pub project_id: String,
    pub plan_kind: String,
    pub parent_plan_id: Option<String>,
    pub chapter_ids: Vec<String>,
    pub raw_output: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLongContentPlansRequest {
    pub project_id: String,
    pub plan_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LongContentPlanIdRequest {
    pub plan_id: String,
}
