use crate::core::error::TaskError;
use crate::db::asset_repository::{
    AssetRepository, NewAssetRecord, NewAssetReferenceRecord, NewGeneratedImageAssetRecord,
};
use crate::db::character_repository::CharacterRepository;
use crate::db::provider_repository::{ProviderModelRecord, ProviderRecord, ProviderRepository};
use crate::db::scene_repository::{
    NewImageCandidateRecord, NewVideoSegmentRecord, SceneRepository,
};
use crate::db::task_repository::{StepSuccessRecord, TaskArtifactRecord, TaskRepository};
use crate::db::Database;
use crate::domain::media::ExecutableMediaOptionDto;
use crate::domain::provider::{
    ImageProviderRequest, ProviderMediaInputDto, ProviderRequestContext, TtsProviderRequest,
    VideoProviderRequest, WorkflowProviderRequest,
};
use crate::domain::scene::{
    ApplyScriptDraftRequest, BuildCharacterResourcePlanRequest, CharacterResourcePlanDto,
    CharacterResourceRequirementDto, ClearHistoricalImageCandidatesRequest,
    ClearHistoricalVideoSegmentsRequest, GenerateImagePromptsRequest, GenerateSubtitlesRequest,
    GenerateSubtitlesResultDto, GeneratedImageAssetDto, ImageCandidateDto,
    ProbeStoryboardAudioRequest, ReplaceStoryboardAudioRequest, SceneDto, ScriptDraftNarrationDto,
    SelectImageCandidateRequest, SelectVideoSegmentRequest, StartImageAssetGenerationRequest,
    StartImageGenerationRequest, StartTtsGenerationRequest, StartVideoGenerationRequest,
    StoryboardDto, SubtitleChunkDto, SubtitleStyleDto, SubtitleTimelineChunkDto,
    SubtitleWordTimingDto, SubtitlesFileDto, UpdateStoryboardSubtitlesRequest, VideoSegmentDto,
};
use crate::domain::structured_output::ValidateStructuredOutputRequest;
use crate::domain::style::ImagePromptPreviewDto;
use crate::security::secret_guard::redact_text;
use crate::services::character_service::{
    character_names_for_ids, validate_storyboard_character_ids,
};
use crate::services::ffmpeg_service;
use crate::services::keyring_service::KeyringService;
use crate::services::location_service::{
    location_scene_description_for_id, validate_storyboard_location_id,
};
use crate::services::media_service::list_executable_media_options;
use crate::services::prompt_service::project_rule_snapshot;
use crate::services::provider_service::ProviderManager;
use crate::services::storage_service::{FileAccessPolicy, FileBucket, StorageService};
use crate::services::structured_output_service::validate_structured_output;
use crate::services::style_service::build_image_prompt_preview_for_item;
use crate::services::task_cancellation::CancellationToken;
use rusqlite::OptionalExtension;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const IMAGE_GENERATION_STEP: &str = "image_generation";
const VIDEO_GENERATION_STEP: &str = "video_generation";
const GENERATION_CONTEXT_SCHEMA_VERSION: u32 = 1;
const CONTROLLED_FAKE_PROVIDER_ID: &str = "provider_controlled_fake_image";
const CONTROLLED_FAKE_MODEL_ID: &str = "model_controlled_fake_t2i";
const CONTROLLED_FAKE_PROVIDER_MODEL_ID: &str = "controlled-fake/text-to-image-v1";
const CONTROLLED_FAKE_VIDEO_PROVIDER_ID: &str = "provider_controlled_fake_video";
const CONTROLLED_FAKE_VIDEO_MODEL_ID: &str = "model_controlled_fake_i2v";
const CONTROLLED_FAKE_VIDEO_PROVIDER_MODEL_ID: &str = "controlled-fake/image-to-video-v1";
const CONTROLLED_FAKE_TTS_PROVIDER_ID: &str = "provider_controlled_fake_tts";
const CONTROLLED_FAKE_TTS_MODEL_ID: &str = "model_controlled_fake_tts";
const CONTROLLED_FAKE_TTS_PROVIDER_MODEL_ID: &str = "controlled-fake/tts-v1";
const SUBTITLE_SCHEMA_VERSION: u32 = 1;
const SUBTITLE_TARGET_MIN_CHARS: usize = 12;
const SUBTITLE_TARGET_MAX_CHARS: usize = 18;
const CHARACTER_REFERENCE_ROLES: &[&str] = &[
    "character_front_view",
    "character_side_view",
    "character_back_view",
    "character_full_body",
    "character_face_closeup",
    "character_expression_sheet",
    "character_outfit",
    "character_pose",
    "character_mood",
];
const LOCATION_REFERENCE_ROLES: &[&str] = &[
    "scene_reference",
    "scene_wide_view",
    "scene_layout_view",
    "scene_detail_view",
    "scene_day_variant",
    "scene_night_variant",
];
const STORYBOARD_ASSET_REFERENCE_ROLES: &[&str] = &[
    "end_frame",
    "control_image",
    "pose_reference",
    "depth_reference",
    "mask_reference",
];

#[derive(Debug, Clone)]
struct MatchedCharacterReference {
    asset_id: Option<String>,
    relative_path: String,
}

pub fn get_storyboard(database: &Database, project_id: String) -> Result<StoryboardDto, String> {
    if let Some(storyboard) = SceneRepository::new(database).get_storyboard(&project_id)? {
        if !storyboard.items.is_empty() {
            return Ok(storyboard);
        }
    }

    Ok(default_storyboard(project_id))
}

pub fn update_storyboard_item(database: &Database, item: SceneDto) -> Result<SceneDto, String> {
    let item = normalize_storyboard_bible_refs(database, item)?;
    SceneRepository::new(database).upsert_storyboard_item(&item)
}

pub fn batch_update_storyboard_items(
    database: &Database,
    items: Vec<SceneDto>,
) -> Result<Vec<SceneDto>, String> {
    let mut normalized = Vec::with_capacity(items.len());
    for item in items {
        normalized.push(normalize_storyboard_bible_refs(database, item)?);
    }
    SceneRepository::new(database).upsert_storyboard_items(normalized)
}

pub fn apply_script_draft(
    database: &Database,
    request: ApplyScriptDraftRequest,
) -> Result<StoryboardDto, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for script draft.".to_string());
    }

    let validation = validate_structured_output(ValidateStructuredOutputRequest {
        raw_output: request.raw_output,
        output_schema: script_draft_output_schema(),
        expected_count: request.expected_count,
        repair_attempt_count: Some(0),
        max_repair_attempts: Some(0),
    })?;
    if !validation.valid {
        return Err(format!(
            "script draft schema invalid: {}",
            validation.errors.join("; ")
        ));
    }

    let parsed_json = validation
        .parsed_json
        .ok_or_else(|| "script draft schema invalid: parsed JSON is missing.".to_string())?;
    let draft_items = parse_script_draft_narrations(&parsed_json)?;
    if draft_items.is_empty() {
        return Err("script draft must contain at least one narration.".to_string());
    }

    let mut storyboard = get_storyboard(database, request.project_id.clone())?;
    let mut next_items = Vec::with_capacity(draft_items.len());
    for draft in draft_items {
        let existing = storyboard
            .items
            .iter()
            .find(|item| item.index == draft.index)
            .cloned();
        next_items.push(merge_script_draft_item(
            request.project_id.clone(),
            draft,
            existing,
        ));
    }

    storyboard.items = batch_update_storyboard_items(database, next_items)?;
    storyboard.confirmed_narrations = storyboard
        .items
        .iter()
        .map(storyboard_item_to_narration)
        .collect();
    storyboard.review_status = "waiting_user".to_string();
    Ok(storyboard)
}

pub fn reorder_storyboard_items(
    database: &Database,
    items: Vec<SceneDto>,
) -> Result<Vec<SceneDto>, String> {
    let reordered = items
        .into_iter()
        .enumerate()
        .map(|(index, mut item)| {
            item.index = (index + 1) as u32;
            item
        })
        .collect::<Vec<_>>();
    batch_update_storyboard_items(database, reordered)
}

pub fn generate_image_prompts(
    database: &Database,
    request: GenerateImagePromptsRequest,
) -> Result<Vec<SceneDto>, String> {
    let items = get_storyboard(database, request.project_id)?.items;
    validate_items_for_image_generation(&items)?;
    if let Some(item_ids) = request.item_ids {
        return Ok(items
            .into_iter()
            .filter(|item| item_ids.contains(&item.item_id))
            .collect());
    }
    Ok(items)
}

pub fn build_character_resource_plan(
    database: &Database,
    request: BuildCharacterResourcePlanRequest,
) -> Result<CharacterResourcePlanDto, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for character resource plan.".to_string());
    }
    if request.item_id.trim().is_empty() {
        return Err("item_id is required for character resource plan.".to_string());
    }
    if request.provider_model_id.is_some() && request.workflow_preset_id.is_some() {
        return Err("provider_model_id and workflow_preset_id cannot both be set.".to_string());
    }

    let item = SceneRepository::new(database)
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Storyboard item {} does not belong to project {}.",
            request.item_id, request.project_id
        ));
    }

    ensure_controlled_fake_image_provider(database)?;
    let option = resolve_image_generation_option(
        database,
        &StartImageGenerationRequest {
            project_id: request.project_id.clone(),
            item_id: request.item_id.clone(),
            count: Some(1),
            image_kind: Some("storyboard_image".to_string()),
            asset_kind: Some("generated_output".to_string()),
            provider_model_id: request.provider_model_id.clone(),
            workflow_preset_id: request.workflow_preset_id.clone(),
            workflow_params: None,
            aspect_ratio: None,
            width: None,
            height: None,
            seed: None,
        },
    )?;
    let characters = load_storyboard_characters(database, &item)?;
    let max_reference_images = option_max_reference_images(&option);
    let mut items = Vec::new();

    for character in characters {
        for role in CHARACTER_REFERENCE_ROLES {
            let mut requirement =
                input_plan_requirement_for_role(&option, role).unwrap_or_else(|| {
                    if *role == "character_front_view" {
                        "optional".to_string()
                    } else {
                        "unused".to_string()
                    }
                });
            if max_reference_images == Some(0) {
                requirement = "unused".to_string();
            }
            let matched = find_character_reference(&character, role);
            let available = matched.is_some();
            let missing_reason = if requirement == "required" && !available {
                Some(format!(
                    "character_resource.required_missing:{}:{}",
                    character.character_id, role
                ))
            } else {
                None
            };
            items.push(CharacterResourceRequirementDto {
                character_id: character.character_id.clone(),
                character_name: character.name.clone(),
                role: role.to_string(),
                requirement,
                available,
                asset_id: matched.as_ref().and_then(|value| value.asset_id.clone()),
                relative_path: matched.as_ref().map(|value| value.relative_path.clone()),
                missing_reason,
                source_options: vec![
                    "upload".to_string(),
                    "select_existing".to_string(),
                    "generate".to_string(),
                ],
            });
        }
    }

    let required_count = items
        .iter()
        .filter(|item| item.requirement == "required")
        .count();
    let optional_count = items
        .iter()
        .filter(|item| item.requirement == "optional")
        .count();
    let unused_count = items
        .iter()
        .filter(|item| item.requirement == "unused")
        .count();
    let missing_required_count = items
        .iter()
        .filter(|item| item.requirement == "required" && !item.available)
        .count();

    Ok(CharacterResourcePlanDto {
        project_id: request.project_id,
        item_id: request.item_id,
        option_id: option.option_id,
        source_type: option.source_type,
        source_id: option.source_id,
        provider_model_id: option.provider_model_id,
        workflow_preset_id: option.workflow_preset_id,
        required_count,
        optional_count,
        unused_count,
        missing_required_count,
        items,
    })
}

fn normalize_storyboard_bible_refs(
    database: &Database,
    mut item: SceneDto,
) -> Result<SceneDto, String> {
    item.character_ids = normalize_scene_string_list(item.character_ids);
    validate_storyboard_character_ids(database, &item.project_id, &item.character_ids)?;
    if !item.character_ids.is_empty() {
        item.characters = character_names_for_ids(database, &item.project_id, &item.character_ids)?;
    } else {
        item.characters = normalize_scene_string_list(item.characters);
    }
    item.location_id = item
        .location_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    validate_storyboard_location_id(database, &item.project_id, item.location_id.as_deref())?;
    if item.location_id.is_some() {
        item.scene_description = location_scene_description_for_id(
            database,
            &item.project_id,
            item.location_id.as_deref(),
        )?
        .unwrap_or_default();
    } else {
        item.scene_description = item.scene_description.trim().to_string();
    }
    Ok(item)
}

fn normalize_scene_string_list(values: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() || output.iter().any(|item: &String| item == trimmed) {
            continue;
        }
        output.push(trimmed.to_string());
    }
    output
}

pub fn start_image_generation(
    database: &Database,
    workspace_root: &Path,
    keyring_service: &KeyringService,
    request: StartImageGenerationRequest,
) -> Result<Vec<ImageCandidateDto>, String> {
    start_image_generation_with_options(
        database,
        workspace_root,
        keyring_service,
        request,
        ImageGenerationRuntimeOptions::default(),
    )
}

fn start_image_generation_with_options(
    database: &Database,
    workspace_root: &Path,
    keyring_service: &KeyringService,
    request: StartImageGenerationRequest,
    options: ImageGenerationRuntimeOptions,
) -> Result<Vec<ImageCandidateDto>, String> {
    let image_kind = request.image_kind.as_deref().unwrap_or("storyboard_image");
    if image_kind != "storyboard_image" {
        return Err(format!(
            "start_image_generation only writes storyboard ImageCandidate records; use start_image_asset_generation for image_kind {image_kind}."
        ));
    }
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for image generation.".to_string());
    }
    if request.item_id.trim().is_empty() {
        return Err("item_id is required for image generation.".to_string());
    }
    if request.provider_model_id.is_some() && request.workflow_preset_id.is_some() {
        return Err("provider_model_id and workflow_preset_id cannot both be set.".to_string());
    }

    let scene_repository = SceneRepository::new(database);
    let item = scene_repository
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Storyboard item {} does not belong to project {}.",
            request.item_id, request.project_id
        ));
    }
    ensure_storyboard_item_unlocked_for_image_generation(&item)?;
    validate_items_for_image_generation(std::slice::from_ref(&item))?;
    let prompt_preview = build_image_prompt_preview_for_item(database, &request.project_id, &item)?;
    let rule_snapshot = project_rule_snapshot(database, workspace_root, &request.project_id)?;

    ensure_controlled_fake_image_provider(database)?;
    let option = resolve_image_generation_option(database, &request)?;
    if let Err(message) =
        validate_required_image_inputs(database, &option, &item, request.workflow_params.as_ref())
    {
        scene_repository.mark_image_generation_failed(
            &request.item_id,
            TaskError::from_code("validation.invalid_input", message.clone()).to_json(),
        )?;
        return Err(message);
    }

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let task_repository = TaskRepository::new(database);
    let task = match task_repository.get_latest_task_detail_by_project(&request.project_id)? {
        Some(task) => task,
        None => task_repository.create_image_to_video_task(&request.project_id)?,
    };
    task_repository.start_task(&request.project_id)?;
    let step_id = task
        .steps
        .iter()
        .find(|step| step.step_name == IMAGE_GENERATION_STEP)
        .map(|step| step.step_id.clone())
        .unwrap_or_else(|| format!("{}_{}", task.task_id, IMAGE_GENERATION_STEP));
    let task_detail =
        task_repository.retry_task_step(&request.project_id, IMAGE_GENERATION_STEP)?;
    let task_id = task_detail.task_id;
    let token = CancellationToken::new(format!("{task_id}_{}", request.item_id));
    if options.cancel_before_success {
        task_repository.request_cancellation(&task_id, Some("scene_service_test_cancel"))?;
        token.cancel();
    }

    let count = request.count.unwrap_or(2).clamp(1, 4);
    let revision = scene_repository.latest_image_revision(&request.item_id)? + 1;
    let derived_from_image_id = item.selected_image_id.clone();
    let mut records = Vec::new();
    let mut artifact_records = Vec::new();
    let mut output_candidates = Vec::new();
    let started_at = current_timestamp_id();

    for variant_index in 1..=count {
        let candidate_id = create_id("img");
        let project_relative_path = format!(
            "{}/images/{}/rev_{:03}/{}_v{}.png",
            sanitize_path_segment(&request.project_id),
            sanitize_path_segment(&request.item_id),
            revision,
            candidate_id,
            variant_index
        );
        let trace_id = create_id("trace");
        let context = ProviderRequestContext {
            trace_id: trace_id.clone(),
            task_id: Some(task_id.clone()),
            task_step_id: Some(step_id.clone()),
            project_id: Some(request.project_id.clone()),
            item_id: Some(request.item_id.clone()),
            provider_id: option.provider_id.clone(),
            provider_model_id: option.provider_model_id.clone(),
            workflow_preset_id: option.workflow_preset_id.clone(),
            timeout_seconds: Some(30),
            idempotency_key: Some(format!(
                "{}:{}:{}:{}:{}",
                task_id, request.item_id, revision, variant_index, started_at
            )),
        };

        let provider_response = ProviderManager::new(database, keyring_service)
            .generate_image(
                ImageProviderRequest {
                    context: context.clone(),
                    prompt: prompt_preview.final_prompt.clone(),
                    negative_prompt: non_empty(prompt_preview.final_negative_prompt.clone()),
                    aspect_ratio: request
                        .aspect_ratio
                        .clone()
                        .unwrap_or_else(|| "9:16".to_string()),
                    width: request.width.or(Some(720)),
                    height: request.height.or(Some(1280)),
                    seed: request.seed.map(u64::from),
                    reference_images: prompt_preview.reference_images.clone(),
                    output_path: project_relative_path.clone(),
                },
                &token,
            )
            .map_err(|error| {
                record_image_generation_failure(
                    &task_repository,
                    &scene_repository,
                    &request,
                    &task_id,
                    &step_id,
                    provider_error_input_json(
                        &option,
                        &item,
                        &rule_snapshot,
                        revision,
                        variant_index,
                    ),
                    error,
                )
            })?;

        token.throw_if_cancelled()?;
        let stored = storage.write_bytes(
            FileBucket::Project,
            &project_relative_path,
            &controlled_fake_png_bytes(revision, variant_index),
            FileAccessPolicy::WriteProject,
        )?;
        let provider_model_id = option
            .provider_model_id
            .clone()
            .unwrap_or_else(|| option.source_id.clone());
        let snapshot = build_image_generation_snapshot(ImageGenerationSnapshotInput {
            request: &request,
            prompt_preview: &prompt_preview,
            option: &option,
            rule_snapshot: &rule_snapshot,
            task_id: &task_id,
            step_id: &step_id,
            trace_id: &trace_id,
            revision,
            variant_index,
            candidate_count: count,
            provider_image_path: &provider_response.image_path,
            provider_output_summary: provider_response.provider_output_summary.clone(),
        });

        records.push(NewImageCandidateRecord {
            image_id: candidate_id.clone(),
            item_id: request.item_id.clone(),
            image_path: stored.relative_path.clone(),
            prompt: prompt_preview.final_prompt.clone(),
            negative_prompt: prompt_preview.final_negative_prompt.clone(),
            model: option.label.clone(),
            provider_model_id: provider_model_id.clone(),
            workflow_preset_id: option.workflow_preset_id.clone(),
            status: "succeeded".to_string(),
            selected: false,
            derived_from_image_id: derived_from_image_id.clone(),
            generation_context_snapshot: snapshot.clone(),
        });
        artifact_records.push(TaskArtifactRecord {
            artifact_id: Some(format!("artifact_{candidate_id}")),
            task_id: task_id.clone(),
            step_id: Some(step_id.clone()),
            project_id: Some(request.project_id.clone()),
            owner_kind: Some("image_candidate".to_string()),
            owner_id: Some(candidate_id.clone()),
            artifact_kind: "image".to_string(),
            media_kind: "image".to_string(),
            relative_path: Some(stored.relative_path),
            metadata_json: snapshot,
        });
        output_candidates.push(json!({
            "imageId": candidate_id,
            "variantIndex": variant_index,
            "revision": revision
        }));
    }

    if options.cancel_after_provider_before_db {
        task_repository
            .request_cancellation(&task_id, Some("scene_service_test_cancel_after_provider"))?;
    }

    let input_json = build_image_task_input_snapshot(
        &request,
        &prompt_preview,
        &option,
        &rule_snapshot,
        revision,
        count,
    );
    task_repository.record_step_success(StepSuccessRecord {
        task_id: task_id.clone(),
        step_name: IMAGE_GENERATION_STEP.to_string(),
        input_json: input_json.clone(),
        output_json: json!({
            "projectId": request.project_id,
            "itemId": request.item_id,
            "revision": revision,
            "imageCandidates": output_candidates,
            "billable": false,
            "externalNetwork": false,
            "controlledFake": true
        }),
        artifacts: artifact_records,
    })?;

    if task_repository.is_cancel_requested(&task_id)? {
        return Err(format!(
            "Task {task_id} was cancelled before image candidates could be persisted."
        ));
    }

    scene_repository.insert_image_candidates(&records)
}

