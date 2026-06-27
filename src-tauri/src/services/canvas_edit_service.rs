use crate::db::scene_repository::{NewImageCandidateRecord, SceneRepository};
use crate::db::Database;
use crate::domain::canvas_edit::{CanvasEditCandidateResultDto, CreateCanvasEditCandidateRequest};
use crate::domain::scene::ImageCandidateDto;
use crate::security::path_guard::PathGuard;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn create_canvas_edit_candidate(
    database: &Database,
    workspace_root: &Path,
    request: CreateCanvasEditCandidateRequest,
) -> Result<CanvasEditCandidateResultDto, String> {
    validate_canvas_edit_request(workspace_root, &request)?;
    let repository = SceneRepository::new(database);
    let source = repository
        .get_image_candidate(&request.source_image_id)?
        .ok_or_else(|| {
            format!(
                "Source image candidate not found: {}",
                request.source_image_id
            )
        })?;
    let item = repository
        .get_storyboard_item(&source.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", source.item_id))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Source image candidate {} does not belong to project {}.",
            request.source_image_id, request.project_id
        ));
    }

    let revision = repository.latest_image_revision(&source.item_id)? + 1;
    let image_id = create_id("canvas_img");
    let edit_kind = request
        .edit_kind
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("manual_canvas_edit")
        .to_string();
    let prompt = request.prompt.unwrap_or_else(|| source.prompt.clone());
    let negative_prompt = request
        .negative_prompt
        .unwrap_or_else(|| source.negative_prompt.clone());
    let provider_model_id = request
        .provider_model_id
        .unwrap_or_else(|| "local_canvas_edit".to_string());
    let workflow_preset_id = request.workflow_preset_id.clone();
    let flow_snapshot = request.flow_snapshot.unwrap_or_else(|| json!({}));
    let snapshot = json!({
        "source": "canvas_edit",
        "renderKind": "canvas_edit",
        "editKind": edit_kind,
        "derivedFromImageId": source.image_id,
        "sourceImagePath": source.image_path,
        "editFlowPath": request.edit_flow_path,
        "editedImagePath": request.edited_image_path,
        "revision": revision,
        "providerModelId": provider_model_id,
        "workflowPresetId": workflow_preset_id,
        "externalNetwork": false,
        "billable": false,
        "flowSnapshot": flow_snapshot
    });

    let saved = repository.insert_image_candidates(&[NewImageCandidateRecord {
        image_id: image_id.clone(),
        item_id: source.item_id.clone(),
        image_path: request.edited_image_path.clone(),
        prompt,
        negative_prompt,
        model: "canvas_edit".to_string(),
        provider_model_id,
        workflow_preset_id,
        status: "succeeded".to_string(),
        selected: false,
        derived_from_image_id: Some(source.image_id.clone()),
        generation_context_snapshot: snapshot,
    }])?;
    let candidate = find_saved_candidate(saved, &image_id)?;
    let selected_item_id = if request.select_after_create.unwrap_or(false) {
        repository.select_image_candidate(&source.item_id, &image_id)?;
        Some(source.item_id.clone())
    } else {
        None
    };

    Ok(CanvasEditCandidateResultDto {
        candidate,
        selected_item_id,
    })
}

fn validate_canvas_edit_request(
    workspace_root: &Path,
    request: &CreateCanvasEditCandidateRequest,
) -> Result<(), String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for canvas edit.".to_string());
    }
    if request.source_image_id.trim().is_empty() {
        return Err("source_image_id is required for canvas edit.".to_string());
    }
    validate_existing_workspace_file(workspace_root, &request.edited_image_path, "edited image")?;
    validate_existing_workspace_file(workspace_root, &request.edit_flow_path, "edit flow")?;
    let flow_path = PathGuard::new(workspace_root)
        .safe_join_existing(&PathGuard::validate_relative_path(&request.edit_flow_path)?)?
        .absolute_path()
        .to_path_buf();
    let flow_text = fs::read_to_string(flow_path).map_err(|error| error.to_string())?;
    let parsed: Value = serde_json::from_str(&flow_text)
        .map_err(|error| format!("edit_flow_path must point to a valid JSON file: {error}"))?;
    if !parsed.is_object() && !parsed.is_array() {
        return Err("edit_flow_path JSON must be an object or array.".to_string());
    }
    Ok(())
}

