use crate::db::project_repository::ProjectRepository;
use crate::db::task_repository::TaskRepository;
use crate::db::Database;
use crate::domain::digital_human::{DigitalHumanProjectStateDto, StartDigitalHumanVideoRequest};
use serde_json::json;

pub fn get_digital_human_project_state(
    database: &Database,
    project_id: String,
) -> Result<DigitalHumanProjectStateDto, String> {
    ensure_digital_human_project(database, &project_id)?;
    let task = TaskRepository::new(database)
        .get_latest_task_detail_by_project(&project_id)?
        .ok_or_else(|| format!("Task not found for project: {project_id}"))?;
    let tts_step = task
        .steps
        .iter()
        .find(|step| step.step_name == "tts_generation");
    let video_step = task
        .steps
        .iter()
        .find(|step| step.step_name == "digital_human_generation");
    let reference_audio_path = tts_step
        .and_then(|step| step.output_json.as_ref())
        .and_then(|value| value.get("referenceAudioPath"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let output_video_path = video_step
        .and_then(|step| step.output_json.as_ref())
        .and_then(|value| value.get("outputVideoPath"))
        .and_then(|value| value.as_str())
        .map(str::to_string);

    Ok(DigitalHumanProjectStateDto {
        project_id,
        tts_status: tts_step
            .map(|step| step.status.clone())
            .unwrap_or_else(|| "pending".to_string()),
        video_status: video_step
            .map(|step| step.status.clone())
            .unwrap_or_else(|| "pending".to_string()),
        reference_image_path: None,
        reference_audio_path,
        output_video_path,
    })
}

pub fn mark_digital_human_tts_succeeded(
    database: &Database,
    project_id: String,
    reference_audio_path: String,
) -> Result<DigitalHumanProjectStateDto, String> {
    ensure_digital_human_project(database, &project_id)?;
    if reference_audio_path.trim().is_empty() {
        return Err("reference_audio_path is required.".to_string());
    }
    TaskRepository::new(database).update_latest_project_step_status(
        &project_id,
        "tts_generation",
        "succeeded",
        Some(json!({ "referenceAudioPath": reference_audio_path })),
    )?;
    get_digital_human_project_state(database, project_id)
}

pub fn mark_digital_human_tts_failed(
    database: &Database,
    project_id: String,
    error_reason: String,
) -> Result<DigitalHumanProjectStateDto, String> {
    ensure_digital_human_project(database, &project_id)?;
    TaskRepository::new(database).update_latest_project_step_status(
        &project_id,
        "tts_generation",
        "failed",
        Some(json!({ "errorReason": error_reason })),
    )?;
    get_digital_human_project_state(database, project_id)
}

pub fn start_digital_human_video(
    database: &Database,
    request: StartDigitalHumanVideoRequest,
) -> Result<DigitalHumanProjectStateDto, String> {
    ensure_digital_human_project(database, &request.project_id)?;
    if request.prompt.trim().is_empty() {
        return Err("prompt is required for digital human video.".to_string());
    }
    let state = get_digital_human_project_state(database, request.project_id.clone())?;
    if state.tts_status != "succeeded" {
        TaskRepository::new(database).update_latest_project_step_status(
            &request.project_id,
            "digital_human_generation",
            "failed",
            Some(json!({
                "blockedBy": "tts_generation",
                "reason": "TTS must succeed before digital human video generation."
            })),
        )?;
        return Err("TTS must succeed before digital human video generation.".to_string());
    }

    TaskRepository::new(database).update_latest_project_step_status(
        &request.project_id,
        "digital_human_generation",
        "succeeded",
        Some(json!({
            "referenceImagePath": request.reference_image_path,
            "referenceAudioPath": state.reference_audio_path,
            "prompt": request.prompt,
            "outputVideoPath": format!("projects/{}/digital-human/output.mp4", request.project_id)
        })),
    )?;
    get_digital_human_project_state(database, request.project_id)
}

fn ensure_digital_human_project(database: &Database, project_id: &str) -> Result<(), String> {
    let project = ProjectRepository::new(database)
        .get_detail(project_id)?
        .ok_or_else(|| format!("Project not found: {project_id}"))?
        .project;
    if project.workflow_type != "digital_human" {
        return Err("digital human actions require workflow_type=digital_human.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::project::CreateProjectRequest;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn digital_human_project_has_independent_steps() {
        let root = test_root("digital_human_steps");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_digital_human_project(&database, "project_dh_steps");

        let task = TaskRepository::new(&database)
            .get_latest_task_detail_by_project("project_dh_steps")
            .expect("task should read")
            .expect("task should exist");
        assert_eq!(task.current_step.as_deref(), Some("script_review"));
        assert!(task
            .steps
            .iter()
            .any(|step| step.step_name == "digital_human_generation"));
        assert!(!task
            .steps
            .iter()
            .any(|step| step.step_name == "storyboard_generation"));

        cleanup(root);
    }

    #[test]
    fn digital_human_video_is_blocked_when_tts_failed() {
        let root = test_root("digital_human_tts_gate");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_digital_human_project(&database, "project_dh_gate");

        mark_digital_human_tts_failed(
            &database,
            "project_dh_gate".to_string(),
            "provider timeout".to_string(),
        )
        .expect("tts can fail");
        let error = start_digital_human_video(
            &database,
            StartDigitalHumanVideoRequest {
                project_id: "project_dh_gate".to_string(),
                reference_image_path: Some("assets/person.png".to_string()),
                prompt: "口播".to_string(),
            },
        )
        .expect_err("video should be blocked");
        assert!(error.contains("TTS must succeed"));

        let state = mark_digital_human_tts_succeeded(
            &database,
            "project_dh_gate".to_string(),
            "projects/project_dh_gate/audio/narration.mp3".to_string(),
        )
        .expect("tts can succeed");
        assert_eq!(state.tts_status, "succeeded");
        let state = start_digital_human_video(
            &database,
            StartDigitalHumanVideoRequest {
                project_id: "project_dh_gate".to_string(),
                reference_image_path: Some("assets/person.png".to_string()),
                prompt: "口播".to_string(),
            },
        )
        .expect("video can start after tts");
        assert_eq!(state.video_status, "succeeded");
        assert!(state.output_video_path.is_some());

        cleanup(root);
    }

    fn create_digital_human_project(database: &Database, project_id: &str) {
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "Digital human".to_string(),
                    workflow_type: "digital_human".to_string(),
                    input_type: "paste".to_string(),
                    topic: None,
                    source_text: Some("口播文案".to_string()),
                    source_text_path: None,
                    content_language: "zh-CN".to_string(),
                    tone: None,
                    aspect_ratio: "9:16".to_string(),
                    target_scene_count: 1,
                    segment_duration_seconds: 20.0,
                    style_prompt: None,
                    active_pack_id: None,
                    rule_refs: None,
                    executable_refs: None,
                    input_process_mode: "fixed".to_string(),
                    input_options: None,
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