pub fn start_image_asset_generation(
    database: &Database,
    workspace_root: &Path,
    keyring_service: &KeyringService,
    request: StartImageAssetGenerationRequest,
) -> Result<Vec<GeneratedImageAssetDto>, String> {
    validate_asset_generation_request(&request)?;
    ensure_controlled_fake_image_provider(database)?;
    let option = resolve_image_asset_generation_option(database, &request)?;
    validate_required_image_asset_inputs(&option, &request)?;
    let rule_snapshot = project_rule_snapshot(database, workspace_root, &request.project_id)?;

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let task_repository = TaskRepository::new(database);
    let task = match task_repository.get_latest_task_detail_by_project(&request.project_id)? {
        Some(task) => task,
        None => task_repository.create_image_to_video_task(&request.project_id)?,
    };
    task_repository.start_task(&request.project_id)?;
    let step_id = task
        .steps
        .iter()
        .find(|step| step.step_name == IMAGE_GENERATION_STEP)
        .map(|step| step.step_id.clone())
        .unwrap_or_else(|| format!("{}_{}", task.task_id, IMAGE_GENERATION_STEP));
    let task_detail =
        task_repository.retry_task_step(&request.project_id, IMAGE_GENERATION_STEP)?;
    let task_id = task_detail.task_id;
    let token = CancellationToken::new(format!(
        "{task_id}_{}_{}",
        request.owner_id, request.image_kind
    ));

    let count = request.count.unwrap_or(1).clamp(1, 4);
    let started_at = current_timestamp_id();
    let asset_kind = request
        .asset_kind
        .clone()
        .unwrap_or_else(|| default_asset_kind_for_image_kind(&request.image_kind).to_string());
    let usage_kind = usage_kind_for_image_kind(&request.image_kind);
    let mut generated = Vec::new();
    let mut pending_asset_records = Vec::new();
    let mut artifact_records = Vec::new();
    let mut output_assets = Vec::new();

    for variant_index in 1..=count {
        let asset_id = create_id("asset");
        let project_relative_path = format!(
            "{}/{}/{}/{}_v{}.png",
            sanitize_path_segment(&request.project_id),
            sanitize_path_segment(&request.image_kind),
            sanitize_path_segment(&request.owner_id),
            asset_id,
            variant_index
        );
        let trace_id = create_id("trace");
        let context = ProviderRequestContext {
            trace_id: trace_id.clone(),
            task_id: Some(task_id.clone()),
            task_step_id: Some(step_id.clone()),
            project_id: Some(request.project_id.clone()),
            item_id: request.item_id.clone(),
            provider_id: option.provider_id.clone(),
            provider_model_id: option.provider_model_id.clone(),
            workflow_preset_id: option.workflow_preset_id.clone(),
            timeout_seconds: Some(30),
            idempotency_key: Some(format!(
                "{}:{}:{}:{}:{}",
                task_id, request.owner_id, request.image_kind, variant_index, started_at
            )),
        };

        let provider_response = ProviderManager::new(database, keyring_service)
            .generate_image(
                ImageProviderRequest {
                    context: context.clone(),
                    prompt: request.prompt.clone(),
                    negative_prompt: request.negative_prompt.clone().and_then(non_empty),
                    aspect_ratio: request
                        .aspect_ratio
                        .clone()
                        .unwrap_or_else(|| "9:16".to_string()),
                    width: request.width.or(Some(720)),
                    height: request.height.or(Some(1280)),
                    seed: request.seed.map(u64::from),
                    reference_images: vec![],
                    output_path: project_relative_path.clone(),
                },
                &token,
            )
            .map_err(|error| {
                record_image_asset_generation_failure(
                    &task_repository,
                    &request,
                    &task_id,
                    &step_id,
                    image_asset_error_input_json(&option, &request, &rule_snapshot, variant_index),
                    error,
                )
            })?;

        token.throw_if_cancelled()?;
        let stored = storage.write_bytes(
            FileBucket::Asset,
            &project_relative_path,
            &controlled_fake_png_bytes(1, variant_index),
            FileAccessPolicy::WriteProject,
        )?;
        let provider_model_id = option
            .provider_model_id
            .clone()
            .unwrap_or_else(|| option.source_id.clone());
        let snapshot = json!({
            "projectId": request.project_id,
            "itemId": request.item_id,
            "imageKind": request.image_kind,
            "assetKind": asset_kind,
            "ownerKind": request.owner_kind,
            "ownerId": request.owner_id,
            "referenceRole": request.reference_role,
            "taskId": task_id,
            "taskStepId": step_id,
            "traceId": trace_id,
            "variantIndex": variant_index,
            "assetCount": count,
            "sourceType": option.source_type,
            "sourceId": option.source_id,
            "providerId": option.provider_id,
            "providerKind": option.provider_kind,
            "providerModelId": option.provider_model_id,
            "workflowPresetId": option.workflow_preset_id,
            "ruleSnapshot": rule_snapshot.clone(),
            "inputPlan": {
                "planKind": option.input_plan.plan_kind,
                "abilityType": option.input_plan.ability_type,
                "imageKind": option.input_plan.image_kind,
                "assetKind": option.input_plan.asset_kind,
                "requiredCount": option.input_plan.required_count,
                "optionalCount": option.input_plan.optional_count
            },
            "providerOutputKind": "controlled_fake_bytes_converted_to_local_path",
            "providerImagePath": provider_response.image_path,
            "billable": false,
            "externalNetwork": false,
            "mock": false,
            "controlledFake": true,
            "sanitizedParamsSnapshot": {
                "aspectRatio": request.aspect_ratio.clone().unwrap_or_else(|| "9:16".to_string()),
                "width": request.width.unwrap_or(720),
                "height": request.height.unwrap_or(1280),
                "seed": request.seed,
                "workflowParams": request.workflow_params.clone().unwrap_or_else(|| json!({}))
            },
            "providerOutputSummary": provider_response.provider_output_summary
        });
        let reference_id = create_id("asset_ref");
        let reference_entry = json!({
            "assetId": asset_id,
            "referenceId": reference_id,
            "role": request.reference_role,
            "imageKind": request.image_kind,
            "assetKind": asset_kind,
            "relativePath": stored.relative_path,
            "source": "generated",
            "providerModelId": provider_model_id,
            "workflowPresetId": option.workflow_preset_id,
            "taskId": task_id,
            "taskStepId": step_id
        });
        pending_asset_records.push(PendingGeneratedImageAsset {
            record: NewGeneratedImageAssetRecord {
                project_id: request.project_id.clone(),
                asset: NewAssetRecord {
                    asset_id: asset_id.clone(),
                    kind: asset_kind.clone(),
                    relative_path: stored.relative_path.clone(),
                    source_kind: "ai_generated".to_string(),
                    mime_type: Some("image/png".to_string()),
                    size_bytes: Some(controlled_fake_png_bytes(1, variant_index).len() as i64),
                    checksum: None,
                    is_builtin: false,
                    metadata: snapshot.clone(),
                },
                reference: NewAssetReferenceRecord {
                    reference_id: reference_id.clone(),
                    asset_id: asset_id.clone(),
                    owner_kind: request.owner_kind.clone(),
                    owner_id: request.owner_id.clone(),
                    usage_kind: usage_kind.to_string(),
                },
                reference_entry,
            },
            snapshot: snapshot.clone(),
            image_kind: request.image_kind.clone(),
            asset_kind: asset_kind.clone(),
            owner_kind: request.owner_kind.clone(),
            owner_id: request.owner_id.clone(),
            reference_role: request.reference_role.clone(),
            usage_kind: usage_kind.to_string(),
        });
        artifact_records.push(TaskArtifactRecord {
            artifact_id: Some(format!("artifact_{asset_id}")),
            task_id: task_id.clone(),
            step_id: Some(step_id.clone()),
            project_id: Some(request.project_id.clone()),
            owner_kind: Some(request.owner_kind.clone()),
            owner_id: Some(request.owner_id.clone()),
            artifact_kind: "image_asset".to_string(),
            media_kind: "image".to_string(),
            relative_path: Some(stored.relative_path.clone()),
            metadata_json: snapshot.clone(),
        });
        output_assets.push(json!({
            "assetId": asset_id,
            "referenceId": reference_id,
            "imageKind": request.image_kind,
            "assetKind": asset_kind,
            "referenceRole": request.reference_role,
            "relativePath": stored.relative_path
        }));
    }

    let input_json = json!({
        "projectId": request.project_id,
        "itemId": request.item_id,
        "imageKind": request.image_kind,
        "assetKind": asset_kind,
        "ownerKind": request.owner_kind,
        "ownerId": request.owner_id,
        "referenceRole": request.reference_role,
        "promptHash": stable_hash_text(&request.prompt),
        "negativePromptHash": stable_hash_text(request.negative_prompt.as_deref().unwrap_or_default()),
        "count": count,
        "providerId": option.provider_id,
        "sourceType": option.source_type,
        "sourceId": option.source_id,
        "providerModelId": option.provider_model_id,
        "workflowPresetId": option.workflow_preset_id,
        "ruleSnapshot": rule_snapshot
    });
    task_repository.record_step_success(StepSuccessRecord {
        task_id: task_id.clone(),
        step_name: IMAGE_GENERATION_STEP.to_string(),
        input_json,
        output_json: json!({
            "projectId": request.project_id,
            "itemId": request.item_id,
            "imageKind": request.image_kind,
            "assetKind": asset_kind,
            "ownerKind": request.owner_kind,
            "ownerId": request.owner_id,
            "referenceRole": request.reference_role,
            "assets": output_assets,
            "billable": false,
            "externalNetwork": false,
            "controlledFake": true
        }),
        artifacts: artifact_records,
    })?;

    if task_repository.is_cancel_requested(&task_id)? {
        return Err(format!(
            "Task {task_id} was cancelled before generated image assets could be persisted."
        ));
    }

    for pending in pending_asset_records {
        let (asset, reference) =
            AssetRepository::new(database).insert_generated_image_asset(&pending.record)?;
        generated.push(GeneratedImageAssetDto {
            relative_path: asset.relative_path.clone(),
            asset,
            reference,
            image_kind: pending.image_kind,
            asset_kind: pending.asset_kind,
            owner_kind: pending.owner_kind,
            owner_id: pending.owner_id,
            reference_role: pending.reference_role,
            usage_kind: pending.usage_kind,
            generation_context_snapshot: pending.snapshot,
        });
    }

    Ok(generated)
}

pub fn select_image_candidate(
    database: &Database,
    request: SelectImageCandidateRequest,
) -> Result<SceneDto, String> {
    let repository = SceneRepository::new(database);
    let item = repository
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    ensure_storyboard_item_unlocked_for_field(&item, "selectedImage")?;
    repository.select_image_candidate(&request.item_id, &request.image_id)
}

pub fn clear_historical_image_candidates(
    database: &Database,
    request: ClearHistoricalImageCandidatesRequest,
) -> Result<SceneDto, String> {
    SceneRepository::new(database).clear_historical_image_candidates(&request.item_id)
}

pub fn start_tts_generation(
    database: &Database,
    workspace_root: &Path,
    keyring_service: &KeyringService,
    request: StartTtsGenerationRequest,
) -> Result<SceneDto, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for TTS generation.".to_string());
    }
    if request.item_id.trim().is_empty() {
        return Err("item_id is required for TTS generation.".to_string());
    }

    let scene_repository = SceneRepository::new(database);
    let item = scene_repository
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Storyboard item {} does not belong to project {}.",
            request.item_id, request.project_id
        ));
    }

    let text = narration_text_for_tts(&item).ok_or_else(|| {
        let message = format!(
            "Storyboard item {} has no narration_text or source_text for TTS.",
            item.index
        );
        let _ = scene_repository.mark_audio_generation_failed(
            &request.item_id,
            TaskError::from_code("validation.invalid_input", message.clone()).to_json(),
        );
        message
    })?;

    ensure_controlled_fake_tts_provider(database)?;
    let option = resolve_tts_generation_option(database, &request)?;
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;

    let project_relative_path = format!(
        "{}/audio/{}/voice.{}",
        sanitize_path_segment(&request.project_id),
        sanitize_path_segment(&request.item_id),
        tts_format(&request)
    );
    let trace_id = create_id("trace");
    let context = ProviderRequestContext {
        trace_id: trace_id.clone(),
        task_id: None,
        task_step_id: None,
        project_id: Some(request.project_id.clone()),
        item_id: Some(request.item_id.clone()),
        provider_id: option.provider_id.clone(),
        provider_model_id: option.provider_model_id.clone(),
        workflow_preset_id: None,
        timeout_seconds: Some(60),
        idempotency_key: Some(format!(
            "tts:{}:{}:{}",
            request.project_id,
            request.item_id,
            stable_hash_text(&text)
        )),
    };
    let token = CancellationToken::new(format!("tts_{}_{}", request.project_id, request.item_id));
    let response = ProviderManager::new(database, keyring_service)
        .generate_tts(
            TtsProviderRequest {
                context,
                text,
                content_language: project_content_language(database, &request.project_id)?
                    .unwrap_or_else(|| "zh-CN".to_string()),
                voice_id: request
                    .voice_id
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "default".to_string()),
                speed: request.speed,
                pitch: request.pitch,
                volume: request.volume,
                format: tts_format(&request),
                sample_rate: request.sample_rate.or(Some(24000)),
                output_path: project_relative_path.clone(),
            },
            &token,
        )
        .map_err(|error| {
            let message = error.message.clone();
            let _ =
                scene_repository.mark_audio_generation_failed(&request.item_id, error.to_json());
            message
        })?;

    token.throw_if_cancelled()?;
    if is_absolute_snapshot_path(&response.audio_path) {
        let error = TaskError::from_code_with_detail(
            "provider.invalid_output_path",
            "TTS provider returned an absolute output path.".to_string(),
            Some(
                json!({ "traceId": trace_id, "path": sanitize_snapshot_path(&response.audio_path) }),
            ),
        );
        scene_repository.mark_audio_generation_failed(&request.item_id, error.to_json())?;
        return Err("TTS provider returned an invalid absolute output path.".to_string());
    }

    scene_repository.update_storyboard_item_audio(
        &request.item_id,
        &response.audio_path,
        None,
        None,
    )
}

pub fn replace_storyboard_audio(
    database: &Database,
    workspace_root: &Path,
    request: ReplaceStoryboardAudioRequest,
) -> Result<SceneDto, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for audio replacement.".to_string());
    }
    if request.item_id.trim().is_empty() {
        return Err("item_id is required for audio replacement.".to_string());
    }
    if request.source_path.trim().is_empty() {
        return Err("source_path is required for audio replacement.".to_string());
    }

    let scene_repository = SceneRepository::new(database);
    let item = scene_repository
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Storyboard item {} does not belong to project {}.",
            request.item_id, request.project_id
        ));
    }

    let source_path = PathBuf::from(&request.source_path);
    let extension = source_path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "audio source file must have an extension.".to_string())?;
    validate_audio_extension(&extension)?;

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let project_relative_path = format!(
        "{}/audio/{}/uploaded.{}",
        sanitize_path_segment(&request.project_id),
        sanitize_path_segment(&request.item_id),
        extension
    );
    let stored = storage.copy_into_bucket(
        &source_path,
        FileBucket::Project,
        &project_relative_path,
        FileAccessPolicy::WriteProject,
    )?;

    scene_repository.update_storyboard_item_audio(
        &request.item_id,
        &stored.relative_path,
        None,
        None,
    )
}

pub fn probe_storyboard_audio(
    database: &Database,
    workspace_root: &Path,
    request: ProbeStoryboardAudioRequest,
) -> Result<SceneDto, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for audio probing.".to_string());
    }
    if request.item_id.trim().is_empty() {
        return Err("item_id is required for audio probing.".to_string());
    }

    let scene_repository = SceneRepository::new(database);
    let item = scene_repository
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Storyboard item {} does not belong to project {}.",
            request.item_id, request.project_id
        ));
    }
    let audio_path = item
        .audio_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Storyboard item {} has no audio_path.", item.index))?;
    let probe = ffmpeg_service::probe_media(workspace_root, audio_path, Some("audio"))?;
    if !probe.has_audio_stream {
        return Err(format!(
            "ffmpeg.invalid_media: audio stream is missing for {audio_path}."
        ));
    }
    scene_repository.update_storyboard_item_audio(
        &request.item_id,
        audio_path,
        Some(probe.duration_seconds),
        Some(&probe),
    )
}

pub fn generate_subtitles(
    database: &Database,
    workspace_root: &Path,
    request: GenerateSubtitlesRequest,
) -> Result<GenerateSubtitlesResultDto, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for subtitle generation.".to_string());
    }

    let repository = SceneRepository::new(database);
    let mut items = get_storyboard(database, request.project_id.clone())?.items;
    if let Some(item_ids) = request.item_ids.as_ref() {
        if item_ids.is_empty() {
            return Err("item_ids cannot be empty when provided.".to_string());
        }
        items.retain(|item| item_ids.contains(&item.item_id));
    }
    if items.is_empty() {
        return Err("No storyboard items available for subtitle generation.".to_string());
    }

    let mut updated_items = Vec::with_capacity(items.len());
    for item in items {
        let chunks = build_subtitle_chunks_for_item(&item)?;
        updated_items.push(repository.update_storyboard_item_subtitles(&item.item_id, &chunks)?);
    }

    let storyboard_items = get_storyboard(database, request.project_id.clone())?.items;
    let subtitles = build_subtitles_file(&request.project_id, &storyboard_items);
    let subtitle_path = write_subtitles_json(workspace_root, &request.project_id, &subtitles)?;

    Ok(GenerateSubtitlesResultDto {
        project_id: request.project_id,
        subtitle_path,
        items: updated_items,
        subtitles,
    })
}

pub fn update_storyboard_subtitles(
    database: &Database,
    workspace_root: &Path,
    request: UpdateStoryboardSubtitlesRequest,
) -> Result<GenerateSubtitlesResultDto, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for subtitle update.".to_string());
    }
    if request.item_id.trim().is_empty() {
        return Err("item_id is required for subtitle update.".to_string());
    }

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
    let chunks = normalize_edited_subtitle_chunks(&item, request.subtitle_chunks)?;
    let updated = repository.update_storyboard_item_subtitles(&request.item_id, &chunks)?;
    let storyboard_items = get_storyboard(database, request.project_id.clone())?.items;
    let subtitles = build_subtitles_file(&request.project_id, &storyboard_items);
    let subtitle_path = write_subtitles_json(workspace_root, &request.project_id, &subtitles)?;

    Ok(GenerateSubtitlesResultDto {
        project_id: request.project_id,
        subtitle_path,
        items: vec![updated],
        subtitles,
    })
}

pub fn start_video_generation(
    database: &Database,
    workspace_root: &Path,
    keyring_service: &KeyringService,
    request: StartVideoGenerationRequest,
) -> Result<Vec<VideoSegmentDto>, String> {
    start_video_generation_with_options(
        database,
        workspace_root,
        keyring_service,
        request,
        VideoGenerationRuntimeOptions::default(),
    )
}

