use crate::db::asset_repository::AssetRepository;
use crate::db::project_repository::ProjectRepository;
use crate::db::scene_repository::SceneRepository;
use crate::db::task_repository::{NewCompositionTaskRecord, TaskRepository};
use crate::db::Database;
use crate::domain::media::MediaProbeDto;
use crate::domain::scene::VideoSegmentDto;
use crate::domain::task::{
    CompositionTaskDto, ListTasksRequest, RetryTaskStepRequest, StartCompositionRequest,
    TaskDetailDto, TaskProjectRequest, TaskSummaryDto,
};
use crate::services::ffmpeg_service;
use crate::services::task_cancellation::ProcessHandleRegistry;
use serde_json::{json, Value};
use std::path::Path;

pub fn create_task(database: &Database, project_id: String) -> Result<TaskDetailDto, String> {
    let repository = TaskRepository::new(database);
    if let Some(existing) = repository.get_latest_task_detail_by_project(&project_id)? {
        return Ok(existing);
    }

    repository.create_image_to_video_task(&project_id)
}

pub fn get_task_detail(database: &Database, project_id: String) -> Result<TaskDetailDto, String> {
    TaskRepository::new(database)
        .get_latest_task_detail_by_project(&project_id)?
        .ok_or_else(|| format!("Task not found for project: {project_id}"))
}

pub fn approve_task_step(
    database: &Database,
    project_id: String,
    step_name: String,
) -> Result<TaskDetailDto, String> {
    TaskRepository::new(database).approve_step(&project_id, &step_name)
}

pub fn start_task(
    database: &Database,
    request: TaskProjectRequest,
) -> Result<TaskDetailDto, String> {
    TaskRepository::new(database).start_task(&request.project_id)
}

pub fn cancel_task_with_registry(
    database: &Database,
    registry: &ProcessHandleRegistry,
    request: TaskProjectRequest,
) -> Result<TaskDetailDto, String> {
    let repository = TaskRepository::new(database);
    let current = repository
        .get_latest_task_detail_by_project(&request.project_id)?
        .ok_or_else(|| format!("Task not found for project: {}", request.project_id))?;
    repository.request_cancellation(&current.task_id, Some("user_requested"))?;
    registry.abort_task(&current.task_id)?;
    repository.complete_cancelled_step(
        &current.task_id,
        current.current_step.as_deref().unwrap_or("cleanup"),
    )
}

pub fn resume_task(
    database: &Database,
    request: TaskProjectRequest,
) -> Result<TaskDetailDto, String> {
    TaskRepository::new(database).resume_task(&request.project_id)
}

pub fn retry_task_step(
    database: &Database,
    request: RetryTaskStepRequest,
) -> Result<TaskDetailDto, String> {
    TaskRepository::new(database).retry_task_step(&request.project_id, &request.step_name)
}

pub fn list_tasks(
    database: &Database,
    request: ListTasksRequest,
) -> Result<Vec<TaskSummaryDto>, String> {
    TaskRepository::new(database).list_tasks(request.project_id.as_deref())
}

