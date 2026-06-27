use crate::db::novel_repository::{NewNovelChapterRecord, NovelRepository};
use crate::db::project_repository::ProjectRepository;
use crate::db::Database;
use crate::domain::novel::{
    ImportNovelRequest, ImportNovelResultDto, MarkNovelChapterEventFailedRequest, NovelChapterDto,
    RetryNovelChapterEventRequest, UpdateNovelChapterEventRequest,
};
use crate::services::storage_service::{FileAccessPolicy, FileBucket, StorageService};
use serde_json::Value;
use std::path::Path;

const FALLBACK_CHAPTER_CHARS: usize = 8_000;

pub fn import_novel(
    database: &Database,
    workspace_root: &Path,
    request: ImportNovelRequest,
) -> Result<ImportNovelResultDto, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for novel import.".to_string());
    }
    let raw_text = request.raw_text.trim();
    if raw_text.is_empty() {
        return Err("novel raw_text is required.".to_string());
    }

    let project = ProjectRepository::new(database)
        .get_detail(&request.project_id)?
        .ok_or_else(|| format!("Project not found: {}", request.project_id))?
        .project;
    if project.input_type != "novel" {
        return Err("novel import requires project input_type=novel.".to_string());
    }

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let stored_file = storage.write_text(
        FileBucket::Project,
        &format!("{}/input/source.txt", request.project_id),
        raw_text,
        FileAccessPolicy::WriteProject,
    )?;
    ProjectRepository::new(database)
        .update_source_text_path(&request.project_id, &stored_file.relative_path)?;

    let chapters = split_novel_chapters(raw_text);
    let chapter_records = chapters
        .into_iter()
        .map(|(chapter_index, chapter_title, chapter_content)| {
            NewNovelChapterRecord::new(chapter_index, chapter_title, chapter_content)
        })
        .collect::<Vec<_>>();
    let chapters = NovelRepository::new(database)
        .replace_project_chapters(&request.project_id, chapter_records)?;

    Ok(ImportNovelResultDto {
        project_id: request.project_id,
        source_text_path: stored_file.relative_path,
        chapters,
    })
}

pub fn list_novel_chapters(
    database: &Database,
    project_id: String,
) -> Result<Vec<NovelChapterDto>, String> {
    if project_id.trim().is_empty() {
        return Err("project_id is required for novel chapters.".to_string());
    }
    NovelRepository::new(database).list_project_chapters(&project_id)
}

pub fn update_novel_chapter_event(
    database: &Database,
    request: UpdateNovelChapterEventRequest,
) -> Result<NovelChapterDto, String> {
    if !request.structured_event.is_object() {
        return Err("structured_event must be a JSON object.".to_string());
    }
    if request.structured_event == Value::Object(Default::default()) {
        return Err("structured_event cannot be empty.".to_string());
    }
    NovelRepository::new(database)
        .mark_event_succeeded(&request.novel_chapter_id, request.structured_event)
}

pub fn mark_novel_chapter_event_failed(
    database: &Database,
    request: MarkNovelChapterEventFailedRequest,
) -> Result<NovelChapterDto, String> {
    if request.error_reason.trim().is_empty() {
        return Err("error_reason is required.".to_string());
    }
    NovelRepository::new(database)
        .mark_event_failed(&request.novel_chapter_id, request.error_reason.trim())
}

pub fn retry_novel_chapter_event(
    database: &Database,
    request: RetryNovelChapterEventRequest,
) -> Result<NovelChapterDto, String> {
    NovelRepository::new(database).reset_event_for_retry(&request.novel_chapter_id)
}

fn split_novel_chapters(raw_text: &str) -> Vec<(u32, String, String)> {
    let lines = raw_text.lines().collect::<Vec<_>>();
    let mut sections = Vec::new();
    let mut current_title = String::new();
    let mut current_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if is_chapter_heading(trimmed) {
            if !current_title.is_empty() || !current_lines.is_empty() {
                sections.push((current_title, current_lines.join("\n").trim().to_string()));
                current_lines = Vec::new();
            }
            current_title = trimmed.to_string();
        } else {
            current_lines.push(line);
        }
    }

    if !current_title.is_empty() || !current_lines.is_empty() {
        sections.push((current_title, current_lines.join("\n").trim().to_string()));
    }

    let sections = sections
        .into_iter()
        .filter(|(_, content)| !content.trim().is_empty())
        .collect::<Vec<_>>();
    if sections.len() > 1 {
        return sections
            .into_iter()
            .enumerate()
            .map(|(index, (title, content))| {
                (
                    (index + 1) as u32,
                    if title.is_empty() {
                        format!("Chapter {}", index + 1)
                    } else {
                        title
                    },
                    content,
                )
            })
            .collect();
    }

    split_by_fixed_chars(raw_text, FALLBACK_CHAPTER_CHARS)
}

