use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateStructuredOutputRequest {
    pub raw_output: String,
    pub output_schema: Value,
    pub expected_count: Option<usize>,
    pub repair_attempt_count: Option<u32>,
    pub max_repair_attempts: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredOutputValidationResult {
    pub valid: bool,
    pub parsed_json: Option<Value>,
    pub errors: Vec<String>,
    pub repair_needed: bool,
    pub attempt_count: u32,
    pub max_attempts: u32,
}
