use crate::db::project_repository::ProjectRepository;
use crate::db::scene_repository::{NewVideoSegmentRecord, SceneRepository};
use crate::db::task_repository::TaskRepository;
use crate::db::Database;
use crate::domain::image_slideshow::{
    ImageSlideshowItemStateDto, ImageSlideshowProjectRequest, ImageSlideshowProjectStateDto,
    RegisterTemplateMotionSegmentRequest,
};
use crate::domain::scene::VideoSegmentDto;
use crate::security::path_guard::PathGuard;
use serde_json::json;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_image_slideshow_project_state(
    database: &Database,
    project_id: String,
) -> Result<ImageSlideshowProjectStateDto, String> {
    ensure_image_slideshow_project(database, &project_id)?;
    let task = TaskRepository::new(database)
        .get_latest_task_detail_by_project(&project_id)?
        .ok_or_else(|| format!("Task not found for project: {project_id}"))?;
    let storyboard = SceneRepository::new(database)
        .get_storyboard(&project_id)?
        .ok_or_else(|| format!("Storyboard not found for project: {project_id}"))?;

    Ok(ImageSlideshowProjectStateDto {
        project_id: project_id.clone(),
        template_motion_status: step_status(&task.steps, "template_motion"),
        segment_composition_status: step_status(&task.steps, "segment_composition"),
        items: storyboard
            .items
            .into_iter()
            .map(|item| {
                let template_segments = item
                    .video_segments
                    .iter()
                    .filter(|segment| is_template_motion_segment(segment))
                    .cloned()
                    .collect::<Vec<_>>();
                ImageSlideshowItemStateDto {
                    item_id: item.item_id,
                    project_id: item.project_id,
                    selected_image_id: item.selected_image_id,
                    selected_video_segment_id: item.selected_video_segment_id.clone(),
                    ready_for_composition: item.selected_video_segment_id.as_ref().is_some_and(
                        |segment_id| {
                            template_segments
                                .iter()
                                .any(|segment| segment.segment_id == *segment_id)
                        },
                    ),
                    template_segments,
                }
            })
            .collect(),
    })
}

pub fn register_template_motion_segment(
    database: &Database,
    workspace_root: &Path,
    request: RegisterTemplateMotionSegmentRequest,
) -> Result<VideoSegmentDto, String> {
    ensure_image_slideshow_project(database, &request.project_id)?;
    validate_template_segment_request(workspace_root, &request)?;
    let repository = SceneRepository::new(database);
    let item = repository
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Storyboard item {} does not belong to project {}.",
            request.item_id, request.project_id
        ));
    }
    if item.selected_image_id.as_deref() != Some(request.input_image_id.as_str()) {
        return Err("template motion segment must use the item's selected image.".to_string());
    }
    if !item
        .image_candidates
        .iter()
        .any(|candidate| candidate.image_id == request.input_image_id && candidate.selected)
    {
        return Err("selected image candidate was not found for template motion.".to_string());
    }

    let segment_id = create_id("slideshow_segment");
    let saved = repository.insert_video_segments(&[NewVideoSegmentRecord {
        segment_id: segment_id.clone(),
        item_id: request.item_id.clone(),
        input_image_id: request.input_image_id.clone(),
        video_path: request.video_path.clone(),
        video_prompt: format!(
            "template_motion:{}:{}",
            request.template_type, request.template_id
        ),
        duration_seconds: request.duration_seconds,
        model: "template_motion".to_string(),
        provider_model_id: "local_template_motion".to_string(),
        workflow_preset_id: request.workflow_preset_id.clone(),
        status: "succeeded".to_string(),
        selected: true,
        generation_context_snapshot: json!({
            "source": "image_slideshow",
            "renderKind": "template_motion",
            "templateId": request.template_id,
            "templateType": request.template_type,
            "externalNetwork": false,
            "billable": false,
            "videoPath": request.video_path,
            "revision": repository.latest_video_revision(&request.item_id).unwrap_or(0) + 1,
        }),
    }])?;
    repository.select_video_segment(&request.item_id, &segment_id)?;
    TaskRepository::new(database).update_latest_project_step_status(
        &request.project_id,
        "template_motion",
        "waiting_user",
        Some(json!({ "latestSegmentId": segment_id })),
    )?;

    saved
        .into_iter()
        .find(|segment| segment.segment_id == segment_id)
        .ok_or_else(|| "Template motion segment was saved but cannot be read.".to_string())
}

