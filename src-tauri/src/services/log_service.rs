use crate::security::path_guard::PathGuard;
use crate::security::secret_guard::{redact_json, redact_text, reject_text_secrets};
use crate::services::storage_service::{FileBucket, StorageService};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const APP_LOG_MAX_BYTES: u64 = 10 * 1024 * 1024;
const ERROR_LOG_MAX_BYTES: u64 = 10 * 1024 * 1024;
const EXPORT_LOG_MAX_BYTES: u64 = 2 * 1024 * 1024;
const ROTATE_KEEP: usize = 7;

#[derive(Debug, Clone)]
pub struct StructuredFileLogRecord {
    pub trace_id: String,
    pub level: String,
    pub message: String,
    pub project_id: Option<String>,
    pub task_id: Option<String>,
    pub task_step_id: Option<String>,
    pub step_kind: Option<String>,
    pub item_id: Option<String>,
    pub provider_id: Option<String>,
    pub provider_kind: Option<String>,
    pub vendor: Option<String>,
    pub model_name: Option<String>,
    pub error_code: Option<String>,
    pub duration_ms: Option<i64>,
    pub retry_count: Option<i64>,
    pub relative_path: Option<String>,
    pub metadata_json: Value,
}

pub fn write_app_log(
    workspace_root: &Path,
    record: &StructuredFileLogRecord,
) -> Result<(), String> {
    write_log(
        workspace_root,
        FileBucket::Log,
        if record.level == "error" {
            "error.log"
        } else {
            "app.log"
        },
        if record.level == "error" {
            ERROR_LOG_MAX_BYTES
        } else {
            APP_LOG_MAX_BYTES
        },
        record,
    )
}

pub fn write_project_export_log(
    workspace_root: &Path,
    project_id: &str,
    record: &StructuredFileLogRecord,
) -> Result<(), String> {
    let relative_path = format!("{}/exports/export.log", sanitize_file_segment(project_id));
    write_log(
        workspace_root,
        FileBucket::Project,
        &relative_path,
        EXPORT_LOG_MAX_BYTES,
        record,
    )
}

fn write_log(
    workspace_root: &Path,
    bucket: FileBucket,
    relative_path: &str,
    max_bytes: u64,
    record: &StructuredFileLogRecord,
) -> Result<(), String> {
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let normalized = PathGuard::validate_relative_path(relative_path)?;
    let target = storage
        .resolver()
        .resolve_bucket_path_for_write(bucket, &normalized)?;
    rotate_if_needed(&target, max_bytes)?;
    let line = build_json_line(record)?;
    reject_text_secrets(&normalized, &line)
        .map_err(|error| format!("log.secret_detected: {error}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&target)
        .map_err(|error| error.to_string())?;
    file.write_all(line.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|error| error.to_string())
}

fn rotate_if_needed(path: &Path, max_bytes: u64) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }
    let size = fs::metadata(path).map_err(|error| error.to_string())?.len();
    if size < max_bytes {
        return Ok(());
    }
    for index in (1..=ROTATE_KEEP).rev() {
        let source = rotated_path(path, index);
        let target = rotated_path(path, index + 1);
        if target.exists() {
            fs::remove_file(&target).map_err(|error| error.to_string())?;
        }
        if source.exists() && index < ROTATE_KEEP {
            fs::rename(&source, &target).map_err(|error| error.to_string())?;
        }
    }
    fs::rename(path, rotated_path(path, 1)).map_err(|error| error.to_string())
}

fn rotated_path(path: &Path, index: usize) -> std::path::PathBuf {
    let file_name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "app.log".to_string());
    path.with_file_name(format!("{file_name}.{index}"))
}

fn build_json_line(record: &StructuredFileLogRecord) -> Result<String, String> {
    let payload = json!({
        "created_at": current_timestamp_string(),
        "trace_id": record.trace_id,
        "level": record.level,
        "message": redact_text(&record.message),
        "project_id": record.project_id,
        "task_id": record.task_id,
        "task_step_id": record.task_step_id,
        "step_kind": record.step_kind,
        "item_id": record.item_id,
        "provider_id": record.provider_id,
        "provider_kind": record.provider_kind,
        "vendor": record.vendor,
        "model_name": record.model_name,
        "error_code": record.error_code,
        "duration_ms": record.duration_ms,
        "retry_count": record.retry_count,
        "relative_path": record.relative_path,
        "metadata": redact_json(&record.metadata_json),
    });
    serde_json::to_string(&payload).map_err(|error| error.to_string())
}

fn current_timestamp_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    seconds.to_string()
}

fn sanitize_file_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if sanitized.is_empty() {
        "untitled".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::{write_app_log, write_project_export_log, StructuredFileLogRecord};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_sanitized_app_and_export_logs() {
        let root = test_root("write");
        let record = fixture_record("info", "completed with api_key=sk-live-secret-token-123456");

        write_app_log(&root, &record).expect("app log should write");
        write_project_export_log(&root, "project_log", &record).expect("export log should write");

        let app_log = fs::read_to_string(root.join("logs/app.log")).expect("app log should read");
        let export_log = fs::read_to_string(root.join("projects/project_log/exports/export.log"))
            .expect("export log should read");
        assert!(!app_log.contains("sk-live"));
        assert!(!export_log.contains("sk-live"));
        assert!(app_log.contains("***REDACTED***"));
        assert!(export_log.contains("export_id"));

        cleanup(root);
    }

    #[test]
    fn rotates_large_app_log() {
        let root = test_root("rotate");
        fs::create_dir_all(root.join("logs")).expect("logs dir");
        fs::write(
            root.join("logs/app.log"),
            "x".repeat((10 * 1024 * 1024) + 1),
        )
        .expect("large log");

        write_app_log(&root, &fixture_record("info", "after rotate")).expect("log should write");

        assert!(root.join("logs/app.log.1").is_file());
        let current = fs::read_to_string(root.join("logs/app.log")).expect("current log");
        assert!(current.contains("after rotate"));

        cleanup(root);
    }

    fn fixture_record(level: &str, message: &str) -> StructuredFileLogRecord {
        StructuredFileLogRecord {
            trace_id: "trace_log_file".to_string(),
            level: level.to_string(),
            message: message.to_string(),
            project_id: Some("project_log".to_string()),
            task_id: Some("task_log".to_string()),
            task_step_id: Some("step_log".to_string()),
            step_kind: Some("export".to_string()),
            item_id: None,
            provider_id: None,
            provider_kind: None,
            vendor: None,
            model_name: None,
            error_code: None,
            duration_ms: Some(10),
            retry_count: Some(0),
            relative_path: Some("outputs/user_exports/project_log/final.mp4".to_string()),
            metadata_json: json!({ "export_id": "export_log" }),
        }
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-log-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
