use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalMemoryEntryDto {
    pub memory_id: String,
    pub project_id: Option<String>,
    pub source_kind: String,
    pub source_id: String,
    pub source_label: String,
    pub content_summary: String,
    pub content_hash: String,
    pub embedding_provider_id: Option<String>,
    pub embedding_model_id: Option<String>,
    pub embedding_vector_path: Option<String>,
    pub metadata: Value,
    pub lifecycle: String,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalMemoryRetrievalCandidateDto {
    pub candidate_id: String,
    pub retrieval_id: String,
    pub memory_id: String,
    pub similarity: f64,
    pub status: String,
    pub reason: Option<String>,
    pub citation: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalMemoryRetrievalDto {
    pub retrieval_id: String,
    pub project_id: String,
    pub query_text: String,
    pub min_similarity: f64,
    pub max_results: i64,
    pub status: String,
    pub candidates: Vec<LocalMemoryRetrievalCandidateDto>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalMemoryContextCitationDto {
    pub candidate_id: String,
    pub memory_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub source_label: String,
    pub similarity: f64,
    pub content_summary: String,
    pub citation: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalMemoryContextDto {
    pub retrieval_id: String,
    pub project_id: String,
    pub min_similarity: f64,
    pub usable: bool,
    pub citations: Vec<LocalMemoryContextCitationDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertLocalMemoryEntryRequest {
    pub memory_id: Option<String>,
    pub project_id: Option<String>,
    pub source_kind: String,
    pub source_id: String,
    pub source_label: Option<String>,
    pub content_summary: String,
    pub embedding_provider_id: Option<String>,
    pub embedding_model_id: Option<String>,
    pub embedding_vector_path: Option<String>,
    pub metadata: Option<Value>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLocalMemoryEntriesRequest {
    pub project_id: Option<String>,
    pub include_global: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocalMemoryRetrievalRequest {
    pub project_id: String,
    pub query_text: String,
    pub min_similarity: f64,
    pub max_results: Option<i64>,
    pub candidates: Vec<LocalMemoryRetrievalCandidateInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMemoryRetrievalCandidateInput {
    pub memory_id: String,
    pub similarity: f64,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMemoryCandidateDecisionRequest {
    pub candidate_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildLocalMemoryContextRequest {
    pub retrieval_id: String,
}
