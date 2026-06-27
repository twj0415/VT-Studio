use crate::core::error::TaskError;
use crate::db::{Database, Repository};
use crate::domain::task::{
    initial_step_status, CompositionTaskDto, TaskDetailDto, TaskStepDto, TaskSummaryDto,
    IMAGE_TO_VIDEO_PIPELINE_STEPS, IMAGE_TO_VIDEO_TASK_KIND,
};
use crate::security::path_guard::PathGuard;
use crate::security::secret_guard::{redact_json, redact_text};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TaskRepository<'db> {
    database: &'db Database,
}

const DEFAULT_MAX_ATTEMPTS: i64 = 3;
const DEFAULT_BACKOFF_SECONDS: [i64; 3] = [2, 5, 10];

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TaskStepFailureRecord {
    pub project_id: String,
    pub task_id: String,
    pub task_step_id: String,
    pub step_kind: String,
    pub item_id: Option<String>,
    pub input_json: Value,
    pub error: TaskError,
    pub duration_ms: Option<i64>,
    pub retry_count: i64,
    pub relative_path: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RetryPolicySnapshot {
    pub max_attempts: i64,
    pub backoff_seconds: Vec<i64>,
}

impl Default for RetryPolicySnapshot {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            backoff_seconds: DEFAULT_BACKOFF_SECONDS.to_vec(),
        }
    }
}