pub fn start_composition(
    database: &Database,
    workspace_root: &Path,
    request: StartCompositionRequest,
) -> Result<CompositionTaskDto, String> {
    ffmpeg_service::require_ffmpeg_sidecars(workspace_root)?;
    let selected_segments = probe_selected_video_segments_for_composition(
        database,
        workspace_root,
        &request.project_id,
    )?;
    let task_id = create_composition_task_id();
    let segment_ids = selected_segments
        .segments
        .iter()
        .map(|segment| segment.segment_id.clone())
        .collect::<Vec<_>>();
    let segment_paths = selected_segments
        .segments
        .iter()
        .map(|segment| segment.video_path.clone())
        .collect::<Vec<_>>();
    let concat_output_path = ffmpeg_service::concat_segments_with_probes(
        workspace_root,
        &request.project_id,
        &task_id,
        &segment_paths,
        &selected_segments.probes,
    )?;
    let mut output_path = concat_output_path.clone();
    let mut steps = vec![json!({
        "step": "concat",
        "status": "succeeded",
        "outputPath": concat_output_path,
    })];
    let subtitle_path = resolve_subtitle_path(&request);
    if let Some(subtitle_path) = subtitle_path.clone() {
        output_path = ffmpeg_service::burn_subtitles_into_video(
            workspace_root,
            &request.project_id,
            &task_id,
            &output_path,
            &subtitle_path,
        )
        .map_err(|error| format!("composition.subtitle_failed: {error}"))?;
        steps.push(json!({
            "step": "subtitle",
            "status": "succeeded",
            "subtitlePath": subtitle_path,
            "outputPath": output_path,
        }));
    } else {
        steps.push(json!({
            "step": "subtitle",
            "status": "skipped",
        }));
    }
    let bgm_mix = resolve_bgm_mix_options(database, &request, &selected_segments)?;
    let mut bgm_enhancement = json!({ "includeBgm": false });
    if let Some(bgm_mix) = bgm_mix {
        output_path = ffmpeg_service::mix_bgm_into_video(
            workspace_root,
            &request.project_id,
            &task_id,
            &output_path,
            &bgm_mix.options,
        )
        .map_err(|error| format!("composition.bgm_failed: {error}"))?;
        bgm_enhancement = json!({
            "includeBgm": true,
            "bgmAssetId": bgm_mix.asset_id,
            "bgmPath": bgm_mix.options.bgm_relative_path,
            "bgmVolume": bgm_mix.options.volume,
            "bgmLoop": bgm_mix.options.loop_bgm,
            "bgmFadeInSeconds": bgm_mix.options.fade_in_seconds,
            "bgmFadeOutSeconds": bgm_mix.options.fade_out_seconds,
            "durationSeconds": bgm_mix.options.duration_seconds,
        });
        steps.push(json!({
            "step": "bgm",
            "status": "succeeded",
            "bgmAssetId": bgm_enhancement.get("bgmAssetId").cloned().unwrap_or(Value::Null),
            "outputPath": output_path,
        }));
    } else {
        steps.push(json!({
            "step": "bgm",
            "status": "skipped",
        }));
    }
    let cover_metadata = resolve_cover_metadata(database, &request)?;
    if cover_metadata
        .get("includeCoverMetadata")
        .and_then(Value::as_bool)
        == Some(true)
    {
        steps.push(json!({
            "step": "cover_metadata",
            "status": "succeeded",
            "coverPath": cover_metadata.get("coverPath").cloned().unwrap_or(Value::Null),
        }));
    } else {
        steps.push(json!({
            "step": "cover_metadata",
            "status": "skipped",
        }));
    };
    let enhancements = json!({
        "includeSubtitle": subtitle_path.is_some(),
        "subtitlePath": subtitle_path,
        "includeCoverMetadata": cover_metadata.get("includeCoverMetadata").and_then(Value::as_bool).unwrap_or(false),
        "coverPath": cover_metadata.get("coverPath").cloned().unwrap_or(Value::Null),
        "steps": steps,
        "bgm": bgm_enhancement,
        "includeBgm": bgm_enhancement.get("includeBgm").and_then(Value::as_bool).unwrap_or(false),
        "bgmAssetId": bgm_enhancement.get("bgmAssetId").cloned().unwrap_or(Value::Null),
        "bgmPath": bgm_enhancement.get("bgmPath").cloned().unwrap_or(Value::Null),
        "bgmVolume": bgm_enhancement.get("bgmVolume").cloned().unwrap_or(Value::Null),
        "bgmLoop": bgm_enhancement.get("bgmLoop").cloned().unwrap_or(Value::Null),
        "bgmFadeInSeconds": bgm_enhancement.get("bgmFadeInSeconds").cloned().unwrap_or(Value::Null),
        "bgmFadeOutSeconds": bgm_enhancement.get("bgmFadeOutSeconds").cloned().unwrap_or(Value::Null),
    });

    TaskRepository::new(database).upsert_composition_task(&NewCompositionTaskRecord {
        task_id,
        project_id: request.project_id,
        segment_ids,
        output_path,
        enhancements,
        status: "succeeded".to_string(),
        progress: 100,
        error_json: None,
    })
}

fn resolve_subtitle_path(request: &StartCompositionRequest) -> Option<String> {
    if request.include_subtitle != Some(true) {
        return None;
    }
    request
        .subtitle_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            Some(format!(
                "projects/{}/subtitles/subtitles.json",
                sanitize_path_segment(&request.project_id)
            ))
        })
}

