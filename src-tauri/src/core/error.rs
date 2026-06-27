use crate::security::secret_guard::{redact_json, redact_text};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorDto {
    pub code: String,
    pub kind: String,
    pub message: String,
    pub detail: Option<Value>,
    pub is_retryable: bool,
    pub recover_action: Option<String>,
    pub trace_id: String,
}

impl AppErrorDto {
    pub fn new(code: &str, kind: &str, message: String) -> Self {
        let task_error = TaskError::from_code(code, message);
        Self {
            code: task_error.error_code,
            kind: kind.to_string(),
            message: task_error.message,
            detail: None,
            is_retryable: task_error.is_retryable,
            recover_action: task_error.recover_action,
            trace_id: task_error.trace_id,
        }
    }

    pub fn from_message(message: String) -> Self {
        let message = redact_text(&message);
        if let Some((code, detail_message)) = parse_prefixed_error_code(&message) {
            return TaskError::from_code(code, detail_message.to_string()).into();
        }
        let lowered = message.to_lowercase();
        let is_validation = lowered.contains("must")
            || lowered.contains("missing")
            || lowered.contains("requires")
            || lowered.contains("not found")
            || lowered.contains("no selected");

        if is_validation {
            return Self {
                detail: Some(json!({ "source": "command_boundary" })),
                ..Self::new("validation.failed", "validation", message)
            };
        }

        Self {
            detail: Some(json!({ "source": "command_boundary" })),
            is_retryable: true,
            recover_action: Some("retry".to_string()),
            ..Self::new("app.command_failed", "unknown", message)
        }
    }
}

impl AppErrorDto {
    #[allow(dead_code)]
    pub fn sanitized(mut self) -> Self {
        self.message = redact_text(&self.message);
        self.detail = self.detail.map(|detail| redact_json(&detail));
        self
    }
}

impl From<TaskError> for AppErrorDto {
    fn from(value: TaskError) -> Self {
        Self {
            code: value.error_code,
            kind: value.error_kind,
            message: value.message,
            detail: value.detail,
            is_retryable: value.is_retryable,
            recover_action: value.recover_action,
            trace_id: value.trace_id,
        }
    }
}