impl RetryPolicySnapshot {
    fn from_json_text(max_attempts: i64, text: &str) -> Self {
        let default = Self::default();
        let parsed = serde_json::from_str::<Value>(text).unwrap_or_else(|_| json!({}));
        let max_attempts = parsed
            .get("maxAttempts")
            .and_then(Value::as_i64)
            .filter(|value| *value > 0)
            .or_else(|| (max_attempts > 0).then_some(max_attempts))
            .unwrap_or(default.max_attempts);
        let backoff_seconds = parsed
            .get("backoffSeconds")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_i64)
                    .filter(|seconds| *seconds > 0)
                    .collect::<Vec<_>>()
            })
            .filter(|values| !values.is_empty())
            .unwrap_or(default.backoff_seconds);

        Self {
            max_attempts,
            backoff_seconds,
        }
    }

    fn to_json(&self) -> Value {
        let backoff_seconds = if self.backoff_seconds.is_empty() {
            DEFAULT_BACKOFF_SECONDS.to_vec()
        } else {
            self.backoff_seconds
                .iter()
                .copied()
                .filter(|seconds| *seconds > 0)
                .collect::<Vec<_>>()
        };

        json!({
            "maxAttempts": self.max_attempts.max(1),
            "backoffSeconds": if backoff_seconds.is_empty() {
                DEFAULT_BACKOFF_SECONDS.to_vec()
            } else {
                backoff_seconds
            },
        })
    }

    fn backoff_for_attempt(&self, attempt_index: i64) -> i64 {
        let backoff_seconds = if self.backoff_seconds.is_empty() {
            DEFAULT_BACKOFF_SECONDS.to_vec()
        } else {
            self.backoff_seconds
                .iter()
                .copied()
                .filter(|seconds| *seconds > 0)
                .collect::<Vec<_>>()
        };
        let backoff_seconds = if backoff_seconds.is_empty() {
            DEFAULT_BACKOFF_SECONDS.to_vec()
        } else {
            backoff_seconds
        };
        let index = attempt_index.saturating_sub(1).max(0) as usize;
        backoff_seconds
            .get(index)
            .or_else(|| backoff_seconds.last())
            .copied()
            .unwrap_or(DEFAULT_BACKOFF_SECONDS[0])
            .max(1)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TaskAttemptDto {
    pub attempt_id: String,
    pub task_id: String,
    pub step_id: Option<String>,
    pub attempt_index: i64,
    pub status: String,
    pub input_json: Value,
    pub output_json: Option<Value>,
    pub error_json: Option<Value>,
    pub error_code: Option<String>,
    pub error_kind: Option<String>,
    pub is_retryable: bool,
    pub recover_action: Option<String>,
    pub trace_id: Option<String>,
    pub duration_ms: Option<i64>,
    pub retry_policy_snapshot: Option<Value>,
    pub next_retry_at: Option<String>,
    pub backoff_seconds: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructuredTaskLogRecord {
    pub trace_id: String,
    pub project_id: String,
    pub task_id: String,
    pub task_step_id: Option<String>,
    pub step_kind: Option<String>,
    pub item_id: Option<String>,
    pub level: String,
    pub message: String,
    pub error_code: Option<String>,
    pub duration_ms: Option<i64>,
    pub retry_count: i64,
    pub relative_path: Option<String>,
    pub metadata_json: Value,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructuredTaskLogDto {
    pub log_id: String,
    pub trace_id: String,
    pub project_id: String,
    pub task_id: String,
    pub task_step_id: Option<String>,
    pub step_kind: Option<String>,
    pub item_id: Option<String>,
    pub level: String,
    pub message: String,
    pub error_code: Option<String>,
    pub duration_ms: Option<i64>,
    pub retry_count: i64,
    pub relative_path: Option<String>,
    pub metadata_json: Value,
    pub created_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TaskLeaseDto {
    pub task_id: String,
    pub worker_id: String,
    pub lease_expires_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StepSuccessRecord {
    pub task_id: String,
    pub step_name: String,
    pub input_json: Value,
    pub output_json: Value,
    pub artifacts: Vec<TaskArtifactRecord>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TaskCancellationRecord {
    pub task_id: String,
    pub project_id: String,
    pub trace_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TaskArtifactRecord {
    pub artifact_id: Option<String>,
    pub task_id: String,
    pub step_id: Option<String>,
    pub project_id: Option<String>,
    pub owner_kind: Option<String>,
    pub owner_id: Option<String>,
    pub artifact_kind: String,
    pub media_kind: String,
    pub relative_path: Option<String>,
    pub metadata_json: Value,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TaskArtifactDto {
    pub artifact_id: String,
    pub task_id: String,
    pub step_id: Option<String>,
    pub project_id: Option<String>,
    pub owner_kind: Option<String>,
    pub owner_id: Option<String>,
    pub artifact_kind: String,
    pub media_kind: String,
    pub relative_path: Option<String>,
    pub metadata_json: Value,
    pub idempotency_key: Option<String>,
    pub input_hash: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NewCompositionTaskRecord {
    pub task_id: String,
    pub project_id: String,
    pub segment_ids: Vec<String>,
    pub output_path: String,
    pub enhancements: Value,
    pub status: String,
    pub progress: u32,
    pub error_json: Option<Value>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IdempotencyHit {
    pub step_id: String,
    pub task_id: String,
    pub step_name: String,
    pub status: String,
    pub output_json: Value,
    pub artifacts: Vec<TaskArtifactDto>,
}

impl<'db> TaskRepository<'db> {
    #[allow(dead_code)]
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    #[allow(dead_code)]
    pub fn create_task_for_test(
        &self,
        project_id: &str,
        task_id: &str,
        step_id: &str,
        step_kind: &str,
    ) -> Result<(), String> {
        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    INSERT INTO projects (
                        project_id, title, workflow_type, input_type, input_process_mode,
                        aspect_ratio, target_scene_count, segment_duration_seconds,
                        content_language, lifecycle
                    )
                    VALUES (?1, '任务测试', 'image_to_video', 'topic', 'generate', '9:16', 1, 4, 'zh-CN', 'draft')
                    "#,
                    [project_id],
                )?;
                transaction.execute(
                    "INSERT INTO tasks (task_id, project_id, task_kind, task_status, current_step, summary) VALUES (?1, ?2, 'image_to_video', 'running', ?3, '')",
                    params![task_id, project_id, step_kind],
                )?;
                transaction.execute(
                    "INSERT INTO task_steps (step_id, task_id, step_name, status, order_index) VALUES (?1, ?2, ?3, 'running', 0)",
                    params![step_id, task_id, step_kind],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn create_image_to_video_task(&self, project_id: &str) -> Result<TaskDetailDto, String> {
        let task_id = create_id("task");
        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    INSERT INTO tasks (task_id, project_id, task_kind, task_status, current_step, summary)
                    VALUES (?1, ?2, ?3, 'running', 'storyboard_review', ?4)
                    "#,
                    params![
                        task_id,
                        project_id,
                        IMAGE_TO_VIDEO_TASK_KIND,
                        "等待确认分镜"
                    ],
                )?;

                insert_image_to_video_steps(transaction, &task_id)?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_task_detail_by_task_id(&task_id)?
            .ok_or_else(|| format!("Task {task_id} was created but cannot be read back."))
    }

    #[allow(dead_code)]
    pub fn update_latest_project_step_status(
        &self,
        project_id: &str,
        step_name: &str,
        status: &str,
        output_json: Option<Value>,
    ) -> Result<TaskDetailDto, String> {
        let task = self
            .get_latest_task_detail_by_project(project_id)?
            .ok_or_else(|| format!("Task not found for project: {project_id}"))?;
        let output_json_text = output_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE task_steps
                    SET status = ?1,
                        output_json = COALESCE(?2, output_json),
                        updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?3 AND step_name = ?4
                    "#,
                    params![status, output_json_text, task.task_id, step_name],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;
        self.get_task_detail_by_task_id(&task.task_id)?
            .ok_or_else(|| format!("Task not found: {}", task.task_id))
    }

    #[allow(dead_code)]
    pub fn get_latest_task_detail_by_project(
        &self,
        project_id: &str,
    ) -> Result<Option<TaskDetailDto>, String> {
        self.database
            .with_connection(|connection| {
                let task_id = connection
                    .query_row(
                        "SELECT task_id FROM tasks WHERE project_id = ?1 ORDER BY updated_at DESC, created_at DESC LIMIT 1",
                        [project_id],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()?;
                task_id
                    .map(|task_id| read_task_detail(connection, &task_id))
                    .transpose()
            })
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn get_task_detail_by_task_id(
        &self,
        task_id: &str,
    ) -> Result<Option<TaskDetailDto>, String> {
        self.database
            .with_connection(|connection| read_task_detail(connection, task_id).optional())
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn upsert_composition_task(
        &self,
        record: &NewCompositionTaskRecord,
    ) -> Result<CompositionTaskDto, String> {
        let output_path = validate_composition_output_path(&record.output_path)?;
        let segment_ids_json =
            serde_json::to_string(&record.segment_ids).map_err(|error| error.to_string())?;
        let enhancements_json = to_json_text(&record.enhancements)?;
        let error_json = record
            .error_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO composition_tasks (
                        task_id, project_id, segment_ids_json, output_path,
                        enhancements_json, status, progress, error_json, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)
                    ON CONFLICT(task_id) DO UPDATE SET
                        segment_ids_json = excluded.segment_ids_json,
                        output_path = excluded.output_path,
                        enhancements_json = excluded.enhancements_json,
                        status = excluded.status,
                        progress = excluded.progress,
                        error_json = excluded.error_json,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        record.task_id,
                        record.project_id,
                        segment_ids_json,
                        output_path,
                        enhancements_json,
                        record.status,
                        record.progress,
                        error_json,
                    ],
                )?;
                read_composition_task(connection, &record.task_id)?
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)
            })
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn latest_composition_task_by_project(
        &self,
        project_id: &str,
    ) -> Result<Option<CompositionTaskDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT task_id
                        FROM composition_tasks
                        WHERE project_id = ?1
                        ORDER BY updated_at DESC, created_at DESC
                        LIMIT 1
                        "#,
                        [project_id],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()?
                    .map(|task_id| read_composition_task(connection, &task_id))
                    .transpose()
                    .map(Option::flatten)
            })
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn approve_step(&self, project_id: &str, step_name: &str) -> Result<TaskDetailDto, String> {
        let Some(current) = self.get_latest_task_detail_by_project(project_id)? else {
            return self.create_image_to_video_task(project_id);
        };
        ensure_task_has_step(&current, step_name)?;

        let next_step = next_task_step(&current, step_name);
        let next_task_status = if next_step.is_some() {
            "running"
        } else {
            "succeeded"
        };

        self.database
            .transaction(|transaction| {
                transaction.execute(
                    "UPDATE task_steps SET status = 'succeeded', updated_at = CURRENT_TIMESTAMP WHERE task_id = ?1 AND step_name = ?2",
                    params![current.task_id, step_name],
                )?;

                if let Some(next_step) = next_step.as_deref() {
                    let next_step_status = if is_review_step(&next_step) {
                        "waiting_user"
                    } else {
                        "pending"
                    };
                    transaction.execute(
                        "UPDATE task_steps SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE task_id = ?2 AND step_name = ?3",
                        params![next_step_status, current.task_id, next_step],
                    )?;
                }

                transaction.execute(
                    "UPDATE tasks SET task_status = ?1, current_step = ?2, updated_at = CURRENT_TIMESTAMP WHERE task_id = ?3",
                    params![next_task_status, next_step, current.task_id],
                )?;

                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_latest_task_detail_by_project(project_id)?
            .ok_or_else(|| {
                format!("Task for project {project_id} was updated but cannot be read back.")
            })
    }

    #[allow(dead_code)]
    pub fn list_tasks(&self, project_id: Option<&str>) -> Result<Vec<TaskSummaryDto>, String> {
        self.database
            .with_connection(|connection| {
                let mut tasks = Vec::new();
                if let Some(project_id) = project_id {
                    let mut statement = connection.prepare(
                        r#"
                        SELECT task_id, project_id, task_status, current_step, summary, created_at, updated_at
                        FROM tasks
                        WHERE project_id = ?1
                        ORDER BY updated_at DESC, created_at DESC
                        "#,
                    )?;
                    let rows = statement.query_map([project_id], row_to_task_summary)?;
                    for row in rows {
                        tasks.push(row?);
                    }
                    return Ok(tasks);
                }

                let mut statement = connection.prepare(
                    r#"
                    SELECT task_id, project_id, task_status, current_step, summary, created_at, updated_at
                    FROM tasks
                    ORDER BY updated_at DESC, created_at DESC
                    "#,
                )?;
                let rows = statement.query_map([], row_to_task_summary)?;
                for row in rows {
                    tasks.push(row?);
                }
                Ok(tasks)
            })
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn start_task(&self, project_id: &str) -> Result<TaskDetailDto, String> {
        let Some(current) = self.get_latest_task_detail_by_project(project_id)? else {
            return self.create_image_to_video_task(project_id);
        };

        self.database
            .transaction(|transaction| {
                transaction.execute(
                    "UPDATE tasks SET task_status = 'running', cancel_requested = 0, worker_id = NULL, lease_expires_at = NULL, started_at = COALESCE(started_at, CURRENT_TIMESTAMP), finished_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE task_id = ?1 AND task_status IN ('pending', 'failed', 'cancelled', 'running')",
                    [current.task_id.as_str()],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_latest_task_detail_by_project(project_id)?
            .ok_or_else(|| {
                format!("Task for project {project_id} was started but cannot be read back.")
            })
    }

    #[allow(dead_code)]
    pub fn cancel_task(&self, project_id: &str) -> Result<TaskDetailDto, String> {
        self.cancel_task_with_reason(project_id, None)
    }

    #[allow(dead_code)]
    pub fn cancel_task_with_reason(
        &self,
        project_id: &str,
        reason: Option<&str>,
    ) -> Result<TaskDetailDto, String> {
        let current = self
            .get_latest_task_detail_by_project(project_id)?
            .ok_or_else(|| format!("Task not found for project: {project_id}"))?;
        let cancellation = self.request_cancellation(&current.task_id, reason)?;
        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    UPDATE task_steps
                    SET
                        status = 'cancelled',
                        error_json = NULL,
                        error_code = NULL,
                        error_kind = NULL,
                        is_retryable = 0,
                        recover_action = NULL,
                        trace_id = ?1,
                        next_retry_at = NULL,
                        finished_at = CURRENT_TIMESTAMP,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?2
                      AND status IN ('pending', 'running', 'retrying', 'waiting_user')
                    "#,
                    params![cancellation.trace_id, current.task_id.as_str()],
                )?;
                transaction.execute(
                    r#"
                    UPDATE tasks
                    SET
                        task_status = 'cancelled',
                        current_step = NULL,
                        worker_id = NULL,
                        lease_expires_at = NULL,
                        finished_at = CURRENT_TIMESTAMP,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?1
                    "#,
                    [current.task_id.as_str()],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_latest_task_detail_by_project(project_id)?
            .ok_or_else(|| {
                format!("Task for project {project_id} was cancelled but cannot be read back.")
            })
    }

    #[allow(dead_code)]
    pub fn request_cancellation(
        &self,
        task_id: &str,
        reason: Option<&str>,
    ) -> Result<TaskCancellationRecord, String> {
        let error = TaskError::from_code_with_detail(
            "task.cancelled",
            "Task cancellation requested.".to_string(),
            Some(json!({
                "taskId": task_id,
                "reason": reason,
            })),
        );
        let trace_id = error.trace_id.clone();
        let reason = reason.map(redact_text);
        self.database
            .transaction(|transaction| {
                let project_id: String = transaction.query_row(
                    "SELECT project_id FROM tasks WHERE task_id = ?1",
                    [task_id],
                    |row| row.get(0),
                )?;
                transaction.execute(
                    r#"
                    UPDATE tasks
                    SET
                        cancel_requested = 1,
                        cancel_requested_at = COALESCE(cancel_requested_at, CURRENT_TIMESTAMP),
                        cancel_reason = COALESCE(cancel_reason, ?1),
                        trace_id = CASE
                            WHEN cancel_requested = 0 OR trace_id IS NULL THEN ?2
                            ELSE trace_id
                        END,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?3
                    "#,
                    params![reason.as_deref(), trace_id.as_str(), task_id],
                )?;
                Ok(TaskCancellationRecord {
                    task_id: task_id.to_string(),
                    project_id,
                    trace_id,
                    reason,
                })
            })
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn complete_cancelled_step(
        &self,
        task_id: &str,
        _step_name: &str,
    ) -> Result<TaskDetailDto, String> {
        let cancellation = self.ensure_cancellation_requested(task_id)?;
        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    UPDATE task_steps
                    SET
                        status = 'cancelled',
                        trace_id = ?1,
                        finished_at = CURRENT_TIMESTAMP,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?2
                      AND status IN ('pending', 'running', 'retrying', 'waiting_user')
                    "#,
                    params![cancellation.trace_id, task_id],
                )?;
                transaction.execute(
                    r#"
                    UPDATE tasks
                    SET
                        task_status = 'cancelled',
                        current_step = NULL,
                        worker_id = NULL,
                        lease_expires_at = NULL,
                        finished_at = CURRENT_TIMESTAMP,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?1
                    "#,
                    [task_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_task_detail_by_task_id(task_id)?
            .ok_or_else(|| format!("Task {task_id} was cancelled but cannot be read back."))
    }

    #[allow(dead_code)]
    pub fn ensure_cancellation_requested(
        &self,
        task_id: &str,
    ) -> Result<TaskCancellationRecord, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT task_id, project_id, COALESCE(trace_id, ''), cancel_reason
                        FROM tasks
                        WHERE task_id = ?1
                          AND cancel_requested = 1
                        "#,
                        [task_id],
                        |row| {
                            let mut trace_id: String = row.get(2)?;
                            if trace_id.is_empty() {
                                trace_id = TaskError::from_code(
                                    "task.cancelled",
                                    "Task cancellation requested.".to_string(),
                                )
                                .trace_id;
                            }
                            Ok(TaskCancellationRecord {
                                task_id: row.get(0)?,
                                project_id: row.get(1)?,
                                trace_id,
                                reason: row.get(3)?,
                            })
                        },
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("Task cancellation was not requested: {task_id}"))
    }

    #[allow(dead_code)]
    pub fn resume_task(&self, project_id: &str) -> Result<TaskDetailDto, String> {
        let current = self
            .get_latest_task_detail_by_project(project_id)?
            .ok_or_else(|| format!("Task not found for project: {project_id}"))?;
        let next_step = current
            .steps
            .iter()
            .find(|step| {
                matches!(
                    step.status.as_str(),
                    "pending" | "retrying" | "failed" | "cancelled" | "waiting_user"
                )
            })
            .map(|step| step.step_name.clone());

        self.database
            .transaction(|transaction| {
                if let Some(next_step) = next_step.as_deref() {
                    let next_step_status = if is_review_step(next_step) {
                        "waiting_user"
                    } else {
                        "pending"
                    };
                    transaction.execute(
                        "UPDATE task_steps SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE task_id = ?2 AND step_name = ?3 AND status IN ('failed', 'cancelled', 'retrying')",
                        params![next_step_status, current.task_id.as_str(), next_step],
                    )?;
                }

                transaction.execute(
                    "UPDATE tasks SET task_status = ?1, current_step = ?2, cancel_requested = 0, worker_id = NULL, lease_expires_at = NULL, finished_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE task_id = ?3",
                    params![
                        if next_step.is_some() { "running" } else { "succeeded" },
                        next_step,
                        current.task_id.as_str(),
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_latest_task_detail_by_project(project_id)?
            .ok_or_else(|| {
                format!("Task for project {project_id} was resumed but cannot be read back.")
            })
    }

    #[allow(dead_code)]
    pub fn retry_task_step(
        &self,
        project_id: &str,
        step_name: &str,
    ) -> Result<TaskDetailDto, String> {
        let current = self
            .get_latest_task_detail_by_project(project_id)?
            .ok_or_else(|| format!("Task not found for project: {project_id}"))?;
        ensure_task_has_step(&current, step_name)?;

        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    UPDATE task_steps
                    SET
                        status = 'pending',
                        error_json = NULL,
                        error_code = NULL,
                        error_kind = NULL,
                        is_retryable = 0,
                        recover_action = NULL,
                        next_retry_at = NULL,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?1 AND step_name = ?2
                    "#,
                    params![current.task_id, step_name],
                )?;
                transaction.execute(
                    "UPDATE tasks SET task_status = 'running', current_step = ?1, cancel_requested = 0, worker_id = NULL, lease_expires_at = NULL, finished_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE task_id = ?2",
                    params![step_name, current.task_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_latest_task_detail_by_project(project_id)?
            .ok_or_else(|| {
                format!("Task for project {project_id} was retried but cannot be read back.")
            })
    }

    #[allow(dead_code)]
    pub fn record_step_success(&self, record: StepSuccessRecord) -> Result<IdempotencyHit, String> {
        let current = self
            .get_task_detail_by_task_id(&record.task_id)?
            .ok_or_else(|| format!("Task not found: {}", record.task_id))?;
        ensure_task_has_step(&current, &record.step_name)?;

        if self.is_cancel_requested(&record.task_id)? {
            return Err(format!(
                "Task {} was cancelled before step {} could write success.",
                record.task_id, record.step_name
            ));
        }

        if let Some(hit) = self.find_idempotent_step_output(
            &record.task_id,
            &record.step_name,
            &record.input_json,
        )? {
            return Ok(hit);
        }

        let input_hash = stable_json_hash(&record.input_json);
        let idempotency_key =
            build_idempotency_key(&record.task_id, &record.step_name, &input_hash);
        let input_json_text = to_json_text(&record.input_json)?;
        let output_json_text = to_json_text(&record.output_json)?;

        self.database
            .transaction(|transaction| {
                let (step_id, project_id, policy): (String, String, RetryPolicySnapshot) = transaction.query_row(
                    r#"
                    SELECT
                        task_steps.step_id,
                        tasks.project_id,
                        tasks.cancel_requested,
                        task_steps.max_attempts,
                        task_steps.retry_policy_snapshot_json
                    FROM task_steps
                    JOIN tasks ON tasks.task_id = task_steps.task_id
                    WHERE task_steps.task_id = ?1 AND task_steps.step_name = ?2
                    "#,
                    params![record.task_id, record.step_name],
                    |row| {
                        let cancel_requested: i64 = row.get(2)?;
                        if cancel_requested != 0 {
                            return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(SimpleSqlError(format!(
                                "Task {} was cancelled before step {} could write success.",
                                record.task_id, record.step_name
                            )))));
                        }
                        let policy_text: String = row.get(4)?;
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            RetryPolicySnapshot::from_json_text(row.get(3)?, &policy_text),
                        ))
                    },
                )?;
                let attempt_index = next_attempt_index(transaction, &step_id)?;
                let retry_policy_snapshot = policy.to_json();
                let retry_policy_snapshot_text = to_json_text(&retry_policy_snapshot)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(SimpleSqlError(error))))?;

                transaction.execute(
                    r#"
                    UPDATE task_steps
                    SET
                        status = 'succeeded',
                        input_json = ?1,
                        output_json = ?2,
                        error_json = NULL,
                        error_code = NULL,
                        error_kind = NULL,
                        is_retryable = 0,
                        recover_action = NULL,
                        idempotency_key = ?3,
                        input_hash = ?4,
                        next_retry_at = NULL,
                        finished_at = CURRENT_TIMESTAMP,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE step_id = ?5
                    "#,
                    params![
                        input_json_text,
                        output_json_text,
                        idempotency_key,
                        input_hash,
                        step_id,
                    ],
                )?;

                transaction.execute(
                    r#"
                    INSERT INTO task_attempts (
                        attempt_id, task_id, step_id, attempt_index, status,
                        input_json, output_json, retry_policy_snapshot_json,
                        started_at, finished_at
                    )
                    VALUES (?1, ?2, ?3, ?4, 'succeeded', ?5, ?6, ?7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                    "#,
                    params![
                        create_id("attempt"),
                        record.task_id,
                        step_id,
                        attempt_index,
                        input_json_text,
                        output_json_text,
                        retry_policy_snapshot_text,
                    ],
                )?;

                for artifact in &record.artifacts {
                    insert_artifact(
                        transaction,
                        artifact,
                        &step_id,
                        &project_id,
                        &idempotency_key,
                        &input_hash,
                    )?;
                }

                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.find_idempotent_step_output(&record.task_id, &record.step_name, &record.input_json)?
            .ok_or_else(|| {
                format!(
                    "Step {} for task {} succeeded but cannot be read back.",
                    record.step_name, record.task_id
                )
            })
    }

    #[allow(dead_code)]
    pub fn find_idempotent_step_output(
        &self,
        task_id: &str,
        step_name: &str,
        input_json: &Value,
    ) -> Result<Option<IdempotencyHit>, String> {
        let input_hash = stable_json_hash(input_json);
        let idempotency_key = build_idempotency_key(task_id, step_name, &input_hash);
        self.database
            .with_connection(|connection| {
                let hit = connection
                    .query_row(
                        r#"
                        SELECT step_id, task_id, step_name, status, output_json
                        FROM task_steps
                        WHERE idempotency_key = ?1 AND status = 'succeeded'
                        "#,
                        [idempotency_key.as_str()],
                        |row| {
                            let output_json_text: Option<String> = row.get(4)?;
                            Ok(IdempotencyHit {
                                step_id: row.get(0)?,
                                task_id: row.get(1)?,
                                step_name: row.get(2)?,
                                status: row.get(3)?,
                                output_json: output_json_text
                                    .and_then(|value| serde_json::from_str::<Value>(&value).ok())
                                    .unwrap_or_else(|| json!({})),
                                artifacts: vec![],
                            })
                        },
                    )
                    .optional()?;

                let Some(mut hit) = hit else {
                    return Ok(None);
                };
                hit.artifacts = read_artifacts_for_step(connection, &hit.step_id)?;
                Ok(Some(hit))
            })
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn find_idempotent_step_output_with_existing_artifacts(
        &self,
        workspace_root: &Path,
        task_id: &str,
        step_name: &str,
        input_json: &Value,
    ) -> Result<Option<IdempotencyHit>, String> {
        let Some(hit) = self.find_idempotent_step_output(task_id, step_name, input_json)? else {
            return Ok(None);
        };

        let guard = PathGuard::new(workspace_root);
        for artifact in &hit.artifacts {
            if let Some(relative_path) = artifact.relative_path.as_deref() {
                guard.safe_join_existing(relative_path)?;
            }
        }

        Ok(Some(hit))
    }

    #[allow(dead_code)]
    pub fn acquire_lease(
        &self,
        task_id: &str,
        worker_id: &str,
        lease_seconds: i64,
    ) -> Result<TaskLeaseDto, String> {
        let lease_modifier = sqlite_datetime_modifier(lease_seconds);
        let changed = self
            .database
            .transaction(|transaction| {
                let changed = transaction.execute(
                    r#"
                    UPDATE tasks
                    SET
                        worker_id = ?1,
                        lease_expires_at = datetime('now', ?2),
                        started_at = COALESCE(started_at, CURRENT_TIMESTAMP),
                        task_status = 'running',
                        updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?3
                      AND task_status = 'running'
                      AND cancel_requested = 0
                      AND current_step IS NOT NULL
                      AND current_step NOT IN ('storyboard_review', 'image_review', 'video_review')
                      AND (worker_id IS NULL OR lease_expires_at IS NULL OR lease_expires_at <= CURRENT_TIMESTAMP)
                      AND EXISTS (
                        SELECT 1
                        FROM task_steps
                        WHERE task_steps.task_id = tasks.task_id
                          AND task_steps.step_name = tasks.current_step
                          AND (
                            task_steps.status = 'pending'
                            OR (
                              task_steps.status = 'retrying'
                              AND (
                                task_steps.next_retry_at IS NULL
                                OR task_steps.next_retry_at <= CURRENT_TIMESTAMP
                              )
                            )
                          )
                      )
                    "#,
                    params![worker_id, lease_modifier, task_id],
                )?;
                if changed > 0 {
                    transaction.execute(
                        r#"
                        UPDATE task_steps
                        SET
                            status = 'running',
                            started_at = COALESCE(started_at, CURRENT_TIMESTAMP),
                            updated_at = CURRENT_TIMESTAMP
                        WHERE task_id = ?1
                          AND step_name = (SELECT current_step FROM tasks WHERE task_id = ?1)
                          AND (
                            status = 'pending'
                            OR (
                              status = 'retrying'
                              AND (
                                next_retry_at IS NULL
                                OR next_retry_at <= CURRENT_TIMESTAMP
                              )
                            )
                          )
                        "#,
                        [task_id],
                    )?;
                }
                Ok(changed)
            })
            .map_err(|error| error.to_string())?;
        if changed == 0 {
            return Err(format!("Task lease was not acquired: {task_id}"));
        }

        self.read_lease(task_id)?
            .ok_or_else(|| format!("Task lease was not acquired: {task_id}"))
    }

    #[allow(dead_code)]
    pub fn renew_lease(
        &self,
        task_id: &str,
        worker_id: &str,
        lease_seconds: i64,
    ) -> Result<TaskLeaseDto, String> {
        let lease_modifier = sqlite_datetime_modifier(lease_seconds);
        let changed = self
            .database
            .transaction(|transaction| {
                let changed = transaction.execute(
                    r#"
                    UPDATE tasks
                    SET lease_expires_at = datetime('now', ?1), updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?2 AND worker_id = ?3 AND task_status = 'running' AND cancel_requested = 0
                    "#,
                    params![lease_modifier, task_id, worker_id],
                )?;
                Ok(changed)
            })
            .map_err(|error| error.to_string())?;
        if changed == 0 {
            return Err(format!("Task lease was not renewed: {task_id}"));
        }

        self.read_lease(task_id)?
            .ok_or_else(|| format!("Task lease was not renewed: {task_id}"))
    }

    #[allow(dead_code)]
    pub fn is_cancel_requested(&self, task_id: &str) -> Result<bool, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT cancel_requested FROM tasks WHERE task_id = ?1",
                        [task_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())?
            .map(|value| value != 0)
            .ok_or_else(|| format!("Task not found: {task_id}"))
    }

    #[allow(dead_code)]
    pub fn scan_and_mark_recoverable_tasks(&self) -> Result<Vec<TaskSummaryDto>, String> {
        let recoverable = self
            .database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT task_id, project_id, task_status, current_step, summary, created_at, updated_at
                    FROM tasks
                    WHERE task_status = 'running'
                      AND current_step IS NOT NULL
                      AND current_step NOT IN ('storyboard_review', 'image_review', 'video_review')
                      AND (
                        worker_id IS NULL
                        OR lease_expires_at IS NULL
                        OR lease_expires_at <= CURRENT_TIMESTAMP
                      )
                      AND EXISTS (
                        SELECT 1
                        FROM task_steps
                        WHERE task_steps.task_id = tasks.task_id
                          AND task_steps.step_name = tasks.current_step
                          AND task_steps.status != 'retrying'
                      )
                    ORDER BY updated_at ASC
                    "#,
                )?;
                let rows = statement.query_map([], row_to_task_summary)?;
                rows.collect::<Result<Vec<_>, _>>()
            })
            .map_err(|error| error.to_string())?;

        for task in &recoverable {
            let error = TaskError::from_code_with_detail(
                "task.resume_required",
                "Task was running without a valid lease and must be resumed from the database state."
                    .to_string(),
                Some(json!({
                    "taskId": task.task_id,
                    "currentStep": task.current_step,
                })),
            );
            let error_json = to_json_text(&error.to_json())?;
            let trace_id = error.trace_id.clone();
            let error_code = error.error_code.clone();
            let error_kind = error.error_kind.clone();
            let recover_action = error.recover_action.clone();
            let is_retryable = i64::from(error.is_retryable);
            self.database
                .transaction(|transaction| {
                    transaction.execute(
                        r#"
                        UPDATE tasks
                        SET
                            task_status = 'failed',
                            worker_id = NULL,
                            lease_expires_at = NULL,
                            last_error_json = ?1,
                            trace_id = ?2,
                            finished_at = CURRENT_TIMESTAMP,
                            updated_at = CURRENT_TIMESTAMP
                        WHERE task_id = ?3
                        "#,
                        params![error_json, trace_id, task.task_id],
                    )?;
                    if let Some(step_name) = task.current_step.as_deref() {
                        transaction.execute(
                            "UPDATE task_steps SET status = 'failed', error_json = ?1, error_code = ?2, error_kind = ?3, trace_id = ?4, is_retryable = ?5, recover_action = ?6, finished_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE task_id = ?7 AND step_name = ?8 AND status IN ('pending', 'running', 'retrying')",
                            params![
                                error_json,
                                error_code,
                                error_kind,
                                trace_id,
                                is_retryable,
                                recover_action,
                                task.task_id,
                                step_name,
                            ],
                        )?;
                    }
                    Ok(())
                })
                .map_err(|error| error.to_string())?;
        }

        Ok(recoverable)
    }

    #[allow(dead_code)]
    fn read_lease(&self, task_id: &str) -> Result<Option<TaskLeaseDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT task_id, worker_id, lease_expires_at FROM tasks WHERE task_id = ?1 AND worker_id IS NOT NULL AND lease_expires_at IS NOT NULL",
                        [task_id],
                        |row| {
                            Ok(TaskLeaseDto {
                                task_id: row.get(0)?,
                                worker_id: row.get(1)?,
                                lease_expires_at: row.get(2)?,
                            })
                        },
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn record_step_failure(
        &self,
        record: TaskStepFailureRecord,
    ) -> Result<TaskAttemptDto, String> {
        let attempt_id = create_id("attempt");
        let error_json = record.error.to_json();
        let error_json_text = to_json_text(&error_json)?;
        let input_json_text = to_json_text(&record.input_json)?;
        let message = record.error.message.clone();
        let trace_id = record.error.trace_id.clone();
        let error_code = record.error.error_code.clone();
        let error_kind = record.error.error_kind.clone();
        let recover_action = record.error.recover_action.clone();
        let is_retryable = i64::from(record.error.is_retryable);
        let error_is_retryable = record.error.is_retryable;
        let attempt_id_for_read = attempt_id.clone();

        self.database
            .transaction(|transaction| {
                let policy = read_retry_policy(transaction, &record.task_step_id)?;
                let attempt_index = next_attempt_index(transaction, &record.task_step_id)?;
                let should_retry = error_is_retryable && attempt_index < policy.max_attempts;
                let step_status = if should_retry { "retrying" } else { "failed" };
                let task_status = if should_retry { "running" } else { "failed" };
                let backoff_seconds =
                    should_retry.then(|| policy.backoff_for_attempt(attempt_index));
                let next_retry_modifier = backoff_seconds.map(sqlite_datetime_modifier);
                let retry_policy_snapshot = policy.to_json();
                let retry_policy_snapshot_text =
                    to_json_text(&retry_policy_snapshot).map_err(|error| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(SimpleSqlError(error)))
                    })?;

                transaction.execute(
                    r#"
                    UPDATE task_steps
                    SET
                        status = ?1,
                        input_json = ?2,
                        error_json = ?3,
                        error_code = ?4,
                        error_kind = ?5,
                        is_retryable = ?6,
                        recover_action = ?7,
                        trace_id = ?8,
                        retry_count = ?9,
                        max_attempts = ?10,
                        retry_policy_snapshot_json = ?11,
                        next_retry_at = CASE
                            WHEN ?12 IS NULL THEN NULL
                            ELSE datetime('now', ?12)
                        END,
                        finished_at = CASE
                            WHEN ?13 = 'failed' THEN CURRENT_TIMESTAMP
                            ELSE NULL
                        END,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE step_id = ?14
                    "#,
                    params![
                        step_status,
                        input_json_text,
                        error_json_text,
                        error_code,
                        error_kind,
                        is_retryable,
                        recover_action,
                        trace_id,
                        attempt_index,
                        policy.max_attempts,
                        retry_policy_snapshot_text,
                        next_retry_modifier,
                        step_status,
                        record.task_step_id,
                    ],
                )?;

                transaction.execute(
                    r#"
                    UPDATE tasks
                    SET
                        task_status = ?1,
                        current_step = ?2,
                        last_error_json = ?3,
                        trace_id = ?4,
                        worker_id = NULL,
                        lease_expires_at = NULL,
                        finished_at = CASE
                            WHEN ?5 = 'failed' THEN CURRENT_TIMESTAMP
                            ELSE NULL
                        END,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE task_id = ?6
                    "#,
                    params![
                        task_status,
                        record.step_kind,
                        error_json_text,
                        trace_id,
                        task_status,
                        record.task_id,
                    ],
                )?;

                transaction.execute(
                    r#"
                    INSERT INTO task_attempts (
                        attempt_id, task_id, step_id, attempt_index, status,
                        input_json, error_json, error_code, error_kind, is_retryable,
                        recover_action, trace_id, duration_ms, retry_policy_snapshot_json,
                        next_retry_at, backoff_seconds, started_at, finished_at
                    )
                    VALUES (
                        ?1, ?2, ?3, ?4, 'failed', ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                        ?13,
                        CASE
                            WHEN ?14 IS NULL THEN NULL
                            ELSE datetime('now', ?14)
                        END,
                        ?15,
                        CURRENT_TIMESTAMP,
                        CURRENT_TIMESTAMP
                    )
                    "#,
                    params![
                        attempt_id,
                        record.task_id,
                        record.task_step_id,
                        attempt_index,
                        input_json_text,
                        error_json_text,
                        error_code,
                        error_kind,
                        is_retryable,
                        recover_action,
                        trace_id,
                        record.duration_ms,
                        retry_policy_snapshot_text,
                        next_retry_modifier,
                        backoff_seconds,
                    ],
                )?;

                insert_log(
                    transaction,
                    &StructuredTaskLogRecord {
                        trace_id,
                        project_id: record.project_id,
                        task_id: record.task_id,
                        task_step_id: Some(record.task_step_id),
                        step_kind: Some(record.step_kind),
                        item_id: record.item_id,
                        level: "error".to_string(),
                        message,
                        error_code: Some(error_code),
                        duration_ms: record.duration_ms,
                        retry_count: attempt_index,
                        relative_path: record.relative_path,
                        metadata_json: json!({
                            "attemptIndex": attempt_index,
                            "backoffSeconds": backoff_seconds,
                            "errorKind": error_kind,
                            "isRetryable": record.error.is_retryable,
                            "maxAttempts": policy.max_attempts,
                            "recoverAction": record.error.recover_action,
                            "scheduledRetry": should_retry,
                        }),
                    },
                )?;

                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_attempt(&attempt_id_for_read)?.ok_or_else(|| {
            format!("Task attempt {attempt_id_for_read} was written but cannot be read back.")
        })
    }

    #[allow(dead_code)]
    pub fn insert_log(
        &self,
        record: StructuredTaskLogRecord,
    ) -> Result<StructuredTaskLogDto, String> {
        let log_id = self
            .database
            .transaction(|transaction| insert_log(transaction, &record))
            .map_err(|error| error.to_string())?;
        self.get_log(&log_id)?
            .ok_or_else(|| format!("Task log {log_id} was written but cannot be read back."))
    }

    #[allow(dead_code)]
    pub fn get_step_error_json(&self, task_step_id: &str) -> Result<Option<Value>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT error_json FROM task_steps WHERE step_id = ?1",
                        [task_step_id],
                        |row| row.get::<_, Option<String>>(0),
                    )
                    .optional()
                    .map(|value| value.flatten())
            })
            .map_err(|error| error.to_string())?
            .map(|text| serde_json::from_str::<Value>(&text).map_err(|error| error.to_string()))
            .transpose()
    }

    #[allow(dead_code)]
    pub fn get_attempt(&self, attempt_id: &str) -> Result<Option<TaskAttemptDto>, String> {
        self.database
            .with_connection(|connection| read_attempt(connection, attempt_id))
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn get_log(&self, log_id: &str) -> Result<Option<StructuredTaskLogDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT
                            log_id, trace_id, project_id, task_id, task_step_id, step_kind,
                            item_id, level, message, error_code, duration_ms, retry_count,
                            relative_path, metadata_json, created_at
                        FROM task_logs
                        WHERE log_id = ?1
                        "#,
                        [log_id],
                        |row| {
                            let metadata_json_text: String = row.get(13)?;
                            Ok(StructuredTaskLogDto {
                                log_id: row.get(0)?,
                                trace_id: row.get(1)?,
                                project_id: row.get(2)?,
                                task_id: row.get(3)?,
                                task_step_id: row.get(4)?,
                                step_kind: row.get(5)?,
                                item_id: row.get(6)?,
                                level: row.get(7)?,
                                message: row.get(8)?,
                                error_code: row.get(9)?,
                                duration_ms: row.get(10)?,
                                retry_count: row.get(11)?,
                                relative_path: row.get(12)?,
                                metadata_json: serde_json::from_str(&metadata_json_text)
                                    .unwrap_or_else(|_| json!({})),
                                created_at: row.get(14)?,
                            })
                        },
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }
}

impl Repository for TaskRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn insert_log(
    transaction: &rusqlite::Transaction<'_>,
    record: &StructuredTaskLogRecord,
) -> Result<String, rusqlite::Error> {
    let log_id = create_id("task_log");
    let metadata_json = serde_json::to_string(&redact_json(&record.metadata_json))
        .unwrap_or_else(|_| "{}".to_string());
    transaction.execute(
        r#"
        INSERT INTO task_logs (
            log_id, trace_id, project_id, task_id, task_step_id, step_kind,
            item_id, level, message, error_code, duration_ms, retry_count,
            relative_path, metadata_json
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        "#,
        params![
            log_id,
            record.trace_id,
            record.project_id,
            record.task_id,
            record.task_step_id,
            record.step_kind,
            record.item_id,
            record.level,
            redact_text(&record.message),
            record.error_code,
            record.duration_ms,
            record.retry_count,
            record.relative_path,
            metadata_json,
        ],
    )?;
    Ok(log_id)
}

fn insert_artifact(
    transaction: &rusqlite::Transaction<'_>,
    record: &TaskArtifactRecord,
    fallback_step_id: &str,
    fallback_project_id: &str,
    idempotency_key: &str,
    input_hash: &str,
) -> Result<String, rusqlite::Error> {
    let artifact_id = record
        .artifact_id
        .clone()
        .unwrap_or_else(|| create_id("artifact"));
    let step_id = record
        .step_id
        .as_deref()
        .unwrap_or(fallback_step_id)
        .to_string();
    let project_id = record
        .project_id
        .as_deref()
        .unwrap_or(fallback_project_id)
        .to_string();
    let relative_path = record
        .relative_path
        .as_deref()
        .map(validate_artifact_relative_path)
        .transpose()
        .map_err(|error| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(SimpleSqlError(error)))
        })?;
    let metadata_json = serde_json::to_string(&redact_json(&record.metadata_json))
        .unwrap_or_else(|_| "{}".to_string());

    transaction.execute(
        r#"
        INSERT INTO artifacts (
            artifact_id, task_id, step_id, project_id, kind, relative_path, data_json,
            owner_kind, owner_id, artifact_kind, media_kind, metadata_json,
            idempotency_key, input_hash
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        "#,
        params![
            artifact_id,
            record.task_id,
            step_id,
            project_id,
            record.artifact_kind,
            relative_path,
            metadata_json,
            record.owner_kind,
            record.owner_id,
            record.artifact_kind,
            record.media_kind,
            metadata_json,
            idempotency_key,
            input_hash,
        ],
    )?;
    Ok(artifact_id)
}

fn insert_image_to_video_steps(
    transaction: &rusqlite::Transaction<'_>,
    task_id: &str,
) -> Result<(), rusqlite::Error> {
    for (order_index, step_name) in IMAGE_TO_VIDEO_PIPELINE_STEPS.iter().enumerate() {
        transaction.execute(
            "INSERT INTO task_steps (step_id, task_id, step_name, status, order_index) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                format!("{task_id}_{step_name}"),
                task_id,
                step_name,
                initial_step_status(step_name),
                order_index as i64,
            ],
        )?;
    }

    Ok(())
}

fn read_task_detail(
    connection: &rusqlite::Connection,
    task_id: &str,
) -> Result<TaskDetailDto, rusqlite::Error> {
    let (task_id, project_id, task_status, current_step): (String, String, String, Option<String>) =
        connection.query_row(
            "SELECT task_id, project_id, task_status, current_step FROM tasks WHERE task_id = ?1",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;

    let mut statement = connection.prepare(
        "SELECT step_id, step_name, status, output_json FROM task_steps WHERE task_id = ?1 ORDER BY order_index ASC, created_at ASC, step_id ASC",
    )?;
    let rows = statement.query_map([&task_id], |row| {
        let output_json_text: Option<String> = row.get(3)?;
        Ok(TaskStepDto {
            step_id: row.get(0)?,
            step_name: row.get(1)?,
            status: row.get(2)?,
            output_json: output_json_text
                .and_then(|value| serde_json::from_str::<Value>(&value).ok()),
        })
    })?;

    Ok(TaskDetailDto {
        task_id,
        composition_task: read_latest_composition_task(connection, &project_id)?,
        project_id,
        task_status,
        current_step,
        steps: rows.collect::<Result<Vec<_>, _>>()?,
    })
}

fn read_latest_composition_task(
    connection: &rusqlite::Connection,
    project_id: &str,
) -> Result<Option<CompositionTaskDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT task_id
            FROM composition_tasks
            WHERE project_id = ?1
            ORDER BY updated_at DESC, created_at DESC
            LIMIT 1
            "#,
            [project_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|task_id| read_composition_task(connection, &task_id))
        .transpose()
        .map(Option::flatten)
}

fn read_composition_task(
    connection: &rusqlite::Connection,
    task_id: &str,
) -> Result<Option<CompositionTaskDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT
                task_id, project_id, segment_ids_json, output_path, status,
                progress, error_json, created_at, updated_at, enhancements_json
            FROM composition_tasks
            WHERE task_id = ?1
            "#,
            [task_id],
            |row| {
                let segment_ids_json: String = row.get(2)?;
                let error_json: Option<String> = row.get(6)?;
                let enhancements_json: String = row.get(9)?;
                Ok(CompositionTaskDto {
                    task_id: row.get(0)?,
                    project_id: row.get(1)?,
                    segment_ids: serde_json::from_str::<Vec<String>>(&segment_ids_json)
                        .unwrap_or_default(),
                    output_path: row.get(3)?,
                    enhancements: serde_json::from_str::<Value>(&enhancements_json)
                        .unwrap_or_else(|_| json!({})),
                    status: row.get(4)?,
                    progress: row.get::<_, i64>(5)?.max(0) as u32,
                    error_json: error_json
                        .and_then(|value| serde_json::from_str::<Value>(&value).ok()),
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .optional()
}

fn read_artifacts_for_step(
    connection: &rusqlite::Connection,
    step_id: &str,
) -> Result<Vec<TaskArtifactDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT
            artifact_id, task_id, step_id, project_id, owner_kind, owner_id,
            COALESCE(artifact_kind, kind), COALESCE(media_kind, 'unknown'), relative_path,
            COALESCE(metadata_json, data_json, '{}'), idempotency_key, input_hash, created_at
        FROM artifacts
        WHERE step_id = ?1
        ORDER BY created_at ASC, artifact_id ASC
        "#,
    )?;
    let rows = statement.query_map([step_id], row_to_artifact)?;
    rows.collect()
}

fn read_attempt(
    connection: &rusqlite::Connection,
    attempt_id: &str,
) -> Result<Option<TaskAttemptDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT
                attempt_id, task_id, step_id, attempt_index, status, input_json, output_json,
                error_json, error_code, error_kind, is_retryable, recover_action, trace_id,
                duration_ms, retry_policy_snapshot_json, next_retry_at, backoff_seconds, created_at
            FROM task_attempts
            WHERE attempt_id = ?1
            "#,
            [attempt_id],
            row_to_task_attempt,
        )
        .optional()
}

fn row_to_task_attempt(row: &rusqlite::Row<'_>) -> Result<TaskAttemptDto, rusqlite::Error> {
    let input_json_text: Option<String> = row.get(5)?;
    let output_json_text: Option<String> = row.get(6)?;
    let error_json_text: Option<String> = row.get(7)?;
    let retry_policy_snapshot_text: Option<String> = row.get(14)?;

    Ok(TaskAttemptDto {
        attempt_id: row.get(0)?,
        task_id: row.get(1)?,
        step_id: row.get(2)?,
        attempt_index: row.get(3)?,
        status: row.get(4)?,
        input_json: input_json_text
            .and_then(|value| serde_json::from_str::<Value>(&value).ok())
            .unwrap_or_else(|| json!({})),
        output_json: output_json_text.and_then(|value| serde_json::from_str::<Value>(&value).ok()),
        error_json: error_json_text.and_then(|value| serde_json::from_str::<Value>(&value).ok()),
        error_code: row.get(8)?,
        error_kind: row.get(9)?,
        is_retryable: row.get::<_, i64>(10)? == 1,
        recover_action: row.get(11)?,
        trace_id: row.get(12)?,
        duration_ms: row.get(13)?,
        retry_policy_snapshot: retry_policy_snapshot_text
            .and_then(|value| serde_json::from_str::<Value>(&value).ok()),
        next_retry_at: row.get(15)?,
        backoff_seconds: row.get(16)?,
        created_at: row.get(17)?,
    })
}

fn row_to_artifact(row: &rusqlite::Row<'_>) -> Result<TaskArtifactDto, rusqlite::Error> {
    let metadata_json_text: String = row.get(9)?;
    Ok(TaskArtifactDto {
        artifact_id: row.get(0)?,
        task_id: row.get(1)?,
        step_id: row.get(2)?,
        project_id: row.get(3)?,
        owner_kind: row.get(4)?,
        owner_id: row.get(5)?,
        artifact_kind: row.get(6)?,
        media_kind: row.get(7)?,
        relative_path: row.get(8)?,
        metadata_json: serde_json::from_str(&metadata_json_text).unwrap_or_else(|_| json!({})),
        idempotency_key: row.get(10)?,
        input_hash: row.get(11)?,
        created_at: row.get(12)?,
    })
}

fn row_to_task_summary(row: &rusqlite::Row<'_>) -> Result<TaskSummaryDto, rusqlite::Error> {
    Ok(TaskSummaryDto {
        task_id: row.get(0)?,
        project_id: row.get(1)?,
        task_status: row.get(2)?,
        current_step: row.get(3)?,
        summary: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn next_task_step(task: &TaskDetailDto, step_name: &str) -> Option<String> {
    let index = task
        .steps
        .iter()
        .position(|pipeline_step| pipeline_step.step_name == step_name)?;
    task.steps.get(index + 1).map(|step| step.step_name.clone())
}

fn ensure_task_has_step(task: &TaskDetailDto, step_name: &str) -> Result<(), String> {
    if task.steps.iter().any(|step| step.step_name == step_name) {
        Ok(())
    } else {
        Err(format!(
            "Task {} does not contain step: {step_name}",
            task.task_id
        ))
    }
}

fn is_review_step(step_name: &str) -> bool {
    matches!(
        step_name,
        "storyboard_review"
            | "image_review"
            | "video_review"
            | "script_review"
            | "digital_human_asset_review"
            | "material_import"
            | "material_analysis"
            | "material_matching"
            | "template_motion"
    )
}

fn next_attempt_index(
    transaction: &rusqlite::Transaction<'_>,
    step_id: &str,
) -> Result<i64, rusqlite::Error> {
    transaction.query_row(
        "SELECT COALESCE(MAX(attempt_index), 0) + 1 FROM task_attempts WHERE step_id = ?1",
        [step_id],
        |row| row.get(0),
    )
}

fn read_retry_policy(
    transaction: &rusqlite::Transaction<'_>,
    step_id: &str,
) -> Result<RetryPolicySnapshot, rusqlite::Error> {
    transaction.query_row(
        "SELECT max_attempts, retry_policy_snapshot_json FROM task_steps WHERE step_id = ?1",
        [step_id],
        |row| {
            let max_attempts: i64 = row.get(0)?;
            let policy_text: String = row.get(1)?;
            Ok(RetryPolicySnapshot::from_json_text(
                max_attempts,
                &policy_text,
            ))
        },
    )
}

fn sqlite_datetime_modifier(seconds: i64) -> String {
    format!("+{} seconds", seconds.max(1))
}

fn build_idempotency_key(task_id: &str, step_name: &str, input_hash: &str) -> String {
    format!("{task_id}:{step_name}:{input_hash}")
}

fn stable_json_hash(value: &Value) -> String {
    let canonical = canonical_json(value);
    let mut hash = 0xcbf29ce484222325u64;
    for byte in canonical.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string()),
        Value::Array(values) => {
            let items = values.iter().map(canonical_json).collect::<Vec<_>>();
            format!("[{}]", items.join(","))
        }
        Value::Object(map) => {
            let mut entries = map.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            let items = entries
                .into_iter()
                .map(|(key, value)| {
                    let key = serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_string());
                    format!("{key}:{}", canonical_json(value))
                })
                .collect::<Vec<_>>();
            format!("{{{}}}", items.join(","))
        }
    }
}

fn validate_artifact_relative_path(path: &str) -> Result<String, String> {
    PathGuard::validate_relative_path(path)
}

fn validate_composition_output_path(path: &str) -> Result<String, String> {
    let normalized = PathGuard::validate_relative_path(path)?;
    if !normalized.starts_with("outputs/") {
        return Err("composition output path must be inside outputs bucket.".to_string());
    }
    Ok(normalized)
}

fn to_json_text(value: &Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| error.to_string())
}

fn create_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}")
}

#[derive(Debug)]
struct SimpleSqlError(String);

impl std::fmt::Display for SimpleSqlError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SimpleSqlError {}

#[cfg(test)]
mod tests {
    use super::{
        NewCompositionTaskRecord, StepSuccessRecord, StructuredTaskLogRecord, TaskArtifactRecord,
        TaskRepository, TaskStepFailureRecord,
    };
    use crate::core::error::TaskError;
    use crate::db::Database;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn records_step_failure_error_json_attempt_and_log() {
        let path = test_database_path("step_failure");
        let database = Database::open(&path).expect("database should open");
        let repository = TaskRepository::new(&database);
        repository
            .create_task_for_test(
                "project_task_failure",
                "task_failure",
                "step_failure",
                "image_generation",
            )
            .expect("task fixture should save");

        let error = TaskError::from_code_with_detail(
            "provider.timeout",
            "Provider timed out with Authorization: Bearer sk-live-secret-token-123456".to_string(),
            Some(json!({
                "providerStatus": 504,
                "headers": {
                    "Authorization": "Bearer sk-live-secret-token-123456"
                }
            })),
        );

        repository
            .record_step_failure(TaskStepFailureRecord {
                project_id: "project_task_failure".to_string(),
                task_id: "task_failure".to_string(),
                task_step_id: "step_failure".to_string(),
                step_kind: "image_generation".to_string(),
                item_id: Some("item_01".to_string()),
                input_json: json!({ "prompt": "safe prompt" }),
                error,
                duration_ms: Some(3200),
                retry_count: 1,
                relative_path: Some("projects/project_task_failure/images/a.png".to_string()),
            })
            .expect("failure should record");

        let error_json = repository
            .get_step_error_json("step_failure")
            .expect("step error should read")
            .expect("step error should exist");
        assert_eq!(error_json["errorCode"], "provider.timeout");
        assert_eq!(error_json["errorKind"], "provider");
        assert_eq!(error_json["isRetryable"], true);
        assert!(!error_json.to_string().contains("sk-live"));

        let counts: (i64, i64) = database
            .with_connection(|connection| {
                Ok((
                    connection.query_row(
                        "SELECT COUNT(*) FROM task_attempts WHERE step_id = 'step_failure'",
                        [],
                        |row| row.get(0),
                    )?,
                    connection.query_row(
                        "SELECT COUNT(*) FROM task_logs WHERE task_step_id = 'step_failure'",
                        [],
                        |row| row.get(0),
                    )?,
                ))
            })
            .expect("counts should read");
        assert_eq!(counts, (1, 1));

        cleanup(path);
    }

    #[test]
    fn retryable_failure_schedules_retry_and_records_attempt_snapshot() {
        let path = test_database_path("task_retry_first_failure");
        let database = Database::open(&path).expect("database should open");
        let repository = TaskRepository::new(&database);
        repository
            .create_task_for_test(
                "project_retry_first",
                "task_retry_first",
                "step_retry_first",
                "image_generation",
            )
            .expect("task fixture should save");

        let attempt = repository
            .record_step_failure(provider_failure_record(
                "project_retry_first",
                "task_retry_first",
                "step_retry_first",
                "image_generation",
                "provider.timeout",
            ))
            .expect("failure should record");

        assert_eq!(attempt.attempt_index, 1);
        assert_eq!(attempt.status, "failed");
        assert_eq!(attempt.error_code.as_deref(), Some("provider.timeout"));
        assert!(attempt.is_retryable);
        assert_eq!(attempt.backoff_seconds, Some(2));
        assert!(attempt.next_retry_at.is_some());
        assert_eq!(
            attempt.retry_policy_snapshot.as_ref().unwrap()["maxAttempts"],
            3
        );

        let state: (String, String, i64, Option<String>, Option<String>) = database
            .with_connection(|connection| {
                connection.query_row(
                    r#"
                    SELECT tasks.task_status, task_steps.status, task_steps.retry_count,
                           task_steps.next_retry_at, tasks.finished_at
                    FROM tasks
                    JOIN task_steps ON task_steps.task_id = tasks.task_id
                    WHERE tasks.task_id = 'task_retry_first'
                      AND task_steps.step_id = 'step_retry_first'
                    "#,
                    [],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    },
                )
            })
            .expect("retry state should read");
        assert_eq!(state.0, "running");
        assert_eq!(state.1, "retrying");
        assert_eq!(state.2, 1);
        assert!(state.3.is_some());
        assert!(state.4.is_none());

        cleanup(path);
    }

    #[test]
    fn retryable_failures_keep_attempt_history_and_fail_at_max_attempts() {
        let path = test_database_path("task_retry_max_attempts");
        let database = Database::open(&path).expect("database should open");
        let repository = TaskRepository::new(&database);
        repository
            .create_task_for_test(
                "project_retry_max",
                "task_retry_max",
                "step_retry_max",
                "image_generation",
            )
            .expect("task fixture should save");

        let first = repository
            .record_step_failure(provider_failure_record(
                "project_retry_max",
                "task_retry_max",
                "step_retry_max",
                "image_generation",
                "provider.timeout",
            ))
            .expect("first failure should record");
        let second = repository
            .record_step_failure(provider_failure_record(
                "project_retry_max",
                "task_retry_max",
                "step_retry_max",
                "image_generation",
                "provider.timeout",
            ))
            .expect("second failure should record");
        let third = repository
            .record_step_failure(provider_failure_record(
                "project_retry_max",
                "task_retry_max",
                "step_retry_max",
                "image_generation",
                "provider.timeout",
            ))
            .expect("third failure should record");

        assert_eq!(first.attempt_index, 1);
        assert_eq!(first.backoff_seconds, Some(2));
        assert_eq!(second.attempt_index, 2);
        assert_eq!(second.backoff_seconds, Some(5));
        assert_eq!(third.attempt_index, 3);
        assert_eq!(third.backoff_seconds, None);
        assert!(third.next_retry_at.is_none());

        let state: (String, String, i64, Option<String>, i64, i64) = database
            .with_connection(|connection| {
                connection.query_row(
                    r#"
                    SELECT tasks.task_status, task_steps.status, task_steps.retry_count,
                           task_steps.next_retry_at, COUNT(task_attempts.attempt_id),
                           MAX(task_attempts.attempt_index)
                    FROM tasks
                    JOIN task_steps ON task_steps.task_id = tasks.task_id
                    LEFT JOIN task_attempts ON task_attempts.step_id = task_steps.step_id
                    WHERE tasks.task_id = 'task_retry_max'
                      AND task_steps.step_id = 'step_retry_max'
                    GROUP BY tasks.task_status, task_steps.status, task_steps.retry_count, task_steps.next_retry_at
                    "#,
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
                )
            })
            .expect("final retry state should read");
        assert_eq!(state.0, "failed");
        assert_eq!(state.1, "failed");
        assert_eq!(state.2, 3);
        assert!(state.3.is_none());
        assert_eq!(state.4, 3);
        assert_eq!(state.5, 3);

        cleanup(path);
    }

    #[test]
    fn non_retryable_failure_fails_immediately_without_backoff() {
        let path = test_database_path("task_retry_non_retryable");
        let database = Database::open(&path).expect("database should open");
        let repository = TaskRepository::new(&database);
        repository
            .create_task_for_test(
                "project_auth_failed",
                "task_auth_failed",
                "step_auth_failed",
                "image_generation",
            )
            .expect("task fixture should save");

        let attempt = repository
            .record_step_failure(provider_failure_record(
                "project_auth_failed",
                "task_auth_failed",
                "step_auth_failed",
                "image_generation",
                "provider.auth_failed",
            ))
            .expect("auth failure should record");

        assert_eq!(attempt.attempt_index, 1);
        assert!(!attempt.is_retryable);
        assert_eq!(attempt.recover_action.as_deref(), Some("update_secret"));
        assert_eq!(attempt.backoff_seconds, None);
        assert!(attempt.next_retry_at.is_none());

        let state: (String, String, Option<String>) = database
            .with_connection(|connection| {
                connection.query_row(
                    r#"
                    SELECT tasks.task_status, task_steps.status, task_steps.next_retry_at
                    FROM tasks
                    JOIN task_steps ON task_steps.task_id = tasks.task_id
                    WHERE tasks.task_id = 'task_auth_failed'
                      AND task_steps.step_id = 'step_auth_failed'
                    "#,
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
            })
            .expect("auth failure state should read");
        assert_eq!(state.0, "failed");
        assert_eq!(state.1, "failed");
        assert!(state.2.is_none());

        cleanup(path);
    }

    #[test]
    fn insert_log_redacts_secret_like_message() {
        let path = test_database_path("task_log_redaction");
        let database = Database::open(&path).expect("database should open");
        let repository = TaskRepository::new(&database);
        repository
            .create_task_for_test("project_task_log", "task_log", "step_log", "provider_call")
            .expect("task fixture should save");

        let log = repository
            .insert_log(StructuredTaskLogRecord {
                trace_id: "trace_log".to_string(),
                project_id: "project_task_log".to_string(),
                task_id: "task_log".to_string(),
                task_step_id: Some("step_log".to_string()),
                step_kind: Some("provider_call".to_string()),
                item_id: None,
                level: "error".to_string(),
                message: "Authorization: Bearer sk-live-secret-token-123456".to_string(),
                error_code: Some("provider.auth_failed".to_string()),
                duration_ms: Some(10),
                retry_count: 0,
                relative_path: Some("projects/project_task_log/logs/provider.log".to_string()),
                metadata_json: json!({ "keyAlias": "main_provider" }),
            })
            .expect("log should insert");

        assert!(!log.message.contains("sk-live"));
        assert!(log.message.contains("***REDACTED***"));
        assert_eq!(log.error_code.as_deref(), Some("provider.auth_failed"));
        assert_eq!(log.metadata_json["keyAlias"], "main_provider");

        cleanup(path);
    }

    #[test]
    fn creates_full_image_to_video_pipeline_steps() {
        let path = test_database_path("pipeline_steps");
        let database = Database::open(&path).expect("database should open");
        database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO projects (
                        project_id, title, workflow_type, input_type, input_process_mode,
                        aspect_ratio, target_scene_count, segment_duration_seconds,
                        content_language, lifecycle
                    )
                    VALUES ('project_pipeline', 'Pipeline', 'image_to_video', 'topic', 'generate', '9:16', 1, 4, 'zh-CN', 'draft')
                    "#,
                    [],
                )
            })
            .expect("project fixture should save");

        let task = TaskRepository::new(&database)
            .create_image_to_video_task("project_pipeline")
            .expect("task should create");

        assert_eq!(task.task_status, "running");
        assert_eq!(task.current_step.as_deref(), Some("storyboard_review"));
        assert_eq!(task.steps.len(), 12);
        assert_eq!(task.steps[0].step_name, "project_init");
        assert_eq!(task.steps[0].status, "succeeded");
        assert_eq!(task.steps[1].step_name, "storyboard_generation");
        assert_eq!(task.steps[1].status, "succeeded");
        assert_eq!(task.steps[2].step_name, "storyboard_review");
        assert_eq!(task.steps[2].status, "waiting_user");
        assert_eq!(task.steps[5].step_name, "image_review");
        assert_eq!(task.steps[5].status, "pending");
        assert_eq!(task.steps[8].step_name, "video_review");
        assert_eq!(task.steps[8].status, "pending");

        cleanup(path);
    }

    #[test]
    fn approve_step_moves_to_next_review_node() {
        let path = test_database_path("approve_pipeline_step");
        let database = Database::open(&path).expect("database should open");
        database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO projects (
                        project_id, title, workflow_type, input_type, input_process_mode,
                        aspect_ratio, target_scene_count, segment_duration_seconds,
                        content_language, lifecycle
                    )
                    VALUES ('project_approve', 'Approve', 'image_to_video', 'topic', 'generate', '9:16', 1, 4, 'zh-CN', 'draft')
                    "#,
                    [],
                )
            })
            .expect("project fixture should save");

        let repository = TaskRepository::new(&database);
        repository
            .create_image_to_video_task("project_approve")
            .expect("task should create");

        let task = repository
            .approve_step("project_approve", "storyboard_review")
            .expect("step should approve");

        assert_eq!(task.task_status, "running");
        assert_eq!(
            task.current_step.as_deref(),
            Some("image_prompt_generation")
        );
        assert_eq!(task.steps[2].status, "succeeded");
        assert_eq!(task.steps[3].status, "pending");

        let task = repository
            .approve_step("project_approve", "cleanup")
            .expect("final review should approve");

        assert_eq!(task.task_status, "succeeded");
        assert!(task.current_step.is_none());
        assert_eq!(task.steps[11].status, "succeeded");

        cleanup(path);
    }

    #[test]
    fn task_lifecycle_commands_update_database_state() {
        let path = test_database_path("task_lifecycle_commands");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_lifecycle", "Lifecycle");

        let repository = TaskRepository::new(&database);
        repository
            .create_image_to_video_task("project_lifecycle")
            .expect("task should create");
        assert_eq!(
            repository
                .list_tasks(Some("project_lifecycle"))
                .expect("tasks should list")
                .len(),
            1
        );

        let cancelled = repository
            .cancel_task("project_lifecycle")
            .expect("task should cancel");
        assert_eq!(cancelled.task_status, "cancelled");
        assert!(cancelled.current_step.is_none());
        assert!(cancelled
            .steps
            .iter()
            .filter(|step| step.status != "succeeded")
            .all(|step| step.status == "cancelled"));

        let resumed = repository
            .resume_task("project_lifecycle")
            .expect("task should resume");
        assert_eq!(resumed.task_status, "running");
        assert_eq!(resumed.current_step.as_deref(), Some("storyboard_review"));
        assert_eq!(resumed.steps[2].status, "waiting_user");

        let retried = repository
            .retry_task_step("project_lifecycle", "image_generation")
            .expect("step should retry");
        assert_eq!(retried.task_status, "running");
        assert_eq!(retried.current_step.as_deref(), Some("image_generation"));
        assert_eq!(retried.steps[4].status, "pending");

        let started = repository
            .start_task("project_lifecycle")
            .expect("task should start");
        assert_eq!(started.task_status, "running");

        cleanup(path);
    }

    #[test]
    fn upsert_composition_task_persists_relative_output_and_task_detail_reads_latest() {
        let path = test_database_path("composition_task_persist");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_composition", "Composition");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_composition")
            .expect("task should create");
        let saved = repository
            .upsert_composition_task(&NewCompositionTaskRecord {
                task_id: "composition_task_saved".to_string(),
                project_id: "project_composition".to_string(),
                segment_ids: vec!["segment_02".to_string(), "segment_01".to_string()],
                output_path: "outputs/exports/project_composition/composition_task_saved_final.mp4"
                    .to_string(),
                enhancements: json!({ "includeBgm": true, "bgmVolume": 0.18 }),
                status: "succeeded".to_string(),
                progress: 100,
                error_json: None,
            })
            .expect("composition task should save");

        assert_eq!(saved.status, "succeeded");
        assert_eq!(saved.progress, 100);
        assert_eq!(
            saved.segment_ids,
            vec!["segment_02".to_string(), "segment_01".to_string()]
        );
        assert_eq!(
            saved.output_path,
            "outputs/exports/project_composition/composition_task_saved_final.mp4"
        );
        assert_eq!(
            saved
                .enhancements
                .get("includeBgm")
                .and_then(|value| value.as_bool()),
            Some(true)
        );

        let latest = repository
            .latest_composition_task_by_project("project_composition")
            .expect("latest composition task should read")
            .expect("latest composition task should exist");
        assert_eq!(latest.task_id, "composition_task_saved");

        let detail = repository
            .get_task_detail_by_task_id(&task.task_id)
            .expect("task detail should read")
            .expect("task should exist");
        let composition_task = detail
            .composition_task
            .expect("task detail should include latest composition task");
        assert_eq!(composition_task.task_id, "composition_task_saved");
        assert_eq!(
            composition_task.output_path,
            "outputs/exports/project_composition/composition_task_saved_final.mp4"
        );

        cleanup(path);
    }

    #[test]
    fn upsert_composition_task_rejects_unsafe_or_non_output_paths() {
        let path = test_database_path("composition_task_unsafe_path");
        let database = Database::open(&path).expect("database should open");
        insert_project(
            &database,
            "project_composition_unsafe",
            "Unsafe composition",
        );

        let repository = TaskRepository::new(&database);
        for (index, output_path) in [
            "../escape/final.mp4",
            "C:/Users/user/final.mp4",
            "projects/project_composition_unsafe/final.mp4",
        ]
        .into_iter()
        .enumerate()
        {
            let result = repository.upsert_composition_task(&NewCompositionTaskRecord {
                task_id: format!("composition_task_unsafe_{index}"),
                project_id: "project_composition_unsafe".to_string(),
                segment_ids: vec!["segment_unsafe".to_string()],
                output_path: output_path.to_string(),
                enhancements: json!({}),
                status: "failed".to_string(),
                progress: 0,
                error_json: Some(json!({ "code": "ffmpeg.concat_failed" })),
            });

            assert!(
                result.is_err(),
                "{output_path} should not be persisted as a composition output path"
            );
        }

        let count: i64 = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM composition_tasks WHERE project_id = ?1",
                    ["project_composition_unsafe"],
                    |row| row.get(0),
                )
            })
            .expect("composition task count should read");
        assert_eq!(count, 0);

        cleanup(path);
    }

    #[test]
    fn cancel_task_records_request_metadata_and_preserves_succeeded_steps() {
        let path = test_database_path("task_cancel_metadata");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_cancel_metadata", "Cancel metadata");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_cancel_metadata")
            .expect("task should create");
        repository
            .approve_step("project_cancel_metadata", "storyboard_review")
            .expect("task should move to auto step");

        let cancelled = repository
            .cancel_task_with_reason("project_cancel_metadata", Some("user clicked cancel"))
            .expect("task should cancel");
        assert_eq!(cancelled.task_status, "cancelled");
        assert!(cancelled.current_step.is_none());
        assert_eq!(cancelled.steps[0].status, "succeeded");
        assert_eq!(cancelled.steps[1].status, "succeeded");
        assert_eq!(cancelled.steps[2].status, "succeeded");
        assert!(cancelled
            .steps
            .iter()
            .skip(3)
            .all(|step| step.status == "cancelled"));

        let metadata: (i64, Option<String>, Option<String>, Option<String>) = database
            .with_connection(|connection| {
                connection.query_row(
                    r#"
                    SELECT cancel_requested, cancel_requested_at, cancel_reason, trace_id
                    FROM tasks
                    WHERE task_id = ?1
                    "#,
                    [task.task_id.as_str()],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
            })
            .expect("cancel metadata should read");
        assert_eq!(metadata.0, 1);
        assert!(metadata.1.is_some());
        assert_eq!(metadata.2.as_deref(), Some("user clicked cancel"));
        assert!(metadata
            .3
            .as_deref()
            .unwrap_or_default()
            .starts_with("trace_"));

        cleanup(path);
    }

    #[test]
    fn cancelling_running_or_retrying_task_blocks_leases_and_success_writes() {
        let path = test_database_path("task_cancel_blocks_work");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_cancel_blocks", "Cancel blocks work");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_cancel_blocks")
            .expect("task should create");
        repository
            .approve_step("project_cancel_blocks", "storyboard_review")
            .expect("task should move to auto step");
        repository
            .acquire_lease(&task.task_id, "worker_cancel", 60)
            .expect("lease should acquire before cancel");
        repository
            .request_cancellation(&task.task_id, Some("stop running work"))
            .expect("cancel request should save");

        assert!(repository
            .acquire_lease(&task.task_id, "worker_after_cancel", 60)
            .is_err());
        assert!(repository
            .renew_lease(&task.task_id, "worker_cancel", 60)
            .is_err());

        let success = repository.record_step_success(StepSuccessRecord {
            task_id: task.task_id.clone(),
            step_name: "image_prompt_generation".to_string(),
            input_json: json!({ "prompt": "cancelled" }),
            output_json: json!({ "prompt": "should not save" }),
            artifacts: vec![TaskArtifactRecord {
                artifact_id: Some("artifact_cancel_should_not_insert".to_string()),
                task_id: task.task_id.clone(),
                step_id: None,
                project_id: None,
                owner_kind: Some("storyboard_item".to_string()),
                owner_id: Some("item_cancel".to_string()),
                artifact_kind: "prompt".to_string(),
                media_kind: "text".to_string(),
                relative_path: None,
                metadata_json: json!({}),
            }],
        });
        assert!(success.is_err());

        let artifact_count: i64 = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM artifacts WHERE task_id = ?1",
                    [task.task_id.as_str()],
                    |row| row.get(0),
                )
            })
            .expect("artifact count should read");
        assert_eq!(artifact_count, 0);

        cleanup(path);
    }

    #[test]
    fn complete_cancelled_step_handles_waiting_user_and_retrying_steps() {
        let path = test_database_path("task_cancel_waiting_retrying");
        let database = Database::open(&path).expect("database should open");
        let repository = TaskRepository::new(&database);
        repository
            .create_task_for_test(
                "project_cancel_retrying",
                "task_cancel_retrying",
                "step_cancel_retrying",
                "image_generation",
            )
            .expect("retrying task fixture should save");
        repository
            .record_step_failure(provider_failure_record(
                "project_cancel_retrying",
                "task_cancel_retrying",
                "step_cancel_retrying",
                "image_generation",
                "provider.timeout",
            ))
            .expect("failure should schedule retry");

        repository
            .request_cancellation("task_cancel_retrying", Some("cancel retrying"))
            .expect("cancel request should save");
        let cancelled = repository
            .complete_cancelled_step("task_cancel_retrying", "image_generation")
            .expect("retrying task should cancel");
        assert_eq!(cancelled.task_status, "cancelled");
        assert_eq!(cancelled.steps[0].status, "cancelled");

        insert_project(&database, "project_cancel_waiting", "Cancel waiting");
        let waiting_task = repository
            .create_image_to_video_task("project_cancel_waiting")
            .expect("waiting task should create");
        repository
            .request_cancellation(&waiting_task.task_id, Some("cancel waiting review"))
            .expect("waiting cancel request should save");
        let waiting_cancelled = repository
            .complete_cancelled_step(&waiting_task.task_id, "storyboard_review")
            .expect("waiting task should cancel");
        assert_eq!(waiting_cancelled.task_status, "cancelled");
        assert_eq!(waiting_cancelled.steps[2].status, "cancelled");

        cleanup(path);
    }

    #[test]
    fn record_step_success_writes_output_json_and_artifact_index() {
        let path = test_database_path("task_step_success_artifact");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_success", "Success");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_success")
            .expect("task should create");
        repository
            .approve_step("project_success", "storyboard_review")
            .expect("task should move to auto step");

        let input = json!({
            "items": [
                { "itemId": "item_01", "prompt": "清晨卧室" }
            ]
        });
        let output = json!({
            "imageCandidates": [
                { "imageId": "image_01", "relativePath": "projects/project_success/images/image_01.png" }
            ]
        });
        let hit = repository
            .record_step_success(StepSuccessRecord {
                task_id: task.task_id.clone(),
                step_name: "image_generation".to_string(),
                input_json: input.clone(),
                output_json: output.clone(),
                artifacts: vec![TaskArtifactRecord {
                    artifact_id: Some("artifact_image_01".to_string()),
                    task_id: task.task_id.clone(),
                    step_id: None,
                    project_id: None,
                    owner_kind: Some("image_candidate".to_string()),
                    owner_id: Some("image_01".to_string()),
                    artifact_kind: "image".to_string(),
                    media_kind: "image".to_string(),
                    relative_path: Some("projects/project_success/images/image_01.png".to_string()),
                    metadata_json: json!({
                        "providerModelId": "mock/image",
                        "prompt": "清晨卧室"
                    }),
                }],
            })
            .expect("step success should record");

        assert_eq!(hit.status, "succeeded");
        assert_eq!(hit.output_json, output);
        assert_eq!(hit.artifacts.len(), 1);
        assert_eq!(hit.artifacts[0].artifact_id, "artifact_image_01");
        assert_eq!(hit.artifacts[0].artifact_kind, "image");
        assert_eq!(
            hit.artifacts[0].relative_path.as_deref(),
            Some("projects/project_success/images/image_01.png")
        );
        assert!(hit.artifacts[0].idempotency_key.is_some());
        assert!(hit.artifacts[0].input_hash.is_some());

        let reread = repository
            .find_idempotent_step_output(&task.task_id, "image_generation", &input)
            .expect("idempotency lookup should run")
            .expect("idempotency hit should exist");
        assert_eq!(reread.output_json, output);
        assert_eq!(reread.artifacts.len(), 1);

        let attempt: (i64, String, String, Option<String>, Option<i64>) = database
            .with_connection(|connection| {
                connection.query_row(
                    r#"
                    SELECT attempt_index, status, output_json, next_retry_at, backoff_seconds
                    FROM task_attempts
                    WHERE step_id = ?1
                    "#,
                    [hit.step_id.as_str()],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    },
                )
            })
            .expect("success attempt should read");
        assert_eq!(attempt.0, 1);
        assert_eq!(attempt.1, "succeeded");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&attempt.2)
                .expect("output json should parse"),
            output
        );
        assert!(attempt.3.is_none());
        assert!(attempt.4.is_none());

        cleanup(path);
    }

    #[test]
    fn record_step_success_is_idempotent_and_does_not_overwrite_confirmed_output() {
        let path = test_database_path("task_step_success_idempotent");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_idempotent", "Idempotent");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_idempotent")
            .expect("task should create");
        repository
            .approve_step("project_idempotent", "storyboard_review")
            .expect("task should move to auto step");

        let input = json!({ "prompt": "same input", "seed": 7 });
        let first = repository
            .record_step_success(StepSuccessRecord {
                task_id: task.task_id.clone(),
                step_name: "image_generation".to_string(),
                input_json: input.clone(),
                output_json: json!({ "selectedImageId": "image_confirmed" }),
                artifacts: vec![TaskArtifactRecord {
                    artifact_id: Some("artifact_confirmed".to_string()),
                    task_id: task.task_id.clone(),
                    step_id: None,
                    project_id: None,
                    owner_kind: Some("image_candidate".to_string()),
                    owner_id: Some("image_confirmed".to_string()),
                    artifact_kind: "image".to_string(),
                    media_kind: "image".to_string(),
                    relative_path: Some(
                        "projects/project_idempotent/images/confirmed.png".to_string(),
                    ),
                    metadata_json: json!({ "revision": 1 }),
                }],
            })
            .expect("first success should record");

        let second = repository
            .record_step_success(StepSuccessRecord {
                task_id: task.task_id.clone(),
                step_name: "image_generation".to_string(),
                input_json: input.clone(),
                output_json: json!({ "selectedImageId": "image_new_should_not_overwrite" }),
                artifacts: vec![TaskArtifactRecord {
                    artifact_id: Some("artifact_should_not_insert".to_string()),
                    task_id: task.task_id.clone(),
                    step_id: None,
                    project_id: None,
                    owner_kind: Some("image_candidate".to_string()),
                    owner_id: Some("image_new_should_not_overwrite".to_string()),
                    artifact_kind: "image".to_string(),
                    media_kind: "image".to_string(),
                    relative_path: Some("projects/project_idempotent/images/new.png".to_string()),
                    metadata_json: json!({ "revision": 2 }),
                }],
            })
            .expect("second success should return idempotency hit");

        assert_eq!(second.output_json, first.output_json);
        assert_eq!(second.artifacts.len(), 1);
        assert_eq!(second.artifacts[0].artifact_id, "artifact_confirmed");

        let artifact_count: i64 = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM artifacts WHERE task_id = ?1",
                    [task.task_id.as_str()],
                    |row| row.get(0),
                )
            })
            .expect("artifact count should read");
        assert_eq!(artifact_count, 1);

        cleanup(path);
    }

    #[test]
    fn record_step_success_rejects_unsafe_artifact_paths() {
        let path = test_database_path("task_step_success_unsafe_path");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_unsafe_artifact", "Unsafe artifact");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_unsafe_artifact")
            .expect("task should create");

        let result = repository.record_step_success(StepSuccessRecord {
            task_id: task.task_id.clone(),
            step_name: "image_generation".to_string(),
            input_json: json!({ "prompt": "unsafe" }),
            output_json: json!({ "imageCandidates": [] }),
            artifacts: vec![TaskArtifactRecord {
                artifact_id: None,
                task_id: task.task_id.clone(),
                step_id: None,
                project_id: None,
                owner_kind: Some("image_candidate".to_string()),
                owner_id: Some("image_unsafe".to_string()),
                artifact_kind: "image".to_string(),
                media_kind: "image".to_string(),
                relative_path: Some("../escape.png".to_string()),
                metadata_json: json!({}),
            }],
        });

        assert!(result.is_err());
        let artifact_count: i64 = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM artifacts WHERE task_id = ?1",
                    [task.task_id.as_str()],
                    |row| row.get(0),
                )
            })
            .expect("artifact count should read");
        assert_eq!(artifact_count, 0);

        cleanup(path);
    }

    #[test]
    fn idempotency_lookup_can_require_existing_artifact_files() {
        let root = test_root("task_step_success_existing_file");
        let database_path = root.join("app.sqlite3");
        let workspace_root = root.join("workspace");
        fs::create_dir_all(workspace_root.join("projects/project_existing/images"))
            .expect("artifact dir should exist");
        fs::write(
            workspace_root.join("projects/project_existing/images/image.png"),
            "png",
        )
        .expect("artifact file should write");
        let database = Database::open(&database_path).expect("database should open");
        insert_project(&database, "project_existing", "Existing");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_existing")
            .expect("task should create");
        let input = json!({ "prompt": "existing file" });
        repository
            .record_step_success(StepSuccessRecord {
                task_id: task.task_id.clone(),
                step_name: "image_generation".to_string(),
                input_json: input.clone(),
                output_json: json!({ "imageCandidates": ["image_existing"] }),
                artifacts: vec![TaskArtifactRecord {
                    artifact_id: Some("artifact_existing".to_string()),
                    task_id: task.task_id.clone(),
                    step_id: None,
                    project_id: None,
                    owner_kind: Some("image_candidate".to_string()),
                    owner_id: Some("image_existing".to_string()),
                    artifact_kind: "image".to_string(),
                    media_kind: "image".to_string(),
                    relative_path: Some("projects/project_existing/images/image.png".to_string()),
                    metadata_json: json!({}),
                }],
            })
            .expect("step success should record");

        assert!(repository
            .find_idempotent_step_output_with_existing_artifacts(
                &workspace_root,
                &task.task_id,
                "image_generation",
                &input,
            )
            .expect("existing artifact lookup should run")
            .is_some());

        fs::remove_file(workspace_root.join("projects/project_existing/images/image.png"))
            .expect("artifact file should remove");
        assert!(repository
            .find_idempotent_step_output_with_existing_artifacts(
                &workspace_root,
                &task.task_id,
                "image_generation",
                &input,
            )
            .is_err());

        cleanup(root);
    }

    #[test]
    fn acquire_lease_marks_worker_and_current_auto_step_running() {
        let path = test_database_path("task_lease_acquire");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_lease", "Lease");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_lease")
            .expect("task should create");
        repository
            .approve_step("project_lease", "storyboard_review")
            .expect("task should move to auto step");

        let lease = repository
            .acquire_lease(&task.task_id, "worker_a", 60)
            .expect("lease should acquire");
        assert_eq!(lease.task_id, task.task_id);
        assert_eq!(lease.worker_id, "worker_a");
        assert!(!lease.lease_expires_at.is_empty());

        let state: (Option<String>, Option<String>, Option<String>, String) = database
            .with_connection(|connection| {
                connection.query_row(
                    r#"
                    SELECT tasks.worker_id, tasks.lease_expires_at, tasks.started_at, task_steps.status
                    FROM tasks
                    JOIN task_steps ON task_steps.task_id = tasks.task_id
                    WHERE tasks.task_id = ?1 AND task_steps.step_name = 'image_prompt_generation'
                    "#,
                    [task.task_id.as_str()],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
            })
            .expect("lease state should read");
        assert_eq!(state.0.as_deref(), Some("worker_a"));
        assert!(state.1.is_some());
        assert!(state.2.is_some());
        assert_eq!(state.3, "running");

        cleanup(path);
    }

    #[test]
    fn acquire_lease_waits_until_retry_backoff_is_due() {
        let path = test_database_path("task_lease_retry_backoff");
        let database = Database::open(&path).expect("database should open");
        let repository = TaskRepository::new(&database);
        repository
            .create_task_for_test(
                "project_retry_backoff",
                "task_retry_backoff",
                "step_retry_backoff",
                "image_generation",
            )
            .expect("task fixture should save");
        repository
            .record_step_failure(provider_failure_record(
                "project_retry_backoff",
                "task_retry_backoff",
                "step_retry_backoff",
                "image_generation",
                "provider.timeout",
            ))
            .expect("failure should schedule retry");

        assert!(repository
            .acquire_lease("task_retry_backoff", "worker_early", 60)
            .is_err());

        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE task_steps SET next_retry_at = datetime('now', '-1 second') WHERE step_id = 'step_retry_backoff'",
                    [],
                )
            })
            .expect("retry due fixture should save");

        let lease = repository
            .acquire_lease("task_retry_backoff", "worker_due", 60)
            .expect("lease should acquire after retry is due");
        assert_eq!(lease.worker_id, "worker_due");

        cleanup(path);
    }

    #[test]
    fn renew_lease_only_allows_current_worker() {
        let path = test_database_path("task_lease_renew");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_renew", "Renew");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_renew")
            .expect("task should create");
        repository
            .approve_step("project_renew", "storyboard_review")
            .expect("task should move to auto step");
        repository
            .acquire_lease(&task.task_id, "worker_a", 60)
            .expect("lease should acquire");

        assert!(repository
            .renew_lease(&task.task_id, "worker_b", 60)
            .is_err());

        let renewed = repository
            .renew_lease(&task.task_id, "worker_a", 90)
            .expect("same worker should renew");
        assert_eq!(renewed.worker_id, "worker_a");

        cleanup(path);
    }

    #[test]
    fn scan_marks_running_auto_task_without_valid_lease_recoverable() {
        let path = test_database_path("task_recoverable_scan");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_recoverable", "Recoverable");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_recoverable")
            .expect("task should create");
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE tasks SET current_step = 'image_generation', worker_id = NULL, lease_expires_at = NULL WHERE task_id = ?1",
                    [task.task_id.as_str()],
                )?;
                connection.execute(
                    "UPDATE task_steps SET status = 'running' WHERE task_id = ?1 AND step_name = 'image_generation'",
                    [task.task_id.as_str()],
                )
            })
            .expect("recoverable fixture should save");

        let recoverable = repository
            .scan_and_mark_recoverable_tasks()
            .expect("scan should run");
        assert_eq!(recoverable.len(), 1);
        assert_eq!(recoverable[0].task_id, task.task_id);

        let state: (
            String,
            Option<String>,
            Option<String>,
            String,
            String,
            Option<String>,
        ) = database
            .with_connection(|connection| {
                connection.query_row(
                    r#"
                    SELECT
                        tasks.task_status,
                        tasks.worker_id,
                        tasks.lease_expires_at,
                        task_steps.status,
                        task_steps.error_code,
                        task_steps.recover_action
                    FROM tasks
                    JOIN task_steps ON task_steps.task_id = tasks.task_id
                    WHERE tasks.task_id = ?1 AND task_steps.step_name = 'image_generation'
                    "#,
                    [task.task_id.as_str()],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                        ))
                    },
                )
            })
            .expect("recoverable state should read");
        assert_eq!(state.0, "failed");
        assert!(state.1.is_none());
        assert!(state.2.is_none());
        assert_eq!(state.3, "failed");
        assert_eq!(state.4, "task.resume_required");
        assert_eq!(state.5.as_deref(), Some("resume_task"));

        cleanup(path);
    }

    #[test]
    fn scan_does_not_mark_scheduled_retry_as_recoverable() {
        let path = test_database_path("task_recoverable_retry_skip");
        let database = Database::open(&path).expect("database should open");
        let repository = TaskRepository::new(&database);
        repository
            .create_task_for_test(
                "project_retry_scan_skip",
                "task_retry_scan_skip",
                "step_retry_scan_skip",
                "image_generation",
            )
            .expect("task fixture should save");
        repository
            .record_step_failure(provider_failure_record(
                "project_retry_scan_skip",
                "task_retry_scan_skip",
                "step_retry_scan_skip",
                "image_generation",
                "provider.timeout",
            ))
            .expect("failure should schedule retry");

        let recoverable = repository
            .scan_and_mark_recoverable_tasks()
            .expect("scan should run");
        assert!(recoverable.is_empty());

        let state: (String, String, Option<String>) = database
            .with_connection(|connection| {
                connection.query_row(
                    r#"
                    SELECT tasks.task_status, task_steps.status, task_steps.next_retry_at
                    FROM tasks
                    JOIN task_steps ON task_steps.task_id = tasks.task_id
                    WHERE tasks.task_id = 'task_retry_scan_skip'
                      AND task_steps.step_id = 'step_retry_scan_skip'
                    "#,
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
            })
            .expect("scheduled retry state should read");
        assert_eq!(state.0, "running");
        assert_eq!(state.1, "retrying");
        assert!(state.2.is_some());

        cleanup(path);
    }

    #[test]
    fn scan_does_not_change_waiting_user_review_task() {
        let path = test_database_path("task_recoverable_review_skip");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_review_wait", "Review wait");

        let repository = TaskRepository::new(&database);
        let task = repository
            .create_image_to_video_task("project_review_wait")
            .expect("task should create");

        let recoverable = repository
            .scan_and_mark_recoverable_tasks()
            .expect("scan should run");
        assert!(recoverable.is_empty());

        let detail = repository
            .get_task_detail_by_task_id(&task.task_id)
            .expect("task should read")
            .expect("task should exist");
        assert_eq!(detail.task_status, "running");
        assert_eq!(detail.current_step.as_deref(), Some("storyboard_review"));
        assert_eq!(detail.steps[2].status, "waiting_user");

        cleanup(path);
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

    fn provider_failure_record(
        project_id: &str,
        task_id: &str,
        step_id: &str,
        step_kind: &str,
        error_code: &str,
    ) -> TaskStepFailureRecord {
        TaskStepFailureRecord {
            project_id: project_id.to_string(),
            task_id: task_id.to_string(),
            task_step_id: step_id.to_string(),
            step_kind: step_kind.to_string(),
            item_id: Some("item_retry_01".to_string()),
            input_json: json!({ "prompt": "retry input" }),
            error: TaskError::from_code(error_code, "Provider failure".to_string()),
            duration_ms: Some(1200),
            retry_count: 999,
            relative_path: None,
        }
    }

    fn test_database_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-task-{name}-{}-{nanos}.sqlite3",
            std::process::id()
        ))
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-task-root-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        if path.is_dir() {
            let _ = fs::remove_dir_all(path);
        } else {
            let _ = fs::remove_file(&path);
            let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
            let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
        }
    }
}