fn resolve_cover_metadata(
    database: &Database,
    request: &StartCompositionRequest,
) -> Result<Value, String> {
    if request.include_cover_metadata != Some(true) {
        return Ok(json!({ "includeCoverMetadata": false }));
    }
    let detail = ProjectRepository::new(database)
        .get_detail(&request.project_id)?
        .ok_or_else(|| format!("Project not found: {}", request.project_id))?;
    let cover_path = request
        .cover_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or(detail.project.cover_path)
        .ok_or_else(|| {
            "composition.cover_missing: generate or upload a cover before including cover metadata."
                .to_string()
        })?;
    if !cover_path.starts_with("projects/") {
        return Err(
            "composition.cover_path_invalid: cover path must live inside projects/.".to_string(),
        );
    }
    Ok(json!({
        "includeCoverMetadata": true,
        "coverPath": cover_path,
        "coverTitle": detail.project.cover_title,
        "coverTemplateId": detail.project.cover_template_id,
        "coverSourceItemId": detail.project.cover_source_item_id,
    }))
}

struct ResolvedBgmMix {
    asset_id: String,
    options: ffmpeg_service::BgmMixOptions,
}

fn resolve_bgm_mix_options(
    database: &Database,
    request: &StartCompositionRequest,
    selected_segments: &SelectedCompositionSegments,
) -> Result<Option<ResolvedBgmMix>, String> {
    if request.include_bgm != Some(true) {
        return Ok(None);
    }
    let asset_id = request
        .bgm_asset_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "composition.bgm_asset_required: select a BGM asset before composition.".to_string()
        })?;
    let asset = AssetRepository::new(database)
        .get_asset(asset_id)?
        .ok_or_else(|| format!("composition.bgm_asset_not_found: {asset_id}"))?;
    if asset.lifecycle == "deleted" {
        return Err(format!("composition.bgm_asset_deleted: {asset_id}"));
    }
    if asset.kind != "bgm" {
        return Err(format!(
            "composition.bgm_asset_invalid_kind: expected bgm, got {}",
            asset.kind
        ));
    }
    if !asset.relative_path.starts_with("assets/") {
        return Err(
            "composition.bgm_asset_path_invalid: BGM asset must live inside assets/.".to_string(),
        );
    }

    let duration_seconds = selected_segments
        .probes
        .iter()
        .map(|probe| probe.duration_seconds)
        .sum::<f64>();
    let video_has_audio = selected_segments
        .probes
        .iter()
        .any(|probe| probe.has_audio_stream);

    Ok(Some(ResolvedBgmMix {
        asset_id: asset.asset_id,
        options: ffmpeg_service::BgmMixOptions {
            bgm_relative_path: asset.relative_path,
            volume: normalize_bgm_volume(request.bgm_volume),
            loop_bgm: request.bgm_loop.unwrap_or(true),
            fade_in_seconds: normalize_bgm_fade_seconds(request.bgm_fade_in_seconds),
            fade_out_seconds: normalize_bgm_fade_seconds(request.bgm_fade_out_seconds),
            duration_seconds,
            video_has_audio,
        },
    }))
}

fn normalize_bgm_volume(value: Option<f64>) -> f64 {
    value
        .filter(|value| value.is_finite())
        .unwrap_or(0.18)
        .clamp(0.0, 1.0)
}

fn normalize_bgm_fade_seconds(value: Option<f64>) -> f64 {
    value
        .filter(|value| value.is_finite())
        .unwrap_or(0.0)
        .clamp(0.0, 30.0)
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

fn probe_selected_video_segments_for_composition(
    database: &Database,
    workspace_root: &Path,
    project_id: &str,
) -> Result<SelectedCompositionSegments, String> {
    let repository = SceneRepository::new(database);
    let storyboard = repository
        .get_storyboard(project_id)?
        .ok_or_else(|| format!("Project not found: {project_id}"))?;
    let mut selected_segments = Vec::new();

    for item in storyboard.items {
        let segment_id = item.selected_video_segment_id.ok_or_else(|| {
            format!(
                "Project {} requires every StoryboardItem to have selected_video_segment_id before starting composition.",
                project_id
            )
        })?;
        let segment = item
            .video_segments
            .into_iter()
            .find(|segment| segment.segment_id == segment_id)
            .ok_or_else(|| format!("Selected video segment not found: {segment_id}"))?;
        selected_segments.push(segment);
    }

    let probes = ffmpeg_service::probe_video_segments(workspace_root, &selected_segments)?;
    for (segment, probe) in selected_segments.iter().zip(probes.iter()) {
        repository.update_video_segment_media_probe(&segment.segment_id, probe)?;
    }

    Ok(SelectedCompositionSegments {
        segments: selected_segments,
        probes,
    })
}

struct SelectedCompositionSegments {
    segments: Vec<VideoSegmentDto>,
    probes: Vec<MediaProbeDto>,
}

fn create_composition_task_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("composition_task_{nanos}")
}

