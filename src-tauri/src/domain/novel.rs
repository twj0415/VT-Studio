use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelChapterDto {
    pub novel_chapter_id: String,
    pub project_id: String,
    pub chapter_index: u32,
    pub volume_title: Option<String>,
    pub chapter_title: String,
    pub chapter_content: String,
    pub structured_event: Value,
    pub event_status: String,
    pub error_reason: Option<String>,
    pub retry_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportNovelRequest {
    pub project_id: String,
    pub raw_text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportNovelResultDto {
    pub project_id: String,
    pub source_text_path: String,
    pub chapters: Vec<NovelChapterDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNovelChapterEventRequest {
    pub novel_chapter_id: String,
    pub structured_event: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkNovelChapterEventFailedRequest {
    pub novel_chapter_id: String,
    pub error_reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryNovelChapterEventRequest {
    pub novel_chapter_id: String,
}
