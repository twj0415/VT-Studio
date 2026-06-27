use crate::db::asset_repository::AssetRepository;
use crate::db::material_edit_repository::{
    MaterialEditRepository, NewMaterialAnalysisSuggestionRecord,
    NewStoryboardMaterialReferenceRecord, NewStoryboardMaterialRequirementRecord,
};
use crate::db::project_repository::ProjectRepository;
use crate::db::scene_repository::SceneRepository;
use crate::db::task_repository::TaskRepository;
use crate::db::Database;
use crate::domain::material_edit::{
    BindStoryboardMaterialRequest, MarkStoryboardNoMaterialRequest, MaterialAnalysisSuggestionDto,
    MaterialAnalysisSuggestionIdRequest, MaterialEditProjectRequest, MaterialEditProjectStateDto,
    SaveMaterialAnalysisSuggestionRequest, StoryboardMaterialCoverageDto,
    StoryboardMaterialRequirementDto,
};
use crate::domain::media::AssetReferenceDto;
use crate::domain::structured_output::ValidateStructuredOutputRequest;
use crate::services::structured_output_service::validate_structured_output;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn get_material_edit_project_state(
    database: &Database,
    project_id: String,
) -> Result<MaterialEditProjectStateDto, String> {
    ensure_material_edit_project(database, &project_id)?;
    let task = TaskRepository::new(database)
        .get_latest_task_detail_by_project(&project_id)?
        .ok_or_else(|| format!("Task not found for project: {project_id}"))?;
    let repository = MaterialEditRepository::new(database);

    Ok(MaterialEditProjectStateDto {
        project_id: project_id.clone(),
        import_status: step_status(&task.steps, "material_import"),
        analysis_status: step_status(&task.steps, "material_analysis"),
        matching_status: step_status(&task.steps, "material_matching"),
        suggestions: repository.list_analysis_suggestions(&project_id)?,
        coverage: build_material_coverage(database, &project_id)?,
    })
}

pub fn save_material_analysis_suggestion(
    database: &Database,
    request: SaveMaterialAnalysisSuggestionRequest,
) -> Result<MaterialAnalysisSuggestionDto, String> {
    ensure_material_edit_project(database, &request.project_id)?;
    ensure_asset_exists(database, &request.asset_id)?;

    let validation = validate_structured_output(ValidateStructuredOutputRequest {
        raw_output: request.raw_output,
        output_schema: material_analysis_schema(),
        expected_count: None,
        repair_attempt_count: Some(0),
        max_repair_attempts: Some(0),
    })?;
    if !validation.valid {
        return Err(format!(
            "material analysis schema invalid: {}",
            validation.errors.join("; ")
        ));
    }
    let suggestion = validation
        .parsed_json
        .ok_or_else(|| "material analysis schema invalid: parsed JSON is missing.".to_string())?;

    let saved = MaterialEditRepository::new(database).insert_analysis_suggestion(
        NewMaterialAnalysisSuggestionRecord::new(
            request.project_id,
            request.asset_id,
            request.provider_id,
            request.model_id,
            suggestion,
        ),
    )?;
    TaskRepository::new(database).update_latest_project_step_status(
        &saved.project_id,
        "material_analysis",
        "waiting_user",
        Some(json!({
            "latestSuggestionId": saved.suggestion_id,
            "note": "VLM analysis is a suggestion and requires user confirmation."
        })),
    )?;
    Ok(saved)
}

pub fn approve_material_analysis_suggestion(
    database: &Database,
    request: MaterialAnalysisSuggestionIdRequest,
) -> Result<MaterialAnalysisSuggestionDto, String> {
    MaterialEditRepository::new(database)
        .set_analysis_suggestion_status(&request.suggestion_id, "approved")
}

pub fn reject_material_analysis_suggestion(
    database: &Database,
    request: MaterialAnalysisSuggestionIdRequest,
) -> Result<MaterialAnalysisSuggestionDto, String> {
    MaterialEditRepository::new(database)
        .set_analysis_suggestion_status(&request.suggestion_id, "rejected")
}