fn start_video_generation_with_options(
    database: &Database,
    workspace_root: &Path,
    keyring_service: &KeyringService,
    request: StartVideoGenerationRequest,
    options: VideoGenerationRuntimeOptions,
) -> Result<Vec<VideoSegmentDto>, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for video generation.".to_string());
    }
    if request.item_id.trim().is_empty() {
        return Err("item_id is required for video generation.".to_string());
    }
    if request.provider_model_id.is_some() && request.workflow_preset_id.is_some() {
        return Err("provider_model_id and workflow_preset_id cannot both be set.".to_string());
    }

    let scene_repository = SceneRepository::new(database);
    let item = scene_repository
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Storyboard item {} does not belong to project {}.",
            request.item_id, request.project_id
        ));
    }
    ensure_storyboard_item_unlocked_for_video_generation(&item)?;
    validate_items_for_video_generation(std::slice::from_ref(&item))?;
    if item.video_prompt.trim().is_empty() {
        return Err(format!(
            "Storyboard item {} is missing required fields: video_prompt.",
            item.index
        ));
    }
    if item.duration_seconds <= 0.0 {
        return Err(format!(
            "Storyboard item {} is missing required fields: duration_seconds.",
            item.index
        ));
    }

    let selected_image_id = item
        .selected_image_id
        .clone()
        .ok_or_else(|| format!("Storyboard item {} has no selected_image_id.", item.index))?;
    let selected_image = scene_repository
        .get_image_candidate(&selected_image_id)?
        .ok_or_else(|| format!("Selected image candidate not found: {selected_image_id}"))?;
    if selected_image.item_id != item.item_id {
        return Err(format!(
            "Selected image candidate {} does not belong to storyboard item {}.",
            selected_image.image_id, item.item_id
        ));
    }

    ensure_controlled_fake_video_provider(database)?;
    let option = resolve_video_generation_option(database, &request)?;
    if let Err(error) = validate_video_option_ability(&option) {
        let message = prefixed_task_error_message(&error);
        scene_repository.mark_video_generation_failed(&request.item_id, error.to_json())?;
        return Err(message);
    }
    if let Err(message) = validate_required_video_inputs(
        &option,
        &item,
        &selected_image,
        request.workflow_params.as_ref(),
    ) {
        scene_repository.mark_video_generation_failed(
            &request.item_id,
            TaskError::from_code("validation.invalid_input", message.clone()).to_json(),
        )?;
        return Err(message);
    }

    let count = request.count.unwrap_or(1).clamp(1, 4);
    let revision = scene_repository.latest_video_revision(&request.item_id)? + 1;
    let started_at = current_timestamp_id();
    let provider_model_id = option
        .provider_model_id
        .clone()
        .unwrap_or_else(|| option.source_id.clone());
    let aspect_ratio = request
        .aspect_ratio
        .clone()
        .unwrap_or_else(|| "9:16".to_string());
    let resolution = request.resolution.clone();
    let fps = request.fps;
    let video_inputs =
        provider_video_inputs(&option, &selected_image, request.workflow_params.as_ref());
    let rule_snapshot = project_rule_snapshot(database, workspace_root, &request.project_id)?;

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let task_repository = TaskRepository::new(database);
    let task = match task_repository.get_latest_task_detail_by_project(&request.project_id)? {
        Some(task) => task,
        None => task_repository.create_image_to_video_task(&request.project_id)?,
    };
    task_repository.start_task(&request.project_id)?;
    let step_id = task
        .steps
        .iter()
        .find(|step| step.step_name == VIDEO_GENERATION_STEP)
        .map(|step| step.step_id.clone())
        .unwrap_or_else(|| format!("{}_{}", task.task_id, VIDEO_GENERATION_STEP));
    let task_detail =
        task_repository.retry_task_step(&request.project_id, VIDEO_GENERATION_STEP)?;
    let task_id = task_detail.task_id;
    if let Err(error) = validate_video_option_limits(
        &option,
        item.duration_seconds,
        &aspect_ratio,
        resolution.as_deref(),
        fps,
        video_inputs.len(),
    ) {
        return Err(record_video_generation_failure(
            &task_repository,
            &scene_repository,
            &request,
            &task_id,
            &step_id,
            video_error_input_json(&option, &item, &selected_image, &rule_snapshot, revision, 0),
            error,
        ));
    }
    let token = CancellationToken::new(format!("{task_id}_{}", request.item_id));
    if options.cancel_before_success {
        task_repository.request_cancellation(&task_id, Some("scene_service_test_video_cancel"))?;
        token.cancel();
    }

    let mut records = Vec::new();
    let mut artifact_records = Vec::new();
    let mut output_segments = Vec::new();

    for variant_index in 1..=count {
        let segment_id = create_id("seg");
        let project_relative_path = format!(
            "{}/videos/{}/rev_{:03}/{}_v{}.mp4",
            sanitize_path_segment(&request.project_id),
            sanitize_path_segment(&request.item_id),
            revision,
            segment_id,
            variant_index
        );
        let trace_id = create_id("trace");
        let context = ProviderRequestContext {
            trace_id: trace_id.clone(),
            task_id: Some(task_id.clone()),
            task_step_id: Some(step_id.clone()),
            project_id: Some(request.project_id.clone()),
            item_id: Some(request.item_id.clone()),
            provider_id: option.provider_id.clone(),
            provider_model_id: option.provider_model_id.clone(),
            workflow_preset_id: option.workflow_preset_id.clone(),
            timeout_seconds: Some(120),
            idempotency_key: Some(format!(
                "{}:{}:{}:{}:{}",
                task_id, request.item_id, revision, variant_index, started_at
            )),
        };

        let provider_output = if option.source_type == "workflow_preset" {
            let workflow_response = ProviderManager::new(database, keyring_service)
                .run_workflow(
                    WorkflowProviderRequest {
                        context: context.clone(),
                        workflow_preset_id: option
                            .workflow_preset_id
                            .clone()
                            .unwrap_or_else(|| option.source_id.clone()),
                        workflow_vendor: option.vendor.clone(),
                        params: video_workflow_params(
                            request.workflow_params.clone(),
                            &item,
                            &selected_image,
                            &aspect_ratio,
                            resolution.as_deref(),
                            fps,
                            request.seed,
                        ),
                        output_path: project_relative_path.clone(),
                    },
                    &token,
                )
                .map_err(|error| {
                    record_video_generation_failure(
                        &task_repository,
                        &scene_repository,
                        &request,
                        &task_id,
                        &step_id,
                        video_error_input_json(
                            &option,
                            &item,
                            &selected_image,
                            &rule_snapshot,
                            revision,
                            variant_index,
                        ),
                        error,
                    )
                })?;
            json!({
                "providerVideoPath": workflow_response.output_path,
                "providerOutputSummary": workflow_response.metadata,
                "providerOutputKind": "workflow_output_converted_to_local_path"
            })
        } else {
            let provider_response = ProviderManager::new(database, keyring_service)
                .generate_video(
                    VideoProviderRequest {
                        context: context.clone(),
                        ability_type: option.input_plan.ability_type.clone(),
                        prompt: item.video_prompt.clone(),
                        negative_prompt: None,
                        aspect_ratio: aspect_ratio.clone(),
                        duration_seconds: item.duration_seconds,
                        resolution: resolution.clone(),
                        fps,
                        seed: request.seed.map(u64::from),
                        input_images: video_inputs.clone(),
                        input_video_path: None,
                        input_audio_path: None,
                        output_path: project_relative_path.clone(),
                    },
                    &token,
                )
                .map_err(|error| {
                    record_video_generation_failure(
                        &task_repository,
                        &scene_repository,
                        &request,
                        &task_id,
                        &step_id,
                        video_error_input_json(
                            &option,
                            &item,
                            &selected_image,
                            &rule_snapshot,
                            revision,
                            variant_index,
                        ),
                        error,
                    )
                })?;
            json!({
                "providerVideoPath": provider_response.video_path,
                "providerDurationSeconds": provider_response.duration_seconds,
                "providerFps": provider_response.fps,
                "providerWidth": provider_response.width,
                "providerHeight": provider_response.height,
                "providerFileSize": provider_response.file_size,
                "providerOutputSummary": provider_response.provider_output_summary,
                "providerOutputKind": "provider_video_converted_to_local_path"
            })
        };

        token.throw_if_cancelled()?;
        let stored = storage.write_bytes(
            FileBucket::Project,
            &project_relative_path,
            &controlled_fake_mp4_bytes(revision, variant_index),
            FileAccessPolicy::WriteProject,
        )?;
        let snapshot = build_video_generation_snapshot(VideoGenerationSnapshotInput {
            request: &request,
            item: &item,
            selected_image: &selected_image,
            option: &option,
            rule_snapshot: &rule_snapshot,
            task_id: &task_id,
            step_id: &step_id,
            trace_id: &trace_id,
            revision,
            variant_index,
            segment_count: count,
            aspect_ratio: &aspect_ratio,
            resolution: resolution.as_deref(),
            fps,
            provider_output: provider_output.clone(),
        });

        records.push(NewVideoSegmentRecord {
            segment_id: segment_id.clone(),
            item_id: request.item_id.clone(),
            input_image_id: selected_image.image_id.clone(),
            video_path: stored.relative_path.clone(),
            video_prompt: item.video_prompt.clone(),
            duration_seconds: item.duration_seconds,
            model: option.label.clone(),
            provider_model_id: provider_model_id.clone(),
            workflow_preset_id: option.workflow_preset_id.clone(),
            status: "succeeded".to_string(),
            selected: false,
            generation_context_snapshot: snapshot.clone(),
        });
        artifact_records.push(TaskArtifactRecord {
            artifact_id: Some(format!("artifact_{segment_id}")),
            task_id: task_id.clone(),
            step_id: Some(step_id.clone()),
            project_id: Some(request.project_id.clone()),
            owner_kind: Some("video_segment".to_string()),
            owner_id: Some(segment_id.clone()),
            artifact_kind: "video".to_string(),
            media_kind: "video".to_string(),
            relative_path: Some(stored.relative_path),
            metadata_json: snapshot,
        });
        output_segments.push(json!({
            "segmentId": segment_id,
            "variantIndex": variant_index,
            "revision": revision
        }));
    }

    if options.cancel_after_provider_before_db {
        task_repository.request_cancellation(
            &task_id,
            Some("scene_service_test_video_cancel_after_provider"),
        )?;
    }

    let input_json = build_video_task_input_snapshot(
        &request,
        &item,
        &selected_image,
        &option,
        &rule_snapshot,
        revision,
        count,
    );
    task_repository.record_step_success(StepSuccessRecord {
        task_id: task_id.clone(),
        step_name: VIDEO_GENERATION_STEP.to_string(),
        input_json,
        output_json: json!({
            "projectId": request.project_id,
            "itemId": request.item_id,
            "revision": revision,
            "videoSegments": output_segments,
            "billable": false,
            "externalNetwork": false,
            "controlledFake": true
        }),
        artifacts: artifact_records,
    })?;

    if task_repository.is_cancel_requested(&task_id)? {
        return Err(format!(
            "Task {task_id} was cancelled before video segments could be persisted."
        ));
    }

    scene_repository.insert_video_segments(&records)
}

pub fn select_video_segment(
    database: &Database,
    request: SelectVideoSegmentRequest,
) -> Result<SceneDto, String> {
    let repository = SceneRepository::new(database);
    let existing = repository
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    ensure_storyboard_item_unlocked_for_field(&existing, "selectedVideoSegment")?;
    let item = repository.select_video_segment(&request.item_id, &request.segment_id)?;
    validate_items_for_composition(std::slice::from_ref(&item))?;
    Ok(item)
}

pub fn clear_historical_video_segments(
    database: &Database,
    request: ClearHistoricalVideoSegmentsRequest,
) -> Result<SceneDto, String> {
    SceneRepository::new(database).clear_historical_video_segments(&request.item_id)
}

#[derive(Debug, Clone, Copy, Default)]
struct ImageGenerationRuntimeOptions {
    cancel_before_success: bool,
    cancel_after_provider_before_db: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct VideoGenerationRuntimeOptions {
    cancel_before_success: bool,
    cancel_after_provider_before_db: bool,
}

struct PendingGeneratedImageAsset {
    record: NewGeneratedImageAssetRecord,
    snapshot: Value,
    image_kind: String,
    asset_kind: String,
    owner_kind: String,
    owner_id: String,
    reference_role: String,
    usage_kind: String,
}

fn record_image_generation_failure(
    task_repository: &TaskRepository<'_>,
    scene_repository: &SceneRepository<'_>,
    request: &StartImageGenerationRequest,
    task_id: &str,
    step_id: &str,
    input_json: Value,
    error: TaskError,
) -> String {
    let message = error.message.clone();
    let error_json = error.to_json();
    let _ =
        task_repository.record_step_failure(crate::db::task_repository::TaskStepFailureRecord {
            project_id: request.project_id.clone(),
            task_id: task_id.to_string(),
            task_step_id: step_id.to_string(),
            step_kind: IMAGE_GENERATION_STEP.to_string(),
            item_id: Some(request.item_id.clone()),
            input_json,
            error,
            duration_ms: None,
            retry_count: 0,
            relative_path: None,
        });
    let _ = scene_repository.mark_image_generation_failed(&request.item_id, error_json);
    message
}

fn record_image_asset_generation_failure(
    task_repository: &TaskRepository<'_>,
    request: &StartImageAssetGenerationRequest,
    task_id: &str,
    step_id: &str,
    input_json: Value,
    error: TaskError,
) -> String {
    let message = error.message.clone();
    let _ =
        task_repository.record_step_failure(crate::db::task_repository::TaskStepFailureRecord {
            project_id: request.project_id.clone(),
            task_id: task_id.to_string(),
            task_step_id: step_id.to_string(),
            step_kind: IMAGE_GENERATION_STEP.to_string(),
            item_id: request.item_id.clone(),
            input_json,
            error,
            duration_ms: None,
            retry_count: 0,
            relative_path: None,
        });
    message
}

fn record_video_generation_failure(
    task_repository: &TaskRepository<'_>,
    scene_repository: &SceneRepository<'_>,
    request: &StartVideoGenerationRequest,
    task_id: &str,
    step_id: &str,
    input_json: Value,
    error: TaskError,
) -> String {
    let message = error.message.clone();
    let error_code = error.error_code.clone();
    let error_json = error.to_json();
    let _ =
        task_repository.record_step_failure(crate::db::task_repository::TaskStepFailureRecord {
            project_id: request.project_id.clone(),
            task_id: task_id.to_string(),
            task_step_id: step_id.to_string(),
            step_kind: VIDEO_GENERATION_STEP.to_string(),
            item_id: Some(request.item_id.clone()),
            input_json,
            error,
            duration_ms: None,
            retry_count: 0,
            relative_path: None,
        });
    let _ = scene_repository.mark_video_generation_failed(&request.item_id, error_json);
    format!("{error_code}: {message}")
}

fn provider_error_input_json(
    option: &ExecutableMediaOptionDto,
    item: &SceneDto,
    rule_snapshot: &Value,
    revision: u32,
    variant_index: u32,
) -> Value {
    json!({
        "itemId": item.item_id,
        "promptHash": stable_hash_text(&item.image_prompt),
        "revision": revision,
        "variantIndex": variant_index,
        "providerId": option.provider_id,
        "sourceType": option.source_type,
        "sourceId": option.source_id,
        "ruleSnapshot": rule_snapshot
    })
}

fn video_error_input_json(
    option: &ExecutableMediaOptionDto,
    item: &SceneDto,
    selected_image: &ImageCandidateDto,
    rule_snapshot: &Value,
    revision: u32,
    variant_index: u32,
) -> Value {
    json!({
        "itemId": item.item_id,
        "inputImageId": selected_image.image_id,
        "inputImagePath": selected_image.image_path,
        "videoPromptHash": stable_hash_text(&item.video_prompt),
        "revision": revision,
        "variantIndex": variant_index,
        "providerId": option.provider_id,
        "sourceType": option.source_type,
        "sourceId": option.source_id,
        "ruleSnapshot": rule_snapshot
    })
}

fn image_asset_error_input_json(
    option: &ExecutableMediaOptionDto,
    request: &StartImageAssetGenerationRequest,
    rule_snapshot: &Value,
    variant_index: u32,
) -> Value {
    json!({
        "itemId": request.item_id,
    "imageKind": request.image_kind,
        "referenceRole": request.reference_role,
        "ownerKind": request.owner_kind,
        "ownerId": request.owner_id,
        "promptHash": stable_hash_text(&request.prompt),
        "variantIndex": variant_index,
        "providerId": option.provider_id,
        "sourceType": option.source_type,
        "sourceId": option.source_id,
        "ruleSnapshot": rule_snapshot
    })
}

struct ImageGenerationSnapshotInput<'a> {
    request: &'a StartImageGenerationRequest,
    prompt_preview: &'a ImagePromptPreviewDto,
    option: &'a ExecutableMediaOptionDto,
    rule_snapshot: &'a Value,
    task_id: &'a str,
    step_id: &'a str,
    trace_id: &'a str,
    revision: u32,
    variant_index: u32,
    candidate_count: u32,
    provider_image_path: &'a str,
    provider_output_summary: Value,
}

fn build_image_generation_snapshot(input: ImageGenerationSnapshotInput<'_>) -> Value {
    let params_snapshot = sanitized_json(&json!({
        "aspectRatio": input.request.aspect_ratio.clone().unwrap_or_else(|| "9:16".to_string()),
        "width": input.request.width.unwrap_or(720),
        "height": input.request.height.unwrap_or(1280),
        "seed": input.request.seed,
        "workflowParams": input.request.workflow_params.clone().unwrap_or_else(|| json!({}))
    }));
    let prompt_snapshot = image_prompt_snapshot(input.prompt_preview);
    let model_snapshot = model_snapshot(input.option);
    let input_plan = input_plan_snapshot(input.option);
    let provider_output_summary = sanitized_json(&input.provider_output_summary);

    json!({
        "schemaVersion": GENERATION_CONTEXT_SCHEMA_VERSION,
        "contextKind": "image_candidate",
        "projectId": input.request.project_id,
        "itemId": input.request.item_id,
        "imageKind": "storyboard_image",
        "assetKind": input.request.asset_kind.clone().unwrap_or_else(|| "generated_output".to_string()),
        "taskId": input.task_id,
        "taskStepId": input.step_id,
        "traceId": input.trace_id,
        "revision": input.revision,
        "variantIndex": input.variant_index,
        "candidateCount": input.candidate_count,
        "sourceType": input.option.source_type,
        "sourceId": input.option.source_id,
        "providerId": input.option.provider_id,
        "providerKind": input.option.provider_kind,
        "providerModelId": input.option.provider_model_id,
        "workflowPresetId": input.option.workflow_preset_id,
        "ruleSnapshot": sanitized_json(input.rule_snapshot),
        "styleBible": sanitized_json(&json!(input.prompt_preview.style_bible)),
        "characterBibles": sanitized_json(&json!(input.prompt_preview.character_bibles)),
        "locationBible": sanitized_json(&json!(input.prompt_preview.location_bible)),
        "promptSnapshot": prompt_snapshot,
        "promptPreview": prompt_snapshot,
        "modelSnapshot": model_snapshot,
        "inputPlan": input_plan,
        "seed": input.request.seed,
        "params": params_snapshot.clone(),
        "providerOutputKind": "controlled_fake_bytes_converted_to_local_path",
        "providerImagePath": sanitize_snapshot_path(input.provider_image_path),
        "billable": false,
        "externalNetwork": false,
        "mock": false,
        "controlledFake": true,
        "sanitizedParamsSnapshot": params_snapshot,
        "providerOutputSummary": provider_output_summary
    })
}

fn build_image_task_input_snapshot(
    request: &StartImageGenerationRequest,
    prompt_preview: &ImagePromptPreviewDto,
    option: &ExecutableMediaOptionDto,
    rule_snapshot: &Value,
    revision: u32,
    count: u32,
) -> Value {
    json!({
        "schemaVersion": GENERATION_CONTEXT_SCHEMA_VERSION,
        "contextKind": "image_generation_task_input",
        "projectId": request.project_id,
        "itemId": request.item_id,
        "promptSnapshot": image_prompt_snapshot(prompt_preview),
        "modelSnapshot": model_snapshot(option),
        "ruleSnapshot": sanitized_json(rule_snapshot),
        "inputPlan": input_plan_snapshot(option),
        "styleBibleId": prompt_preview.style_bible.as_ref().map(|style| style.style_bible_id.clone()),
        "characterBibleIds": prompt_preview.character_bibles.iter().map(|character| character.character_id.clone()).collect::<Vec<_>>(),
        "locationBibleId": prompt_preview.location_bible.as_ref().map(|location| location.location_id.clone()),
        "referenceImageCount": prompt_preview.reference_images.len(),
        "revision": revision,
        "count": count,
        "providerId": option.provider_id,
        "sourceType": option.source_type,
        "sourceId": option.source_id,
        "providerModelId": option.provider_model_id,
        "workflowPresetId": option.workflow_preset_id
    })
}

struct VideoGenerationSnapshotInput<'a> {
    request: &'a StartVideoGenerationRequest,
    item: &'a SceneDto,
    selected_image: &'a ImageCandidateDto,
    option: &'a ExecutableMediaOptionDto,
    rule_snapshot: &'a Value,
    task_id: &'a str,
    step_id: &'a str,
    trace_id: &'a str,
    revision: u32,
    variant_index: u32,
    segment_count: u32,
    aspect_ratio: &'a str,
    resolution: Option<&'a str>,
    fps: Option<u32>,
    provider_output: Value,
}

fn build_video_generation_snapshot(input: VideoGenerationSnapshotInput<'_>) -> Value {
    let params_snapshot = sanitized_json(&json!({
        "aspectRatio": input.aspect_ratio,
        "resolution": input.resolution,
        "fps": input.fps,
        "seed": input.request.seed,
        "durationSeconds": input.item.duration_seconds,
        "workflowParams": input.request.workflow_params.clone().unwrap_or_else(|| json!({}))
    }));
    let video_prompt_snapshot = video_prompt_snapshot(input.item);
    let input_image_snapshot = input_image_snapshot(input.selected_image);
    let model_snapshot = model_snapshot(input.option);
    let input_plan = input_plan_snapshot(input.option);

    json!({
        "schemaVersion": GENERATION_CONTEXT_SCHEMA_VERSION,
        "contextKind": "video_segment",
        "projectId": input.request.project_id,
        "itemId": input.request.item_id,
        "inputImageId": input.selected_image.image_id,
        "inputImagePath": sanitize_snapshot_path(&input.selected_image.image_path),
        "inputImageSnapshot": input_image_snapshot,
        "taskId": input.task_id,
        "taskStepId": input.step_id,
        "traceId": input.trace_id,
        "revision": input.revision,
        "variantIndex": input.variant_index,
        "segmentCount": input.segment_count,
        "sourceType": input.option.source_type,
        "sourceId": input.option.source_id,
        "providerId": input.option.provider_id,
        "providerKind": input.option.provider_kind,
        "providerModelId": input.option.provider_model_id,
        "workflowPresetId": input.option.workflow_preset_id,
        "videoPromptSnapshot": video_prompt_snapshot,
        "modelSnapshot": model_snapshot,
        "ruleSnapshot": sanitized_json(input.rule_snapshot),
        "inputPlan": input_plan,
        "workflowType": "image_to_video",
        "seed": input.request.seed,
        "params": params_snapshot.clone(),
        "billable": false,
        "externalNetwork": false,
        "mock": false,
        "controlledFake": true,
        "sanitizedParamsSnapshot": params_snapshot,
        "providerOutput": sanitized_json(&input.provider_output)
    })
}

fn build_video_task_input_snapshot(
    request: &StartVideoGenerationRequest,
    item: &SceneDto,
    selected_image: &ImageCandidateDto,
    option: &ExecutableMediaOptionDto,
    rule_snapshot: &Value,
    revision: u32,
    count: u32,
) -> Value {
    json!({
        "schemaVersion": GENERATION_CONTEXT_SCHEMA_VERSION,
        "contextKind": "video_generation_task_input",
        "projectId": request.project_id,
        "itemId": request.item_id,
        "videoPromptSnapshot": video_prompt_snapshot(item),
        "inputImageSnapshot": input_image_snapshot(selected_image),
        "modelSnapshot": model_snapshot(option),
        "ruleSnapshot": sanitized_json(rule_snapshot),
        "inputPlan": input_plan_snapshot(option),
        "inputImageId": selected_image.image_id,
        "inputImagePath": sanitize_snapshot_path(&selected_image.image_path),
        "revision": revision,
        "count": count,
        "providerId": option.provider_id,
        "sourceType": option.source_type,
        "sourceId": option.source_id,
        "providerModelId": option.provider_model_id,
        "workflowPresetId": option.workflow_preset_id
    })
}

fn image_prompt_snapshot(preview: &ImagePromptPreviewDto) -> Value {
    sanitized_json(&json!({
        "finalPromptHash": stable_hash_text(&preview.final_prompt),
        "finalNegativePromptHash": stable_hash_text(&preview.final_negative_prompt),
        "sections": preview.sections,
        "referenceImages": preview.reference_images,
        "negativePromptTruncated": preview.negative_prompt_truncated,
        "negativePromptMaxLength": preview.negative_prompt_max_length
    }))
}

fn video_prompt_snapshot(item: &SceneDto) -> Value {
    sanitized_json(&json!({
        "videoPromptHash": stable_hash_text(&item.video_prompt),
        "durationSeconds": item.duration_seconds,
        "shotSize": item.shot_size,
        "cameraMotion": item.camera_motion,
        "composition": item.composition,
        "pace": item.pace,
        "transitionType": item.transition_type
    }))
}

