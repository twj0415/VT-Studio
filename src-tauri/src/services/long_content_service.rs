use crate::db::long_content_repository::{LongContentRepository, NewLongContentPlanRecord};
use crate::db::project_repository::ProjectRepository;
use crate::db::Database;
use crate::domain::long_content::{
    ListLongContentPlansRequest, LongContentPlanDto, LongContentPlanIdRequest,
    SaveLongContentPlanRequest, LONG_CONTENT_PLAN_SCHEMA_VERSION,
};
use crate::domain::structured_output::ValidateStructuredOutputRequest;
use crate::services::structured_output_service::validate_structured_output;
use serde_json::{json, Value};

const PLAN_KINDS: &[&str] = &[
    "story_skeleton",
    "adaptation_strategy",
    "episode_script",
    "storyboard_table",
    "asset_extraction",
];

pub fn save_long_content_plan(
    database: &Database,
    request: SaveLongContentPlanRequest,
) -> Result<LongContentPlanDto, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for long content plan.".to_string());
    }
    if !PLAN_KINDS.contains(&request.plan_kind.as_str()) {
        return Err(format!(
            "Unsupported long content plan_kind: {}",
            request.plan_kind
        ));
    }
    ProjectRepository::new(database)
        .get_detail(&request.project_id)?
        .ok_or_else(|| format!("Project not found: {}", request.project_id))?;

    let validation = validate_structured_output(ValidateStructuredOutputRequest {
        raw_output: request.raw_output,
        output_schema: long_content_plan_schema(&request.plan_kind),
        expected_count: None,
        repair_attempt_count: Some(0),
        max_repair_attempts: Some(0),
    })?;
    if !validation.valid {
        return Err(format!(
            "long content plan schema invalid: {}",
            validation.errors.join("; ")
        ));
    }
    let content = validation
        .parsed_json
        .ok_or_else(|| "long content plan schema invalid: parsed JSON is missing.".to_string())?;

    LongContentRepository::new(database).insert_plan(NewLongContentPlanRecord::new(
        request.project_id,
        request.plan_kind,
        request.parent_plan_id,
        request.chapter_ids,
        content,
        LONG_CONTENT_PLAN_SCHEMA_VERSION,
    ))
}

pub fn list_long_content_plans(
    database: &Database,
    request: ListLongContentPlansRequest,
) -> Result<Vec<LongContentPlanDto>, String> {
    if request.project_id.trim().is_empty() {
        return Err("project_id is required for long content plans.".to_string());
    }
    if let Some(plan_kind) = request.plan_kind.as_deref() {
        if !PLAN_KINDS.contains(&plan_kind) {
            return Err(format!("Unsupported long content plan_kind: {plan_kind}"));
        }
    }
    LongContentRepository::new(database)
        .list_plans(&request.project_id, request.plan_kind.as_deref())
}

pub fn approve_long_content_plan(
    database: &Database,
    request: LongContentPlanIdRequest,
) -> Result<LongContentPlanDto, String> {
    LongContentRepository::new(database).set_plan_status(&request.plan_id, "approved")
}

pub fn reject_long_content_plan(
    database: &Database,
    request: LongContentPlanIdRequest,
) -> Result<LongContentPlanDto, String> {
    LongContentRepository::new(database).set_plan_status(&request.plan_id, "rejected")
}

fn long_content_plan_schema(plan_kind: &str) -> Value {
    match plan_kind {
        "story_skeleton" => json!({
            "type": "object",
            "required": ["storyArc", "keyBeats"],
            "properties": {
                "storyArc": { "type": "string" },
                "keyBeats": {
                    "type": "array",
                    "minItems": 1,
                    "items": { "type": "string" }
                }
            }
        }),
        "adaptation_strategy" => json!({
            "type": "object",
            "required": ["targetFormat", "strategy"],
            "properties": {
                "targetFormat": { "type": "string" },
                "strategy": { "type": "string" },
                "constraints": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            }
        }),
        "episode_script" => json!({
            "type": "object",
            "required": ["episodes"],
            "properties": {
                "episodes": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["title", "summary"],
                        "properties": {
                            "title": { "type": "string" },
                            "summary": { "type": "string" }
                        }
                    }
                }
            }
        }),
        "storyboard_table" => json!({
            "type": "object",
            "required": ["rows"],
            "properties": {
                "rows": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["sourceText", "visualGoal"],
                        "properties": {
                            "sourceText": { "type": "string" },
                            "visualGoal": { "type": "string" }
                        }
                    }
                }
            }
        }),
        "asset_extraction" => json!({
            "type": "object",
            "required": ["characters", "locations"],
            "properties": {
                "characters": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "locations": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            }
        }),
        _ => json!({ "type": "object" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::project::CreateProjectRequest;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn long_content_plan_requires_schema_and_waits_for_user_approval() {
        let root = test_root("long_content_plan");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_long_plan");

        let error = save_long_content_plan(
            &database,
            SaveLongContentPlanRequest {
                project_id: "project_long_plan".to_string(),
                plan_kind: "story_skeleton".to_string(),
                parent_plan_id: None,
                chapter_ids: vec![],
                raw_output: json!({ "storyArc": "成长" }).to_string(),
            },
        )
        .expect_err("missing keyBeats should fail");
        assert!(error.contains("long content plan schema invalid"));

        let plan = save_long_content_plan(
            &database,
            SaveLongContentPlanRequest {
                project_id: "project_long_plan".to_string(),
                plan_kind: "story_skeleton".to_string(),
                parent_plan_id: None,
                chapter_ids: vec!["chapter_1".to_string()],
                raw_output: json!({
                    "storyArc": "成长",
                    "keyBeats": ["觉醒", "冲突", "选择"]
                })
                .to_string(),
            },
        )
        .expect("valid plan should save");
        assert_eq!(plan.status, "waiting_user");
        assert_eq!(plan.schema_version, LONG_CONTENT_PLAN_SCHEMA_VERSION);

        let approved = approve_long_content_plan(
            &database,
            LongContentPlanIdRequest {
                plan_id: plan.plan_id,
            },
        )
        .expect("plan should approve");
        assert_eq!(approved.status, "approved");

        cleanup(root);
    }

    fn create_project(database: &Database, project_id: &str) {
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "Long content".to_string(),
                    workflow_type: "image_to_video".to_string(),
                    input_type: "novel".to_string(),
                    topic: None,
                    source_text: Some("seed".to_string()),
                    source_text_path: None,
                    content_language: "zh-CN".to_string(),
                    tone: None,
                    aspect_ratio: "9:16".to_string(),
                    target_scene_count: 8,
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
