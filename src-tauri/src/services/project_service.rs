use crate::db::project_repository::ProjectRepository;
use crate::db::scene_repository::SceneRepository;
use crate::db::Database;
use crate::domain::project::{
    CreateProjectRequest, GenerateProjectCoverRequest, ListProjectsRequest, PageResult,
    ProjectDetailDto, ProjectSummaryDto, ReplaceProjectCoverImageRequest, UpdateProjectFields,
    UpdateProjectLifecycleRequest, UpdateProjectRequest,
};
use crate::security::path_guard::PathGuard;
use crate::services::storage_service::{FileAccessPolicy, FileBucket, StorageService};
use crate::services::{prompt_service, video_pack_service};
use std::path::{Path, PathBuf};

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
    if let Some(pack_id) = request
        .active_pack_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pack = video_pack_service::get_video_pack(
            database,
            workspace_root,
            crate::domain::video_pack::VideoPackIdRequest {
                pack_id: pack_id.to_string(),
            },
        )?;
        if request.rule_refs.is_none() {
            request.rule_refs = Some(pack.rule_refs);
        }
        if request.executable_refs.is_none() {
            request.executable_refs = Some(pack.recommended_executable_refs);
        }
    }
    if let Some(rule_refs) = request.rule_refs.as_ref() {
        request.rule_refs = Some(prompt_service::resolve_creative_rule_refs(
            workspace_root,
            rule_refs,
        )?);
    }
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

pub fn update_project(
    database: &Database,
    workspace_root: &Path,
    request: UpdateProjectRequest,
) -> Result<ProjectDetailDto, String> {
    validate_project_id(&request.project_id)?;
    let patch = request
        .patch
        .as_object()
        .ok_or_else(|| "project patch must be an object.".to_string())?;
    validate_project_patch_keys(patch)?;

    let current = get_project_detail(database, request.project_id.clone())?;
    let mut input_options = patch_value(
        patch,
        "inputOptions",
        current.project.input_options.clone(),
        "inputOptions",
    )?;
    if let Some(content_category) = patch_nullable_string(patch, "contentCategory")? {
        input_options["contentCategory"] = serde_json::Value::String(content_category);
    }

    let mut fields = UpdateProjectFields {
        title: patch_required_string(patch, "title", current.project.title.clone())?,
        input_options,
        source_text: patch_nullable_string(patch, "sourceText")?
            .or_else(|| patch_nullable_string(patch, "topic").ok().flatten())
            .or(current.project.source_text),
        source_text_path: patch_nullable_string(patch, "sourceTextPath")?
            .or(current.project.source_text_path),
        aspect_ratio: patch_required_string(
            patch,
            "aspectRatio",
            current.project.aspect_ratio.clone(),
        )?,
        target_scene_count: patch_u32(
            patch,
            "targetSceneCount",
            current.project.target_scene_count,
        )?,
        segment_duration_seconds: patch_f64(
            patch,
            "segmentDurationSeconds",
            current.project.segment_duration_seconds,
        )?,
        style_prompt: patch_nullable_string(patch, "stylePrompt")?.or(current.project.style_prompt),
        active_pack_id: patch_nullable_string(patch, "activePackId")?
            .or(current.project.active_pack_id),
        rule_refs: patch_value(patch, "ruleRefs", current.project.rule_refs, "ruleRefs")?,
        executable_refs: patch_value(
            patch,
            "executableRefs",
            current.project.executable_refs,
            "executableRefs",
        )?,
        cover_title: patch_nullable_string(patch, "coverTitle")?.or(current.project.cover_title),
        tone: patch_nullable_string(patch, "tone")?.or(current.project.tone),
        content_language: patch_required_string(
            patch,
            "contentLanguage",
            current.project.content_language.clone(),
        )?,
    };

    validate_update_project_fields(&fields)?;
    if let Some(source_text_path) = fields.source_text_path.as_deref() {
        validate_controlled_relative_path(source_text_path)?;
    }
    if let Some(pack_id) = patch_nullable_string(patch, "activePackId")?
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pack = video_pack_service::get_video_pack(
            database,
            workspace_root,
            crate::domain::video_pack::VideoPackIdRequest {
                pack_id: pack_id.to_string(),
            },
        )?;
        if !patch.contains_key("ruleRefs") {
            fields.rule_refs = pack.rule_refs;
        }
        if !patch.contains_key("executableRefs") {
            fields.executable_refs = pack.recommended_executable_refs;
        }
    }
    fields.rule_refs =
        prompt_service::resolve_creative_rule_refs(workspace_root, &fields.rule_refs)?;

    ProjectRepository::new(database).update_basic_fields(&request.project_id, fields)
}

