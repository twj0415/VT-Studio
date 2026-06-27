use crate::db::project_repository::ProjectRepository;
use crate::db::video_pack_repository::{VideoPackRecord, VideoPackRepository};
use crate::db::Database;
use crate::domain::video_pack::{
    ListVideoPacksRequest, SaveProjectConfigAsVideoPackRequest, SetVideoPackEnabledRequest,
    UpsertUserVideoPackRequest, VideoPackDto, VideoPackIdRequest,
};
use crate::security::secret_guard::reject_json_secrets;
use crate::services::prompt_service::resolve_creative_rule_refs;
use serde_json::{json, Value};
use std::path::Path;

pub fn list_video_packs(
    database: &Database,
    workspace_root: &Path,
    request: ListVideoPacksRequest,
) -> Result<Vec<VideoPackDto>, String> {
    ensure_builtin_video_packs(database, workspace_root)?;
    let mut packs = VideoPackRepository::new(database).list()?;

    if let Some(source_type) = request.source_type.as_deref() {
        packs.retain(|pack| pack.source_type == source_type);
    }
    if !request.include_disabled.unwrap_or(false) {
        packs.retain(|pack| pack.is_enabled);
    }

    Ok(packs)
}

pub fn get_video_pack(
    database: &Database,
    workspace_root: &Path,
    request: VideoPackIdRequest,
) -> Result<VideoPackDto, String> {
    validate_identifier("pack_id", &request.pack_id)?;
    ensure_builtin_video_packs(database, workspace_root)?;
    VideoPackRepository::new(database)
        .get(&request.pack_id)?
        .ok_or_else(|| format!("video_pack.not_found: {}", request.pack_id))
}

pub fn clone_video_pack_to_user(
    database: &Database,
    workspace_root: &Path,
    request: VideoPackIdRequest,
) -> Result<VideoPackDto, String> {
    let source = get_video_pack(database, workspace_root, request)?;
    if source.source_type != "builtin" {
        return Err(
            "video_pack.clone_source_invalid: only builtin video packs can be cloned.".to_string(),
        );
    }

    let pack = VideoPackRecord {
        pack_id: VideoPackRepository::create_pack_id(),
        source_type: "user".to_string(),
        name: format!("{} Copy", source.name),
        description: source.description,
        applicable_input_types: source.applicable_input_types,
        content_category: source.content_category,
        default_tone: source.default_tone,
        default_aspect_ratio: source.default_aspect_ratio,
        default_duration_seconds: source.default_duration_seconds,
        default_scene_count: source.default_scene_count,
        rule_refs: source.rule_refs,
        recommended_executable_refs: source.recommended_executable_refs,
        asset_refs: source.asset_refs,
        is_enabled: false,
        created_at: None,
        updated_at: None,
    };
    validate_pack_record(&pack, true)?;
    VideoPackRepository::new(database).upsert(&pack)
}

pub fn upsert_user_video_pack(
    database: &Database,
    workspace_root: &Path,
    request: UpsertUserVideoPackRequest,
) -> Result<VideoPackDto, String> {
    let pack_id = request
        .pack_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(VideoPackRepository::create_pack_id);
    if let Some(existing) = VideoPackRepository::new(database).get(&pack_id)? {
        if existing.source_type != "user" {
            return Err(
                "video_pack.builtin_readonly: builtin video packs cannot be edited.".to_string(),
            );
        }
    }

    let pack = VideoPackRecord {
        pack_id,
        source_type: "user".to_string(),
        name: request.name,
        description: request.description,
        applicable_input_types: request.applicable_input_types,
        content_category: request.content_category,
        default_tone: request.default_tone,
        default_aspect_ratio: request.default_aspect_ratio,
        default_duration_seconds: request.default_duration_seconds,
        default_scene_count: request.default_scene_count,
        rule_refs: resolve_creative_rule_refs(workspace_root, &request.rule_refs)?,
        recommended_executable_refs: request.recommended_executable_refs,
        asset_refs: request.asset_refs,
        is_enabled: request.is_enabled,
        created_at: None,
        updated_at: None,
    };
    validate_pack_record(&pack, true)?;
    VideoPackRepository::new(database).upsert(&pack)
}