fn input_image_snapshot(candidate: &ImageCandidateDto) -> Value {
    let source = &candidate.generation_context_snapshot;
    json!({
        "imageId": candidate.image_id,
        "imagePath": sanitize_snapshot_path(&candidate.image_path),
        "revision": source.get("revision").and_then(Value::as_u64).unwrap_or(1),
        "variantIndex": source.get("variantIndex").and_then(Value::as_u64).unwrap_or(1),
        "promptHash": source
            .get("promptSnapshot")
            .or_else(|| source.get("promptPreview"))
            .and_then(|prompt| prompt.get("finalPromptHash"))
            .cloned()
            .unwrap_or_else(|| json!(stable_hash_text(&candidate.prompt))),
        "negativePromptHash": source
            .get("promptSnapshot")
            .or_else(|| source.get("promptPreview"))
            .and_then(|prompt| prompt.get("finalNegativePromptHash"))
            .cloned()
            .unwrap_or_else(|| json!(stable_hash_text(&candidate.negative_prompt))),
        "styleBibleId": source
            .get("styleBible")
            .and_then(|style| style.get("styleBibleId").or_else(|| style.get("style_bible_id")))
            .cloned(),
        "characterBibleIds": source
            .get("characterBibles")
            .and_then(Value::as_array)
            .map(|characters| {
                characters
                    .iter()
                    .filter_map(|character| {
                        character
                            .get("characterId")
                            .or_else(|| character.get("character_id"))
                            .and_then(Value::as_str)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        "locationBibleId": source
            .get("locationBible")
            .and_then(|location| location.get("locationId").or_else(|| location.get("location_id")))
            .cloned(),
        "referenceImages": source
            .get("promptSnapshot")
            .or_else(|| source.get("promptPreview"))
            .and_then(|prompt| prompt.get("referenceImages"))
            .cloned()
            .unwrap_or_else(|| json!([])),
        "modelSnapshot": source.get("modelSnapshot").cloned().unwrap_or_else(|| {
            json!({
                "providerModelId": candidate.provider_model_id,
                "workflowPresetId": candidate.workflow_preset_id,
                "label": candidate.model
            })
        })
    })
}

fn model_snapshot(option: &ExecutableMediaOptionDto) -> Value {
    sanitized_json(&json!({
        "optionId": option.option_id,
        "sourceType": option.source_type,
        "sourceId": option.source_id,
        "label": option.label,
        "providerId": option.provider_id,
        "providerKind": option.provider_kind,
        "vendor": option.vendor,
        "kind": option.kind,
        "capability": option.capability,
        "capabilities": option.capabilities,
        "providerModelId": option.provider_model_id,
        "workflowPresetId": option.workflow_preset_id,
        "constraints": option.constraints,
        "normalizedParams": option.normalized_params
    }))
}

fn input_plan_snapshot(option: &ExecutableMediaOptionDto) -> Value {
    sanitized_json(&json!({
        "planKind": option.input_plan.plan_kind,
        "abilityType": option.input_plan.ability_type,
        "imageKind": option.input_plan.image_kind,
        "assetKind": option.input_plan.asset_kind,
        "requiredCount": option.input_plan.required_count,
        "optionalCount": option.input_plan.optional_count,
        "unusedCount": option.input_plan.unused_count,
        "items": option.input_plan.items
    }))
}

fn sanitized_json(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut sanitized = Map::new();
            let mut redacted_count = 0usize;
            for (key, child) in object {
                if snapshot_key_is_sensitive(key) {
                    redacted_count += 1;
                    continue;
                }
                sanitized.insert(key.clone(), sanitized_json(child));
            }
            if redacted_count > 0 {
                sanitized.insert("redactedFieldCount".to_string(), json!(redacted_count));
            }
            Value::Object(sanitized)
        }
        Value::Array(items) => Value::Array(items.iter().map(sanitized_json).collect()),
        Value::String(text) => Value::String(sanitize_snapshot_text(text)),
        _ => value.clone(),
    }
}

fn snapshot_key_is_sensitive(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    [
        "secret",
        "authorization",
        "token",
        "password",
        "api_key",
        "apikey",
        "private_key",
    ]
    .iter()
    .any(|keyword| normalized.contains(keyword))
}

fn sanitize_snapshot_text(value: &str) -> String {
    let redacted = redact_text(value);
    if is_absolute_snapshot_path(&redacted) {
        "<blocked:absolute-path>".to_string()
    } else {
        redacted
    }
}

fn sanitize_snapshot_path(value: &str) -> String {
    sanitize_snapshot_text(value)
}

fn is_absolute_snapshot_path(value: &str) -> bool {
    let normalized = value.replace('\\', "/");
    normalized.starts_with('/') || value.as_bytes().get(1).is_some_and(|byte| *byte == b':')
}

fn resolve_image_generation_option(
    database: &Database,
    request: &StartImageGenerationRequest,
) -> Result<ExecutableMediaOptionDto, String> {
    let options = list_executable_media_options(database)?;
    let selected = options.into_iter().find(|option| {
        option.enabled
            && option
                .capabilities
                .iter()
                .any(|capability| matches!(capability.as_str(), "text_to_image" | "image_to_image"))
            && match (
                request.provider_model_id.as_deref(),
                request.workflow_preset_id.as_deref(),
            ) {
                (Some(provider_model_id), None) => {
                    option.provider_model_id.as_deref() == Some(provider_model_id)
                        || option.source_id == provider_model_id
                }
                (None, Some(workflow_preset_id)) => {
                    option.workflow_preset_id.as_deref() == Some(workflow_preset_id)
                        || option.source_id == workflow_preset_id
                }
                (None, None) => true,
                (Some(_), Some(_)) => false,
            }
    });

    selected.ok_or_else(|| {
        if request.provider_model_id.is_some() || request.workflow_preset_id.is_some() {
            "Selected image provider or workflow preset is not executable for text_to_image."
                .to_string()
        } else {
            "No executable text_to_image provider or workflow preset is configured.".to_string()
        }
    })
}

fn resolve_image_asset_generation_option(
    database: &Database,
    request: &StartImageAssetGenerationRequest,
) -> Result<ExecutableMediaOptionDto, String> {
    let options = list_executable_media_options(database)?;
    let selected = options.into_iter().find(|option| {
        option.enabled
            && option
                .capabilities
                .iter()
                .any(|capability| matches!(capability.as_str(), "text_to_image" | "image_to_image"))
            && match (
                request.provider_model_id.as_deref(),
                request.workflow_preset_id.as_deref(),
            ) {
                (Some(provider_model_id), None) => {
                    option.provider_model_id.as_deref() == Some(provider_model_id)
                        || option.source_id == provider_model_id
                }
                (None, Some(workflow_preset_id)) => {
                    option.workflow_preset_id.as_deref() == Some(workflow_preset_id)
                        || option.source_id == workflow_preset_id
                }
                (None, None) => true,
                (Some(_), Some(_)) => false,
            }
    });

    selected.ok_or_else(|| {
        if request.provider_model_id.is_some() || request.workflow_preset_id.is_some() {
            "Selected image provider or workflow preset is not executable for generated image assets."
                .to_string()
        } else {
            "No executable text_to_image provider or workflow preset is configured.".to_string()
        }
    })
}

fn resolve_video_generation_option(
    database: &Database,
    request: &StartVideoGenerationRequest,
) -> Result<ExecutableMediaOptionDto, String> {
    let options = list_executable_media_options(database)?;
    let selected = options.into_iter().find(|option| {
        option.enabled
            && option.capabilities.iter().any(|capability| {
                matches!(capability.as_str(), "image_to_video" | "first_frame_i2v")
            })
            && matches!(
                option.capability.as_str(),
                "image_to_video" | "first_frame_i2v"
            )
            && match (
                request.provider_model_id.as_deref(),
                request.workflow_preset_id.as_deref(),
            ) {
                (Some(provider_model_id), None) => {
                    option.provider_model_id.as_deref() == Some(provider_model_id)
                        || option.source_id == provider_model_id
                }
                (None, Some(workflow_preset_id)) => {
                    option.workflow_preset_id.as_deref() == Some(workflow_preset_id)
                        || option.source_id == workflow_preset_id
                }
                (None, None) => true,
                (Some(_), Some(_)) => false,
            }
    });

    selected.ok_or_else(|| {
        if request.provider_model_id.is_some() || request.workflow_preset_id.is_some() {
            "Selected video provider or workflow preset is not executable for image_to_video."
                .to_string()
        } else {
            "No executable image_to_video provider or workflow preset is configured.".to_string()
        }
    })
}

fn resolve_tts_generation_option(
    database: &Database,
    request: &StartTtsGenerationRequest,
) -> Result<ExecutableMediaOptionDto, String> {
    let options = list_executable_media_options(database)?;
    let selected = options.into_iter().find(|option| {
        option.enabled
            && option.provider_kind == "tts"
            && option
                .capabilities
                .iter()
                .any(|capability| matches!(capability.as_str(), "tts" | "text_to_speech"))
            && match request.provider_model_id.as_deref() {
                Some(provider_model_id) => {
                    option.provider_model_id.as_deref() == Some(provider_model_id)
                        || option.source_id == provider_model_id
                }
                None => true,
            }
    });

    selected.ok_or_else(|| {
        if request.provider_model_id.is_some() {
            "Selected TTS provider model is not executable for text_to_speech.".to_string()
        } else {
            "No executable TTS provider model is configured.".to_string()
        }
    })
}

fn narration_text_for_tts(item: &SceneDto) -> Option<String> {
    let narration = item.narration_text.trim();
    if !narration.is_empty() {
        return Some(narration.to_string());
    }
    let source = item.source_text.trim();
    if source.is_empty() {
        None
    } else {
        Some(source.to_string())
    }
}

fn build_subtitle_chunks_for_item(item: &SceneDto) -> Result<Vec<SubtitleChunkDto>, String> {
    let text = narration_text_for_tts(item).ok_or_else(|| {
        format!(
            "Storyboard item {} has no narration_text or source_text for subtitles.",
            item.index
        )
    })?;
    let lines = split_subtitle_text(&text);
    Ok(timed_subtitle_chunks_for_item(item, lines))
}

fn normalize_edited_subtitle_chunks(
    item: &SceneDto,
    chunks: Vec<SubtitleChunkDto>,
) -> Result<Vec<SubtitleChunkDto>, String> {
    let lines = chunks
        .into_iter()
        .map(|chunk| normalize_inline_text(&chunk.text))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return Err("subtitle_chunks must contain at least one non-empty text chunk.".to_string());
    }
    Ok(timed_subtitle_chunks_for_item(item, lines))
}

fn timed_subtitle_chunks_for_item(item: &SceneDto, lines: Vec<String>) -> Vec<SubtitleChunkDto> {
    let duration = subtitle_item_duration(item);
    let total_weight = lines
        .iter()
        .map(|text| subtitle_text_weight(text))
        .sum::<usize>()
        .max(lines.len());
    let mut cursor = 0.0;
    let last_index = lines.len().saturating_sub(1);

    lines
        .into_iter()
        .enumerate()
        .map(|(index, text)| {
            let start_seconds = cursor;
            let end_seconds = if index == last_index {
                duration
            } else {
                let weight = subtitle_text_weight(&text).max(1);
                cursor = (cursor + duration * (weight as f64 / total_weight as f64)).min(duration);
                cursor
            };
            SubtitleChunkDto {
                chunk_id: format!("sub_{}_{}", item.item_id, index + 1),
                text,
                start_seconds: Some(round_seconds(start_seconds)),
                end_seconds: Some(round_seconds(end_seconds.max(start_seconds))),
                estimated: true,
            }
        })
        .collect()
}

fn split_subtitle_text(text: &str) -> Vec<String> {
    let sentences = split_text_by_sentence_boundary(text);
    let mut chunks = Vec::new();
    let mut current = String::new();

    for sentence in sentences {
        for part in split_long_subtitle_sentence(&sentence) {
            let part_len = subtitle_text_weight(&part);
            let current_len = subtitle_text_weight(&current);
            if current.is_empty() {
                current = part;
            } else if current_len + part_len <= SUBTITLE_TARGET_MAX_CHARS {
                current.push_str(&part);
            } else {
                chunks.push(current);
                current = part;
            }
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }
    if chunks.is_empty() {
        return vec![normalize_inline_text(text)];
    }
    chunks
}

fn split_text_by_sentence_boundary(text: &str) -> Vec<String> {
    let normalized = normalize_inline_text(text);
    let mut output = Vec::new();
    let mut current = String::new();
    for character in normalized.chars() {
        current.push(character);
        if matches!(
            character,
            '。' | '！' | '？' | '；' | '!' | '?' | ';' | '.' | '\n'
        ) {
            let value = normalize_inline_text(&current);
            if !value.is_empty() {
                output.push(value);
            }
            current.clear();
        }
    }
    let value = normalize_inline_text(&current);
    if !value.is_empty() {
        output.push(value);
    }
    output
}

fn split_long_subtitle_sentence(sentence: &str) -> Vec<String> {
    if subtitle_text_weight(sentence) <= SUBTITLE_TARGET_MAX_CHARS {
        return vec![sentence.to_string()];
    }

    let mut output = Vec::new();
    let mut current = String::new();
    for character in sentence.chars() {
        current.push(character);
        let should_break = subtitle_text_weight(&current) >= SUBTITLE_TARGET_MIN_CHARS
            && (matches!(character, '，' | '、' | ',' | ' ' | '：' | ':')
                || subtitle_text_weight(&current) >= SUBTITLE_TARGET_MAX_CHARS);
        if should_break {
            let value = normalize_inline_text(&current);
            if !value.is_empty() {
                output.push(value);
            }
            current.clear();
        }
    }
    let value = normalize_inline_text(&current);
    if !value.is_empty() {
        output.push(value);
    }
    output
}

fn subtitle_item_duration(item: &SceneDto) -> f64 {
    item.audio_duration_seconds
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or_else(|| {
            if item.duration_seconds.is_finite() && item.duration_seconds > 0.0 {
                item.duration_seconds
            } else {
                1.0
            }
        })
}

fn subtitle_text_weight(text: &str) -> usize {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .count()
}

fn normalize_inline_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn round_seconds(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn build_subtitles_file(project_id: &str, items: &[SceneDto]) -> SubtitlesFileDto {
    let mut cursor = 0.0;
    let mut chunks = Vec::new();
    let mut sorted_items = items.to_vec();
    sorted_items.sort_by_key(|item| item.index);

    for item in sorted_items {
        let item_duration = subtitle_item_duration(&item);
        for chunk in &item.subtitle_chunks {
            let start = chunk.start_seconds.unwrap_or(0.0).clamp(0.0, item_duration);
            let end = chunk
                .end_seconds
                .unwrap_or(item_duration)
                .clamp(start, item_duration);
            chunks.push(SubtitleTimelineChunkDto {
                item_id: item.item_id.clone(),
                item_index: item.index,
                chunk_id: chunk.chunk_id.clone(),
                text: chunk.text.clone(),
                start_seconds: round_seconds(cursor + start),
                end_seconds: round_seconds(cursor + end),
                estimated: chunk.estimated,
                word_timings: build_estimated_word_timings(
                    &chunk.text,
                    cursor + start,
                    cursor + end,
                ),
            });
        }
        cursor += item_duration;
    }

    SubtitlesFileDto {
        schema_version: SUBTITLE_SCHEMA_VERSION,
        project_id: project_id.to_string(),
        generated_at: current_timestamp_id().to_string(),
        style: default_subtitle_style(),
        chunks,
    }
}

fn default_subtitle_style() -> SubtitleStyleDto {
    SubtitleStyleDto {
        preset_id: "vertical_cn_default".to_string(),
        position: "bottom".to_string(),
        font_size: 42,
        color: "#FFFFFF".to_string(),
        outline_color: "#111111".to_string(),
        outline_width: 3,
        highlight_color: "#FFD54A".to_string(),
        mode: "karaoke_estimated".to_string(),
        safe_top: 96,
        safe_bottom: 160,
        safe_left: 48,
        safe_right: 48,
        max_chars_per_line: SUBTITLE_TARGET_MAX_CHARS as u32,
    }
}

fn write_subtitles_json(
    workspace_root: &Path,
    project_id: &str,
    subtitles: &SubtitlesFileDto,
) -> Result<String, String> {
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let content = serde_json::to_string_pretty(subtitles).map_err(|error| error.to_string())?;
    let project_relative_path = format!(
        "{}/subtitles/subtitles.json",
        sanitize_path_segment(project_id)
    );
    let stored = storage.write_text(
        FileBucket::Project,
        &project_relative_path,
        &content,
        FileAccessPolicy::WriteProject,
    )?;
    Ok(stored.relative_path)
}

fn build_estimated_word_timings(
    text: &str,
    start_seconds: f64,
    end_seconds: f64,
) -> Vec<SubtitleWordTimingDto> {
    let tokens = subtitle_timing_tokens(text);
    if tokens.is_empty() {
        return Vec::new();
    }
    let duration = (end_seconds - start_seconds).max(0.001);
    let total_weight = tokens
        .iter()
        .map(|token| subtitle_text_weight(token))
        .sum::<usize>()
        .max(tokens.len());
    let mut cursor = start_seconds;
    let last_index = tokens.len().saturating_sub(1);

    tokens
        .into_iter()
        .enumerate()
        .map(|(index, token)| {
            let token_start = cursor;
            let token_end = if index == last_index {
                end_seconds
            } else {
                let weight = subtitle_text_weight(&token).max(1);
                cursor =
                    (cursor + duration * (weight as f64 / total_weight as f64)).min(end_seconds);
                cursor
            };
            SubtitleWordTimingDto {
                token,
                start_seconds: round_seconds(token_start),
                end_seconds: round_seconds(token_end.max(token_start)),
                estimated: true,
            }
        })
        .collect()
}

fn subtitle_timing_tokens(text: &str) -> Vec<String> {
    let normalized = normalize_inline_text(text);
    if normalized.contains(' ') {
        return normalized
            .split(' ')
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(str::to_string)
            .collect();
    }
    normalized
        .chars()
        .filter(|character| !character.is_whitespace())
        .map(|character| character.to_string())
        .collect()
}

fn tts_format(request: &StartTtsGenerationRequest) -> String {
    match request.format.as_deref().map(str::trim) {
        Some("wav") => "wav".to_string(),
        Some("mp3") | None | Some("") => "mp3".to_string(),
        Some(_) => "mp3".to_string(),
    }
}

fn validate_audio_extension(extension: &str) -> Result<(), String> {
    match extension {
        "mp3" | "wav" | "m4a" | "aac" | "flac" | "ogg" => Ok(()),
        _ => Err(format!("unsupported audio extension: {extension}")),
    }
}

fn project_content_language(
    database: &Database,
    project_id: &str,
) -> Result<Option<String>, String> {
    database
        .with_connection(|connection| {
            connection
                .query_row(
                    "SELECT content_language FROM projects WHERE project_id = ?1",
                    [project_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
        })
        .map_err(|error| error.to_string())
}

fn validate_required_image_inputs(
    database: &Database,
    option: &ExecutableMediaOptionDto,
    item: &SceneDto,
    workflow_params: Option<&Value>,
) -> Result<(), String> {
    let mut missing = Vec::new();
    let character_missing = missing_required_character_resources(database, option, item)?;
    for input in &option.input_plan.items {
        if input.requirement != "required" {
            continue;
        }
        match input.input_key.as_str() {
            "prompt" if item.image_prompt.trim().is_empty() => missing.push("prompt".to_string()),
            "negativePrompt" if item.negative_prompt.trim().is_empty() => {
                missing.push("negativePrompt".to_string())
            }
            key if key.starts_with("workflowParams.") => {
                let param_key = key.trim_start_matches("workflowParams.");
                let has_value =
                    workflow_param_value(workflow_params, param_key).is_some_and(value_is_present);
                if !has_value {
                    missing.push(key.to_string());
                }
            }
            key if is_character_reference_role(key) => {
                missing.extend(character_missing.get(key).cloned().unwrap_or_default());
            }
            key if !matches!(
                key,
                "prompt" | "negativePrompt" | "aspectRatio" | "resolution" | "seed"
            ) =>
            {
                missing.push(key.to_string());
            }
            _ => {}
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    Err(format!(
        "Image generation inputPlan.required is missing: {}.",
        missing.join(", ")
    ))
}

fn validate_required_image_asset_inputs(
    option: &ExecutableMediaOptionDto,
    request: &StartImageAssetGenerationRequest,
) -> Result<(), String> {
    let mut missing = Vec::new();
    for input in &option.input_plan.items {
        if input.requirement != "required" {
            continue;
        }
        match input.input_key.as_str() {
            "prompt" if request.prompt.trim().is_empty() => missing.push("prompt".to_string()),
            "negativePrompt"
                if request
                    .negative_prompt
                    .as_deref()
                    .unwrap_or_default()
                    .trim()
                    .is_empty() =>
            {
                missing.push("negativePrompt".to_string())
            }
            key if key.starts_with("workflowParams.") => {
                let param_key = key.trim_start_matches("workflowParams.");
                let has_value = request
                    .workflow_params
                    .as_ref()
                    .and_then(|params| params.get(param_key))
                    .is_some_and(value_is_present);
                if !has_value {
                    missing.push(key.to_string());
                }
            }
            key if !matches!(
                key,
                "prompt" | "negativePrompt" | "aspectRatio" | "resolution" | "seed"
            ) =>
            {
                missing.push(key.to_string());
            }
            _ => {}
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    Err(format!(
        "Image asset generation inputPlan.required is missing: {}.",
        missing.join(", ")
    ))
}

fn missing_required_character_resources(
    database: &Database,
    option: &ExecutableMediaOptionDto,
    item: &SceneDto,
) -> Result<HashMap<String, Vec<String>>, String> {
    let mut missing: HashMap<String, Vec<String>> = HashMap::new();
    if item.character_ids.is_empty() {
        for input in &option.input_plan.items {
            if input.requirement == "required" && is_character_reference_role(&input.input_key) {
                missing
                    .entry(input.input_key.clone())
                    .or_default()
                    .push(format!("character_id:{}", input.input_key));
            }
        }
        return Ok(missing);
    }
    let characters = load_storyboard_characters(database, item)?;
    for input in &option.input_plan.items {
        if input.requirement != "required" || !is_character_reference_role(&input.input_key) {
            continue;
        }
        for character in &characters {
            if find_character_reference(character, &input.input_key).is_none() {
                missing
                    .entry(input.input_key.clone())
                    .or_default()
                    .push(format!("{}:{}", character.character_id, input.input_key));
            }
        }
    }
    Ok(missing)
}

fn load_storyboard_characters(
    database: &Database,
    item: &SceneDto,
) -> Result<Vec<crate::domain::character::CharacterBibleDto>, String> {
    let repository = CharacterRepository::new(database);
    let mut characters = Vec::new();
    for character_id in &item.character_ids {
        let character_id = character_id.trim();
        if character_id.is_empty() {
            continue;
        }
        let character = repository
            .get_project_character_bible(&item.project_id, character_id)?
            .ok_or_else(|| format!("Character Bible not found: {character_id}"))?;
        characters.push(character);
    }
    Ok(characters)
}

fn input_plan_requirement_for_role(
    option: &ExecutableMediaOptionDto,
    role: &str,
) -> Option<String> {
    option
        .input_plan
        .items
        .iter()
        .find(|item| item.input_key == role)
        .map(|item| item.requirement.clone())
}

fn find_character_reference(
    character: &crate::domain::character::CharacterBibleDto,
    role: &str,
) -> Option<MatchedCharacterReference> {
    if role == "character_front_view" {
        if let Some(path) = character.reference_image_path.as_deref() {
            if path.starts_with("assets/") {
                return Some(MatchedCharacterReference {
                    asset_id: None,
                    relative_path: path.to_string(),
                });
            }
        }
    }
    for image in &character.reference_images {
        let image_role = image.get("role").and_then(Value::as_str);
        if image_role != Some(role) {
            continue;
        }
        let Some(path) = image
            .get("relativePath")
            .or_else(|| image.get("relative_path"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        if !path.starts_with("assets/") {
            continue;
        }
        return Some(MatchedCharacterReference {
            asset_id: image
                .get("assetId")
                .or_else(|| image.get("asset_id"))
                .and_then(Value::as_str)
                .map(str::to_string),
            relative_path: path.to_string(),
        });
    }
    None
}

fn is_character_reference_role(input_key: &str) -> bool {
    CHARACTER_REFERENCE_ROLES.contains(&input_key)
}

fn option_max_reference_images(option: &ExecutableMediaOptionDto) -> Option<u64> {
    option
        .constraints
        .get("limits")
        .and_then(|limits| read_json_u64(limits, &["maxReferenceImages", "max_reference_images"]))
}

fn validate_required_video_inputs(
    option: &ExecutableMediaOptionDto,
    item: &SceneDto,
    selected_image: &ImageCandidateDto,
    workflow_params: Option<&Value>,
) -> Result<(), String> {
    let mut missing = Vec::new();
    for input in &option.input_plan.items {
        if input.requirement != "required" {
            continue;
        }
        match input.input_key.as_str() {
            "startFrame" if selected_image.image_path.trim().is_empty() => {
                missing.push("startFrame".to_string())
            }
            "selectedImage" if selected_image.image_path.trim().is_empty() => {
                missing.push("selectedImage".to_string())
            }
            "videoPrompt" if item.video_prompt.trim().is_empty() => {
                missing.push("videoPrompt".to_string())
            }
            "durationSeconds" if item.duration_seconds <= 0.0 => {
                missing.push("durationSeconds".to_string())
            }
            "endFrame" if workflow_param_string(workflow_params, "endFrame").is_none() => {
                missing.push("endFrame".to_string())
            }
            "referenceAsset"
                if workflow_param_string(workflow_params, "referenceAsset").is_none() =>
            {
                missing.push("referenceAsset".to_string())
            }
            key if key.starts_with("workflowParams.") => {
                let param_key = key.trim_start_matches("workflowParams.");
                let has_value = workflow_params
                    .and_then(|params| params.get(param_key))
                    .is_some_and(value_is_present);
                if !has_value {
                    missing.push(key.to_string());
                }
            }
            key if !matches!(
                key,
                "startFrame"
                    | "selectedImage"
                    | "videoPrompt"
                    | "durationSeconds"
                    | "aspectRatio"
                    | "resolution"
                    | "fps"
                    | "seed"
            ) =>
            {
                missing.push(key.to_string());
            }
            _ => {}
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    Err(format!(
        "Video generation inputPlan.required is missing: {}.",
        missing.join(", ")
    ))
}

fn validate_video_option_limits(
    option: &ExecutableMediaOptionDto,
    duration_seconds: f64,
    aspect_ratio: &str,
    resolution: Option<&str>,
    fps: Option<u32>,
    input_image_count: usize,
) -> Result<(), TaskError> {
    validate_video_option_ability(option)?;
    let empty_limits = json!({});
    let limits = option.constraints.get("limits").unwrap_or(&empty_limits);
    validate_video_duration(option, limits, duration_seconds)?;
    validate_video_aspect_ratio(option, limits, aspect_ratio)?;
    validate_video_resolution(option, limits, resolution)?;
    validate_video_fps(option, limits, fps)?;
    validate_video_input_image_count(option, limits, input_image_count)?;
    Ok(())
}

fn validate_video_option_ability(option: &ExecutableMediaOptionDto) -> Result<(), TaskError> {
    let ability_type = option.input_plan.ability_type.as_str();
    if !is_video_ability_type(ability_type) {
        return Err(video_capability_error(
            option,
            "Video generation option must use a video ability type.",
            json!({
                "abilityType": ability_type,
                "capabilities": option.capabilities
            }),
        ));
    }
    if !option
        .capabilities
        .iter()
        .any(|capability| capability == ability_type)
    {
        return Err(video_capability_error(
            option,
            "Video generation ability type is not declared in option capabilities.",
            json!({
                "abilityType": ability_type,
                "capabilities": option.capabilities
            }),
        ));
    }
    Ok(())
}

fn prefixed_task_error_message(error: &TaskError) -> String {
    format!("{}: {}", error.error_code, error.message)
}

fn is_video_ability_type(ability_type: &str) -> bool {
    matches!(
        ability_type,
        "image_to_video" | "first_frame_i2v" | "start_end_frame_i2v" | "reference_to_video"
    )
}

fn validate_video_duration(
    option: &ExecutableMediaOptionDto,
    limits: &Value,
    duration_seconds: f64,
) -> Result<(), TaskError> {
    let Some(range) = read_json_value(
        limits,
        &[
            "durationSeconds",
            "duration_seconds",
            "durationRange",
            "duration_range",
        ],
    ) else {
        return Ok(());
    };
    let min = range.get("min").and_then(Value::as_f64).unwrap_or(0.0);
    let max = range.get("max").and_then(Value::as_f64).unwrap_or(f64::MAX);
    let integer = range
        .get("integer")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if duration_seconds < min
        || duration_seconds > max
        || (integer && duration_seconds.fract() != 0.0)
    {
        return Err(video_limit_error(
            option,
            "Duration is outside provider video limits.",
            json!({ "field": "durationSeconds", "durationSeconds": duration_seconds, "limits": range }),
        ));
    }
    Ok(())
}

fn validate_video_aspect_ratio(
    option: &ExecutableMediaOptionDto,
    limits: &Value,
    aspect_ratio: &str,
) -> Result<(), TaskError> {
    let Some(aspect_ratios) = read_json_string_array(
        limits,
        &[
            "aspectRatios",
            "aspect_ratios",
            "supportedAspectRatios",
            "supported_aspect_ratios",
        ],
    ) else {
        return Ok(());
    };
    if !aspect_ratios.is_empty() && !aspect_ratios.iter().any(|item| item == aspect_ratio) {
        return Err(video_limit_error(
            option,
            "Aspect ratio is not supported by provider video limits.",
            json!({ "field": "aspectRatio", "aspectRatio": aspect_ratio, "supportedAspectRatios": aspect_ratios }),
        ));
    }
    Ok(())
}

fn validate_video_resolution(
    option: &ExecutableMediaOptionDto,
    limits: &Value,
    resolution: Option<&str>,
) -> Result<(), TaskError> {
    let Some(resolution) = resolution else {
        return Ok(());
    };
    let Some(resolutions) = read_json_string_array(limits, &["resolutions"]) else {
        return Ok(());
    };
    if !resolutions.is_empty() && !resolutions.iter().any(|item| item == resolution) {
        return Err(video_limit_error(
            option,
            "Resolution is not supported by provider video limits.",
            json!({ "field": "resolution", "resolution": resolution, "resolutions": resolutions }),
        ));
    }
    Ok(())
}

fn validate_video_fps(
    option: &ExecutableMediaOptionDto,
    limits: &Value,
    fps: Option<u32>,
) -> Result<(), TaskError> {
    let Some(fps) = fps else {
        return Ok(());
    };
    if let Some(range) = read_json_value(limits, &["fpsRange", "fps_range"]) {
        let min = range.get("min").and_then(Value::as_f64).unwrap_or(0.0);
        let max = range.get("max").and_then(Value::as_f64).unwrap_or(f64::MAX);
        if (fps as f64) < min || (fps as f64) > max {
            return Err(video_limit_error(
                option,
                "FPS is outside provider video limits.",
                json!({ "field": "fps", "fps": fps, "limits": range }),
            ));
        }
    }
    if let Some(values) = read_json_number_array(limits, &["fps", "fpsValues", "fps_values"]) {
        if !values.is_empty()
            && !values
                .iter()
                .any(|value| (*value - fps as f64).abs() < f64::EPSILON)
        {
            return Err(video_limit_error(
                option,
                "FPS is not supported by provider video limits.",
                json!({ "field": "fps", "fps": fps, "supportedFps": values }),
            ));
        }
    }
    Ok(())
}

fn validate_video_input_image_count(
    option: &ExecutableMediaOptionDto,
    limits: &Value,
    input_image_count: usize,
) -> Result<(), TaskError> {
    let Some(max_reference_images) =
        read_json_u64(limits, &["maxReferenceImages", "max_reference_images"])
    else {
        return Ok(());
    };
    if input_image_count as u64 > max_reference_images {
        return Err(video_limit_error(
            option,
            "Too many input images for provider video limits.",
            json!({
                "field": "inputImages",
                "inputImageCount": input_image_count,
                "maxReferenceImages": max_reference_images
            }),
        ));
    }
    Ok(())
}

fn video_capability_error(
    option: &ExecutableMediaOptionDto,
    message: &str,
    detail: Value,
) -> TaskError {
    video_option_error("provider.capability_unsupported", option, message, detail)
}

fn video_limit_error(option: &ExecutableMediaOptionDto, message: &str, detail: Value) -> TaskError {
    video_option_error("provider.limit_exceeded", option, message, detail)
}

fn video_option_error(
    code: &str,
    option: &ExecutableMediaOptionDto,
    message: &str,
    detail: Value,
) -> TaskError {
    TaskError::from_code_with_detail(
        code,
        message.to_string(),
        Some(json!({
            "providerId": option.provider_id,
            "providerModelId": option.provider_model_id,
            "workflowPresetId": option.workflow_preset_id,
            "sourceType": option.source_type,
            "sourceId": option.source_id,
            "recoverAction": "edit_input",
            "detail": detail
        })),
    )
}

fn read_json_value(value: &Value, keys: &[&str]) -> Option<Value> {
    keys.iter().find_map(|key| value.get(*key).cloned())
}

fn read_json_string_array(value: &Value, keys: &[&str]) -> Option<Vec<String>> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(Value::as_array).map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
    })
}

fn read_json_number_array(value: &Value, keys: &[&str]) -> Option<Vec<f64>> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
    })
}

fn read_json_u64(value: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_u64))
}

fn validate_asset_generation_request(
    request: &StartImageAssetGenerationRequest,
) -> Result<(), String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for image asset generation.".to_string());
    }
    if request.image_kind.trim().is_empty() {
        return Err("image_kind is required for image asset generation.".to_string());
    }
    if request.owner_kind.trim().is_empty() {
        return Err("owner_kind is required for image asset generation.".to_string());
    }
    if request.owner_id.trim().is_empty() {
        return Err("owner_id is required for image asset generation.".to_string());
    }
    if request.reference_role.trim().is_empty() {
        return Err("reference_role is required for image asset generation.".to_string());
    }
    if request.prompt.trim().is_empty() {
        return Err("prompt is required for image asset generation.".to_string());
    }
    if request.provider_model_id.is_some() && request.workflow_preset_id.is_some() {
        return Err("provider_model_id and workflow_preset_id cannot both be set.".to_string());
    }
    validate_image_kind(&request.image_kind)?;
    let expected_owner_kind = default_owner_kind_for_image_kind(&request.image_kind);
    if request.owner_kind != expected_owner_kind {
        return Err(format!(
            "image_kind {} must use owner_kind {}, got {}.",
            request.image_kind, expected_owner_kind, request.owner_kind
        ));
    }
    if let Some(asset_kind) = request.asset_kind.as_deref() {
        validate_asset_kind_for_image_kind(&request.image_kind, asset_kind)?;
    }
    validate_reference_role_for_image_kind(&request.image_kind, &request.reference_role)?;
    Ok(())
}

fn provider_video_inputs(
    option: &ExecutableMediaOptionDto,
    selected_image: &ImageCandidateDto,
    workflow_params: Option<&Value>,
) -> Vec<ProviderMediaInputDto> {
    let mut inputs = Vec::new();
    let mut has_start_frame = false;
    for input in &option.input_plan.items {
        match input.input_key.as_str() {
            "startFrame" | "selectedImage" => {
                if !has_start_frame {
                    inputs.push(ProviderMediaInputDto {
                        path: selected_image.image_path.clone(),
                        role: "first_frame".to_string(),
                        weight: None,
                    });
                    has_start_frame = true;
                }
            }
            "endFrame" => {
                if let Some(path) = workflow_param_string(workflow_params, "endFrame") {
                    inputs.push(ProviderMediaInputDto {
                        path,
                        role: "last_frame".to_string(),
                        weight: None,
                    });
                }
            }
            "referenceAsset" => {
                if let Some(path) = workflow_param_string(workflow_params, "referenceAsset") {
                    inputs.push(ProviderMediaInputDto {
                        path,
                        role: "reference".to_string(),
                        weight: None,
                    });
                }
            }
            _ => {}
        }
    }
    if !has_start_frame {
        inputs.push(ProviderMediaInputDto {
            path: selected_image.image_path.clone(),
            role: "first_frame".to_string(),
            weight: None,
        });
    }
    inputs
}

fn video_workflow_params(
    workflow_params: Option<Value>,
    item: &SceneDto,
    selected_image: &ImageCandidateDto,
    aspect_ratio: &str,
    resolution: Option<&str>,
    fps: Option<u32>,
    seed: Option<u32>,
) -> Value {
    let mut params = workflow_params.unwrap_or_else(|| json!({}));
    if let Some(object) = params.as_object_mut() {
        object
            .entry("startFrame".to_string())
            .or_insert_with(|| json!(selected_image.image_path));
        object
            .entry("selectedImage".to_string())
            .or_insert_with(|| json!(selected_image.image_path));
        object
            .entry("videoPrompt".to_string())
            .or_insert_with(|| json!(item.video_prompt));
        object
            .entry("durationSeconds".to_string())
            .or_insert_with(|| json!(item.duration_seconds));
        object
            .entry("aspectRatio".to_string())
            .or_insert_with(|| json!(aspect_ratio));
        if let Some(resolution) = resolution {
            object
                .entry("resolution".to_string())
                .or_insert_with(|| json!(resolution));
        }
        if let Some(fps) = fps {
            object
                .entry("fps".to_string())
                .or_insert_with(|| json!(fps));
        }
        if let Some(seed) = seed {
            object
                .entry("seed".to_string())
                .or_insert_with(|| json!(seed));
        }
    }
    params
}

fn workflow_param_string(workflow_params: Option<&Value>, key: &str) -> Option<String> {
    workflow_param_value(workflow_params, key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

fn workflow_param_value<'a>(workflow_params: Option<&'a Value>, key: &str) -> Option<&'a Value> {
    let params = workflow_params?;
    params.get(key).or_else(|| {
        params
            .get("workflowParams")
            .and_then(|nested| nested.get(key))
    })
}

fn value_is_present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        Value::Object(values) => !values.is_empty(),
        _ => true,
    }
}

fn validate_image_kind(image_kind: &str) -> Result<(), String> {
    if matches!(
        image_kind,
        "character_reference"
            | "scene_reference"
            | "style_reference"
            | "prop_reference"
            | "end_frame"
            | "control_image"
            | "cover_image"
    ) {
        Ok(())
    } else {
        Err(format!(
            "Unsupported generated image asset kind: {image_kind}."
        ))
    }
}

fn validate_asset_kind_for_image_kind(image_kind: &str, asset_kind: &str) -> Result<(), String> {
    let expected = default_asset_kind_for_image_kind(image_kind);
    if asset_kind == expected {
        Ok(())
    } else {
        Err(format!(
            "image_kind {image_kind} must use asset_kind {expected}, got {asset_kind}."
        ))
    }
}

fn validate_reference_role_for_image_kind(
    image_kind: &str,
    reference_role: &str,
) -> Result<(), String> {
    let allowed = match image_kind {
        "character_reference" => CHARACTER_REFERENCE_ROLES,
        "scene_reference" => LOCATION_REFERENCE_ROLES,
        "style_reference" => &["style_reference"],
        "prop_reference" => &["prop_reference"],
        "cover_image" => &["cover_image"],
        "end_frame" | "control_image" => STORYBOARD_ASSET_REFERENCE_ROLES,
        _ => &[],
    };
    if allowed.contains(&reference_role) {
        Ok(())
    } else {
        Err(format!(
            "image_kind {image_kind} does not support reference_role {reference_role}."
        ))
    }
}

fn default_asset_kind_for_image_kind(image_kind: &str) -> &'static str {
    match image_kind {
        "character_reference" => "character_reference_image",
        "scene_reference" => "scene_reference_image",
        "style_reference" => "style_reference_image",
        "cover_image" => "cover_source",
        "prop_reference" | "end_frame" | "control_image" => "generated_output",
        _ => "generated_output",
    }
}