pub fn update_project_lifecycle(
    database: &Database,
    request: UpdateProjectLifecycleRequest,
) -> Result<ProjectDetailDto, String> {
    validate_project_id(&request.project_id)?;
    validate_project_lifecycle(&request.lifecycle)?;
    ProjectRepository::new(database).update_lifecycle(&request.project_id, &request.lifecycle)
}

pub fn generate_project_cover(
    database: &Database,
    workspace_root: &Path,
    request: GenerateProjectCoverRequest,
) -> Result<ProjectDetailDto, String> {
    validate_project_id(&request.project_id)?;
    let detail = get_project_detail(database, request.project_id.clone())?;
    let cover_title = normalize_cover_title(
        request
            .cover_title
            .as_deref()
            .unwrap_or(&detail.project.title),
    );
    let cover_template_id = normalize_cover_template_id(request.cover_template_id.as_deref());
    let cover_source_item_id = resolve_cover_source_item_id(database, &request)?;

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let source_image_path = request
        .source_image_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            cover_source_item_id
                .as_ref()
                .and_then(|item_id| selected_image_path(database, item_id).ok().flatten())
        });
    if let Some(path) = source_image_path.as_deref() {
        validate_controlled_project_or_asset_path(path)?;
    }

    let cover_relative_path = format!(
        "{}/cover/cover.png",
        sanitize_path_segment(&request.project_id)
    );
    storage.write_bytes(
        FileBucket::Project,
        &cover_relative_path,
        &basic_cover_png_bytes(&cover_title, source_image_path.as_deref()),
        FileAccessPolicy::WriteProject,
    )?;

    ProjectRepository::new(database).update_cover(
        &request.project_id,
        &format!("projects/{cover_relative_path}"),
        &cover_title,
        &cover_template_id,
        cover_source_item_id.as_deref(),
    )
}

pub fn replace_project_cover_image(
    database: &Database,
    workspace_root: &Path,
    request: ReplaceProjectCoverImageRequest,
) -> Result<ProjectDetailDto, String> {
    validate_project_id(&request.project_id)?;
    let detail = get_project_detail(database, request.project_id.clone())?;
    let source_path = PathBuf::from(&request.source_path);
    let extension = source_path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "cover source file must have an extension.".to_string())?;
    validate_cover_image_extension(&extension)?;

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let imported_relative_path = format!(
        "{}/cover/source.{}",
        sanitize_path_segment(&request.project_id),
        extension
    );
    let stored_source = storage.copy_into_bucket(
        &source_path,
        FileBucket::Project,
        &imported_relative_path,
        FileAccessPolicy::WriteProject,
    )?;
    generate_project_cover(
        database,
        workspace_root,
        GenerateProjectCoverRequest {
            project_id: request.project_id,
            cover_title: request.cover_title.or(detail.project.cover_title),
            cover_template_id: request
                .cover_template_id
                .or(detail.project.cover_template_id),
            cover_source_item_id: None,
            source_image_path: Some(stored_source.relative_path),
        },
    )
}

