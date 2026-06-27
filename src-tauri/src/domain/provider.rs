use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRequestContext {
    pub trace_id: String,
    pub task_id: Option<String>,
    pub task_step_id: Option<String>,
    pub project_id: Option<String>,
    pub item_id: Option<String>,
    pub provider_id: String,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmChatMessageDto {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmChatRequest {
    pub context: ProviderRequestContext,
    pub messages: Vec<LlmChatMessageDto>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub response_format: Option<String>,
    pub json_schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmChatResponse {
    pub content: String,
    pub parsed_json: Option<Value>,
    pub usage: Option<Value>,
    pub raw_response_summary: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageProviderRequest {
    pub context: ProviderRequestContext,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub aspect_ratio: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub seed: Option<u64>,
    pub reference_images: Vec<ProviderMediaInputDto>,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageProviderResponse {
    pub image_path: String,
    pub seed: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size: Option<u64>,
    pub provider_output_summary: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProviderRequest {
    pub context: ProviderRequestContext,
    pub ability_type: String,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub aspect_ratio: String,
    pub duration_seconds: f64,
    pub resolution: Option<String>,
    pub fps: Option<u32>,
    pub seed: Option<u64>,
    pub input_images: Vec<ProviderMediaInputDto>,
    pub input_video_path: Option<String>,
    pub input_audio_path: Option<String>,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProviderResponse {
    pub video_path: String,
    pub duration_seconds: f64,
    pub fps: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size: Option<u64>,
    pub provider_output_summary: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsProviderRequest {
    pub context: ProviderRequestContext,
    pub text: String,
    pub content_language: String,
    pub voice_id: String,
    pub speed: Option<f64>,
    pub pitch: Option<f64>,
    pub volume: Option<f64>,
    pub format: String,
    pub sample_rate: Option<u32>,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsProviderResponse {
    pub audio_path: String,
    pub audio_duration_seconds: f64,
    pub format: String,
    pub sample_rate: Option<u32>,
    pub file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VlmAnalyzeRequest {
    pub context: ProviderRequestContext,
    pub input_path: String,
    pub prompt: String,
    pub output_schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VlmAnalyzeResponse {
    pub description: String,
    pub parsed_json: Option<Value>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowProviderRequest {
    pub context: ProviderRequestContext,
    pub workflow_preset_id: String,
    pub workflow_vendor: String,
    pub params: Value,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowProviderResponse {
    pub output_path: String,
    pub output_kind: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderMediaInputDto {
    pub path: String,
    pub role: String,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDryRunRequest {
    pub provider_id: String,
    pub provider_kind: String,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub simulate_failure: Option<bool>,
    pub simulate_cancelled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDryRunResponse {
    pub trace_id: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub status: String,
    pub message: String,
    pub output_summary: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGenerationTestRequest {
    pub provider_id: String,
    pub provider_kind: String,
    pub provider_model_id: Option<String>,
    pub workflow_preset_id: Option<String>,
    pub test_mode: String,
    pub real_generate_confirmed: Option<bool>,
    pub confirm_token: Option<String>,
    pub simulate_failure: Option<bool>,
    pub simulate_cancelled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGenerationTestResponse {
    pub trace_id: String,
    pub test_mode: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub status: String,
    pub message: String,
    pub output_summary: Value,
    pub billable: bool,
    pub real_generate_confirmed: bool,
}