pub fn set_video_pack_enabled(
    database: &Database,
    workspace_root: &Path,
    request: SetVideoPackEnabledRequest,
) -> Result<VideoPackDto, String> {
    validate_identifier("pack_id", &request.pack_id)?;
    let pack = VideoPackRepository::new(database)
        .get(&request.pack_id)?
        .ok_or_else(|| format!("video_pack.not_found: {}", request.pack_id))?;
    if pack.source_type != "user" {
        return Err(
            "video_pack.builtin_readonly: builtin video packs cannot be disabled from UI."
                .to_string(),
        );
    }

    upsert_user_video_pack(
        database,
        workspace_root,
        UpsertUserVideoPackRequest {
            pack_id: Some(pack.pack_id),
            name: pack.name,
            description: pack.description,
            applicable_input_types: pack.applicable_input_types,
            content_category: pack.content_category,
            default_tone: pack.default_tone,
            default_aspect_ratio: pack.default_aspect_ratio,
            default_duration_seconds: pack.default_duration_seconds,
            default_scene_count: pack.default_scene_count,
            rule_refs: pack.rule_refs,
            recommended_executable_refs: pack.recommended_executable_refs,
            asset_refs: pack.asset_refs,
            is_enabled: request.is_enabled,
        },
    )
}

pub fn delete_user_video_pack(
    database: &Database,
    request: VideoPackIdRequest,
) -> Result<VideoPackDto, String> {
    validate_identifier("pack_id", &request.pack_id)?;
    let repository = VideoPackRepository::new(database);
    let pack = repository
        .get(&request.pack_id)?
        .ok_or_else(|| format!("video_pack.not_found: {}", request.pack_id))?;
    if pack.source_type != "user" {
        return Err(
            "video_pack.builtin_readonly: builtin video packs cannot be deleted.".to_string(),
        );
    }
    if pack.project_reference_count > 0 {
        return Err(format!(
            "video_pack.in_use: {} project(s) reference this video pack.",
            pack.project_reference_count
        ));
    }
    if task_snapshot_reference_count(database, &request.pack_id)? > 0 {
        return Err("video_pack.in_use: task snapshots reference this video pack.".to_string());
    }
    repository.delete(&request.pack_id)
}

pub fn save_project_config_as_video_pack(
    database: &Database,
    workspace_root: &Path,
    request: SaveProjectConfigAsVideoPackRequest,
) -> Result<VideoPackDto, String> {
    validate_identifier("project_id", &request.project_id)?;
    let project = ProjectRepository::new(database)
        .get_detail(&request.project_id)?
        .ok_or_else(|| format!("project.not_found: {}", request.project_id))?
        .project;

    let default_duration_seconds = (project.target_scene_count as f64
        * project.segment_duration_seconds)
        .round()
        .max(1.0) as u32;
    let pack = VideoPackRecord {
        pack_id: VideoPackRepository::create_pack_id(),
        source_type: "user".to_string(),
        name: request.name,
        description: request.description,
        applicable_input_types: vec![project.input_type],
        content_category: None,
        default_tone: project.tone,
        default_aspect_ratio: project.aspect_ratio,
        default_duration_seconds,
        default_scene_count: project.target_scene_count,
        rule_refs: resolve_creative_rule_refs(workspace_root, &project.rule_refs)?,
        recommended_executable_refs: project.executable_refs,
        asset_refs: json!([]),
        is_enabled: true,
        created_at: None,
        updated_at: None,
    };
    validate_pack_record(&pack, true)?;
    VideoPackRepository::new(database).upsert(&pack)
}