fn validate_create_project_request(request: &CreateProjectRequest) -> Result<(), String> {
    if !matches!(
        request.workflow_type.as_str(),
        "image_to_video" | "digital_human" | "material_edit" | "image_slideshow"
    ) {
        return Err("Only workflow_type=image_to_video, workflow_type=digital_human, workflow_type=material_edit or workflow_type=image_slideshow is enabled.".to_string());
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

fn validate_update_project_fields(fields: &UpdateProjectFields) -> Result<(), String> {
    if fields.title.trim().is_empty() {
        return Err("title cannot be empty.".to_string());
    }
    if fields.target_scene_count == 0 {
        return Err("target_scene_count must be greater than 0.".to_string());
    }
    if fields.segment_duration_seconds <= 0.0 {
        return Err("segment_duration_seconds must be greater than 0.".to_string());
    }
    if fields.aspect_ratio.trim().is_empty() {
        return Err("aspect_ratio cannot be empty.".to_string());
    }
    if fields.content_language.trim().is_empty() {
        return Err("content_language cannot be empty.".to_string());
    }
    Ok(())
}

fn validate_project_patch_keys(
    patch: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    for key in patch.keys() {
        if matches!(
            key.as_str(),
            "lifecycle" | "projectLifecycle" | "project_lifecycle"
        ) {
            return Err("update_project cannot modify project lifecycle.".to_string());
        }
        if !matches!(
            key.as_str(),
            "title"
                | "topic"
                | "sourceText"
                | "sourceTextPath"
                | "inputOptions"
                | "contentCategory"
                | "aspectRatio"
                | "targetSceneCount"
                | "segmentDurationSeconds"
                | "stylePrompt"
                | "activePackId"
                | "ruleRefs"
                | "executableRefs"
                | "coverTitle"
                | "tone"
                | "contentLanguage"
        ) {
            return Err(format!("Unsupported project patch field: {key}"));
        }
    }
    Ok(())
}

fn patch_required_string(
    patch: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    current: String,
) -> Result<String, String> {
    let Some(value) = patch.get(key) else {
        return Ok(current);
    };
    value
        .as_str()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{key} must be a non-empty string."))
}

fn patch_nullable_string(
    patch: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<String>, String> {
    let Some(value) = patch.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_str()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(Some)
        .ok_or_else(|| format!("{key} must be a string or null."))
}

fn patch_u32(
    patch: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    current: u32,
) -> Result<u32, String> {
    let Some(value) = patch.get(key) else {
        return Ok(current);
    };
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("{key} must be a positive integer."))
}

fn patch_f64(
    patch: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    current: f64,
) -> Result<f64, String> {
    let Some(value) = patch.get(key) else {
        return Ok(current);
    };
    value
        .as_f64()
        .ok_or_else(|| format!("{key} must be a number."))
}

fn patch_value(
    patch: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    current: serde_json::Value,
    label: &str,
) -> Result<serde_json::Value, String> {
    let Some(value) = patch.get(key) else {
        return Ok(current);
    };
    if !value.is_object() {
        return Err(format!("{label} must be an object."));
    }
    Ok(value.clone())
}

fn validate_project_id(project_id: &str) -> Result<(), String> {
    if project_id.trim().is_empty() {
        Err("project_id is required.".to_string())
    } else {
        Ok(())
    }
}

fn validate_project_lifecycle(lifecycle: &str) -> Result<(), String> {
    if matches!(lifecycle, "draft" | "active" | "archived" | "deleted") {
        Ok(())
    } else {
        Err(format!("Unsupported project lifecycle: {lifecycle}"))
    }
}

fn resolve_cover_source_item_id(
    database: &Database,
    request: &GenerateProjectCoverRequest,
) -> Result<Option<String>, String> {
    let Some(item_id) = request
        .cover_source_item_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let item = SceneRepository::new(database)
        .get_storyboard_item(item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {item_id}"))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Storyboard item {item_id} does not belong to project {}.",
            request.project_id
        ));
    }
    Ok(Some(item_id.to_string()))
}

