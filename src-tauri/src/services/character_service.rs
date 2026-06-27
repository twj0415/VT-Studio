use crate::db::asset_repository::{AssetRepository, NewAssetReferenceRecord};
use crate::db::character_repository::{
    normalize_character_data, CharacterBibleRecordInput, CharacterRepository,
};
use crate::db::Database;
use crate::domain::character::{
    BindCharacterReferenceAssetRequest, BindCharacterReferenceAssetResponse, CharacterBibleDto,
    CharacterBibleIdRequest, UpsertProjectCharacterBibleRequest,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

const CHARACTER_ID_MAX_LENGTH: usize = 80;

pub fn list_project_character_bibles(
    database: &Database,
    project_id: String,
) -> Result<Vec<CharacterBibleDto>, String> {
    validate_project_id(&project_id)?;
    CharacterRepository::new(database).list_project_character_bibles(&project_id)
}

pub fn upsert_project_character_bible(
    database: &Database,
    request: UpsertProjectCharacterBibleRequest,
) -> Result<CharacterBibleDto, String> {
    validate_project_id(&request.project_id)?;
    if request.name.trim().is_empty() {
        return Err("character name is required.".to_string());
    }

    let repository = CharacterRepository::new(database);
    let character_id = request
        .character_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| create_character_id(&request.name));
    validate_character_id(&character_id)?;

    let existing = repository.get_character_bible(&character_id)?;
    if let Some(existing) = &existing {
        if existing.project_id != request.project_id {
            return Err(format!(
                "Character Bible {} does not belong to project {}.",
                character_id, request.project_id
            ));
        }
    }

    let data = character_data_from_request(&request, existing.as_ref(), &character_id)?;
    repository.upsert_character_bible(CharacterBibleRecordInput {
        character_bible_id: character_id,
        project_id: request.project_id,
        name: normalize_name(&request.name),
        data,
    })
}

pub fn delete_project_character_bible(
    database: &Database,
    request: CharacterBibleIdRequest,
) -> Result<CharacterBibleDto, String> {
    validate_character_id(&request.character_id)?;
    let repository = CharacterRepository::new(database);
    let storyboard_references = repository.count_storyboard_references(&request.character_id)?;
    let asset_references = repository.count_asset_references(&request.character_id)?;
    if storyboard_references > 0 || asset_references > 0 {
        return Err(format!(
            "Character Bible {} is still referenced by storyboard_items={} asset_references={}. Remove references first.",
            request.character_id, storyboard_references, asset_references
        ));
    }
    repository.delete_character_bible(&request.character_id)
}

pub fn bind_character_reference_asset(
    database: &Database,
    request: BindCharacterReferenceAssetRequest,
) -> Result<BindCharacterReferenceAssetResponse, String> {
    validate_project_id(&request.project_id)?;
    validate_character_id(&request.character_id)?;
    let reference_role = request
        .reference_role
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("character_front_view");
    validate_reference_role(reference_role)?;

    let character = CharacterRepository::new(database)
        .get_project_character_bible(&request.project_id, &request.character_id)?
        .ok_or_else(|| format!("Character Bible not found: {}", request.character_id))?;
    let asset = AssetRepository::new(database)
        .get_asset(&request.asset_id)?
        .ok_or_else(|| format!("Asset not found: {}", request.asset_id))?;
    if asset.lifecycle == "deleted" {
        return Err("deleted asset cannot be used as a character reference.".to_string());
    }
    if !asset.relative_path.starts_with("assets/") {
        return Err(
            "character reference asset must be in the controlled assets bucket.".to_string(),
        );
    }
    if asset.kind != "character_reference_image" {
        return Err(format!(
            "character reference asset kind must be character_reference_image, got {}.",
            asset.kind
        ));
    }

    let reference_id = create_id("asset_ref");
    let entry = json!({
        "assetId": asset.asset_id,
        "referenceId": reference_id,
        "role": reference_role,
        "imageKind": "character_reference",
        "assetKind": asset.kind,
        "relativePath": asset.relative_path,
        "source": asset.source_kind
    });
    let reference = AssetRepository::new(database).create_reference_and_append_to_character_bible(
        &NewAssetReferenceRecord {
            reference_id,
            asset_id: asset.asset_id,
            owner_kind: "character_bible".to_string(),
            owner_id: character.character_id.clone(),
            usage_kind: "character_reference".to_string(),
        },
        &entry,
    )?;
    let character_bible = CharacterRepository::new(database)
        .get_project_character_bible(&request.project_id, &request.character_id)?
        .ok_or_else(|| format!("Character Bible not found: {}", request.character_id))?;

    Ok(BindCharacterReferenceAssetResponse {
        character_bible,
        reference,
    })
}