pub fn ensure_builtin_video_packs(
    database: &Database,
    workspace_root: &Path,
) -> Result<(), String> {
    let repository = VideoPackRepository::new(database);
    repository.upsert(&builtin_knowledge_pack(workspace_root)?)?;
    repository.upsert(&builtin_story_pack(workspace_root)?)?;
    Ok(())
}

fn builtin_knowledge_pack(workspace_root: &Path) -> Result<VideoPackRecord, String> {
    let rule_refs = resolve_creative_rule_refs(
        workspace_root,
        &json!({
            "script": {
                "ruleKey": "script.topic_narration",
                "ruleId": "builtin:script:script_topic_narration"
            },
            "storyboard": {
                "ruleKey": "storyboard.default",
                "ruleId": "builtin:storyboard:storyboard_default"
            },
            "character": {
                "ruleKey": "character.default",
                "ruleId": "builtin:character:character_default"
            },
            "scene": {
                "ruleKey": "scene.default",
                "ruleId": "builtin:scene:scene_default"
            },
            "style": {
                "ruleKey": "style.default",
                "ruleId": "builtin:style:style_default"
            },
            "image_prompt": {
                "ruleKey": "image_prompt.shot_frame",
                "ruleId": "builtin:image_prompt:image_prompt_shot_frame"
            },
            "storyboard_image": {
                "ruleKey": "storyboard_image.default",
                "ruleId": "builtin:storyboard_image:storyboard_image_default"
            },
            "video_prompt": {
                "ruleKey": "video_prompt.image_to_video",
                "ruleId": "builtin:video_prompt:video_prompt_image_to_video"
            },
            "review": {
                "ruleKey": "review.safety",
                "ruleId": "builtin:review:review_safety"
            }
        }),
    )?;
    Ok(VideoPackRecord {
        pack_id: "pack_knowledge_short".to_string(),
        source_type: "builtin".to_string(),
        name: "知识科普短视频".to_string(),
        description: "适合 30-90 秒竖屏知识内容，强调结构清楚、信息密度和镜头可生成性。"
            .to_string(),
        applicable_input_types: vec![
            "topic".to_string(),
            "paste".to_string(),
            "article".to_string(),
        ],
        content_category: Some("knowledge".to_string()),
        default_tone: Some("清楚、克制、有信息量".to_string()),
        default_aspect_ratio: "9:16".to_string(),
        default_duration_seconds: 60,
        default_scene_count: 8,
        rule_refs,
        recommended_executable_refs: json!({
            "llm": {},
            "image": {},
            "video": {}
        }),
        asset_refs: json!([]),
        is_enabled: true,
        created_at: None,
        updated_at: None,
    })
}

fn builtin_story_pack(workspace_root: &Path) -> Result<VideoPackRecord, String> {
    let rule_refs = resolve_creative_rule_refs(
        workspace_root,
        &json!({
            "script": {
                "ruleKey": "script.topic_narration",
                "ruleId": "builtin:script:script_topic_narration"
            },
            "storyboard": {
                "ruleKey": "storyboard.default",
                "ruleId": "builtin:storyboard:storyboard_default"
            },
            "character": {
                "ruleKey": "character.default",
                "ruleId": "builtin:character:character_default"
            },
            "scene": {
                "ruleKey": "scene.default",
                "ruleId": "builtin:scene:scene_default"
            },
            "style": {
                "ruleKey": "style.default",
                "ruleId": "builtin:style:style_default"
            },
            "image_prompt": {
                "ruleKey": "image_prompt.shot_frame",
                "ruleId": "builtin:image_prompt:image_prompt_shot_frame"
            },
            "storyboard_image": {
                "ruleKey": "storyboard_image.default",
                "ruleId": "builtin:storyboard_image:storyboard_image_default"
            },
            "video_prompt": {
                "ruleKey": "video_prompt.image_to_video",
                "ruleId": "builtin:video_prompt:video_prompt_image_to_video"
            },
            "review": {
                "ruleKey": "review.safety",
                "ruleId": "builtin:review:review_safety"
            }
        }),
    )?;
    Ok(VideoPackRecord {
        pack_id: "pack_story_short".to_string(),
        source_type: "builtin".to_string(),
        name: "故事叙事短视频".to_string(),
        description: "适合人物故事、经历复盘和情绪叙事，保留角色、场景和镜头提示词的连续性。"
            .to_string(),
        applicable_input_types: vec![
            "topic".to_string(),
            "paste".to_string(),
            "article".to_string(),
        ],
        content_category: Some("story".to_string()),
        default_tone: Some("真实、克制、带一点悬念".to_string()),
        default_aspect_ratio: "9:16".to_string(),
        default_duration_seconds: 45,
        default_scene_count: 7,
        rule_refs,
        recommended_executable_refs: json!({
            "llm": {},
            "image": {},
            "video": {}
        }),
        asset_refs: json!([]),
        is_enabled: true,
        created_at: None,
        updated_at: None,
    })
}