fn default_owner_kind_for_image_kind(image_kind: &str) -> &'static str {
    match image_kind {
        "character_reference" => "character_bible",
        "scene_reference" => "location_bible",
        "style_reference" => "style_bible",
        "prop_reference" | "cover_image" => "project",
        "end_frame" | "control_image" => "storyboard_item",
        _ => "project",
    }
}

fn usage_kind_for_image_kind(image_kind: &str) -> &'static str {
    match image_kind {
        "character_reference" => "character_reference",
        "scene_reference" => "location_reference",
        "style_reference" => "style_reference",
        "cover_image" => "cover",
        _ => "reference_image",
    }
}

fn ensure_controlled_fake_image_provider(database: &Database) -> Result<(), String> {
    let repository = ProviderRepository::new(database);
    let providers = repository.list_providers()?;
    if !providers
        .iter()
        .any(|provider| provider.provider_id == CONTROLLED_FAKE_PROVIDER_ID)
    {
        repository.upsert_provider(&ProviderRecord {
            provider_id: CONTROLLED_FAKE_PROVIDER_ID.to_string(),
            vendor: "controlled_fake".to_string(),
            kind: "image".to_string(),
            display_name: "Controlled fake image provider".to_string(),
            auth_type: "none".to_string(),
            key_alias: None,
            base_url: None,
            status: "ready".to_string(),
            enabled: true,
            config_json: json!({ "adapter": "dummy", "externalNetwork": false, "billable": false }),
        })?;
    }

    let models = repository.list_provider_models(Some(CONTROLLED_FAKE_PROVIDER_ID))?;
    if !models
        .iter()
        .any(|model| model.model_id == CONTROLLED_FAKE_MODEL_ID)
    {
        repository.upsert_provider_model(&ProviderModelRecord {
            model_id: CONTROLLED_FAKE_MODEL_ID.to_string(),
            provider_id: CONTROLLED_FAKE_PROVIDER_ID.to_string(),
            provider_model_id: CONTROLLED_FAKE_PROVIDER_MODEL_ID.to_string(),
            display_name: "Controlled fake text-to-image".to_string(),
            capability: "text_to_image".to_string(),
            config_json: json!({
                "providerKind": "image",
                "vendor": "controlled_fake",
                "modelName": "controlled-fake-text-to-image",
                "abilityTypes": ["text_to_image"],
                "inputModalities": ["text"],
                "outputModalities": ["image"],
                "inputRequirements": {},
                "limits": {
                    "supportedAspectRatios": ["9:16", "16:9", "1:1"],
                    "resolutions": ["720p"],
                    "maxReferenceImages": 0
                },
                "status": "ready",
                "apiContractVerified": true
            }),
            enabled: true,
        })?;
    }

    Ok(())
}