fn selected_image_path(database: &Database, item_id: &str) -> Result<Option<String>, String> {
    let item = SceneRepository::new(database)
        .get_storyboard_item(item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {item_id}"))?;
    let Some(selected_image_id) = item.selected_image_id.as_deref() else {
        return Ok(None);
    };
    Ok(item
        .image_candidates
        .into_iter()
        .find(|candidate| candidate.image_id == selected_image_id || candidate.selected)
        .map(|candidate| candidate.image_path))
}

fn normalize_cover_title(value: &str) -> String {
    let title = value.trim();
    let title = if title.is_empty() {
        "未命名封面"
    } else {
        title
    };
    title.chars().take(15).collect()
}

fn normalize_cover_template_id(value: Option<&str>) -> String {
    let normalized = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("knowledge_bold")
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
        .collect::<String>();
    if normalized.is_empty() {
        "knowledge_bold".to_string()
    } else {
        normalized
    }
}

fn validate_controlled_project_or_asset_path(path: &str) -> Result<(), String> {
    let normalized = PathGuard::validate_relative_path(path)?;
    if normalized.starts_with("projects/") || normalized.starts_with("assets/") {
        return Ok(());
    }
    Err("cover source image path must point to a controlled projects or assets bucket.".to_string())
}

fn validate_cover_image_extension(extension: &str) -> Result<(), String> {
    match extension {
        "png" | "jpg" | "jpeg" | "webp" => Ok(()),
        _ => Err(format!("Unsupported cover image extension: {extension}")),
    }
}

fn sanitize_path_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    let sanitized = sanitized.trim_matches('_');
    if sanitized.is_empty() {
        "unnamed".to_string()
    } else {
        sanitized.to_string()
    }
}