impl From<String> for AppErrorDto {
    fn from(value: String) -> Self {
        Self::from_message(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskError {
    pub error_code: String,
    pub error_kind: String,
    pub message: String,
    pub detail: Option<Value>,
    pub is_retryable: bool,
    pub recover_action: Option<String>,
    pub trace_id: String,
}

impl TaskError {
    pub fn from_code(code: &str, message: String) -> Self {
        Self::from_code_with_detail(code, message, None)
    }

    pub fn from_code_with_detail(code: &str, message: String, detail: Option<Value>) -> Self {
        let kind = error_kind_from_code(code);
        let is_retryable = is_retryable_error(code, kind);
        Self {
            error_code: code.to_string(),
            error_kind: kind.to_string(),
            message: redact_text(&message),
            detail: detail.map(|value| redact_json(&value)),
            is_retryable,
            recover_action: recover_action_for(code, kind, is_retryable),
            trace_id: create_trace_id(),
        }
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| json!({}))
    }
}

fn error_kind_from_code(code: &str) -> &'static str {
    match code.split_once('.').map(|(domain, _)| domain) {
        Some("task") => "unknown",
        Some("template") => "template",
        Some("storage") => "storage",
        Some("provider") => "provider",
        Some("workflow") => "workflow",
        Some("ffmpeg") => "ffmpeg",
        Some("db") => "db",
        Some("security") => "security",
        Some("export") => "export",
        Some("import") => "import",
        Some("backup") => "backup",
        Some("diagnostic") => "diagnostic",
        Some("validation") => "validation",
        Some("auth") => "auth",
        Some("rate_limit") => "rate_limit",
        Some("network") => "network",
        _ => "unknown",
    }
}

fn is_retryable_error(code: &str, kind: &str) -> bool {
    matches!(
        code,
        "provider.timeout"
            | "provider.rate_limited"
            | "provider.server_error"
            | "provider.invalid_response"
            | "provider.output_missing"
            | "workflow.timeout"
            | "workflow.worker_unavailable"
            | "ffmpeg.concat_failed"
            | "ffmpeg.output_missing"
            | "ffmpeg.process_failed"
            | "ffmpeg.process_interrupted"
            | "ffmpeg.transcode_failed"
            | "ffmpeg.timeout"
            | "template.render_failed"
            | "template.browser_crashed"
            | "template.timeout"
            | "template.output_missing"
            | "db.query_failed"
            | "db.transaction_failed"
            | "db.busy"
            | "db.locked"
    ) || matches!(kind, "network" | "rate_limit")
}

fn recover_action_for(code: &str, kind: &str, is_retryable: bool) -> Option<String> {
    if is_retryable {
        return Some("retry".to_string());
    }

    let action = match code {
        "task.resume_required" => "resume_task",
        "provider.auth_failed" => "update_secret",
        "provider.cancelled" => "manual_check",
        "provider.capability_unsupported" => "change_provider_or_plan",
        "provider.content_policy" => "edit_input",
        "provider.limit_exceeded" => "edit_input",
        "provider.quota_exceeded" => "change_provider_or_plan",
        "storage.path_denied" | "security.path_escape" => "choose_controlled_path",
        "ffmpeg.not_found" => "configure_sidecar",
        "ffmpeg.sidecar_missing" => "configure_sidecar",
        "ffmpeg.probe_failed" => "check_media_file",
        "ffmpeg.invalid_media" => "regenerate_or_replace_media",
        "ffmpeg.transcode_failed" => "regenerate_or_replace_media",
        "workflow.invalid_node_map" => "change_provider_or_plan",
        "workflow.output_missing" => "change_provider_or_plan",
        "template.param_invalid" => "edit_input",
        "template.resource_denied" => "choose_controlled_path",
        "template.not_found" => "edit_input",
        "template.sidecar_missing" => "configure_sidecar",
        "db.migration_failed" => "restart_app_or_check_database",
        "export.final_missing" => "start_composition",
        "export.composition_not_ready" => "start_composition",
        "export.target_denied" => "choose_controlled_path",
        "export.secret_detected" => "remove_secret_and_retry",
        "export.open_failed" => "manual_check",
        "import.package_invalid" => "choose_controlled_path",
        "import.zip_slip_detected" => "reject_package",
        "backup.restore_failed" => "choose_controlled_path",
        "backup.zip_slip_detected" => "reject_package",
        "diagnostic.secret_detected" => "remove_secret_and_retry",
        "diagnostic.media_permission_required" => "grant_media_permission",
        "validation.invalid_input" => "edit_input",
        _ if kind == "db" => "restart_app_or_check_database",
        _ => "manual_check",
    };

    Some(action.to_string())
}

fn parse_prefixed_error_code(message: &str) -> Option<(&str, &str)> {
    let (code, detail) = message.split_once(": ")?;
    let domain = code.split_once('.').map(|(domain, _)| domain)?;
    let known_domain = matches!(
        domain,
        "task"
            | "template"
            | "storage"
            | "provider"
            | "workflow"
            | "ffmpeg"
            | "db"
            | "security"
            | "export"
            | "import"
            | "backup"
            | "diagnostic"
            | "validation"
            | "auth"
            | "rate_limit"
            | "network"
    );
    if known_domain && !detail.trim().is_empty() {
        Some((code, detail))
    } else {
        None
    }
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
    use super::{AppErrorDto, TaskError};
    use serde_json::json;

    #[test]
    fn task_error_classifies_retryable_provider_timeout() {
        let error = TaskError::from_code("provider.timeout", "Provider timed out".to_string());

        assert_eq!(error.error_kind, "provider");
        assert!(error.is_retryable);
        assert_eq!(error.recover_action.as_deref(), Some("retry"));
        assert!(error.trace_id.starts_with("trace_"));
    }

    #[test]
    fn task_error_redacts_detail_and_message() {
        let error = TaskError::from_code_with_detail(
            "provider.auth_failed",
            "Authorization: Bearer sk-live-secret-token-123456".to_string(),
            Some(json!({
                "headers": {
                    "Authorization": "Bearer sk-live-secret-token-123456"
                }
            })),
        );

        assert_eq!(error.error_kind, "provider");
        assert!(!error.is_retryable);
        assert_eq!(error.recover_action.as_deref(), Some("update_secret"));
        assert!(!error.message.contains("sk-live"));
        assert_eq!(
            error.detail.unwrap()["headers"]["Authorization"],
            "***REDACTED***"
        );
    }

    #[test]
    fn app_error_can_be_built_from_task_error() {
        let error = TaskError::from_code("storage.path_denied", "Path denied".to_string());
        let dto = AppErrorDto::from(error);

        assert_eq!(dto.code, "storage.path_denied");
        assert_eq!(dto.kind, "storage");
        assert!(!dto.is_retryable);
    }

    #[test]
    fn app_error_preserves_prefixed_provider_error_code() {
        let dto = AppErrorDto::from(
            "provider.limit_exceeded: Duration is outside provider limits.".to_string(),
        );

        assert_eq!(dto.code, "provider.limit_exceeded");
        assert_eq!(dto.kind, "provider");
        assert_eq!(dto.recover_action.as_deref(), Some("edit_input"));
        assert!(!dto.is_retryable);
    }

    #[test]
    fn app_error_classifies_export_import_backup_and_diagnostic_codes() {
        let cases = [
            (
                "export.target_denied: target path is outside workspace.",
                "export.target_denied",
                "export",
                "choose_controlled_path",
            ),
            (
                "export.secret_detected: secret-like content was found.",
                "export.secret_detected",
                "export",
                "remove_secret_and_retry",
            ),
            (
                "import.package_invalid: manifest is missing.",
                "import.package_invalid",
                "import",
                "choose_controlled_path",
            ),
            (
                "import.zip_slip_detected: relative path escaped.",
                "import.zip_slip_detected",
                "import",
                "reject_package",
            ),
            (
                "backup.restore_failed: backup is newer than current app.",
                "backup.restore_failed",
                "backup",
                "choose_controlled_path",
            ),
            (
                "diagnostic.media_permission_required: media attachment needs confirmation.",
                "diagnostic.media_permission_required",
                "diagnostic",
                "grant_media_permission",
            ),
        ];

        for (message, code, kind, recover_action) in cases {
            let dto = AppErrorDto::from(message.to_string());
            assert_eq!(dto.code, code);
            assert_eq!(dto.kind, kind);
            assert!(!dto.is_retryable);
            assert_eq!(dto.recover_action.as_deref(), Some(recover_action));
        }
    }

    #[test]
    fn task_error_registry_covers_todo_09_error_codes() {
        let cases = [
            ("provider.auth_failed", "provider", false, "update_secret"),
            ("provider.rate_limited", "provider", true, "retry"),
            ("provider.timeout", "provider", true, "retry"),
            ("provider.content_policy", "provider", false, "edit_input"),
            ("provider.invalid_response", "provider", true, "retry"),
            (
                "workflow.invalid_node_map",
                "workflow",
                false,
                "change_provider_or_plan",
            ),
            (
                "workflow.output_missing",
                "workflow",
                false,
                "change_provider_or_plan",
            ),
            (
                "ffmpeg.sidecar_missing",
                "ffmpeg",
                false,
                "configure_sidecar",
            ),
            ("ffmpeg.probe_failed", "ffmpeg", false, "check_media_file"),
            ("ffmpeg.concat_failed", "ffmpeg", true, "retry"),
            ("template.param_invalid", "template", false, "edit_input"),
            ("template.render_failed", "template", true, "retry"),
            (
                "template.sidecar_missing",
                "template",
                false,
                "configure_sidecar",
            ),
            (
                "db.migration_failed",
                "db",
                false,
                "restart_app_or_check_database",
            ),
            (
                "export.target_denied",
                "export",
                false,
                "choose_controlled_path",
            ),
            (
                "import.zip_slip_detected",
                "import",
                false,
                "reject_package",
            ),
            (
                "diagnostic.secret_detected",
                "diagnostic",
                false,
                "remove_secret_and_retry",
            ),
        ];

        for (code, kind, retryable, recover_action) in cases {
            let error = TaskError::from_code(code, "failure".to_string());
            assert_eq!(error.error_kind, kind, "{code}");
            assert_eq!(error.is_retryable, retryable, "{code}");
            assert_eq!(
                error.recover_action.as_deref(),
                Some(recover_action),
                "{code}"
            );
        }
    }
}
