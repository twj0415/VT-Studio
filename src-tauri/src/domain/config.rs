use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigDto {
    pub app_locale: String,
    pub theme_preset: String,
    pub layout_density: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfigDto {
    pub provider_id: String,
    pub provider_kind: String,
    pub vendor: String,
    pub display_name: String,
    pub base_url: Option<String>,
    pub auth_type: String,
    pub key_alias: Option<String>,
    pub status: String,
    pub is_enabled: bool,
    pub config: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProviderConfigsRequest {
    pub provider_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteProviderConfigRequest {
    pub provider_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelDto {
    pub model_id: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub vendor: String,
    pub provider_model_id: String,
    pub model_name: String,
    pub display_name: String,
    pub ability_types: Vec<String>,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
    pub feature_flags: Vec<String>,
    pub limits: Value,
    pub input_requirements: Value,
    pub api_contract_verified: bool,
    pub status: String,
    pub is_enabled: bool,
    pub config: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProviderModelsRequest {
    pub provider_id: Option<String>,
    pub provider_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteProviderModelRequest {
    pub model_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowPresetDto {
    pub workflow_preset_id: String,
    pub provider_id: String,
    pub vendor: String,
    pub workflow_key: String,
    pub workflow_id: Option<String>,
    pub display_name: String,
    pub workflow_version: String,
    pub ability_types: Vec<String>,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
    pub limits: Value,
    pub param_schema: Value,
    pub node_map: Value,
    pub output_map: Value,
    pub default_params: Value,
    pub status: String,
    pub is_builtin: bool,
    pub is_enabled: bool,
    pub config: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkflowPresetsRequest {
    pub provider_id: Option<String>,
    pub vendor: Option<String>,
    pub ability_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorkflowPresetRequest {
    pub workflow_preset_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderSecretRequest {
    pub provider_id: String,
    pub auth_type: String,
    pub key_alias: Option<String>,
    pub secret: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSecretAliasRequest {
    pub key_alias: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSecretHandleDto {
    pub provider_id: String,
    pub auth_type: String,
    pub key_alias: String,
    pub has_secret: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSecretStatusDto {
    pub key_alias: String,
    pub has_secret: bool,
}
