use crate::domain::task::{TaskDetailDto, TaskStepDto};
use crate::security::secret_guard::redact_text;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

pub const TASK_PROGRESS_EVENT: &str = "task://progress";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub trace_id: String,
    pub project_id: String,
    pub task_id: String,
    pub task_step_id: Option<String>,
    pub step_kind: Option<String>,
    pub status: String,
    pub progress: u8,
    pub message: String,
    pub error_code: Option<String>,
    pub item_id: Option<String>,
}

impl ProgressEvent {
    pub fn from_task_detail(detail: &TaskDetailDto, message: impl Into<String>) -> Self {
        let current_step = detail
            .current_step
            .as_deref()
            .and_then(|step_name| detail.steps.iter().find(|step| step.step_name == step_name))
            .or_else(|| terminal_step(detail));

        Self {
            trace_id: create_trace_id(),
            project_id: detail.project_id.clone(),
            task_id: detail.task_id.clone(),
            task_step_id: current_step.map(|step| step.step_id.clone()),
            step_kind: current_step.map(|step| step.step_name.clone()),
            status: detail.task_status.clone(),
            progress: task_progress(&detail.steps),
            message: redact_text(&message.into()),
            error_code: None,
            item_id: None,
        }
    }
}

pub fn emit_task_progress(app_handle: &AppHandle, event: ProgressEvent) -> Result<(), String> {
    app_handle
        .emit(TASK_PROGRESS_EVENT, event)
        .map_err(|error| format!("Failed to emit task progress event: {error}"))
}

fn task_progress(steps: &[TaskStepDto]) -> u8 {
    if steps.is_empty() {
        return 0;
    }

    let finished = steps
        .iter()
        .filter(|step| matches!(step.status.as_str(), "succeeded" | "skipped" | "cancelled"))
        .count();
    ((finished * 100) / steps.len()).min(100) as u8
}

fn terminal_step(detail: &TaskDetailDto) -> Option<&TaskStepDto> {
    detail
        .steps
        .iter()
        .rev()
        .find(|step| matches!(step.status.as_str(), "succeeded" | "failed" | "cancelled"))
}

fn create_trace_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("trace_{nanos}")
}

#[cfg(test)]
mod tests {
    use super::{task_progress, ProgressEvent};
    use crate::domain::task::{TaskDetailDto, TaskStepDto};

    #[test]
    fn progress_event_redacts_message_and_uses_task_state() {
        let detail = TaskDetailDto {
            task_id: "task_event".to_string(),
            project_id: "project_event".to_string(),
            task_status: "running".to_string(),
            current_step: Some("image_generation".to_string()),
            steps: vec![
                step("step_a", "project_init", "succeeded"),
                step("step_b", "storyboard_generation", "succeeded"),
                step("step_c", "image_generation", "running"),
                step("step_d", "video_generation", "pending"),
            ],
            composition_task: None,
        };

        let event = ProgressEvent::from_task_detail(
            &detail,
            "Authorization: Bearer sk-live-secret-token-123456",
        );

        assert_eq!(event.task_id, "task_event");
        assert_eq!(event.project_id, "project_event");
        assert_eq!(event.task_step_id.as_deref(), Some("step_c"));
        assert_eq!(event.step_kind.as_deref(), Some("image_generation"));
        assert_eq!(event.status, "running");
        assert_eq!(event.progress, 50);
        assert!(!event.message.contains("sk-live"));
    }

    #[test]
    fn task_progress_counts_terminal_steps_only() {
        let steps = vec![
            step("step_a", "project_init", "succeeded"),
            step("step_b", "storyboard_generation", "skipped"),
            step("step_c", "image_generation", "cancelled"),
            step("step_d", "video_generation", "failed"),
        ];

        assert_eq!(task_progress(&steps), 75);
    }

    fn step(step_id: &str, step_name: &str, status: &str) -> TaskStepDto {
        TaskStepDto {
            step_id: step_id.to_string(),
            step_name: step_name.to_string(),
            status: status.to_string(),
            output_json: None,
        }
    }
}