pub fn bind_storyboard_material(
    database: &Database,
    request: BindStoryboardMaterialRequest,
) -> Result<StoryboardMaterialCoverageDto, String> {
    ensure_material_edit_project(database, &request.project_id)?;
    ensure_asset_exists(database, &request.asset_id)?;
    ensure_storyboard_item_in_project(database, &request.project_id, &request.item_id)?;

    MaterialEditRepository::new(database).create_storyboard_material_reference(
        NewStoryboardMaterialReferenceRecord::new(request.item_id.clone(), request.asset_id),
    )?;
    let requirement = MaterialEditRepository::new(database)
        .upsert_storyboard_material_requirement(NewStoryboardMaterialRequirementRecord {
            item_id: request.item_id.clone(),
            project_id: request.project_id.clone(),
            requirement_status: "needs_material".to_string(),
            no_material_reason: None,
            confirmed_by_user: true,
        })?;
    let coverage = build_material_coverage(database, &request.project_id)?
        .into_iter()
        .find(|coverage| coverage.item_id == request.item_id)
        .ok_or_else(|| "Storyboard material coverage cannot be read after binding.".to_string())?;
    if coverage.requirement.is_none() {
        return Ok(StoryboardMaterialCoverageDto {
            requirement: Some(requirement),
            ..coverage
        });
    }
    Ok(coverage)
}

pub fn mark_storyboard_no_material_needed(
    database: &Database,
    request: MarkStoryboardNoMaterialRequest,
) -> Result<StoryboardMaterialCoverageDto, String> {
    ensure_material_edit_project(database, &request.project_id)?;
    ensure_storyboard_item_in_project(database, &request.project_id, &request.item_id)?;
    let reason = request.reason.trim();
    if reason.is_empty() {
        return Err("no material reason is required.".to_string());
    }

    MaterialEditRepository::new(database).upsert_storyboard_material_requirement(
        NewStoryboardMaterialRequirementRecord {
            item_id: request.item_id.clone(),
            project_id: request.project_id.clone(),
            requirement_status: "no_material_needed".to_string(),
            no_material_reason: Some(reason.to_string()),
            confirmed_by_user: true,
        },
    )?;
    build_material_coverage(database, &request.project_id)?
        .into_iter()
        .find(|coverage| coverage.item_id == request.item_id)
        .ok_or_else(|| "Storyboard material coverage cannot be read after update.".to_string())
}

pub fn validate_material_storyboard_coverage(
    database: &Database,
    request: MaterialEditProjectRequest,
) -> Result<MaterialEditProjectStateDto, String> {
    let state = get_material_edit_project_state(database, request.project_id)?;
    let missing = state
        .coverage
        .iter()
        .filter(|coverage| !coverage.satisfied)
        .map(|coverage| coverage.item_id.clone())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "Storyboard items require source material or no-material confirmation: {}",
            missing.join(", ")
        ));
    }
    TaskRepository::new(database).update_latest_project_step_status(
        &state.project_id,
        "material_matching",
        "succeeded",
        Some(json!({
            "checkedItemCount": state.coverage.len(),
            "allItemsSatisfied": true
        })),
    )?;
    get_material_edit_project_state(database, state.project_id)
}

fn build_material_coverage(
    database: &Database,
    project_id: &str,
) -> Result<Vec<StoryboardMaterialCoverageDto>, String> {
    let storyboard = SceneRepository::new(database)
        .get_storyboard(project_id)?
        .ok_or_else(|| format!("Storyboard not found for project: {project_id}"))?;
    let references =
        MaterialEditRepository::new(database).list_storyboard_material_references(project_id)?;
    let requirements =
        MaterialEditRepository::new(database).list_storyboard_material_requirements(project_id)?;
    let mut references_by_item: HashMap<String, Vec<AssetReferenceDto>> = HashMap::new();
    for reference in references {
        references_by_item
            .entry(reference.owner_id.clone())
            .or_default()
            .push(reference);
    }
    let requirements_by_item = requirements
        .into_iter()
        .map(|requirement| (requirement.item_id.clone(), requirement))
        .collect::<HashMap<_, _>>();

    Ok(storyboard
        .items
        .into_iter()
        .map(|item| {
            let bound_assets = references_by_item.remove(&item.item_id).unwrap_or_default();
            let requirement = requirements_by_item.get(&item.item_id).cloned();
            let no_material_confirmed = requirement
                .as_ref()
                .map(is_no_material_requirement_confirmed)
                .unwrap_or(false);
            StoryboardMaterialCoverageDto {
                item_id: item.item_id,
                project_id: item.project_id,
                satisfied: !bound_assets.is_empty() || no_material_confirmed,
                bound_assets,
                requirement,
            }
        })
        .collect())
}