pub fn validate_image_slideshow_segments(
    database: &Database,
    request: ImageSlideshowProjectRequest,
) -> Result<ImageSlideshowProjectStateDto, String> {
    let state = get_image_slideshow_project_state(database, request.project_id)?;
    let missing = state
        .items
        .iter()
        .filter(|item| !item.ready_for_composition)
        .map(|item| item.item_id.clone())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "Image slideshow items require selected template motion segments: {}",
            missing.join(", ")
        ));
    }
    TaskRepository::new(database).update_latest_project_step_status(
        &state.project_id,
        "segment_composition",
        "succeeded",
        Some(json!({
            "checkedItemCount": state.items.len(),
            "allItemsReady": true
        })),
    )?;
    get_image_slideshow_project_state(database, state.project_id)
}

fn validate_template_segment_request(
    workspace_root: &Path,
    request: &RegisterTemplateMotionSegmentRequest,
) -> Result<(), String> {
    if request.duration_seconds <= 0.0 {
        return Err("duration_seconds must be greater than 0.".to_string());
    }
    if request.template_id.trim().is_empty() {
        return Err("template_id is required.".to_string());
    }
    if request.template_type.trim().is_empty() {
        return Err("template_type is required.".to_string());
    }
    let normalized = PathGuard::validate_relative_path(&request.video_path)?;
    if !normalized.starts_with("projects/") && !normalized.starts_with("outputs/") {
        return Err("template motion video_path must be under projects or outputs.".to_string());
    }
    PathGuard::new(workspace_root).safe_join_existing(&normalized)?;
    Ok(())
}

fn ensure_image_slideshow_project(database: &Database, project_id: &str) -> Result<(), String> {
    let project = ProjectRepository::new(database)
        .get_detail(project_id)?
        .ok_or_else(|| format!("Project not found: {project_id}"))?
        .project;
    if project.workflow_type != "image_slideshow" {
        return Err("image slideshow actions require workflow_type=image_slideshow.".to_string());
    }
    Ok(())
}

fn is_template_motion_segment(segment: &VideoSegmentDto) -> bool {
    segment
        .generation_context_snapshot
        .get("renderKind")
        .and_then(|value| value.as_str())
        == Some("template_motion")
}

fn step_status(steps: &[crate::domain::task::TaskStepDto], step_name: &str) -> String {
    steps
        .iter()
        .find(|step| step.step_name == step_name)
        .map(|step| step.status.clone())
        .unwrap_or_else(|| "pending".to_string())
}