fn ensure_controlled_fake_video_provider(database: &Database) -> Result<(), String> {
    let repository = ProviderRepository::new(database);
    let providers = repository.list_providers()?;
    if !providers
        .iter()
        .any(|provider| provider.provider_id == CONTROLLED_FAKE_VIDEO_PROVIDER_ID)
    {
        repository.upsert_provider(&ProviderRecord {
            provider_id: CONTROLLED_FAKE_VIDEO_PROVIDER_ID.to_string(),
            vendor: "controlled_fake".to_string(),
            kind: "video".to_string(),
            display_name: "Controlled fake video provider".to_string(),
            auth_type: "none".to_string(),
            key_alias: None,
            base_url: None,
            status: "ready".to_string(),
            enabled: true,
            config_json: json!({ "adapter": "dummy", "externalNetwork": false, "billable": false }),
        })?;
    }

    let models = repository.list_provider_models(Some(CONTROLLED_FAKE_VIDEO_PROVIDER_ID))?;
    if !models
        .iter()
        .any(|model| model.model_id == CONTROLLED_FAKE_VIDEO_MODEL_ID)
    {
        repository.upsert_provider_model(&ProviderModelRecord {
            model_id: CONTROLLED_FAKE_VIDEO_MODEL_ID.to_string(),
            provider_id: CONTROLLED_FAKE_VIDEO_PROVIDER_ID.to_string(),
            provider_model_id: CONTROLLED_FAKE_VIDEO_PROVIDER_MODEL_ID.to_string(),
            display_name: "Controlled fake image-to-video".to_string(),
            capability: "image_to_video".to_string(),
            config_json: json!({
                "providerKind": "video",
                "vendor": "controlled_fake",
                "modelName": "controlled-fake-image-to-video",
                "abilityTypes": ["image_to_video"],
                "inputModalities": ["text", "image"],
                "outputModalities": ["video"],
                "inputRequirements": {},
                "limits": {
                    "durationSeconds": { "min": 1, "max": 30, "integer": false },
                    "supportedAspectRatios": ["9:16", "16:9", "1:1"],
                    "resolutions": ["720p"],
                    "fps": [24, 30],
                    "maxReferenceImages": 1
                },
                "status": "ready",
                "apiContractVerified": true
            }),
            enabled: true,
        })?;
    }

    Ok(())
}

fn ensure_controlled_fake_tts_provider(database: &Database) -> Result<(), String> {
    let repository = ProviderRepository::new(database);
    let providers = repository.list_providers()?;
    if !providers
        .iter()
        .any(|provider| provider.provider_id == CONTROLLED_FAKE_TTS_PROVIDER_ID)
    {
        repository.upsert_provider(&ProviderRecord {
            provider_id: CONTROLLED_FAKE_TTS_PROVIDER_ID.to_string(),
            vendor: "controlled_fake".to_string(),
            kind: "tts".to_string(),
            display_name: "Controlled fake TTS provider".to_string(),
            auth_type: "none".to_string(),
            key_alias: None,
            base_url: None,
            status: "ready".to_string(),
            enabled: true,
            config_json: json!({ "adapter": "dummy", "externalNetwork": false, "billable": false }),
        })?;
    }

    let models = repository.list_provider_models(Some(CONTROLLED_FAKE_TTS_PROVIDER_ID))?;
    if !models
        .iter()
        .any(|model| model.model_id == CONTROLLED_FAKE_TTS_MODEL_ID)
    {
        repository.upsert_provider_model(&ProviderModelRecord {
            model_id: CONTROLLED_FAKE_TTS_MODEL_ID.to_string(),
            provider_id: CONTROLLED_FAKE_TTS_PROVIDER_ID.to_string(),
            provider_model_id: CONTROLLED_FAKE_TTS_PROVIDER_MODEL_ID.to_string(),
            display_name: "Controlled fake text-to-speech".to_string(),
            capability: "text_to_speech".to_string(),
            config_json: json!({
                "providerKind": "tts",
                "vendor": "controlled_fake",
                "modelName": "controlled-fake-tts",
                "abilityTypes": ["text_to_speech"],
                "inputModalities": ["text"],
                "outputModalities": ["audio"],
                "inputRequirements": {},
                "limits": {
                    "sampleRates": [16000, 24000, 44100],
                    "formats": ["mp3", "wav"]
                },
                "status": "ready",
                "apiContractVerified": true
            }),
            enabled: true,
        })?;
    }

    Ok(())
}

fn default_storyboard(project_id: String) -> StoryboardDto {
    let narrations = vec![
        narration(1, "很多人以为早睡只是自律。"),
        narration(2, "其实它先改变的是你的清醒感。"),
        narration(3, "规律作息会让注意力和记忆力更稳定。"),
    ];

    StoryboardDto {
        storyboard_id: "storyboard_default".to_string(),
        project_id: project_id.clone(),
        confirmed_narrations: narrations.clone(),
        review_status: "waiting_user".to_string(),
        items: narrations
            .into_iter()
            .map(|item| {
                storyboard_item(project_id.clone(), item.index, item.text.clone(), item.text)
            })
            .collect(),
    }
}

fn script_draft_output_schema() -> Value {
    json!({
        "type": "object",
        "required": ["narrations"],
        "properties": {
            "narrations": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "required": ["index", "sourceText"],
                    "properties": {
                        "index": { "type": "integer" },
                        "sourceText": { "type": "string" },
                        "narrationText": { "type": "string" }
                    }
                }
            }
        }
    })
}

fn parse_script_draft_narrations(value: &Value) -> Result<Vec<ScriptDraftNarrationDto>, String> {
    let narrations = value
        .get("narrations")
        .cloned()
        .ok_or_else(|| "script draft narrations are required.".to_string())?;
    serde_json::from_value(narrations)
        .map_err(|error| format!("script draft parse failed: {error}"))
}

fn merge_script_draft_item(
    project_id: String,
    draft: ScriptDraftNarrationDto,
    existing: Option<SceneDto>,
) -> SceneDto {
    let source_text = draft.source_text.trim().to_string();
    let narration_text = draft
        .narration_text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(source_text.as_str())
        .to_string();

    if let Some(mut item) = existing {
        if !storyboard_lock_flag_enabled(&item, "sourceText") {
            item.source_text = source_text;
        }
        if !storyboard_lock_flag_enabled(&item, "narrationText") {
            item.narration_text = narration_text;
        }
        return item;
    }

    storyboard_item(project_id, draft.index, source_text, narration_text)
}

fn storyboard_item_to_narration(item: &SceneDto) -> crate::domain::scene::NarrationDto {
    crate::domain::scene::NarrationDto {
        index: item.index,
        text: if item.narration_text.trim().is_empty() {
            item.source_text.clone()
        } else {
            item.narration_text.clone()
        },
        locked: script_text_locked(item),
    }
}

fn narration(index: u32, text: &str) -> crate::domain::scene::NarrationDto {
    crate::domain::scene::NarrationDto {
        index,
        text: text.to_string(),
        locked: false,
    }
}

fn storyboard_item(
    project_id: String,
    index: u32,
    source_text: String,
    narration_text: String,
) -> SceneDto {
    let (visual_description, scene_description, image_prompt, video_prompt) = match index {
        1 => (
            "清晨卧室，主角醒来。",
            "清晨卧室",
            "清晨卧室，主角醒来，柔和自然光，竖屏构图",
            "晨光缓慢进入房间，主角睁眼坐起。",
        ),
        2 => (
            "对比熬夜和早睡状态。",
            "对比场景",
            "熬夜疲惫和早睡清醒状态对比，真实生活方式短视频，竖屏构图",
            "画面在疲惫和清醒状态之间平滑对比。",
        ),
        _ => (
            "办公室里保持清醒工作。",
            "办公室",
            "办公室里保持清醒工作，干净真实光线，竖屏构图",
            "主角专注工作，镜头轻微推进。",
        ),
    };

    SceneDto {
        item_id: format!("item_{index}"),
        project_id,
        index,
        source_text,
        narration_text,
        visual_goal: "表达旁白核心信息".to_string(),
        visual_description: visual_description.to_string(),
        characters: vec!["主角".to_string()],
        character_ids: vec![],
        location_id: None,
        scene_description: scene_description.to_string(),
        image_prompt: image_prompt.to_string(),
        negative_prompt: "低清晰度，畸形手指，扭曲面部，文字水印".to_string(),
        video_prompt: video_prompt.to_string(),
        duration_seconds: 4.0,
        subtitle_chunks: vec![],
        audio_path: None,
        audio_duration_seconds: None,
        audio_probe: None,
        selected_image_id: None,
        selected_video_segment_id: None,
        status: "pending".to_string(),
        lock_flags_json: json!({}),
        shot_size: Some("medium".to_string()),
        camera_motion: Some("static".to_string()),
        composition: Some("center".to_string()),
        pace: Some("normal".to_string()),
        transition_type: Some("cut".to_string()),
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
    }
}

fn validate_items_for_image_generation(items: &[SceneDto]) -> Result<(), String> {
    if items.is_empty() {
        return Err(
            "Storyboard must contain at least one item before image generation.".to_string(),
        );
    }

    for item in items {
        let mut missing = vec![];

        if item.source_text.trim().is_empty() && item.narration_text.trim().is_empty() {
            missing.push("source_text_or_narration_text");
        }
        if item.visual_description.trim().is_empty() {
            missing.push("visual_description");
        }
        if item.image_prompt.trim().is_empty() {
            missing.push("image_prompt");
        }
        if item.duration_seconds <= 0.0 {
            missing.push("duration_seconds");
        }

        if !missing.is_empty() {
            return Err(format!(
                "Storyboard item {} is missing required fields: {}.",
                item.index,
                missing.join(", ")
            ));
        }
    }

    Ok(())
}

fn ensure_storyboard_item_unlocked_for_image_generation(item: &SceneDto) -> Result<(), String> {
    if storyboard_lock_flag_enabled(item, "image") {
        return Err(format!(
            "Storyboard item {} is locked for image generation.",
            item.index
        ));
    }
    for field in ["imagePrompt", "negativePrompt", "selectedImage"] {
        ensure_storyboard_item_unlocked_for_field(item, field)?;
    }
    Ok(())
}

fn ensure_storyboard_item_unlocked_for_video_generation(item: &SceneDto) -> Result<(), String> {
    if storyboard_lock_flag_enabled(item, "video") {
        return Err(format!(
            "Storyboard item {} is locked for video generation.",
            item.index
        ));
    }
    for field in ["videoPrompt", "selectedVideoSegment"] {
        ensure_storyboard_item_unlocked_for_field(item, field)?;
    }
    Ok(())
}

fn ensure_storyboard_item_unlocked_for_field(item: &SceneDto, field: &str) -> Result<(), String> {
    if storyboard_lock_flag_enabled(item, field) {
        return Err(format!(
            "Storyboard item {} field {field} is locked.",
            item.index
        ));
    }
    Ok(())
}