pub fn validate_storyboard_character_ids(
    database: &Database,
    project_id: &str,
    character_ids: &[String],
) -> Result<(), String> {
    let normalized = normalize_string_list(character_ids);
    if normalized.is_empty() {
        return Ok(());
    }
    let missing =
        CharacterRepository::new(database).find_missing_character_ids(project_id, &normalized)?;
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Storyboard item references missing Character Bible ids: {}.",
            missing.join(", ")
        ))
    }
}

pub fn character_names_for_ids(
    database: &Database,
    project_id: &str,
    character_ids: &[String],
) -> Result<Vec<String>, String> {
    let repository = CharacterRepository::new(database);
    let mut names = Vec::new();
    for character_id in normalize_string_list(character_ids) {
        let character = repository
            .get_project_character_bible(project_id, &character_id)?
            .ok_or_else(|| {
                format!(
                    "Storyboard item references missing Character Bible id: {}.",
                    character_id
                )
            })?;
        names.push(character.name);
    }
    Ok(names)
}

fn character_data_from_request(
    request: &UpsertProjectCharacterBibleRequest,
    existing: Option<&CharacterBibleDto>,
    character_id: &str,
) -> Result<Value, String> {
    let mut data = normalize_character_data(
        existing
            .map(|character| character.data.clone())
            .unwrap_or_else(|| json!({})),
    );
    let visual_prompt = request
        .visual_prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            build_visual_prompt(
                &request.name,
                &request.age,
                &request.gender,
                &request.appearance,
                &request.clothing,
                &request.personality,
            )
        });
    let negative_prompt = request
        .negative_prompt
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .to_string();

    if let Some(object) = data.as_object_mut() {
        object.insert(
            "character_id".to_string(),
            Value::String(character_id.to_string()),
        );
        object.insert(
            "alias".to_string(),
            json!(normalize_string_list(&request.alias)),
        );
        object.insert(
            "age".to_string(),
            Value::String(request.age.trim().to_string()),
        );
        object.insert(
            "gender".to_string(),
            Value::String(request.gender.trim().to_string()),
        );
        object.insert(
            "appearance".to_string(),
            Value::String(request.appearance.trim().to_string()),
        );
        object.insert(
            "clothing".to_string(),
            Value::String(request.clothing.trim().to_string()),
        );
        object.insert(
            "personality".to_string(),
            Value::String(request.personality.trim().to_string()),
        );
        object.insert("visual_prompt".to_string(), Value::String(visual_prompt));
        object.insert(
            "negative_prompt".to_string(),
            Value::String(negative_prompt),
        );
        if let Some(path) = request
            .reference_image_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            if !path.starts_with("assets/") {
                return Err(
                    "reference_image_path must point to the controlled assets bucket.".to_string(),
                );
            }
            object.insert(
                "reference_image_path".to_string(),
                Value::String(path.to_string()),
            );
        } else if existing.is_none() {
            object.insert("reference_image_path".to_string(), Value::Null);
        }
        if let Some(reference_images) = &request.reference_images {
            validate_reference_images(reference_images)?;
            object.insert(
                "reference_images_json".to_string(),
                Value::Array(reference_images.clone()),
            );
            object.insert(
                "reference_images".to_string(),
                Value::Array(reference_images.clone()),
            );
        }
        if let Some(lock_flags) = &request.lock_flags {
            if !lock_flags.is_object() {
                return Err("lock_flags must be a JSON object.".to_string());
            }
            object.insert("lock_flags".to_string(), lock_flags.clone());
        }
    }
    Ok(data)
}

fn build_visual_prompt(
    name: &str,
    age: &str,
    gender: &str,
    appearance: &str,
    clothing: &str,
    personality: &str,
) -> String {
    [
        normalize_name(name),
        age.trim().to_string(),
        gender.trim().to_string(),
        appearance.trim().to_string(),
        clothing.trim().to_string(),
        personality.trim().to_string(),
    ]
    .into_iter()
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(", ")
}