fn ensure_material_edit_project(database: &Database, project_id: &str) -> Result<(), String> {
    let project = ProjectRepository::new(database)
        .get_detail(project_id)?
        .ok_or_else(|| format!("Project not found: {project_id}"))?
        .project;
    if project.workflow_type != "material_edit" {
        return Err("material edit actions require workflow_type=material_edit.".to_string());
    }
    Ok(())
}

fn ensure_asset_exists(database: &Database, asset_id: &str) -> Result<(), String> {
    if asset_id.trim().is_empty() {
        return Err("asset_id is required.".to_string());
    }
    AssetRepository::new(database)
        .get_asset(asset_id)?
        .ok_or_else(|| format!("Asset not found: {asset_id}"))?;
    Ok(())
}

fn ensure_storyboard_item_in_project(
    database: &Database,
    project_id: &str,
    item_id: &str,
) -> Result<(), String> {
    let item = SceneRepository::new(database)
        .get_storyboard_item(item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {item_id}"))?;
    if item.project_id != project_id {
        return Err(format!(
            "Storyboard item {item_id} does not belong to project {project_id}."
        ));
    }
    Ok(())
}

fn is_no_material_requirement_confirmed(requirement: &StoryboardMaterialRequirementDto) -> bool {
    requirement.requirement_status == "no_material_needed" && requirement.confirmed_by_user
}

fn step_status(steps: &[crate::domain::task::TaskStepDto], step_name: &str) -> String {
    steps
        .iter()
        .find(|step| step.step_name == step_name)
        .map(|step| step.status.clone())
        .unwrap_or_else(|| "pending".to_string())
}

fn material_analysis_schema() -> Value {
    json!({
        "type": "object",
        "required": ["summary", "tags", "shots"],
        "properties": {
            "summary": { "type": "string" },
            "tags": {
                "type": "array",
                "items": { "type": "string" }
            },
            "durationSeconds": { "type": "number" },
            "shots": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "required": ["description"],
                    "properties": {
                        "description": { "type": "string" },
                        "startSeconds": { "type": "number" },
                        "endSeconds": { "type": "number" },
                        "characters": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "location": { "type": "string" },
                        "confidence": { "type": "number" }
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::asset_repository::{AssetRepository, NewAssetRecord};
    use crate::db::project_repository::ProjectRepository;
    use crate::domain::project::CreateProjectRequest;
    use crate::domain::scene::SceneDto;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn material_edit_project_has_independent_steps() {
        let root = test_root("material_steps");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_material_steps", "material_edit");

        let task = TaskRepository::new(&database)
            .get_latest_task_detail_by_project("project_material_steps")
            .expect("task should read")
            .expect("task should exist");
        assert_eq!(task.current_step.as_deref(), Some("material_import"));
        assert!(task
            .steps
            .iter()
            .any(|step| step.step_name == "material_matching"));
        assert!(!task
            .steps
            .iter()
            .any(|step| step.step_name == "image_generation"));

        cleanup(root);
    }

    #[test]
    fn storyboard_material_binding_writes_asset_reference_and_satisfies_coverage() {
        let root = test_root("material_binding");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_material_bind", "material_edit");
        insert_storyboard_item(&database, "project_material_bind", "item_material_bind");
        insert_asset(&database, "asset_material_bind");

        let before =
            get_material_edit_project_state(&database, "project_material_bind".to_string())
                .expect("state should read");
        assert_eq!(before.coverage.len(), 1);
        assert!(!before.coverage[0].satisfied);

        let coverage = bind_storyboard_material(
            &database,
            BindStoryboardMaterialRequest {
                project_id: "project_material_bind".to_string(),
                item_id: "item_material_bind".to_string(),
                asset_id: "asset_material_bind".to_string(),
            },
        )
        .expect("material should bind");
        assert!(coverage.satisfied);
        assert_eq!(coverage.bound_assets.len(), 1);
        assert_eq!(coverage.bound_assets[0].owner_kind, "storyboard_item");
        assert_eq!(coverage.bound_assets[0].usage_kind, "source_material");

        cleanup(root);
    }

    #[test]
    fn every_storyboard_item_requires_material_or_no_material_confirmation() {
        let root = test_root("material_coverage");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_material_coverage", "material_edit");
        insert_storyboard_item(
            &database,
            "project_material_coverage",
            "item_material_missing",
        );

        let error = validate_material_storyboard_coverage(
            &database,
            MaterialEditProjectRequest {
                project_id: "project_material_coverage".to_string(),
            },
        )
        .expect_err("missing material should block");
        assert!(error.contains("require source material"));

        mark_storyboard_no_material_needed(
            &database,
            MarkStoryboardNoMaterialRequest {
                project_id: "project_material_coverage".to_string(),
                item_id: "item_material_missing".to_string(),
                reason: "旁白过渡，无需画面素材".to_string(),
            },
        )
        .expect("no material marker should save");
        let state = validate_material_storyboard_coverage(
            &database,
            MaterialEditProjectRequest {
                project_id: "project_material_coverage".to_string(),
            },
        )
        .expect("coverage should pass");
        assert_eq!(state.matching_status, "succeeded");

        cleanup(root);
    }

    #[test]
    fn vlm_suggestion_waits_for_user_and_does_not_modify_storyboard() {
        let root = test_root("material_vlm_suggestion");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_material_vlm", "material_edit");
        insert_storyboard_item(&database, "project_material_vlm", "item_material_vlm");
        insert_asset(&database, "asset_material_vlm");

        let suggestion = save_material_analysis_suggestion(
            &database,
            SaveMaterialAnalysisSuggestionRequest {
                project_id: "project_material_vlm".to_string(),
                asset_id: "asset_material_vlm".to_string(),
                provider_id: Some("provider_vlm".to_string()),
                model_id: Some("model_vlm".to_string()),
                raw_output: json!({
                    "summary": "一段室内素材",
                    "tags": ["室内", "人物"],
                    "shots": [{ "description": "人物坐在桌前" }]
                })
                .to_string(),
            },
        )
        .expect("suggestion should save");
        assert_eq!(suggestion.status, "waiting_user");

        let item = SceneRepository::new(&database)
            .get_storyboard_item("item_material_vlm")
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(item.visual_description, "原始画面描述");
        assert!(item.image_prompt.is_empty());

        cleanup(root);
    }

    #[test]
    fn material_edit_actions_reject_image_to_video_project() {
        let root = test_root("material_wrong_workflow");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_image_to_video", "image_to_video");

        let error =
            get_material_edit_project_state(&database, "project_image_to_video".to_string())
                .expect_err("wrong workflow should be rejected");
        assert!(error.contains("workflow_type=material_edit"));

        cleanup(root);
    }

    fn create_project(database: &Database, project_id: &str, workflow_type: &str) {
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "Material edit".to_string(),
                    workflow_type: workflow_type.to_string(),
                    input_type: "material".to_string(),
                    topic: None,
                    source_text: None,
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
                    input_process_mode: "generate".to_string(),
                    input_options: Some(json!({})),
                },
            )
            .expect("project should create");
    }

    fn insert_asset(database: &Database, asset_id: &str) {
        AssetRepository::new(database)
            .insert_asset(&NewAssetRecord {
                asset_id: asset_id.to_string(),
                kind: "source_material".to_string(),
                relative_path: format!("assets/source_material/{asset_id}.mp4"),
                source_kind: "upload".to_string(),
                mime_type: Some("video/mp4".to_string()),
                size_bytes: Some(1024),
                checksum: None,
                is_builtin: false,
                metadata: json!({}),
            })
            .expect("asset should insert");
    }

    fn insert_storyboard_item(database: &Database, project_id: &str, item_id: &str) {
        SceneRepository::new(database)
            .upsert_storyboard_item(&SceneDto {
                item_id: item_id.to_string(),
                project_id: project_id.to_string(),
                index: 1,
                source_text: "文案".to_string(),
                narration_text: "文案".to_string(),
                visual_goal: "目标".to_string(),
                visual_description: "原始画面描述".to_string(),
                characters: vec![],
                character_ids: vec![],
                location_id: None,
                scene_description: String::new(),
                image_prompt: String::new(),
                negative_prompt: String::new(),
                video_prompt: String::new(),
                duration_seconds: 4.0,
                subtitle_chunks: vec![],
                audio_path: None,
                audio_duration_seconds: None,
                audio_probe: None,
                selected_image_id: None,
                selected_video_segment_id: None,
                status: "pending".to_string(),
                lock_flags_json: json!({}),
                shot_size: None,
                camera_motion: None,
                composition: None,
                pace: None,
                transition_type: None,
                image_status: "pending".to_string(),
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
            .expect("storyboard item should insert");
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