fn validate_existing_workspace_file(
    workspace_root: &Path,
    relative_path: &str,
    label: &str,
) -> Result<(), String> {
    let normalized = PathGuard::validate_relative_path(relative_path)?;
    if !normalized.starts_with("projects/") && !normalized.starts_with("outputs/") {
        return Err(format!("{label} path must be under projects or outputs."));
    }
    let absolute = PathGuard::new(workspace_root)
        .safe_join_existing(&normalized)?
        .absolute_path()
        .to_path_buf();
    if !absolute.is_file() {
        return Err(format!("{label} path must point to an existing file."));
    }
    Ok(())
}

fn find_saved_candidate(
    candidates: Vec<ImageCandidateDto>,
    image_id: &str,
) -> Result<ImageCandidateDto, String> {
    candidates
        .into_iter()
        .find(|candidate| candidate.image_id == image_id)
        .ok_or_else(|| "Canvas edit candidate was saved but cannot be read.".to_string())
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
    use std::path::PathBuf;

    #[test]
    fn canvas_edit_result_is_new_candidate_and_keeps_source() {
        let root = test_root("canvas_edit_new_candidate");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace = root.join("workspace");
        let (item_id, source_image_id) =
            seed_project_with_source_image(&database, &workspace, "project_canvas");
        write_workspace_file(
            &workspace,
            "projects/project_canvas/images/item_canvas/edited.png",
            b"edited image",
        );
        write_workspace_file(
            &workspace,
            "projects/project_canvas/edit_flows/item_canvas/edit.json",
            br#"{"nodes":[],"edges":[]}"#,
        );

        let result = create_canvas_edit_candidate(
            &database,
            &workspace,
            CreateCanvasEditCandidateRequest {
                project_id: "project_canvas".to_string(),
                source_image_id: source_image_id.clone(),
                edited_image_path: "projects/project_canvas/images/item_canvas/edited.png"
                    .to_string(),
                edit_flow_path: "projects/project_canvas/edit_flows/item_canvas/edit.json"
                    .to_string(),
                prompt: Some("局部修复后的画面".to_string()),
                negative_prompt: None,
                provider_model_id: None,
                workflow_preset_id: None,
                edit_kind: Some("inpaint".to_string()),
                flow_snapshot: None,
                select_after_create: None,
            },
        )
        .expect("canvas edit candidate should save");

        assert_ne!(result.candidate.image_id, source_image_id);
        assert_eq!(
            result.candidate.derived_from_image_id.as_deref(),
            Some(source_image_id.as_str())
        );
        assert_eq!(
            result.candidate.generation_context_snapshot["editFlowPath"],
            "projects/project_canvas/edit_flows/item_canvas/edit.json"
        );
        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(item.image_candidates.len(), 2);
        assert!(item
            .image_candidates
            .iter()
            .any(|candidate| candidate.image_id == source_image_id));

        cleanup(root);
    }

    #[test]
    fn canvas_edit_rejects_missing_image_or_flow_json() {
        let root = test_root("canvas_edit_requires_real_files");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace = root.join("workspace");
        let (_, source_image_id) =
            seed_project_with_source_image(&database, &workspace, "project_canvas_missing");
        write_workspace_file(
            &workspace,
            "projects/project_canvas_missing/images/item_canvas/source.png",
            b"source image",
        );

        let error = create_canvas_edit_candidate(
            &database,
            &workspace,
            CreateCanvasEditCandidateRequest {
                project_id: "project_canvas_missing".to_string(),
                source_image_id,
                edited_image_path: "projects/project_canvas_missing/images/item_canvas/missing.png"
                    .to_string(),
                edit_flow_path:
                    "projects/project_canvas_missing/edit_flows/item_canvas/missing.json"
                        .to_string(),
                prompt: None,
                negative_prompt: None,
                provider_model_id: None,
                workflow_preset_id: None,
                edit_kind: None,
                flow_snapshot: None,
                select_after_create: None,
            },
        )
        .expect_err("missing files must reject");
        assert!(!error.trim().is_empty());

        cleanup(root);
    }

    #[test]
    fn selecting_canvas_edit_candidate_resets_video_and_composition_state() {
        let root = test_root("canvas_edit_select_resets");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace = root.join("workspace");
        let (item_id, source_image_id) =
            seed_project_with_source_image(&database, &workspace, "project_canvas_select");
        let repository = SceneRepository::new(&database);
        repository
            .insert_video_segments(&[crate::db::scene_repository::NewVideoSegmentRecord {
                segment_id: "segment_before_edit".to_string(),
                item_id: item_id.clone(),
                input_image_id: source_image_id.clone(),
                video_path: "projects/project_canvas_select/videos/segment_before_edit.mp4"
                    .to_string(),
                video_prompt: "move".to_string(),
                duration_seconds: 4.0,
                model: "video".to_string(),
                provider_model_id: "video_model".to_string(),
                workflow_preset_id: None,
                status: "succeeded".to_string(),
                selected: true,
                generation_context_snapshot: json!({ "revision": 1 }),
            }])
            .expect("video segment should save");
        repository
            .select_video_segment(&item_id, "segment_before_edit")
            .expect("video should select");
        write_workspace_file(
            &workspace,
            "projects/project_canvas_select/images/item_canvas/edited.png",
            b"edited image",
        );
        write_workspace_file(
            &workspace,
            "projects/project_canvas_select/edit_flows/item_canvas/edit.json",
            br#"{"nodes":[]}"#,
        );

        let result = create_canvas_edit_candidate(
            &database,
            &workspace,
            CreateCanvasEditCandidateRequest {
                project_id: "project_canvas_select".to_string(),
                source_image_id,
                edited_image_path: "projects/project_canvas_select/images/item_canvas/edited.png"
                    .to_string(),
                edit_flow_path: "projects/project_canvas_select/edit_flows/item_canvas/edit.json"
                    .to_string(),
                prompt: None,
                negative_prompt: None,
                provider_model_id: None,
                workflow_preset_id: None,
                edit_kind: None,
                flow_snapshot: None,
                select_after_create: Some(true),
            },
        )
        .expect("canvas edit candidate should save and select");
        assert_eq!(result.selected_item_id.as_deref(), Some(item_id.as_str()));

        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(
            item.selected_image_id.as_deref(),
            Some(result.candidate.image_id.as_str())
        );
        assert!(item.selected_video_segment_id.is_none());
        assert_eq!(item.video_status, "pending");
        assert_eq!(item.segment_status, "pending");
        assert_eq!(item.render_status, "pending");
        assert!(item.video_segments.iter().all(|segment| !segment.selected));

        cleanup(root);
    }

    fn seed_project_with_source_image(
        database: &Database,
        workspace: &Path,
        project_id: &str,
    ) -> (String, String) {
        fs::create_dir_all(workspace.join("projects")).expect("projects dir should exist");
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "Canvas edit".to_string(),
                    workflow_type: "image_to_video".to_string(),
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
        let item_id = "item_canvas".to_string();
        let source_image_id = "image_canvas_source".to_string();
        let repository = SceneRepository::new(database);
        repository
            .upsert_storyboard_item(&SceneDto {
                item_id: item_id.clone(),
                project_id: project_id.to_string(),
                index: 1,
                source_text: "文案".to_string(),
                narration_text: "文案".to_string(),
                visual_goal: "目标".to_string(),
                visual_description: "画面".to_string(),
                characters: vec![],
                character_ids: vec![],
                location_id: None,
                scene_description: String::new(),
                image_prompt: "prompt".to_string(),
                negative_prompt: String::new(),
                video_prompt: "move".to_string(),
                duration_seconds: 4.0,
                subtitle_chunks: vec![],
                audio_path: None,
                audio_duration_seconds: None,
                audio_probe: None,
                selected_image_id: Some(source_image_id.clone()),
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
            .expect("item should save");
        write_workspace_file(
            workspace,
            &format!("projects/{project_id}/images/item_canvas/source.png"),
            b"source image",
        );
        repository
            .insert_image_candidates(&[NewImageCandidateRecord {
                image_id: source_image_id.clone(),
                item_id: item_id.clone(),
                image_path: format!("projects/{project_id}/images/item_canvas/source.png"),
                prompt: "prompt".to_string(),
                negative_prompt: String::new(),
                model: "source".to_string(),
                provider_model_id: "source_model".to_string(),
                workflow_preset_id: None,
                status: "succeeded".to_string(),
                selected: true,
                derived_from_image_id: None,
                generation_context_snapshot: json!({ "revision": 1 }),
            }])
            .expect("source candidate should save");
        (item_id, source_image_id)
    }

    fn write_workspace_file(workspace: &Path, relative_path: &str, content: &[u8]) {
        let path = workspace.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).expect("parent should exist");
        fs::write(path, content).expect("workspace file should write");
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
