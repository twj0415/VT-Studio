use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
        Self {
            code: code.to_string(),
            kind: kind.to_string(),
            message,
            detail: None,
            is_retryable: false,
            recover_action: None,
            trace_id: "trace_mock".to_string(),
        }
    }

    pub fn from_message(message: String) -> Self {
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

impl From<String> for AppErrorDto {
    fn from(value: String) -> Self {
        Self::from_message(value)
    }
}