fn basic_cover_png_bytes(title: &str, source_image_path: Option<&str>) -> Vec<u8> {
    let mut bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    bytes.extend_from_slice(title.as_bytes());
    if let Some(source_image_path) = source_image_path {
        bytes.extend_from_slice(source_image_path.as_bytes());
    }
    bytes
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
    use super::{
        create_project, generate_project_cover, replace_project_cover_image,
        update_project_lifecycle,
    };
    use crate::db::project_repository::ProjectRepository;
    use crate::db::scene_repository::{NewImageCandidateRecord, SceneRepository};
    use crate::db::Database;
    use crate::domain::project::{
        CreateProjectRequest, GenerateProjectCoverRequest, ReplaceProjectCoverImageRequest,
        UpdateProjectLifecycleRequest, UpdateProjectRequest,
    };
    use crate::domain::scene::SceneDto;
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

    #[test]
    fn generate_project_cover_writes_project_cover_fields_and_file() {
        let root = test_root("cover_generate");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let detail = create_project(&database, &workspace_root, create_request("封面测试"))
            .expect("project should be created");
        let project_id = detail.project.project_id.clone();
        insert_selected_image_storyboard_item(&database, &project_id);

        let updated = generate_project_cover(
            &database,
            &workspace_root,
            GenerateProjectCoverRequest {
                project_id: project_id.clone(),
                cover_title: Some("这是一个很长的封面标题应该截断".to_string()),
                cover_template_id: Some("***".to_string()),
                cover_source_item_id: Some(format!("item_{project_id}")),
                source_image_path: None,
            },
        )
        .expect("cover should generate");

        assert_eq!(
            updated.project.cover_path.as_deref(),
            Some(format!("projects/{project_id}/cover/cover.png").as_str())
        );
        let cover_title = updated
            .project
            .cover_title
            .as_deref()
            .expect("cover title should persist");
        assert_eq!(cover_title.chars().count(), 15);
        assert!(cover_title.starts_with("这是一个很长的封面"));
        assert_eq!(
            updated.project.cover_template_id.as_deref(),
            Some("knowledge_bold")
        );
        assert_eq!(
            updated.project.cover_source_item_id.as_deref(),
            Some(format!("item_{project_id}").as_str())
        );
        let cover_path = workspace_root.join(format!("projects/{project_id}/cover/cover.png"));
        let bytes = fs::read(&cover_path).expect("cover.png should exist");
        assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));

        cleanup(root);
    }

    #[test]
    fn replace_project_cover_image_copies_upload_into_workspace() {
        let root = test_root("cover_upload");
        let source_root = test_root("cover_upload_source");
        let workspace_root = root.join("workspace");
        fs::create_dir_all(&source_root).expect("source root should exist");
        let source_path = source_root.join("local-cover.png");
        fs::write(&source_path, [0x89, b'P', b'N', b'G']).expect("source image should write");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let detail = create_project(&database, &workspace_root, create_request("上传封面"))
            .expect("project should be created");
        let project_id = detail.project.project_id.clone();

        let updated = replace_project_cover_image(
            &database,
            &workspace_root,
            ReplaceProjectCoverImageRequest {
                project_id: project_id.clone(),
                source_path: source_path.to_string_lossy().to_string(),
                cover_title: Some("上传封面".to_string()),
                cover_template_id: Some("cover-basic".to_string()),
            },
        )
        .expect("uploaded cover image should import and generate");

        assert_eq!(
            updated.project.cover_path.as_deref(),
            Some(format!("projects/{project_id}/cover/cover.png").as_str())
        );
        assert!(workspace_root
            .join(format!("projects/{project_id}/cover/source.png"))
            .is_file());
        assert!(workspace_root
            .join(format!("projects/{project_id}/cover/cover.png"))
            .is_file());

        cleanup(root);
        cleanup(source_root);
    }

    #[test]
    fn generate_project_cover_accepts_controlled_asset_source_path() {
        let root = test_root("cover_asset_source");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let detail = create_project(&database, &workspace_root, create_request("资产封面"))
            .expect("project should be created");
        let project_id = detail.project.project_id.clone();

        let updated = generate_project_cover(
            &database,
            &workspace_root,
            GenerateProjectCoverRequest {
                project_id: project_id.clone(),
                cover_title: Some("资产封面".to_string()),
                cover_template_id: Some("asset.cover".to_string()),
                cover_source_item_id: None,
                source_image_path: Some("assets/generated/cover.png".to_string()),
            },
        )
        .expect("controlled asset path should be accepted");

        assert_eq!(
            updated.project.cover_template_id.as_deref(),
            Some("asset.cover")
        );
        assert!(workspace_root
            .join(format!("projects/{project_id}/cover/cover.png"))
            .is_file());

        cleanup(root);
    }

    #[test]
    fn paste_input_must_use_fixed_process_mode_and_preserves_original_text() {
        let root = test_root("paste_fixed");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let mut request = create_request("用户原文第一段\n用户原文第二段");
        request.input_type = "paste".to_string();
        request.input_process_mode = "fixed".to_string();

        let detail = create_project(&database, &workspace_root, request)
            .expect("paste fixed project should create");
        assert_eq!(detail.project.input_process_mode, "fixed");
        assert_eq!(
            detail.project.source_text.as_deref(),
            Some("用户原文第一段\n用户原文第二段")
        );

        cleanup(root);
    }

    #[test]
    fn paste_input_rejects_generate_process_mode() {
        let root = test_root("paste_generate_rejected");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let mut request = create_request("用户原文");
        request.input_type = "paste".to_string();
        request.input_process_mode = "generate".to_string();

        let error = create_project(&database, &workspace_root, request)
            .expect_err("paste generate should be rejected");
        assert!(error.contains("paste input_type must use fixed input_process_mode"));

        cleanup(root);
    }

    #[test]
    fn update_project_lifecycle_persists_and_list_excludes_deleted_by_default() {
        let root = test_root("project_lifecycle");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let detail = create_project(&database, &workspace_root, create_request("生命周期"))
            .expect("project should be created");
        let project_id = detail.project.project_id.clone();

        let archived = update_project_lifecycle(
            &database,
            UpdateProjectLifecycleRequest {
                project_id: project_id.clone(),
                lifecycle: "archived".to_string(),
            },
        )
        .expect("project should archive");
        assert_eq!(archived.project.lifecycle, "archived");

        let deleted = update_project_lifecycle(
            &database,
            UpdateProjectLifecycleRequest {
                project_id: project_id.clone(),
                lifecycle: "deleted".to_string(),
            },
        )
        .expect("project should soft delete");
        assert_eq!(deleted.project.lifecycle, "deleted");

        let visible = ProjectRepository::new(&database)
            .list(crate::domain::project::ListProjectsRequest {
                page: 1,
                page_size: 20,
                keyword: None,
                lifecycle: None,
                sort_by: None,
                sort_order: None,
            })
            .expect("projects should list");
        assert!(visible
            .items
            .iter()
            .all(|project| project.project_id != project_id));

        let deleted_list = ProjectRepository::new(&database)
            .list(crate::domain::project::ListProjectsRequest {
                page: 1,
                page_size: 20,
                keyword: None,
                lifecycle: Some("deleted".to_string()),
                sort_by: None,
                sort_order: None,
            })
            .expect("deleted projects should list");
        assert_eq!(deleted_list.items.len(), 1);
        assert_eq!(deleted_list.items[0].project_id, project_id);

        cleanup(root);
    }

    #[test]
    fn update_project_persists_basic_fields_without_lifecycle() {
        let root = test_root("project_update_basic");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let detail = create_project(&database, &workspace_root, create_request("更新前"))
            .expect("project should be created");
        let project_id = detail.project.project_id.clone();

        let updated = super::update_project(
            &database,
            &workspace_root,
            UpdateProjectRequest {
                project_id: project_id.clone(),
                patch: json!({
                    "title": "更新后的作品",
                    "targetSceneCount": 12,
                    "segmentDurationSeconds": 5,
                    "aspectRatio": "16:9",
                    "contentLanguage": "en-US",
                    "stylePrompt": "cinematic",
                    "inputOptions": { "confirmedSegments": [{ "sourceText": "A", "narrationText": "B" }] }
                }),
            },
        )
        .expect("project should update");

        assert_eq!(updated.project.title, "更新后的作品");
        assert_eq!(updated.project.target_scene_count, 12);
        assert_eq!(updated.project.segment_duration_seconds, 5.0);
        assert_eq!(updated.project.aspect_ratio, "16:9");
        assert_eq!(updated.project.content_language, "en-US");
        assert_eq!(updated.project.lifecycle, "draft");
        assert_eq!(
            updated.project.input_options["confirmedSegments"][0]["narrationText"],
            "B"
        );

        let error = super::update_project(
            &database,
            &workspace_root,
            UpdateProjectRequest {
                project_id,
                patch: json!({ "lifecycle": "deleted" }),
            },
        )
        .expect_err("project lifecycle cannot be updated through update_project");
        assert!(error.contains("cannot modify project lifecycle"));

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
            active_pack_id: None,
            rule_refs: None,
            executable_refs: None,
            input_process_mode: "generate".to_string(),
            input_options: Some(json!({ "splitMode": "paragraph" })),
        }
    }

    fn insert_selected_image_storyboard_item(database: &Database, project_id: &str) {
        let item_id = format!("item_{project_id}");
        let image_id = format!("image_{project_id}");
        let repository = SceneRepository::new(database);
        repository
            .upsert_storyboard_item(&SceneDto {
                item_id: item_id.clone(),
                project_id: project_id.to_string(),
                index: 1,
                source_text: "source".to_string(),
                narration_text: "source".to_string(),
                visual_goal: "goal".to_string(),
                visual_description: "visual".to_string(),
                characters: vec![],
                character_ids: vec![],
                location_id: None,
                scene_description: "scene".to_string(),
                image_prompt: "prompt".to_string(),
                negative_prompt: String::new(),
                video_prompt: "move".to_string(),
                duration_seconds: 4.0,
                subtitle_chunks: vec![],
                audio_path: None,
                audio_duration_seconds: None,
                audio_probe: None,
                selected_image_id: Some(image_id.clone()),
                selected_video_segment_id: None,
                status: "pending".to_string(),
                lock_flags_json: json!({}),
                shot_size: None,
                camera_motion: None,
                composition: None,
                pace: None,
                transition_type: None,
                image_status: "succeeded".to_string(),
                image_last_error_json: None,
                image_retry_count: 0,
                audio_status: "pending".to_string(),
                audio_last_error_json: None,
                audio_retry_count: 0,
                video_status: "pending".to_string(),
                subtitle_status: "pending".to_string(),
                render_status: "pending".to_string(),
                segment_status: "pending".to_string(),
                image_candidates: vec![],
                video_segments: vec![],
                downstream_reset_records: Some(vec![]),
            })
            .expect("storyboard item should save");
        repository
            .insert_image_candidates(&[NewImageCandidateRecord {
                image_id,
                item_id,
                image_path: format!("projects/{project_id}/images/cover-source.png"),
                prompt: "prompt".to_string(),
                negative_prompt: String::new(),
                model: "model".to_string(),
                provider_model_id: "model_cover".to_string(),
                workflow_preset_id: None,
                status: "succeeded".to_string(),
                selected: true,
                derived_from_image_id: None,
                generation_context_snapshot: json!({}),
            }])
            .expect("image candidate should save");
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