fn storyboard_lock_flag_enabled(item: &SceneDto, field: &str) -> bool {
    item.lock_flags_json
        .get(field)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(crate) fn script_text_locked(item: &SceneDto) -> bool {
    storyboard_lock_flag_enabled(item, "sourceText")
        || storyboard_lock_flag_enabled(item, "narrationText")
}

fn validate_items_for_video_generation(items: &[SceneDto]) -> Result<(), String> {
    for item in items {
        if item.selected_image_id.is_none() {
            return Err(format!(
                "Storyboard item {} has no selected_image_id.",
                item.index
            ));
        }
    }

    Ok(())
}

fn validate_items_for_composition(items: &[SceneDto]) -> Result<(), String> {
    for item in items {
        if item.selected_video_segment_id.is_none() {
            return Err(format!(
                "Storyboard item {} has no selected_video_segment_id.",
                item.index
            ));
        }
    }

    Ok(())
}

fn non_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
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

fn create_id(prefix: &str) -> String {
    let nanos = current_timestamp_id();
    format!("{prefix}_{nanos}")
}

fn current_timestamp_id() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn stable_hash_text(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn controlled_fake_png_bytes(revision: u32, variant_index: u32) -> Vec<u8> {
    // Minimal valid 1x1 PNG. The metadata is stored in DB; the pixel payload only proves file persistence.
    let mut bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    bytes.push((revision % 255) as u8);
    bytes.push((variant_index % 255) as u8);
    bytes
}

fn controlled_fake_mp4_bytes(revision: u32, variant_index: u32) -> Vec<u8> {
    let mut bytes =
        b"\x00\x00\x00\x18ftypisom\x00\x00\x02\x00isomiso2mp41\x00\x00\x00\x08free".to_vec();
    bytes.push((revision % 255) as u8);
    bytes.push((variant_index % 255) as u8);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::project_repository::ProjectRepository;
    use crate::domain::project::CreateProjectRequest;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn apply_script_draft_rejects_invalid_schema_without_overwriting_storyboard() {
        let root = test_root("script_draft_invalid_schema");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_one_item(&database, &workspace_root, "project_script_bad");
        let before = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");

        let error = apply_script_draft(
            &database,
            ApplyScriptDraftRequest {
                project_id: "project_script_bad".to_string(),
                raw_output: json!({ "items": [{ "index": 1, "sourceText": "new" }] }).to_string(),
                expected_count: Some(1),
            },
        )
        .expect_err("invalid script draft schema should fail");
        assert!(error.contains("script draft schema invalid"));

        let after = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(after.source_text, before.source_text);
        assert_eq!(after.narration_text, before.narration_text);

        cleanup(root);
    }

    #[test]
    fn apply_script_draft_keeps_locked_script_text_fields() {
        let root = test_root("script_draft_locked_text");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_one_item(&database, &workspace_root, "project_script_lock");
        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.source_text = "locked source".to_string();
        item.narration_text = "locked narration".to_string();
        item.lock_flags_json = json!({
            "sourceText": true,
            "narrationText": true
        });
        update_storyboard_item(&database, item).expect("item should update");

        let storyboard = apply_script_draft(
            &database,
            ApplyScriptDraftRequest {
                project_id: "project_script_lock".to_string(),
                raw_output: json!({
                    "narrations": [{
                        "index": 1,
                        "sourceText": "new source",
                        "narrationText": "new narration"
                    }]
                })
                .to_string(),
                expected_count: Some(1),
            },
        )
        .expect("valid script draft should apply");

        let updated = storyboard
            .items
            .first()
            .expect("storyboard should contain item");
        assert_eq!(updated.source_text, "locked source");
        assert_eq!(updated.narration_text, "locked narration");
        assert_eq!(storyboard.confirmed_narrations[0].text, "locked narration");
        assert!(storyboard.confirmed_narrations[0].locked);

        cleanup(root);
    }

    #[test]
    fn image_generation_persists_candidates_and_files_with_revisions() {
        let root = test_root("image_generation_revisions");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_one_item(&database, &workspace_root, "project_revision");

        let first = start_image_generation_with_options(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_revision".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
            ImageGenerationRuntimeOptions::default(),
        )
        .expect("first generation should pass");
        let second = start_image_generation_with_options(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_revision".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
            ImageGenerationRuntimeOptions::default(),
        )
        .expect("second generation should pass");

        assert_eq!(first[0].generation_context_snapshot["revision"], 1);
        assert_eq!(second[0].generation_context_snapshot["revision"], 2);
        assert!(workspace_root.join(&first[0].image_path).is_file());
        assert!(workspace_root.join(&second[0].image_path).is_file());

        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(item.image_candidates.len(), 2);
        cleanup(root);
    }

    #[test]
    fn image_generation_keeps_selected_candidate_when_regenerating() {
        let root = test_root("image_generation_keeps_selected");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_one_item(&database, &workspace_root, "project_selected");
        let first = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_selected".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect("generation should pass");
        SceneRepository::new(&database)
            .select_image_candidate(&item_id, &first[0].image_id)
            .expect("selection should save");

        start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_selected".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect("regeneration should pass");

        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(
            item.selected_image_id.as_deref(),
            Some(first[0].image_id.as_str())
        );
        assert!(item
            .image_candidates
            .iter()
            .any(|candidate| candidate.image_id == first[0].image_id && candidate.selected));
        assert_eq!(item.image_candidates.len(), 2);
        cleanup(root);
    }

    #[test]
    fn image_generation_respects_locked_prompt_and_selected_image() {
        let root = test_root("image_generation_locked_fields");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_one_item(&database, &workspace_root, "project_img_lock");
        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.lock_flags_json = json!({"imagePrompt": true});
        update_storyboard_item(&database, item).expect("item should update");

        let error = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_img_lock".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect_err("locked imagePrompt should block image generation");
        assert!(error.contains("imagePrompt"));

        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.lock_flags_json = json!({});
        update_storyboard_item(&database, item).expect("item should update");
        let candidates = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_img_lock".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect("unlocked image generation should pass");

        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.lock_flags_json = json!({"selectedImage": true});
        update_storyboard_item(&database, item).expect("item should update");
        let error = select_image_candidate(
            &database,
            SelectImageCandidateRequest {
                item_id: item_id.clone(),
                image_id: candidates[0].image_id.clone(),
            },
        )
        .expect_err("locked selectedImage should block selection");
        assert!(error.contains("selectedImage"));
        cleanup(root);
    }

    #[test]
    fn image_generation_rejects_cancelled_task_before_db_write() {
        let root = test_root("image_generation_cancelled");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_one_item(&database, &workspace_root, "project_cancelled");

        let error = start_image_generation_with_options(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_cancelled".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
            ImageGenerationRuntimeOptions {
                cancel_before_success: false,
                cancel_after_provider_before_db: true,
            },
        )
        .expect_err("cancelled task should reject success write");

        assert!(error.contains("cancelled"));
        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert!(item.image_candidates.is_empty());
        cleanup(root);
    }

    #[test]
    fn image_generation_rejects_required_input_plan_missing_fields() {
        let root = test_root("image_generation_required_missing");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_one_item(&database, &workspace_root, "project_required");
        ensure_controlled_fake_image_provider(&database).expect("fake provider should seed");
        ProviderRepository::new(&database)
            .upsert_provider_model(&ProviderModelRecord {
                model_id: "model_requires_pose".to_string(),
                provider_id: CONTROLLED_FAKE_PROVIDER_ID.to_string(),
                provider_model_id: "controlled-fake/requires-pose".to_string(),
                display_name: "Requires pose".to_string(),
                capability: "text_to_image".to_string(),
                config_json: json!({
                    "providerKind": "image",
                    "vendor": "controlled_fake",
                    "modelName": "requires-pose",
                    "abilityTypes": ["text_to_image"],
                    "inputRequirements": {
                        "requiredInputs": ["character_pose"]
                    },
                    "limits": {},
                    "status": "ready"
                }),
                enabled: true,
            })
            .expect("model should save");

        let error = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_required".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: Some("model_requires_pose".to_string()),
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect_err("missing inputPlan required should fail");

        assert!(error.contains("inputPlan.required"));
        assert!(error.contains("character_pose"));
        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(item.image_status, "failed");
        assert_eq!(item.image_retry_count, 1);
        assert!(item.image_last_error_json.is_some());
        cleanup(root);
    }

    #[test]
    fn image_generation_rejects_character_reference_when_model_does_not_support_references() {
        let root = test_root("image_generation_character_ref_limit");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_character_reference(
            &database,
            &workspace_root,
            "project_character_ref_limit",
        );

        let error = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_character_ref_limit".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect_err("default text-to-image model should reject character references");

        assert!(error.contains("Too many reference images"));
        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(item.image_status, "failed");
        assert_eq!(item.image_retry_count, 1);
        assert_eq!(
            item.image_last_error_json
                .as_ref()
                .and_then(|value| value.get("errorCode").or_else(|| value.get("error_code")))
                .and_then(Value::as_str),
            Some("provider.limit_exceeded")
        );
        cleanup(root);
    }

    #[test]
    fn image_generation_sends_character_reference_images_to_supported_model() {
        let root = test_root("image_generation_character_ref_supported");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_character_reference(
            &database,
            &workspace_root,
            "project_character_ref_supported",
        );
        seed_limited_image_model(
            &database,
            "model_image_character_refs",
            json!({
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "maxReferenceImages": 2
            }),
            json!({}),
        );

        let candidates = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_character_ref_supported".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: Some("model_image_character_refs".to_string()),
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect("reference-capable model should generate");

        let references = candidates[0].generation_context_snapshot["promptPreview"]
            ["referenceImages"]
            .as_array()
            .expect("reference images should be recorded");
        assert_eq!(references.len(), 1);
        assert_eq!(
            references[0]["path"],
            json!("assets/character_reference_image/hero_front.png")
        );
        assert_eq!(references[0]["role"], json!("character_front_view:hero"));
        cleanup(root);
    }

    #[test]
    fn image_generation_snapshot_records_bibles_model_and_sanitized_params() {
        let root = test_root("image_generation_context_snapshot");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_character_reference(&database, &workspace_root, "project_img_snap");
        seed_limited_image_model(
            &database,
            "model_image_snapshot_refs",
            json!({
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "maxReferenceImages": 2
            }),
            json!({}),
        );

        let candidates = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_img_snap".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: Some("model_image_snapshot_refs".to_string()),
                workflow_preset_id: None,
                workflow_params: Some(json!({
                    "strength": 0.8,
                    "Authorization": "Bearer sk-abcdefghijklmnopqrstuvwxyz012345",
                    "localPath": "C:\\Users\\Twj\\secret\\input.png"
                })),
                aspect_ratio: None,
                width: None,
                height: None,
                seed: Some(42),
            },
        )
        .expect("image generation should pass");
        let snapshot = &candidates[0].generation_context_snapshot;

        assert_eq!(
            snapshot["schemaVersion"],
            json!(GENERATION_CONTEXT_SCHEMA_VERSION)
        );
        assert_eq!(snapshot["contextKind"], json!("image_candidate"));
        assert_eq!(
            snapshot["promptSnapshot"]["finalPromptHash"],
            json!(stable_hash_text(&candidates[0].prompt))
        );
        assert_eq!(snapshot["promptPreview"], snapshot["promptSnapshot"]);
        assert_eq!(snapshot["characterBibles"][0]["characterId"], json!("hero"));
        assert_eq!(
            snapshot["modelSnapshot"]["providerModelId"],
            json!("model_image_snapshot_refs")
        );
        assert!(snapshot["ruleSnapshot"].is_object());
        assert_eq!(snapshot["inputPlan"]["planKind"], json!("image"));
        assert_eq!(
            snapshot["params"]["workflowParams"]["redactedFieldCount"],
            json!(1)
        );
        assert_eq!(
            snapshot["params"]["workflowParams"]["localPath"],
            json!("<blocked:absolute-path>")
        );
        assert_snapshot_has_no_sensitive_or_absolute_text(snapshot);
        cleanup(root);
    }

    #[test]
    fn image_generation_snapshot_keeps_old_character_bible_after_edit() {
        let root = test_root("image_generation_snapshot_old_bible");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_character_reference(
            &database,
            &workspace_root,
            "project_old_bible_snapshot",
        );
        seed_limited_image_model(
            &database,
            "model_image_snapshot_old_bible",
            json!({
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "maxReferenceImages": 2
            }),
            json!({}),
        );

        let first = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_old_bible_snapshot".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: Some("model_image_snapshot_old_bible".to_string()),
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect("first generation should pass");
        crate::services::character_service::upsert_project_character_bible(
            &database,
            crate::domain::character::UpsertProjectCharacterBibleRequest {
                project_id: "project_old_bible_snapshot".to_string(),
                character_id: Some("hero".to_string()),
                name: "Hero".to_string(),
                alias: vec![],
                age: "28".to_string(),
                gender: "female".to_string(),
                appearance: "black hair after edit".to_string(),
                clothing: "blue coat after edit".to_string(),
                personality: "calm".to_string(),
                visual_prompt: Some("edited visual prompt".to_string()),
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                lock_flags: None,
            },
        )
        .expect("character should update");

        let saved = SceneRepository::new(&database)
            .get_image_candidate(&first[0].image_id)
            .expect("candidate should read")
            .expect("candidate should exist");
        let serialized = serde_json::to_string(&saved.generation_context_snapshot)
            .expect("snapshot should serialize");
        assert!(serialized.contains("silver hair"));
        assert!(serialized.contains("red pilot jacket"));
        assert!(!serialized.contains("black hair after edit"));
        assert!(!serialized.contains("blue coat after edit"));
        cleanup(root);
    }

    #[test]
    fn character_resource_plan_marks_unused_when_model_accepts_no_reference_images() {
        let root = test_root("character_plan_no_reference_model");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_character_reference(
            &database,
            &workspace_root,
            "project_character_plan_no_refs",
        );

        let plan = build_character_resource_plan(
            &database,
            BuildCharacterResourcePlanRequest {
                project_id: "project_character_plan_no_refs".to_string(),
                item_id,
                provider_model_id: None,
                workflow_preset_id: None,
            },
        )
        .expect("plan should build");

        assert_eq!(plan.required_count, 0);
        assert_eq!(plan.optional_count, 0);
        assert_eq!(plan.missing_required_count, 0);
        assert!(plan.items.iter().all(|item| item.requirement == "unused"));
        cleanup(root);
    }

    #[test]
    fn character_resource_plan_expands_required_roles_per_character() {
        let root = test_root("character_plan_required_roles");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_one_item(&database, &workspace_root, "project_character_plan");
        crate::services::character_service::upsert_project_character_bible(
            &database,
            crate::domain::character::UpsertProjectCharacterBibleRequest {
                project_id: "project_character_plan".to_string(),
                character_id: Some("hero".to_string()),
                name: "Hero".to_string(),
                alias: vec![],
                age: "".to_string(),
                gender: "".to_string(),
                appearance: "silver hair".to_string(),
                clothing: "red jacket".to_string(),
                personality: "".to_string(),
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                lock_flags: None,
            },
        )
        .expect("character should save");
        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.character_ids = vec!["hero".to_string()];
        update_storyboard_item(&database, item).expect("item should update");
        seed_limited_image_model(
            &database,
            "model_image_requires_front",
            json!({
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "maxReferenceImages": 2
            }),
            json!({
                "requiredInputs": ["character_front_view"],
                "optionalInputs": ["character_face_closeup"]
            }),
        );

        let missing_plan = build_character_resource_plan(
            &database,
            BuildCharacterResourcePlanRequest {
                project_id: "project_character_plan".to_string(),
                item_id: item_id.clone(),
                provider_model_id: Some("model_image_requires_front".to_string()),
                workflow_preset_id: None,
            },
        )
        .expect("plan should build");
        let front = missing_plan
            .items
            .iter()
            .find(|item| item.character_id == "hero" && item.role == "character_front_view")
            .expect("front role should exist");
        assert_eq!(front.requirement, "required");
        assert!(!front.available);
        assert_eq!(missing_plan.missing_required_count, 1);

        let item_id_with_ref = create_project_with_character_reference_id(
            &database,
            &workspace_root,
            "project_character_plan_with_ref",
            "hero_ref",
        );
        seed_limited_image_model(
            &database,
            "model_image_requires_front_2",
            json!({
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "maxReferenceImages": 2
            }),
            json!({
                "requiredInputs": ["character_front_view"],
                "optionalInputs": ["character_face_closeup"]
            }),
        );
        let available_plan = build_character_resource_plan(
            &database,
            BuildCharacterResourcePlanRequest {
                project_id: "project_character_plan_with_ref".to_string(),
                item_id: item_id_with_ref,
                provider_model_id: Some("model_image_requires_front_2".to_string()),
                workflow_preset_id: None,
            },
        )
        .expect("plan should build");
        let available_front = available_plan
            .items
            .iter()
            .find(|item| item.character_id == "hero_ref" && item.role == "character_front_view")
            .expect("front role should exist");
        assert!(available_front.available);
        assert_eq!(
            available_front.relative_path.as_deref(),
            Some("assets/character_reference_image/hero_front.png")
        );
        assert_eq!(available_plan.missing_required_count, 0);
        cleanup(root);
    }

    #[test]
    fn image_generation_failure_is_isolated_to_one_storyboard_item() {
        let root = test_root("image_generation_row_isolation");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let failing_item_id =
            create_project_with_one_item(&database, &workspace_root, "project_row_isolation");
        let mut second_item = storyboard_item(
            "project_row_isolation".to_string(),
            2,
            "其实它先改变的是你的清醒感。".to_string(),
            "其实它先改变的是你的清醒感。".to_string(),
        );
        second_item.item_id = "project_row_isolation_item_2".to_string();
        let success_item_id = SceneRepository::new(&database)
            .upsert_storyboard_item(&second_item)
            .expect("second item should save")
            .item_id;
        ensure_controlled_fake_image_provider(&database).expect("fake provider should seed");
        ProviderRepository::new(&database)
            .upsert_provider_model(&ProviderModelRecord {
                model_id: "model_requires_character_pose".to_string(),
                provider_id: CONTROLLED_FAKE_PROVIDER_ID.to_string(),
                provider_model_id: "controlled-fake/requires-character-pose".to_string(),
                display_name: "Requires character pose".to_string(),
                capability: "text_to_image".to_string(),
                config_json: json!({
                    "providerKind": "image",
                    "vendor": "controlled_fake",
                    "modelName": "requires-character-pose",
                    "abilityTypes": ["text_to_image"],
                    "inputRequirements": {
                        "requiredInputs": ["character_pose"]
                    },
                    "limits": {},
                    "status": "ready"
                }),
                enabled: true,
            })
            .expect("model should save");

        let failure = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_row_isolation".to_string(),
                item_id: failing_item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: Some("model_requires_character_pose".to_string()),
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect_err("first row should fail");
        assert!(failure.contains("character_pose"));

        let success = start_image_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: "project_row_isolation".to_string(),
                item_id: success_item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect("second row should still succeed");

        let repository = SceneRepository::new(&database);
        let failed_item = repository
            .get_storyboard_item(&failing_item_id)
            .expect("failed item should read")
            .expect("failed item should exist");
        let succeeded_item = repository
            .get_storyboard_item(&success_item_id)
            .expect("succeeded item should read")
            .expect("succeeded item should exist");
        assert_eq!(failed_item.image_status, "failed");
        assert_eq!(failed_item.image_candidates.len(), 0);
        assert_eq!(succeeded_item.image_status, "succeeded");
        assert_eq!(succeeded_item.image_candidates.len(), 1);
        assert_eq!(
            succeeded_item.image_candidates[0].image_id,
            success[0].image_id
        );
        cleanup(root);
    }

    #[test]
    fn image_asset_generation_writes_character_reference_without_image_candidate() {
        let root = test_root("image_asset_character_reference");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_one_item(&database, &workspace_root, "project_asset_ref");
        database
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO character_bibles (character_bible_id, project_id, name, data_json) VALUES (?1, ?2, ?3, ?4)",
                    ("character_main", "project_asset_ref", "主角", "{}"),
                )
            })
            .expect("character bible should save");

        let generated = start_image_asset_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartImageAssetGenerationRequest {
                project_id: "project_asset_ref".to_string(),
                image_kind: "character_reference".to_string(),
                asset_kind: None,
                owner_kind: "character_bible".to_string(),
                owner_id: "character_main".to_string(),
                reference_role: "character_front_view".to_string(),
                item_id: Some(item_id.clone()),
                prompt: "角色正面参考图，清晰全身".to_string(),
                negative_prompt: None,
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect("asset image generation should pass");

        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].image_kind, "character_reference");
        assert_eq!(generated[0].asset.kind, "character_reference_image");
        assert_eq!(generated[0].reference.owner_kind, "character_bible");
        assert_eq!(generated[0].reference.owner_id, "character_main");
        assert_eq!(generated[0].reference_role, "character_front_view");
        assert_eq!(generated[0].reference.usage_kind, "character_reference");
        assert!(generated[0].relative_path.starts_with("assets/"));
        assert!(workspace_root.join(&generated[0].relative_path).is_file());

        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert!(item.image_candidates.is_empty());
        assert!(item.selected_image_id.is_none());

        let bible_data: String = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT data_json FROM character_bibles WHERE character_bible_id = ?1",
                    ["character_main"],
                    |row| row.get(0),
                )
            })
            .expect("bible should read");
        let bible_json: Value = serde_json::from_str(&bible_data).expect("bible json should parse");
        let references = bible_json["reference_images_json"]
            .as_array()
            .expect("reference_images_json should be array");
        assert_eq!(references.len(), 1);
        assert_eq!(references[0]["assetId"], generated[0].asset.asset_id);
        assert_eq!(references[0]["role"], "character_front_view");
        assert_eq!(references[0]["relativePath"], generated[0].relative_path);

        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.character_ids = vec!["character_main".to_string()];
        update_storyboard_item(&database, item).expect("item should update");
        let plan = build_character_resource_plan(
            &database,
            BuildCharacterResourcePlanRequest {
                project_id: "project_asset_ref".to_string(),
                item_id,
                provider_model_id: None,
                workflow_preset_id: None,
            },
        )
        .expect("plan should build");
        let front_view = plan
            .items
            .iter()
            .find(|item| {
                item.character_id == "character_main" && item.role == "character_front_view"
            })
            .expect("front view entry should exist");
        assert!(front_view.available);
        assert_eq!(
            front_view.asset_id.as_deref(),
            Some(generated[0].asset.asset_id.as_str())
        );
        cleanup(root);
    }

    #[test]
    fn video_generation_rejects_missing_selected_image() {
        let root = test_root("video_generation_missing_selected_image");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_one_item(&database, &workspace_root, "project_video_missing");

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_video_missing".to_string(),
                item_id,
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
        )
        .expect_err("video generation without selected image should fail");

        assert!(error.contains("selected_image_id"));
        cleanup(root);
    }

    #[test]
    fn video_generation_rejects_missing_selected_image_candidate() {
        let root = test_root("video_generation_missing_candidate");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_one_item(&database, &workspace_root, "project_video_no_candidate");
        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.selected_image_id = Some("img_missing".to_string());
        SceneRepository::new(&database)
            .upsert_storyboard_item(&item)
            .expect("item should update");

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_video_no_candidate".to_string(),
                item_id,
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
        )
        .expect_err("missing selected ImageCandidate should fail");

        assert!(error.contains("Selected image candidate not found"));
        cleanup(root);
    }

    #[test]
    fn video_generation_persists_segments_and_files_with_revisions() {
        let root = test_root("video_generation_revisions");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_selected_image(
            &database,
            &workspace_root,
            "project_video_revision",
        );

        let first = start_video_generation_with_options(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_video_revision".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
            VideoGenerationRuntimeOptions::default(),
        )
        .expect("first video generation should pass");
        let second = start_video_generation_with_options(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_video_revision".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
            VideoGenerationRuntimeOptions::default(),
        )
        .expect("second video generation should pass");

        assert_eq!(first[0].generation_context_snapshot["revision"], 1);
        assert_eq!(second[0].generation_context_snapshot["revision"], 2);
        assert!(workspace_root.join(&first[0].video_path).is_file());
        assert!(workspace_root.join(&second[0].video_path).is_file());
        assert!(first[0].video_path.starts_with("projects/"));
        assert_eq!(first[0].selected, false);
        assert_eq!(
            first[0].generation_context_snapshot["billable"],
            json!(false)
        );
        assert_eq!(
            first[0].generation_context_snapshot["externalNetwork"],
            json!(false)
        );

        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(item.video_segments.len(), 2);
        assert!(item.selected_video_segment_id.is_none());
        cleanup(root);
    }

    #[test]
    fn video_generation_snapshot_records_input_image_summary_and_sanitized_params() {
        let root = test_root("video_generation_context_snapshot");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_selected_image(&database, &workspace_root, "project_video_snap");

        let segments = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_video_snap".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: Some(json!({
                    "cfg": 7,
                    "password": "sk-abcdefghijklmnopqrstuvwxyz012345",
                    "debugPath": "/Users/twj/secret/input.png"
                })),
                aspect_ratio: None,
                resolution: Some("720p".to_string()),
                fps: Some(24),
                seed: Some(7),
            },
        )
        .expect("video generation should pass");
        let snapshot = &segments[0].generation_context_snapshot;

        assert_eq!(
            snapshot["schemaVersion"],
            json!(GENERATION_CONTEXT_SCHEMA_VERSION)
        );
        assert_eq!(snapshot["contextKind"], json!("video_segment"));
        assert_eq!(
            snapshot["videoPromptSnapshot"]["videoPromptHash"],
            json!(stable_hash_text(&segments[0].video_prompt))
        );
        assert_eq!(
            snapshot["inputImageSnapshot"]["imageId"],
            snapshot["inputImageId"]
        );
        assert_eq!(snapshot["inputImageSnapshot"]["revision"], json!(1));
        assert!(snapshot["inputImageSnapshot"]["promptHash"].is_string());
        assert!(snapshot["inputImageSnapshot"]["modelSnapshot"].is_object());
        assert_eq!(snapshot["modelSnapshot"]["providerKind"], json!("video"));
        assert_eq!(
            snapshot["params"]["workflowParams"]["redactedFieldCount"],
            json!(1)
        );
        assert_eq!(
            snapshot["params"]["workflowParams"]["debugPath"],
            json!("<blocked:absolute-path>")
        );
        assert_snapshot_has_no_sensitive_or_absolute_text(snapshot);
        cleanup(root);
    }

    #[test]
    fn video_generation_keeps_confirmed_segment_when_regenerating() {
        let root = test_root("video_generation_keeps_selected");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_selected_image(
            &database,
            &workspace_root,
            "project_video_selected",
        );

        let first = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_video_selected".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
        )
        .expect("video generation should pass");
        SceneRepository::new(&database)
            .select_video_segment(&item_id, &first[0].segment_id)
            .expect("segment selection should save");

        start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_video_selected".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
        )
        .expect("video regeneration should pass");

        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(
            item.selected_video_segment_id.as_deref(),
            Some(first[0].segment_id.as_str())
        );
        assert!(item
            .video_segments
            .iter()
            .any(|segment| segment.segment_id == first[0].segment_id && segment.selected));
        assert_eq!(item.video_segments.len(), 2);
        cleanup(root);
    }

    #[test]
    fn video_generation_respects_locked_prompt_and_selected_segment() {
        let root = test_root("video_generation_locked_fields");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_selected_image(&database, &workspace_root, "project_vid_lock");
        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.lock_flags_json = json!({"videoPrompt": true});
        update_storyboard_item(&database, item).expect("item should update");

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_vid_lock".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
        )
        .expect_err("locked videoPrompt should block video generation");
        assert!(error.contains("videoPrompt"));

        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.lock_flags_json = json!({});
        update_storyboard_item(&database, item).expect("item should update");
        let segments = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_vid_lock".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
        )
        .expect("unlocked video generation should pass");

        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.lock_flags_json = json!({"selectedVideoSegment": true});
        update_storyboard_item(&database, item).expect("item should update");
        let error = select_video_segment(
            &database,
            SelectVideoSegmentRequest {
                item_id: item_id.clone(),
                segment_id: segments[0].segment_id.clone(),
            },
        )
        .expect_err("locked selectedVideoSegment should block selection");
        assert!(error.contains("selectedVideoSegment"));
        cleanup(root);
    }

    #[test]
    fn video_generation_rejects_cancelled_task_before_db_write() {
        let root = test_root("video_generation_cancelled");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_selected_image(
            &database,
            &workspace_root,
            "project_video_cancelled",
        );

        let error = start_video_generation_with_options(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_video_cancelled".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
            VideoGenerationRuntimeOptions {
                cancel_before_success: false,
                cancel_after_provider_before_db: true,
            },
        )
        .expect_err("cancelled task should reject success write");

        assert!(error.contains("cancelled"));
        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert!(item.video_segments.is_empty());
        cleanup(root);
    }

    #[test]
    fn video_generation_rejects_required_input_plan_missing_fields() {
        let root = test_root("video_generation_required_missing");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_selected_image(
            &database,
            &workspace_root,
            "project_video_required",
        );
        ensure_controlled_fake_video_provider(&database).expect("fake video provider should seed");
        ProviderRepository::new(&database)
            .upsert_provider_model(&ProviderModelRecord {
                model_id: "model_requires_motion_strength".to_string(),
                provider_id: CONTROLLED_FAKE_VIDEO_PROVIDER_ID.to_string(),
                provider_model_id: "controlled-fake/requires-motion-strength".to_string(),
                display_name: "Requires motion strength".to_string(),
                capability: "image_to_video".to_string(),
                config_json: json!({
                    "providerKind": "video",
                    "vendor": "controlled_fake",
                    "modelName": "requires-motion-strength",
                    "abilityTypes": ["image_to_video"],
                    "inputModalities": ["text", "image"],
                    "outputModalities": ["video"],
                    "inputRequirements": {
                        "requiredInputs": ["startFrame", "videoPrompt", "workflowParams.motionStrength"]
                    },
                    "paramSchema": {
                        "motionStrength": { "type": "number", "required": true }
                    },
                    "limits": {
                        "durationSeconds": { "min": 1, "max": 30, "integer": false },
                        "supportedAspectRatios": ["9:16"],
                        "fps": [24]
                    },
                    "status": "ready"
                }),
                enabled: true,
            })
            .expect("model should save");

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_video_required".to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                provider_model_id: Some("model_requires_motion_strength".to_string()),
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
        )
        .expect_err("missing required workflow param should fail");

        assert!(error.contains("inputPlan.required"));
        assert!(error.contains("workflowParams.motionStrength"));
        let item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(item.video_status, "failed");
        assert_eq!(item.segment_status, "failed");
        assert!(item.video_segments.is_empty());
        cleanup(root);
    }

    #[test]
    fn video_generation_rejects_duration_outside_provider_limits() {
        let root = test_root("video_generation_duration_limit");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_selected_image(
            &database,
            &workspace_root,
            "project_video_duration_limit",
        );
        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.duration_seconds = 12.0;
        SceneRepository::new(&database)
            .upsert_storyboard_item(&item)
            .expect("item should update");
        seed_limited_video_model(
            &database,
            "model_video_duration_limit",
            "image_to_video",
            json!({
                "durationSeconds": { "min": 1, "max": 5, "integer": false },
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "fps": [24],
                "maxReferenceImages": 1
            }),
            json!({}),
        );

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            video_request(
                "project_video_duration_limit",
                &item_id,
                Some("model_video_duration_limit"),
            ),
        )
        .expect_err("duration overflow should fail");

        assert!(error.contains("provider.limit_exceeded"));
        assert!(error.contains("Duration"));
        assert_no_video_segments(&database, &item_id);
        cleanup(root);
    }

    #[test]
    fn video_generation_rejects_aspect_ratio_outside_provider_limits() {
        let root = test_root("video_generation_aspect_limit");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_selected_image(
            &database,
            &workspace_root,
            "project_video_aspect_limit",
        );
        seed_limited_video_model(
            &database,
            "model_video_aspect_limit",
            "image_to_video",
            json!({
                "durationSeconds": { "min": 1, "max": 30, "integer": false },
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "fps": [24],
                "maxReferenceImages": 1
            }),
            json!({}),
        );
        let mut request = video_request(
            "project_video_aspect_limit",
            &item_id,
            Some("model_video_aspect_limit"),
        );
        request.aspect_ratio = Some("16:9".to_string());

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            request,
        )
        .expect_err("unsupported aspect ratio should fail");

        assert!(error.contains("provider.limit_exceeded"));
        assert!(error.contains("Aspect ratio"));
        assert_no_video_segments(&database, &item_id);
        cleanup(root);
    }

    #[test]
    fn video_generation_rejects_resolution_outside_provider_limits() {
        let root = test_root("video_generation_resolution_limit");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_selected_image(
            &database,
            &workspace_root,
            "project_video_resolution_limit",
        );
        seed_limited_video_model(
            &database,
            "model_video_resolution_limit",
            "image_to_video",
            json!({
                "durationSeconds": { "min": 1, "max": 30, "integer": false },
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "fps": [24],
                "maxReferenceImages": 1
            }),
            json!({}),
        );
        let mut request = video_request(
            "project_video_resolution_limit",
            &item_id,
            Some("model_video_resolution_limit"),
        );
        request.resolution = Some("1080p".to_string());

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            request,
        )
        .expect_err("unsupported resolution should fail");

        assert!(error.contains("provider.limit_exceeded"));
        assert!(error.contains("Resolution"));
        assert_no_video_segments(&database, &item_id);
        cleanup(root);
    }

    #[test]
    fn video_generation_rejects_fps_outside_provider_limits() {
        let root = test_root("video_generation_fps_limit");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_selected_image(
            &database,
            &workspace_root,
            "project_video_fps_limit",
        );
        seed_limited_video_model(
            &database,
            "model_video_fps_limit",
            "image_to_video",
            json!({
                "durationSeconds": { "min": 1, "max": 30, "integer": false },
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "fps": [24],
                "maxReferenceImages": 1
            }),
            json!({}),
        );
        let mut request = video_request(
            "project_video_fps_limit",
            &item_id,
            Some("model_video_fps_limit"),
        );
        request.fps = Some(30);

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            request,
        )
        .expect_err("unsupported fps should fail");

        assert!(error.contains("provider.limit_exceeded"));
        assert!(error.contains("FPS"));
        assert_no_video_segments(&database, &item_id);
        cleanup(root);
    }

    #[test]
    fn image_to_video_mainline_rejects_advanced_video_only_models() {
        let root = test_root("video_generation_advanced_only");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_selected_image(&database, &workspace_root, "project_adv_video");
        seed_limited_video_model(
            &database,
            "video_start_end_only",
            "start_end_frame_i2v",
            json!({ "durationSeconds": { "min": 2, "max": 6 } }),
            json!({}),
        );

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartVideoGenerationRequest {
                project_id: "project_adv_video".to_string(),
                item_id,
                count: Some(1),
                provider_model_id: Some("video_start_end_only".to_string()),
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                resolution: None,
                fps: None,
                seed: None,
            },
        )
        .expect_err("advanced-only model must not run current image_to_video mainline");

        assert!(error.contains("image_to_video"));
        cleanup(root);
    }

    #[test]
    fn video_limit_validation_rejects_too_many_input_images() {
        let option = video_option_for_limit_tests(
            "start_end_frame_i2v",
            vec!["start_end_frame_i2v"],
            json!({
                "durationSeconds": { "min": 1, "max": 30, "integer": false },
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "fps": [24],
                "maxReferenceImages": 1
            }),
        );

        let error = validate_video_option_limits(&option, 4.0, "9:16", Some("720p"), Some(24), 2)
            .expect_err("input image overflow should fail");

        assert_eq!(error.error_code, "provider.limit_exceeded");
        assert_eq!(error.recover_action.as_deref(), Some("edit_input"));
        assert_eq!(
            error.detail.as_ref().unwrap()["detail"]["field"],
            json!("inputImages")
        );
    }

    #[test]
    fn video_limit_validation_rejects_option_ability_type_mismatch() {
        let option = video_option_for_limit_tests(
            "image_to_video",
            vec!["reference_to_video"],
            json!({
                "durationSeconds": { "min": 1, "max": 30, "integer": false },
                "supportedAspectRatios": ["9:16"],
                "resolutions": ["720p"],
                "fps": [24],
                "maxReferenceImages": 1
            }),
        );

        let error = validate_video_option_limits(&option, 4.0, "9:16", Some("720p"), Some(24), 1)
            .expect_err("ability mismatch should fail");

        assert_eq!(error.error_code, "provider.capability_unsupported");
        assert_eq!(
            error.recover_action.as_deref(),
            Some("change_provider_or_plan")
        );
    }

    #[test]
    fn video_generation_rejects_workflow_preset_limits() {
        let root = test_root("video_generation_workflow_limit");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_selected_image(
            &database,
            &workspace_root,
            "project_video_workflow_limit",
        );
        seed_limited_video_workflow(
            &database,
            "workflow_video_limit",
            json!({
                "durationSeconds": { "min": 1, "max": 30, "integer": false },
                "supportedAspectRatios": ["1:1"],
                "resolutions": ["720p"],
                "fpsRange": { "min": 16, "max": 24 },
                "maxReferenceImages": 1
            }),
        );
        let mut request = video_request("project_video_workflow_limit", &item_id, None);
        request.workflow_preset_id = Some("workflow_video_limit".to_string());
        request.aspect_ratio = Some("9:16".to_string());

        let error = start_video_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            request,
        )
        .expect_err("workflow limit overflow should fail");

        assert!(error.contains("provider.limit_exceeded"));
        assert!(error.contains("Aspect ratio"));
        assert_no_video_segments(&database, &item_id);
        cleanup(root);
    }

    #[test]
    fn storage_rejects_escaped_image_candidate_paths() {
        let root = test_root("image_generation_storage_escape");
        let storage = StorageService::new(root.join("workspace"));
        storage
            .initialize_workspace()
            .expect("workspace should init");

        let result = storage.write_bytes(
            FileBucket::Project,
            "../escape.png",
            b"bad",
            FileAccessPolicy::WriteProject,
        );

        assert!(result.is_err());
        cleanup(root);
    }

    fn create_project_with_selected_image(
        database: &Database,
        workspace_root: &Path,
        project_id: &str,
    ) -> String {
        let item_id = create_project_with_one_item(database, workspace_root, project_id);
        let candidates = start_image_generation(
            database,
            workspace_root,
            &KeyringService::memory(),
            StartImageGenerationRequest {
                project_id: project_id.to_string(),
                item_id: item_id.clone(),
                count: Some(1),
                image_kind: None,
                asset_kind: None,
                provider_model_id: None,
                workflow_preset_id: None,
                workflow_params: None,
                aspect_ratio: None,
                width: None,
                height: None,
                seed: None,
            },
        )
        .expect("image generation should pass");
        SceneRepository::new(database)
            .select_image_candidate(&item_id, &candidates[0].image_id)
            .expect("image selection should save");
        item_id
    }

    fn create_project_with_character_reference(
        database: &Database,
        workspace_root: &Path,
        project_id: &str,
    ) -> String {
        create_project_with_character_reference_id(database, workspace_root, project_id, "hero")
    }

    fn create_project_with_character_reference_id(
        database: &Database,
        workspace_root: &Path,
        project_id: &str,
        character_id: &str,
    ) -> String {
        let item_id = create_project_with_one_item(database, workspace_root, project_id);
        fs::create_dir_all(workspace_root.join("assets/character_reference_image"))
            .expect("character reference dir should exist");
        fs::write(
            workspace_root.join("assets/character_reference_image/hero_front.png"),
            "png",
        )
        .expect("character reference should write");
        crate::services::character_service::upsert_project_character_bible(
            database,
            crate::domain::character::UpsertProjectCharacterBibleRequest {
                project_id: project_id.to_string(),
                character_id: Some(character_id.to_string()),
                name: "Hero".to_string(),
                alias: vec![],
                age: "28".to_string(),
                gender: "female".to_string(),
                appearance: "silver hair".to_string(),
                clothing: "red pilot jacket".to_string(),
                personality: "calm".to_string(),
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                lock_flags: None,
            },
        )
        .expect("character should save");
        let asset = AssetRepository::new(database)
            .insert_asset(&NewAssetRecord {
                asset_id: format!("{project_id}_hero_front_asset"),
                kind: "character_reference_image".to_string(),
                relative_path: "assets/character_reference_image/hero_front.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(3),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "mediaKind": "image" }),
            })
            .expect("asset should save");
        crate::services::character_service::bind_character_reference_asset(
            database,
            crate::domain::character::BindCharacterReferenceAssetRequest {
                project_id: project_id.to_string(),
                character_id: character_id.to_string(),
                asset_id: asset.asset_id,
                reference_role: Some("character_front_view".to_string()),
            },
        )
        .expect("character reference should bind");
        let mut item = SceneRepository::new(database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.character_ids = vec![character_id.to_string()];
        update_storyboard_item(database, item).expect("item should update");
        item_id
    }

    fn video_request(
        project_id: &str,
        item_id: &str,
        provider_model_id: Option<&str>,
    ) -> StartVideoGenerationRequest {
        StartVideoGenerationRequest {
            project_id: project_id.to_string(),
            item_id: item_id.to_string(),
            count: Some(1),
            provider_model_id: provider_model_id.map(str::to_string),
            workflow_preset_id: None,
            workflow_params: None,
            aspect_ratio: None,
            resolution: Some("720p".to_string()),
            fps: Some(24),
            seed: None,
        }
    }

    fn seed_limited_video_model(
        database: &Database,
        model_id: &str,
        ability_type: &str,
        limits: Value,
        input_requirements: Value,
    ) {
        ensure_controlled_fake_video_provider(database).expect("fake video provider should seed");
        ProviderRepository::new(database)
            .upsert_provider_model(&ProviderModelRecord {
                model_id: model_id.to_string(),
                provider_id: CONTROLLED_FAKE_VIDEO_PROVIDER_ID.to_string(),
                provider_model_id: format!("controlled-fake/{model_id}"),
                display_name: format!("Limited {model_id}"),
                capability: ability_type.to_string(),
                config_json: json!({
                    "providerKind": "video",
                    "vendor": "controlled_fake",
                    "modelName": model_id,
                    "abilityTypes": [ability_type],
                    "inputModalities": ["text", "image"],
                    "outputModalities": ["video"],
                    "inputRequirements": input_requirements,
                    "limits": limits,
                    "status": "ready"
                }),
                enabled: true,
            })
            .expect("model should save");
    }

    fn seed_limited_image_model(
        database: &Database,
        model_id: &str,
        limits: Value,
        input_requirements: Value,
    ) {
        ensure_controlled_fake_image_provider(database).expect("fake image provider should seed");
        ProviderRepository::new(database)
            .upsert_provider_model(&ProviderModelRecord {
                model_id: model_id.to_string(),
                provider_id: CONTROLLED_FAKE_PROVIDER_ID.to_string(),
                provider_model_id: format!("controlled-fake/{model_id}"),
                display_name: format!("Limited {model_id}"),
                capability: "text_to_image".to_string(),
                config_json: json!({
                    "providerKind": "image",
                    "vendor": "controlled_fake",
                    "modelName": model_id,
                    "abilityTypes": ["text_to_image"],
                    "inputModalities": ["text", "image"],
                    "outputModalities": ["image"],
                    "inputRequirements": input_requirements,
                    "limits": limits,
                    "status": "ready"
                }),
                enabled: true,
            })
            .expect("model should save");
    }

    fn seed_limited_video_workflow(database: &Database, preset_id: &str, limits: Value) {
        ensure_controlled_fake_workflow_provider(database)
            .expect("fake workflow provider should seed");
        ProviderRepository::new(database)
            .upsert_workflow_preset(&crate::db::provider_repository::WorkflowPresetRecord {
                preset_id: preset_id.to_string(),
                provider_id: Some("provider_controlled_fake_workflow".to_string()),
                model_id: None,
                name: format!("Limited {preset_id}"),
                kind: "controlled_fake".to_string(),
                capability: "image_to_video".to_string(),
                config_json: json!({
                    "providerId": "provider_controlled_fake_workflow",
                    "vendor": "controlled_fake",
                    "workflowKey": "controlled-fake/video-workflow.json",
                    "workflowVersion": "1.0.0",
                    "abilityTypes": ["image_to_video"],
                    "inputModalities": ["text", "image"],
                    "outputModalities": ["video"],
                    "paramSchema": {},
                    "nodeMap": { "videoPrompt": "1.inputs.prompt" },
                    "outputMap": { "video": "99.outputs.video" },
                    "defaultParams": {},
                    "limits": limits,
                    "status": "ready"
                }),
                enabled: true,
            })
            .expect("workflow preset should save");
    }

    fn video_option_for_limit_tests(
        ability_type: &str,
        capabilities: Vec<&str>,
        limits: Value,
    ) -> ExecutableMediaOptionDto {
        ExecutableMediaOptionDto {
            option_id: "test_video_option".to_string(),
            source_type: "provider_model".to_string(),
            source_id: "test_video_source".to_string(),
            label: "Test video option".to_string(),
            provider_id: "provider_test_video".to_string(),
            provider_kind: "video".to_string(),
            vendor: "controlled_fake".to_string(),
            kind: "provider_model".to_string(),
            capability: capabilities
                .first()
                .copied()
                .unwrap_or(ability_type)
                .to_string(),
            capabilities: capabilities.into_iter().map(str::to_string).collect(),
            constraints: json!({ "limits": limits }),
            input_plan: crate::domain::media::MediaInputPlanDto {
                plan_kind: "video".to_string(),
                ability_type: ability_type.to_string(),
                image_kind: None,
                asset_kind: None,
                items: vec![],
                required_count: 0,
                optional_count: 0,
                unused_count: 0,
            },
            status: "ready".to_string(),
            provider_model_id: Some("model_test_video".to_string()),
            workflow_preset_id: None,
            enabled: true,
            disabled_reason: None,
            normalized_params: json!({}),
        }
    }

    fn ensure_controlled_fake_workflow_provider(database: &Database) -> Result<(), String> {
        let repository = ProviderRepository::new(database);
        if !repository
            .list_providers()?
            .iter()
            .any(|provider| provider.provider_id == "provider_controlled_fake_workflow")
        {
            repository.upsert_provider(&ProviderRecord {
                provider_id: "provider_controlled_fake_workflow".to_string(),
                vendor: "controlled_fake".to_string(),
                kind: "workflow".to_string(),
                display_name: "Controlled fake workflow provider".to_string(),
                auth_type: "none".to_string(),
                key_alias: None,
                base_url: None,
                status: "ready".to_string(),
                enabled: true,
                config_json: json!({
                    "adapter": "dummy",
                    "externalNetwork": false,
                    "billable": false
                }),
            })?;
        }
        Ok(())
    }

    fn assert_no_video_segments(database: &Database, item_id: &str) {
        let item = SceneRepository::new(database)
            .get_storyboard_item(item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(item.video_status, "failed");
        assert_eq!(item.segment_status, "failed");
        assert!(item.video_segments.is_empty());
    }

    fn create_project_with_one_item(
        database: &Database,
        workspace_root: &Path,
        project_id: &str,
    ) -> String {
        StorageService::new(workspace_root)
            .initialize_workspace()
            .expect("workspace should init");
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: project_id.to_string(),
                    workflow_type: "image_to_video".to_string(),
                    input_type: "topic".to_string(),
                    topic: Some("早睡".to_string()),
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
        let mut item = storyboard_item(
            project_id.to_string(),
            1,
            "很多人以为早睡只是自律。".to_string(),
            "很多人以为早睡只是自律。".to_string(),
        );
        item.item_id = format!("{project_id}_item_1");
        SceneRepository::new(database)
            .upsert_storyboard_item(&item)
            .expect("item should save")
            .item_id
    }

    #[test]
    fn tts_generation_writes_audio_fields_without_requiring_video() {
        let root = test_root("tts_generation_fields");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_one_item(&database, &workspace_root, "project_tts");

        let item = start_tts_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartTtsGenerationRequest {
                project_id: "project_tts".to_string(),
                item_id: item_id.clone(),
                provider_model_id: None,
                voice_id: Some("voice_a".to_string()),
                speed: Some(1.1),
                pitch: None,
                volume: None,
                format: Some("mp3".to_string()),
                sample_rate: Some(24000),
            },
        )
        .expect("tts generation should pass");

        assert_eq!(item.audio_status, "succeeded");
        let expected_audio_path = format!("project_tts/audio/{item_id}/voice.mp3");
        assert_eq!(
            item.audio_path.as_deref(),
            Some(expected_audio_path.as_str())
        );
        assert_eq!(item.audio_duration_seconds, None);
        assert_eq!(item.audio_last_error_json, None);
        assert_eq!(item.audio_retry_count, 0);
        assert!(item.video_segments.is_empty());
        cleanup(root);
    }

    #[test]
    fn tts_generation_failure_records_error_and_retry_count() {
        let root = test_root("tts_generation_failure");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_one_item(&database, &workspace_root, "project_tts_fail");
        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.narration_text.clear();
        item.source_text.clear();
        SceneRepository::new(&database)
            .upsert_storyboard_item(&item)
            .expect("item should update");

        let error = start_tts_generation(
            &database,
            &workspace_root,
            &KeyringService::memory(),
            StartTtsGenerationRequest {
                project_id: "project_tts_fail".to_string(),
                item_id: item_id.clone(),
                provider_model_id: None,
                voice_id: None,
                speed: None,
                pitch: None,
                volume: None,
                format: None,
                sample_rate: None,
            },
        )
        .expect_err("missing narration text should fail");

        assert!(error.contains("no narration_text or source_text"));
        let updated = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(updated.audio_status, "failed");
        assert_eq!(updated.audio_retry_count, 1);
        assert_eq!(
            updated
                .audio_last_error_json
                .as_ref()
                .and_then(|value| value.get("errorCode"))
                .and_then(Value::as_str),
            Some("validation.invalid_input")
        );
        cleanup(root);
    }

    #[test]
    fn replace_storyboard_audio_copies_file_into_workspace() {
        let root = test_root("replace_storyboard_audio");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_one_item(&database, &workspace_root, "project_audio_upload");
        let source_dir = root.join("user-picked");
        fs::create_dir_all(&source_dir).expect("source dir should exist");
        let source_path = source_dir.join("voice.mp3");
        fs::write(&source_path, b"fake audio bytes").expect("audio fixture should write");

        let item = replace_storyboard_audio(
            &database,
            &workspace_root,
            ReplaceStoryboardAudioRequest {
                project_id: "project_audio_upload".to_string(),
                item_id: item_id.clone(),
                source_path: source_path.to_string_lossy().to_string(),
            },
        )
        .expect("audio replacement should pass");

        let expected_audio_path =
            format!("projects/project_audio_upload/audio/{item_id}/uploaded.mp3");
        assert_eq!(item.audio_status, "succeeded");
        assert_eq!(
            item.audio_path.as_deref(),
            Some(expected_audio_path.as_str())
        );
        assert_eq!(item.audio_duration_seconds, None);
        assert!(workspace_root.join(expected_audio_path).is_file());
        assert_ne!(
            item.audio_path.as_deref(),
            Some(source_path.to_string_lossy().as_ref())
        );
        cleanup(root);
    }

    #[test]
    fn generate_subtitles_uses_audio_duration_and_writes_file_without_changing_narration() {
        let root = test_root("generate_subtitles");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id = create_project_with_one_item(&database, &workspace_root, "project_subtitles");
        let mut item = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        item.narration_text =
            "很多人以为早睡只是自律，其实它先改变的是你的清醒感。规律作息会让注意力更稳定。"
                .to_string();
        item.audio_duration_seconds = Some(6.0);
        SceneRepository::new(&database)
            .upsert_storyboard_item(&item)
            .expect("item should update");

        let result = generate_subtitles(
            &database,
            &workspace_root,
            GenerateSubtitlesRequest {
                project_id: "project_subtitles".to_string(),
                item_ids: None,
            },
        )
        .expect("subtitle generation should pass");

        let updated = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(updated.narration_text, item.narration_text);
        assert_eq!(updated.subtitle_status, "succeeded");
        assert_eq!(updated.render_status, "pending");
        assert!(!updated.subtitle_chunks.is_empty());
        assert_eq!(
            updated
                .subtitle_chunks
                .last()
                .and_then(|chunk| chunk.end_seconds),
            Some(6.0)
        );
        assert!(workspace_root
            .join("projects/project_subtitles/subtitles/subtitles.json")
            .is_file());
        assert_eq!(
            result.subtitle_path,
            "projects/project_subtitles/subtitles/subtitles.json"
        );
        assert!(result.subtitles.chunks.iter().all(|chunk| chunk.estimated));
        assert_eq!(result.subtitles.style.mode, "karaoke_estimated");
        assert!(result
            .subtitles
            .chunks
            .iter()
            .all(|chunk| !chunk.word_timings.is_empty()
                && chunk.word_timings.iter().all(|word| word.estimated)));
        cleanup(root);
    }

    #[test]
    fn update_storyboard_subtitles_only_updates_chunks() {
        let root = test_root("update_storyboard_subtitles");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let item_id =
            create_project_with_one_item(&database, &workspace_root, "project_subtitle_edit");
        let original = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");

        update_storyboard_subtitles(
            &database,
            &workspace_root,
            UpdateStoryboardSubtitlesRequest {
                project_id: "project_subtitle_edit".to_string(),
                item_id: item_id.clone(),
                subtitle_chunks: vec![
                    SubtitleChunkDto {
                        chunk_id: "ignored_1".to_string(),
                        text: "第一行字幕".to_string(),
                        start_seconds: None,
                        end_seconds: None,
                        estimated: false,
                    },
                    SubtitleChunkDto {
                        chunk_id: "ignored_2".to_string(),
                        text: "第二行字幕".to_string(),
                        start_seconds: None,
                        end_seconds: None,
                        estimated: false,
                    },
                ],
            },
        )
        .expect("subtitle edit should pass");

        let updated = SceneRepository::new(&database)
            .get_storyboard_item(&item_id)
            .expect("item should read")
            .expect("item should exist");
        assert_eq!(updated.narration_text, original.narration_text);
        assert_eq!(updated.source_text, original.source_text);
        assert_eq!(
            updated
                .subtitle_chunks
                .iter()
                .map(|chunk| chunk.text.as_str())
                .collect::<Vec<_>>(),
            vec!["第一行字幕", "第二行字幕"]
        );
        assert!(updated.subtitle_chunks.iter().all(|chunk| chunk.estimated));
        cleanup(root);
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-scene-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }

    fn assert_snapshot_has_no_sensitive_or_absolute_text(value: &Value) {
        let serialized = serde_json::to_string(value).expect("snapshot should serialize");
        let lowered = serialized.to_ascii_lowercase();
        for forbidden in [
            "authorization",
            "secret",
            "token",
            "password",
            "api_key",
            "apikey",
        ] {
            assert!(
                !lowered.contains(forbidden),
                "snapshot leaked forbidden term: {forbidden} in {serialized}"
            );
        }
        assert!(!serialized.contains("C:\\"));
        assert!(!serialized.contains("D:\\"));
        assert!(!serialized.contains("/Users/"));
        assert!(!serialized.contains("/home/"));
    }
}