fn validate_pack_record(pack: &VideoPackRecord, require_user: bool) -> Result<(), String> {
    validate_identifier("pack_id", &pack.pack_id)?;
    if require_user && pack.source_type != "user" {
        return Err(
            "video_pack.source_type_invalid: only user video packs can be edited.".to_string(),
        );
    }
    validate_one_of("source_type", &pack.source_type, &["builtin", "user"])?;
    if pack.name.trim().is_empty() {
        return Err("video_pack.name_required: name cannot be empty.".to_string());
    }
    if pack.default_duration_seconds == 0 {
        return Err(
            "video_pack.default_duration_invalid: default duration must be greater than 0."
                .to_string(),
        );
    }
    if pack.default_scene_count == 0 {
        return Err(
            "video_pack.default_scene_count_invalid: default scene count must be greater than 0."
                .to_string(),
        );
    }
    if pack.applicable_input_types.is_empty() {
        return Err(
            "video_pack.input_types_required: at least one input type is required.".to_string(),
        );
    }
    for input_type in &pack.applicable_input_types {
        validate_one_of(
            "input_type",
            input_type,
            &["topic", "paste", "article", "novel", "material"],
        )?;
    }
    validate_one_of(
        "default_aspect_ratio",
        &pack.default_aspect_ratio,
        &["9:16", "16:9", "1:1", "4:3", "3:4", "vertical_9_16"],
    )?;
    validate_json_object("rule_refs", &pack.rule_refs)?;
    validate_rule_refs_shape(&pack.rule_refs)?;
    validate_json_object(
        "recommended_executable_refs",
        &pack.recommended_executable_refs,
    )?;
    validate_json_array("asset_refs", &pack.asset_refs)?;
    reject_json_secrets(&pack.rule_refs)?;
    reject_json_secrets(&pack.recommended_executable_refs)?;
    reject_json_secrets(&pack.asset_refs)?;
    reject_forbidden_json_keys(&pack.rule_refs, &["body", "prompt_body", "promptBody"])?;
    reject_forbidden_json_keys(
        &pack.recommended_executable_refs,
        &[
            "apiKey",
            "api_key",
            "authorization",
            "headers",
            "baseUrl",
            "base_url",
            "providerConfig",
        ],
    )?;
    reject_forbidden_json_keys(&pack.asset_refs, &["absolutePath", "absolute_path", "path"])?;
    Ok(())
}

fn task_snapshot_reference_count(database: &Database, pack_id: &str) -> Result<i64, String> {
    database
        .with_connection(|connection| {
            let task_count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM tasks WHERE summary LIKE '%' || ?1 || '%' OR last_error_json LIKE '%' || ?1 || '%'",
                [pack_id],
                |row| row.get(0),
            )?;
            let step_count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM task_steps WHERE input_json LIKE '%' || ?1 || '%' OR output_json LIKE '%' || ?1 || '%'",
                [pack_id],
                |row| row.get(0),
            )?;
            Ok(task_count + step_count)
        })
        .map_err(|error| error.to_string())
}