fn is_chapter_heading(line: &str) -> bool {
    if line.len() > 80 {
        return false;
    }
    let lower = line.to_ascii_lowercase();
    (line.starts_with('第') && (line.contains('章') || line.contains('回') || line.contains('节')))
        || lower.starts_with("chapter ")
}

fn split_by_fixed_chars(raw_text: &str, max_chars: usize) -> Vec<(u32, String, String)> {
    let chars = raw_text.chars().collect::<Vec<_>>();
    chars
        .chunks(max_chars)
        .enumerate()
        .map(|(index, chunk)| {
            (
                (index + 1) as u32,
                format!("Chapter {}", index + 1),
                chunk.iter().collect::<String>().trim().to_string(),
            )
        })
        .filter(|(_, _, content)| !content.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::project::CreateProjectRequest;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn import_novel_writes_source_file_and_chapters_without_inline_text() {
        let root = test_root("novel_import");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_novel_project(&database, "project_novel_import");

        let result = import_novel(
            &database,
            &workspace_root,
            ImportNovelRequest {
                project_id: "project_novel_import".to_string(),
                raw_text: "第一章 开端\n主角醒来。\n第二章 冲突\n敌人出现。".to_string(),
            },
        )
        .expect("novel should import");

        assert_eq!(result.chapters.len(), 2);
        assert!(result.source_text_path.starts_with("projects/"));
        assert!(workspace_root.join(&result.source_text_path).is_file());
        let project = ProjectRepository::new(&database)
            .get_detail("project_novel_import")
            .expect("project should read")
            .expect("project should exist")
            .project;
        assert!(project.source_text.is_none());
        assert_eq!(project.source_text_path, Some(result.source_text_path));
        assert_eq!(result.chapters[0].event_status, "pending");

        cleanup(root);
    }

    #[test]
    fn novel_chapter_event_can_fail_and_retry_independently() {
        let root = test_root("novel_event_retry");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_novel_project(&database, "project_novel_retry");
        let result = import_novel(
            &database,
            &workspace_root,
            ImportNovelRequest {
                project_id: "project_novel_retry".to_string(),
                raw_text: "第一章 开端\n主角醒来。\n第二章 冲突\n敌人出现。".to_string(),
            },
        )
        .expect("novel should import");
        let chapter_id = result.chapters[1].novel_chapter_id.clone();

        let failed = mark_novel_chapter_event_failed(
            &database,
            MarkNovelChapterEventFailedRequest {
                novel_chapter_id: chapter_id.clone(),
                error_reason: "provider timeout".to_string(),
            },
        )
        .expect("chapter can fail");
        assert_eq!(failed.event_status, "failed");
        assert_eq!(failed.retry_count, 1);

        let retrying = retry_novel_chapter_event(
            &database,
            RetryNovelChapterEventRequest {
                novel_chapter_id: chapter_id.clone(),
            },
        )
        .expect("chapter can retry");
        assert_eq!(retrying.event_status, "pending");
        assert!(retrying.error_reason.is_none());

        let succeeded = update_novel_chapter_event(
            &database,
            UpdateNovelChapterEventRequest {
                novel_chapter_id: chapter_id,
                structured_event: json!({
                    "characters": ["主角"],
                    "locations": ["房间"],
                    "actions": ["醒来"],
                    "conflict": "敌人出现"
                }),
            },
        )
        .expect("chapter event can succeed");
        assert_eq!(succeeded.event_status, "succeeded");
        assert_eq!(succeeded.structured_event["conflict"], "敌人出现");

        cleanup(root);
    }

    fn create_novel_project(database: &Database, project_id: &str) {
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "Novel".to_string(),
                    workflow_type: "image_to_video".to_string(),
                    input_type: "novel".to_string(),
                    topic: None,
                    source_text: Some("seed".to_string()),
                    source_text_path: None,
                    content_language: "zh-CN".to_string(),
                    tone: None,
                    aspect_ratio: "9:16".to_string(),
                    target_scene_count: 8,
                    segment_duration_seconds: 4.0,
                    style_prompt: None,
                    active_pack_id: None,
                    rule_refs: None,
                    executable_refs: None,
                    input_process_mode: "generate".to_string(),
                    input_options: Some(json!({})),
                },
            )
            .expect("project should create");
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