fn validate_reference_images(reference_images: &[Value]) -> Result<(), String> {
    for image in reference_images {
        let Some(path) = image
            .get("relativePath")
            .or_else(|| image.get("relative_path"))
            .and_then(Value::as_str)
        else {
            return Err("character reference image must include relativePath.".to_string());
        };
        if !path.starts_with("assets/") {
            return Err("character reference image must use assets/ relativePath.".to_string());
        }
        if let Some(role) = image.get("role").and_then(Value::as_str) {
            validate_reference_role(role)?;
        }
    }
    Ok(())
}

fn validate_reference_role(role: &str) -> Result<(), String> {
    let allowed = [
        "character_reference",
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
    if allowed.contains(&role) {
        Ok(())
    } else {
        Err(format!("Unsupported character reference role: {role}."))
    }
}

fn validate_project_id(project_id: &str) -> Result<(), String> {
    if project_id.trim().is_empty() {
        Err("project_id is required.".to_string())
    } else {
        Ok(())
    }
}

fn validate_character_id(character_id: &str) -> Result<(), String> {
    let trimmed = character_id.trim();
    if trimmed.is_empty() {
        return Err("character_id is required.".to_string());
    }
    if trimmed.len() > CHARACTER_ID_MAX_LENGTH {
        return Err(format!(
            "character_id cannot exceed {} characters.",
            CHARACTER_ID_MAX_LENGTH
        ));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
    {
        return Err(
            "character_id may only contain lowercase letters, digits, hyphen, and underscore."
                .to_string(),
        );
    }
    Ok(())
}

fn create_character_id(name: &str) -> String {
    let slug = slugify(name);
    let prefix = if slug.is_empty() { "character" } else { &slug };
    format!("{prefix}_{}", current_timestamp_id())
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_separator = false;
    for ch in value.trim().chars() {
        let next = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_lowercase())
        } else if ch == '-' || ch == '_' || ch.is_whitespace() {
            Some('_')
        } else {
            None
        };
        if let Some(ch) = next {
            if ch == '_' {
                if !previous_separator && !slug.is_empty() {
                    slug.push('_');
                    previous_separator = true;
                }
            } else {
                slug.push(ch);
                previous_separator = false;
            }
        }
    }
    slug.trim_matches('_').chars().take(32).collect()
}

fn normalize_name(name: &str) -> String {
    name.trim().to_string()
}

fn normalize_string_list(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            output.push(trimmed.to_string());
        }
    }
    output
}

fn create_id(prefix: &str) -> String {
    format!("{prefix}_{}", current_timestamp_id())
}

