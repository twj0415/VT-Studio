use crate::db::project_repository::ProjectRepository;
use crate::db::Database;
use crate::domain::project::{
    CreateProjectRequest, ListProjectsRequest, PageResult, ProjectDetailDto, ProjectSummaryDto,
};
use crate::security::path_guard::PathGuard;
use crate::services::storage_service::{FileAccessPolicy, FileBucket, StorageService};
use std::path::Path;

const INLINE_SOURCE_TEXT_LIMIT_BYTES: usize = 20 * 1024;

pub fn list_projects(
    database: &Database,
    request: ListProjectsRequest,
) -> Result<PageResult<ProjectSummaryDto>, String> {
    ProjectRepository::new(database).list(request)
}

pub fn create_project(
    database: &Database,
    workspace_root: &Path,
    mut request: CreateProjectRequest,
) -> Result<ProjectDetailDto, String> {
    validate_create_project_request(&request)?;
    let project_id = ProjectRepository::create_project_id();
    apply_long_source_text_strategy(workspace_root, &project_id, &mut request)?;
    ProjectRepository::new(database).create_with_id(project_id, request)
}

pub fn get_project_detail(
    database: &Database,
    project_id: String,
) -> Result<ProjectDetailDto, String> {
    ProjectRepository::new(database)
        .get_detail(&project_id)?
        .ok_or_else(|| format!("Project not found: {project_id}"))
}

pub fn update_project(database: &Database, project_id: String) -> Result<ProjectDetailDto, String> {
    get_project_detail(database, project_id)
}

fn validate_create_project_request(request: &CreateProjectRequest) -> Result<(), String> {
    if request.workflow_type != "image_to_video" {
        return Err(
            "Only workflow_type=image_to_video is enabled in the current mainline.".to_string(),
        );
    }

    if request.input_type == "paste" && request.input_process_mode != "fixed" {
        return Err("paste input_type must use fixed input_process_mode.".to_string());
    }

    if request.input_type != "paste" && request.input_process_mode == "fixed" {
        return Err("fixed input_process_mode is only allowed for paste input_type.".to_string());
    }

    if request.target_scene_count == 0 {
        return Err("target_scene_count must be greater than 0.".to_string());
    }

    if request.segment_duration_seconds <= 0.0 {
        return Err("segment_duration_seconds must be greater than 0.".to_string());
    }

    Ok(())
}

fn apply_long_source_text_strategy(
    workspace_root: &Path,
    project_id: &str,
    request: &mut CreateProjectRequest,
) -> Result<(), String> {
    if let Some(source_text_path) = request.source_text_path.as_deref() {
        validate_controlled_relative_path(source_text_path)?;
    }

    let inline_text = request
        .source_text
        .clone()
        .or_else(|| request.topic.clone());
    let Some(text) = inline_text else {
        return Ok(());
    };

    if request.input_type != "novel" && text.len() <= INLINE_SOURCE_TEXT_LIMIT_BYTES {
        return Ok(());
    }

    if request.title.trim().is_empty() && request.input_type == "topic" {
        request.title = text.chars().take(15).collect();
    }

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let stored_file = storage.write_text(
        FileBucket::Project,
        &format!("{project_id}/input/source.txt"),
        &text,
        FileAccessPolicy::WriteProject,
    )?;

    request.topic = None;
    request.source_text = None;
    request.source_text_path = Some(stored_file.relative_path);

    Ok(())
}

fn validate_controlled_relative_path(path: &str) -> Result<(), String> {
    let normalized = PathGuard::validate_relative_path(path)?;
    if !normalized.starts_with("projects/") {
        return Err("source_text_path must point to the controlled projects bucket.".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::create_project;
    use crate::db::project_repository::ProjectRepository;
    use crate::db::Database;
    use crate::domain::project::CreateProjectRequest;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn short_source_text_stays_inline() {
        let root = test_root("short_inline");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");

        let detail = create_project(&database, &root.join("workspace"), create_request("短文案"))
            .expect("project should be created");

        assert_eq!(detail.project.source_text.as_deref(), Some("短文案"));
        assert!(detail.project.source_text_path.is_none());

        cleanup(root);
    }

    #[test]
    fn long_source_text_is_written_to_controlled_workspace_path() {
        let root = test_root("long_text");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let long_text = "长文本".repeat(8 * 1024);

        let detail = create_project(&database, &workspace_root, create_request(&long_text))
            .expect("project should be created");

        let relative_path = detail
            .project
            .source_text_path
            .as_deref()
            .expect("long text should use relative path");
        assert!(detail.project.source_text.is_none());
        assert!(relative_path.starts_with("projects/"));
        assert!(!PathBuf::from(relative_path).is_absolute());

        let stored_text = fs::read_to_string(workspace_root.join(relative_path))
            .expect("long text file should exist");
        assert_eq!(stored_text, long_text);

        let reread = ProjectRepository::new(&database)
            .get_detail(&detail.project.project_id)
            .expect("project should read")
            .expect("project should exist");
        assert_eq!(
            reread.project.source_text_path.as_deref(),
            Some(relative_path)
        );
        assert!(reread.project.source_text.is_none());

        cleanup(root);
    }

    fn create_request(source_text: &str) -> CreateProjectRequest {
        CreateProjectRequest {
            title: "长文本策略测试".to_string(),
            workflow_type: "image_to_video".to_string(),
            input_type: "article".to_string(),
            topic: None,
            source_text: Some(source_text.to_string()),
            source_text_path: None,
            content_language: "zh-CN".to_string(),
            tone: None,
            aspect_ratio: "9:16".to_string(),
            target_scene_count: 8,
            segment_duration_seconds: 4.0,
            style_prompt: Some("干净真实".to_string()),
            input_process_mode: "generate".to_string(),
            input_options: Some(json!({ "splitMode": "paragraph" })),
        }
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-service-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