fn validate_identifier(name: &str, value: &str) -> Result<(), String> {
    let valid = !value.trim().is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | ':')
        });
    if valid {
        Ok(())
    } else {
        Err(format!("{name} contains unsupported characters."))
    }
}

fn validate_one_of(name: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!("{name} has unsupported value: {value}"))
    }
}

fn validate_json_object(name: &str, value: &Value) -> Result<(), String> {
    if value.is_object() {
        Ok(())
    } else {
        Err(format!("{name} must be a JSON object."))
    }
}

fn validate_json_array(name: &str, value: &Value) -> Result<(), String> {
    if value.is_array() {
        Ok(())
    } else {
        Err(format!("{name} must be a JSON array."))
    }
}

fn validate_rule_refs_shape(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "rule_refs must be a JSON object.".to_string())?;
    let allowed_slots = [
        "script",
        "storyboard",
        "character",
        "scene",
        "style",
        "image_prompt",
        "storyboard_image",
        "video_prompt",
        "review",
    ];
    for (slot, entry) in object {
        if !allowed_slots.contains(&slot.as_str()) {
            return Err(format!(
                "video_pack.rule_refs_invalid: unsupported slot {slot}."
            ));
        }
        let entry = entry.as_object().ok_or_else(|| {
            format!("video_pack.rule_refs_invalid: {slot} must be an object, not a legacy string.")
        })?;
        for key in ["ruleKey", "ruleId", "sourceType", "ruleType"] {
            let Some(value) = entry.get(key).and_then(Value::as_str) else {
                return Err(format!(
                    "video_pack.rule_refs_invalid: {slot}.{key} is required."
                ));
            };
            if value.trim().is_empty() {
                return Err(format!(
                    "video_pack.rule_refs_invalid: {slot}.{key} cannot be empty."
                ));
            }
        }
    }
    Ok(())
}