#[cfg(test)]
mod tests {
    use super::{cancel_task_with_registry, start_composition};
    use crate::db::task_repository::TaskRepository;
    use crate::db::Database;
    use crate::domain::task::{StartCompositionRequest, TaskProjectRequest};
    use crate::services::task_cancellation::{CancellableProcessHandle, ProcessHandleRegistry};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct FakeHandle {
        abort_count: Arc<AtomicUsize>,
    }

    impl CancellableProcessHandle for FakeHandle {
        fn abort(&self) -> Result<(), String> {
            self.abort_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn cancel_task_with_registry_aborts_handles_and_is_idempotent() {
        let path = test_database_path("task_service_cancel_registry");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_cancel_registry", "Cancel registry");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_cancel_registry")
            .expect("task should create");
        repository
            .approve_step("project_cancel_registry", "storyboard_review")
            .expect("task should move to auto step");

        let registry = ProcessHandleRegistry::new();
        let abort_count = Arc::new(AtomicUsize::new(0));
        registry
            .register(
                &task.task_id,
                Arc::new(FakeHandle {
                    abort_count: Arc::clone(&abort_count),
                }),
            )
            .expect("fake handle should register");

        let request = TaskProjectRequest {
            project_id: "project_cancel_registry".to_string(),
        };
        let cancelled = cancel_task_with_registry(&database, &registry, request)
            .expect("task should cancel through registry");
        assert_eq!(cancelled.task_status, "cancelled");
        assert_eq!(abort_count.load(Ordering::SeqCst), 1);

        let request = TaskProjectRequest {
            project_id: "project_cancel_registry".to_string(),
        };
        let cancelled_again = cancel_task_with_registry(&database, &registry, request)
            .expect("second cancellation should be idempotent");
        assert_eq!(cancelled_again.task_status, "cancelled");
        assert_eq!(abort_count.load(Ordering::SeqCst), 1);

        cleanup(path);
    }

    #[test]
    fn start_composition_requires_ffmpeg_sidecars_first() {
        let database_path = test_database_path("composition_missing_sidecar_db");
        let database = Database::open(&database_path).expect("database should open");
        let workspace_root = test_workspace_root("composition_missing_sidecar");
        let error = start_composition(
            &database,
            &workspace_root,
            StartCompositionRequest {
                project_id: "project_composition_missing_sidecar".to_string(),
                include_subtitle: None,
                subtitle_path: None,
                include_bgm: None,
                bgm_asset_id: None,
                bgm_volume: None,
                bgm_loop: None,
                bgm_fade_in_seconds: None,
                bgm_fade_out_seconds: None,
                include_cover_metadata: None,
                cover_path: None,
            },
        )
        .expect_err("missing ffmpeg sidecars should block composition");

        assert!(error.starts_with("ffmpeg.not_found:"));
        assert!(error.contains("sidecars/ffmpeg.exe"));
        assert!(!error.contains(&workspace_root.display().to_string()));

        let _ = fs::remove_dir_all(workspace_root);
        cleanup(database_path);
    }

    fn insert_project(database: &Database, project_id: &str, title: &str) {
        database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO projects (
                        project_id, title, workflow_type, input_type, input_process_mode,
                        aspect_ratio, target_scene_count, segment_duration_seconds,
                        content_language, lifecycle
                    )
                    VALUES (?1, ?2, 'image_to_video', 'topic', 'generate', '9:16', 1, 4, 'zh-CN', 'draft')
                    "#,
                    (project_id, title),
                )
            })
            .expect("project fixture should save");
    }

    fn test_database_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-service-{name}-{}-{nanos}.sqlite3",
            std::process::id()
        ))
    }

    fn test_workspace_root(name: &str) -> PathBuf {
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
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
    }
}