fn current_timestamp_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    nanos.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::asset_repository::NewAssetRecord;
    use crate::db::project_repository::ProjectRepository;
    use crate::db::scene_repository::SceneRepository;
    use crate::domain::project::CreateProjectRequest;
    use crate::domain::scene::SceneDto;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn character_id_is_stable_and_not_name() {
        let root = test_root("character_id");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_test_project(&database, "project_character_id");

        let character = upsert_project_character_bible(
            &database,
            UpsertProjectCharacterBibleRequest {
                project_id: "project_character_id".to_string(),
                character_id: None,
                name: "小林".to_string(),
                alias: vec!["主角".to_string()],
                age: "28".to_string(),
                gender: "male".to_string(),
                appearance: "short black hair, round glasses".to_string(),
                clothing: "white shirt and blue cardigan".to_string(),
                personality: "calm".to_string(),
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                lock_flags: None,
            },
        )
        .expect("character should save");

        assert_ne!(character.character_id, "小林");
        assert!(character.character_id.starts_with("character_"));
        assert!(character.visual_prompt.contains("round glasses"));
        cleanup(root);
    }

    #[test]
    fn storyboard_reference_requires_existing_character_id() {
        let root = test_root("storyboard_character_reference");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_test_project(&database, "project_character_ref");
        let character = upsert_project_character_bible(
            &database,
            UpsertProjectCharacterBibleRequest {
                project_id: "project_character_ref".to_string(),
                character_id: Some("hero".to_string()),
                name: "Hero".to_string(),
                alias: vec![],
                age: "28".to_string(),
                gender: "female".to_string(),
                appearance: "sharp eyes".to_string(),
                clothing: "green coat".to_string(),
                personality: "decisive".to_string(),
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                lock_flags: None,
            },
        )
        .expect("character should save");

        validate_storyboard_character_ids(
            &database,
            "project_character_ref",
            &[character.character_id.clone()],
        )
        .expect("existing character should pass");
        let error = validate_storyboard_character_ids(
            &database,
            "project_character_ref",
            &["missing".to_string()],
        )
        .expect_err("missing character should fail");
        assert!(error.contains("missing"));
        cleanup(root);
    }

    #[test]
    fn deleting_referenced_character_is_blocked() {
        let root = test_root("delete_referenced_character");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_test_project(&database, "project_delete_character");
        upsert_project_character_bible(
            &database,
            UpsertProjectCharacterBibleRequest {
                project_id: "project_delete_character".to_string(),
                character_id: Some("hero".to_string()),
                name: "Hero".to_string(),
                alias: vec![],
                age: "".to_string(),
                gender: "".to_string(),
                appearance: "same face".to_string(),
                clothing: "same outfit".to_string(),
                personality: "".to_string(),
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                lock_flags: None,
            },
        )
        .expect("character should save");
        SceneRepository::new(&database)
            .upsert_storyboard_item(&test_item(
                "project_delete_character",
                vec!["hero".to_string()],
            ))
            .expect("item should save");

        let error = delete_project_character_bible(
            &database,
            CharacterBibleIdRequest {
                character_id: "hero".to_string(),
            },
        )
        .expect_err("referenced character delete should fail");

        assert!(error.contains("storyboard_items=1"));
        cleanup(root);
    }

    #[test]
    fn binding_reference_asset_requires_controlled_relative_path() {
        let root = test_root("character_reference_asset");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_test_project(&database, "project_character_asset");
        upsert_project_character_bible(
            &database,
            UpsertProjectCharacterBibleRequest {
                project_id: "project_character_asset".to_string(),
                character_id: Some("hero".to_string()),
                name: "Hero".to_string(),
                alias: vec![],
                age: "".to_string(),
                gender: "".to_string(),
                appearance: "same face".to_string(),
                clothing: "same outfit".to_string(),
                personality: "".to_string(),
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                lock_flags: None,
            },
        )
        .expect("character should save");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_hero_ref".to_string(),
                kind: "character_reference_image".to_string(),
                relative_path: "assets/character_reference_image/hero.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(3),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "mediaKind": "image" }),
            })
            .expect("asset should save");

        let bound = bind_character_reference_asset(
            &database,
            BindCharacterReferenceAssetRequest {
                project_id: "project_character_asset".to_string(),
                character_id: "hero".to_string(),
                asset_id: asset.asset_id,
                reference_role: Some("character_front_view".to_string()),
            },
        )
        .expect("reference should bind");

        assert_eq!(bound.reference.owner_kind, "character_bible");
        assert_eq!(bound.reference.usage_kind, "character_reference");
        assert_eq!(
            bound.character_bible.reference_image_path.as_deref(),
            Some("assets/character_reference_image/hero.png")
        );
        assert_eq!(bound.character_bible.reference_images.len(), 1);
        cleanup(root);
    }

    fn create_test_project(database: &Database, project_id: &str) {
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "角色测试".to_string(),
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
    }

    fn test_item(project_id: &str, character_ids: Vec<String>) -> SceneDto {
        SceneDto {
            item_id: format!("{project_id}_item_1"),
            project_id: project_id.to_string(),
            index: 1,
            source_text: "source".to_string(),
            narration_text: "source".to_string(),
            visual_goal: "goal".to_string(),
            visual_description: "visual".to_string(),
            characters: vec!["Hero".to_string()],
            character_ids,
            location_id: None,
            scene_description: "scene".to_string(),
            image_prompt: "prompt".to_string(),
            negative_prompt: String::new(),
            video_prompt: "move".to_string(),
            duration_seconds: 4.0,
            subtitle_chunks: vec![],
            audio_path: None,
            audio_duration_seconds: None,
            audio_probe: None,
            selected_image_id: None,
            selected_video_segment_id: None,
            status: "pending".to_string(),
            lock_flags_json: json!({}),
            shot_size: None,
            camera_motion: None,
            composition: None,
            pace: None,
            transition_type: None,
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

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-character-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