fn reject_forbidden_json_keys(value: &Value, forbidden_keys: &[&str]) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if forbidden_keys
                    .iter()
                    .any(|forbidden| key.eq_ignore_ascii_case(forbidden))
                {
                    return Err(format!("video_pack.forbidden_field: {key} is not allowed."));
                }
                reject_forbidden_json_keys(child, forbidden_keys)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                reject_forbidden_json_keys(item, forbidden_keys)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        clone_video_pack_to_user, delete_user_video_pack, ensure_builtin_video_packs,
        list_video_packs, save_project_config_as_video_pack, upsert_user_video_pack,
    };
    use crate::db::Database;
    use crate::domain::project::CreateProjectRequest;
    use crate::domain::video_pack::{
        ListVideoPacksRequest, SaveProjectConfigAsVideoPackRequest, UpsertUserVideoPackRequest,
        VideoPackIdRequest,
    };
    use crate::services::project_service::create_project;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builtin_packs_seed_and_clone_to_user() {
        let root = test_root("builtin_clone");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");

        ensure_builtin_video_packs(&database, &workspace_root).expect("builtin packs should seed");
        let packs = list_video_packs(
            &database,
            &workspace_root,
            ListVideoPacksRequest {
                source_type: Some("builtin".to_string()),
                include_disabled: Some(true),
            },
        )
        .expect("packs should list");
        assert!(packs
            .iter()
            .any(|pack| pack.pack_id == "pack_knowledge_short"));

        let cloned = clone_video_pack_to_user(
            &database,
            &workspace_root,
            VideoPackIdRequest {
                pack_id: "pack_knowledge_short".to_string(),
            },
        )
        .expect("builtin pack should clone");
        assert_eq!(cloned.source_type, "user");
        assert!(!cloned.is_enabled);
        assert!(cloned.rule_refs.get("image_prompt").is_some());

        cleanup(root);
    }

    #[test]
    fn create_project_copies_pack_refs_to_project_config() {
        let root = test_root("project_pack");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        ensure_builtin_video_packs(&database, &workspace_root).expect("builtin packs should seed");

        let detail = create_project(
            &database,
            &workspace_root,
            create_request(Some("pack_knowledge_short")),
        )
        .expect("project should create");

        assert_eq!(
            detail.project.active_pack_id.as_deref(),
            Some("pack_knowledge_short")
        );
        assert_eq!(
            detail.project.rule_refs["image_prompt"]["ruleKey"],
            "image_prompt.shot_frame"
        );
        assert_eq!(
            detail.project.rule_refs["image_prompt"]["ruleId"],
            "builtin:image_prompt:image_prompt_shot_frame"
        );
        assert!(detail.project.executable_refs.get("video").is_some());

        cleanup(root);
    }

    #[test]
    fn deleting_user_pack_is_blocked_when_project_references_it() {
        let root = test_root("delete_blocked");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let pack = upsert_user_video_pack(
            &database,
            &workspace_root,
            UpsertUserVideoPackRequest {
                pack_id: Some("pack_user_blocked".to_string()),
                name: "User pack".to_string(),
                description: "desc".to_string(),
                applicable_input_types: vec!["topic".to_string()],
                content_category: Some("knowledge".to_string()),
                default_tone: None,
                default_aspect_ratio: "9:16".to_string(),
                default_duration_seconds: 30,
                default_scene_count: 5,
                rule_refs: json!({
                    "storyboard": {
                        "ruleKey": "storyboard.default",
                        "ruleId": "builtin:storyboard:storyboard_default"
                    }
                }),
                recommended_executable_refs: json!({ "image": { "provider_model_id": "pm_image" } }),
                asset_refs: json!([]),
                is_enabled: true,
            },
        )
        .expect("user pack should save");
        create_project(
            &database,
            &workspace_root,
            create_request(Some(&pack.pack_id)),
        )
        .expect("project should create");

        let delete_result = delete_user_video_pack(
            &database,
            VideoPackIdRequest {
                pack_id: pack.pack_id,
            },
        );
        assert!(delete_result.is_err());

        cleanup(root);
    }

    #[test]
    fn project_config_can_be_saved_as_user_pack_without_outputs_or_secrets() {
        let root = test_root("save_project_pack");
        let workspace_root = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let detail = create_project(&database, &workspace_root, create_request(None))
            .expect("project should create");

        let pack = save_project_config_as_video_pack(
            &database,
            &workspace_root,
            SaveProjectConfigAsVideoPackRequest {
                project_id: detail.project.project_id,
                name: "Current config".to_string(),
                description: "Saved from project".to_string(),
            },
        )
        .expect("project config should save as pack");

        assert_eq!(pack.source_type, "user");
        assert_eq!(pack.default_scene_count, 8);
        assert!(pack
            .asset_refs
            .as_array()
            .is_some_and(|items| items.is_empty()));

        cleanup(root);
    }

    fn create_request(active_pack_id: Option<&str>) -> CreateProjectRequest {
        CreateProjectRequest {
            title: "视频包测试".to_string(),
            workflow_type: "image_to_video".to_string(),
            input_type: "topic".to_string(),
            topic: Some("为什么要早睡".to_string()),
            source_text: None,
            source_text_path: None,
            content_language: "zh-CN".to_string(),
            tone: None,
            aspect_ratio: "9:16".to_string(),
            target_scene_count: 8,
            segment_duration_seconds: 4.0,
            style_prompt: None,
            active_pack_id: active_pack_id.map(str::to_string),
            rule_refs: None,
            executable_refs: None,
            input_process_mode: "generate".to_string(),
            input_options: Some(json!({})),
        }
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-pack-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