fn create_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::project_repository::ProjectRepository;
    use crate::domain::project::CreateProjectRequest;
    use crate::domain::scene::SceneDto;
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn image_slideshow_project_has_independent_steps_without_video_provider() {
        let root = test_root("slideshow_steps");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_slideshow_steps", "image_slideshow");

        let task = TaskRepository::new(&database)
            .get_latest_task_detail_by_project("project_slideshow_steps")
            .expect("task should read")
            .expect("task should exist");
        assert_eq!(task.current_step.as_deref(), Some("storyboard_review"));
        assert!(task
            .steps
            .iter()
            .any(|step| step.step_name == "template_motion"));
        assert!(!task
            .steps
            .iter()
            .any(|step| step.step_name == "video_generation"));

        cleanup(root);
    }

    #[test]
    fn template_motion_segment_requires_real_workspace_video() {
        let root = test_root("slideshow_real_video_required");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace = root.join("workspace");
        create_project(&database, "project_slideshow_missing", "image_slideshow");
        insert_storyboard_item_with_selected_image(
            &database,
            "project_slideshow_missing",
            "item_slideshow_missing",
            "image_slideshow_missing",
        );

        let error = register_template_motion_segment(
            &database,
            &workspace,
            RegisterTemplateMotionSegmentRequest {
                project_id: "project_slideshow_missing".to_string(),
                item_id: "item_slideshow_missing".to_string(),
                input_image_id: "image_slideshow_missing".to_string(),
                video_path: "projects/project_slideshow_missing/slideshow/missing.mp4".to_string(),
                duration_seconds: 4.0,
                template_id: "fade".to_string(),
                template_type: "layout".to_string(),
                workflow_preset_id: None,
            },
        )
        .expect_err("missing video must not be registered");
        assert!(!error.trim().is_empty());
        let state =
            get_image_slideshow_project_state(&database, "project_slideshow_missing".to_string())
                .expect("state should read");
        assert!(state.items[0].template_segments.is_empty());

        cleanup(root);
    }

    #[test]
    fn template_motion_segment_enters_video_segments_and_composition_gate() {
        let root = test_root("slideshow_register");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace = root.join("workspace");
        let video_path = workspace.join("projects/project_slideshow/slideshow/segment_0001.mp4");
        fs::create_dir_all(video_path.parent().unwrap()).expect("video dir");
        fs::write(&video_path, "real template output").expect("video file");
        create_project(&database, "project_slideshow", "image_slideshow");
        insert_storyboard_item_with_selected_image(
            &database,
            "project_slideshow",
            "item_slideshow",
            "image_slideshow",
        );

        let segment = register_template_motion_segment(
            &database,
            &workspace,
            RegisterTemplateMotionSegmentRequest {
                project_id: "project_slideshow".to_string(),
                item_id: "item_slideshow".to_string(),
                input_image_id: "image_slideshow".to_string(),
                video_path: "projects/project_slideshow/slideshow/segment_0001.mp4".to_string(),
                duration_seconds: 4.0,
                template_id: "fade".to_string(),
                template_type: "layout".to_string(),
                workflow_preset_id: Some("preset_slideshow_fade".to_string()),
            },
        )
        .expect("segment should register");
        assert_eq!(segment.model, "template_motion");
        assert_eq!(
            segment.generation_context_snapshot["renderKind"],
            "template_motion"
        );

        let state = validate_image_slideshow_segments(
            &database,
            ImageSlideshowProjectRequest {
                project_id: "project_slideshow".to_string(),
            },
        )
        .expect("segments should validate");
        assert_eq!(state.segment_composition_status, "succeeded");
        assert!(state.items[0].ready_for_composition);

        cleanup(root);
    }

    #[test]
    fn image_slideshow_actions_reject_image_to_video_project() {
        let root = test_root("slideshow_wrong_workflow");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_image_to_video", "image_to_video");

        let error =
            get_image_slideshow_project_state(&database, "project_image_to_video".to_string())
                .expect_err("wrong workflow should reject");
        assert!(error.contains("workflow_type=image_slideshow"));

        cleanup(root);
    }

    fn create_project(database: &Database, project_id: &str, workflow_type: &str) {
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "Image slideshow".to_string(),
                    workflow_type: workflow_type.to_string(),
                    input_type: "paste".to_string(),
                    topic: None,
                    source_text: Some("文案".to_string()),
                    source_text_path: None,
                    content_language: "zh-CN".to_string(),
                    tone: None,
                    aspect_ratio: "9:16".to_string(),
                    target_scene_count: 1,
                    segment_duration_seconds: 4.0,
                    style_prompt: None,
                    active_pack_id: None,
                    rule_refs: None,
                    executable_refs: None,
                    input_process_mode: "fixed".to_string(),
                    input_options: Some(json!({})),
                },
            )
            .expect("project should create");
    }

    fn insert_storyboard_item_with_selected_image(
        database: &Database,
        project_id: &str,
        item_id: &str,
        image_id: &str,
    ) {
        let repository = SceneRepository::new(database);
        repository
            .upsert_storyboard_item(&SceneDto {
                item_id: item_id.to_string(),
                project_id: project_id.to_string(),
                index: 1,
                source_text: "文案".to_string(),
                narration_text: "文案".to_string(),
                visual_goal: "目标".to_string(),
                visual_description: "图文画面".to_string(),
                characters: vec![],
                character_ids: vec![],
                location_id: None,
                scene_description: String::new(),
                image_prompt: "prompt".to_string(),
                negative_prompt: String::new(),
                video_prompt: String::new(),
                duration_seconds: 4.0,
                subtitle_chunks: vec![],
                audio_path: None,
                audio_duration_seconds: None,
                audio_probe: None,
                selected_image_id: Some(image_id.to_string()),
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
            .insert_image_candidates(&[crate::db::scene_repository::NewImageCandidateRecord {
                image_id: image_id.to_string(),
                item_id: item_id.to_string(),
                image_path: format!("projects/{project_id}/images/{image_id}.png"),
                prompt: "prompt".to_string(),
                negative_prompt: String::new(),
                model: "mock".to_string(),
                provider_model_id: "mock_image".to_string(),
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
            "vt-ai-short-video-maker-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: impl AsRef<Path>) {
        let _ = fs::remove_dir_all(path);
    }
}
