use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutableMediaOptionDto {
    pub option_id: String,
    pub label: String,
    pub kind: String,
    pub capability: String,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub enabled: bool,
}
