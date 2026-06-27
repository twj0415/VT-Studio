use crate::db::asset_repository::{AssetRepository, NewAssetReferenceRecord};
use crate::db::location_repository::{
    normalize_location_data, LocationBibleRecordInput, LocationRepository,
};
use crate::db::Database;
use crate::domain::location::{
    BindLocationReferenceAssetRequest, BindLocationReferenceAssetResponse, LocationBibleDto,
    LocationBibleIdRequest, UpsertProjectLocationBibleRequest,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

const LOCATION_ID_MAX_LENGTH: usize = 80;

pub fn list_project_location_bibles(
    database: &Database,
    project_id: String,
) -> Result<Vec<LocationBibleDto>, String> {
    validate_project_id(&project_id)?;
    LocationRepository::new(database).list_project_location_bibles(&project_id)
}

pub fn upsert_project_location_bible(
    database: &Database,
    request: UpsertProjectLocationBibleRequest,
) -> Result<LocationBibleDto, String> {
    validate_project_id(&request.project_id)?;
    if request.name.trim().is_empty() {
        return Err("location name is required.".to_string());
    }

    let repository = LocationRepository::new(database);
    let location_id = request
        .location_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| create_location_id(&request.name));
    validate_location_id(&location_id)?;

    let existing = repository.get_location_bible(&location_id)?;
    if let Some(existing) = &existing {
        if existing.project_id != request.project_id {
            return Err(format!(
                "Location Bible {} does not belong to project {}.",
                location_id, request.project_id
            ));
        }
    }

    let data = location_data_from_request(&request, existing.as_ref(), &location_id)?;
    repository.upsert_location_bible(LocationBibleRecordInput {
        location_bible_id: location_id,
        project_id: request.project_id,
        name: normalize_name(&request.name),
        data,
    })
}

pub fn delete_project_location_bible(
    database: &Database,
    request: LocationBibleIdRequest,
) -> Result<LocationBibleDto, String> {
    validate_location_id(&request.location_id)?;
    let repository = LocationRepository::new(database);
    let storyboard_references = repository.count_storyboard_references(&request.location_id)?;
    let asset_references = repository.count_asset_references(&request.location_id)?;
    if storyboard_references > 0 || asset_references > 0 {
        return Err(format!(
            "Location Bible {} is still referenced by storyboard_items={} asset_references={}. Remove references first.",
            request.location_id, storyboard_references, asset_references
        ));
    }
    repository.delete_location_bible(&request.location_id)
}

pub fn bind_location_reference_asset(
    database: &Database,
    request: BindLocationReferenceAssetRequest,
) -> Result<BindLocationReferenceAssetResponse, String> {
    validate_project_id(&request.project_id)?;
    validate_location_id(&request.location_id)?;
    let reference_role = request
        .reference_role
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("scene_wide_view");
    validate_reference_role(reference_role)?;

    let location = LocationRepository::new(database)
        .get_project_location_bible(&request.project_id, &request.location_id)?
        .ok_or_else(|| format!("Location Bible not found: {}", request.location_id))?;
    let asset = AssetRepository::new(database)
        .get_asset(&request.asset_id)?
        .ok_or_else(|| format!("Asset not found: {}", request.asset_id))?;
    if asset.lifecycle == "deleted" {
        return Err("deleted asset cannot be used as a location reference.".to_string());
    }
    if !asset.relative_path.starts_with("assets/") {
        return Err(
            "location reference asset must be in the controlled assets bucket.".to_string(),
        );
    }
    if asset.kind != "scene_reference_image" {
        return Err(format!(
            "location reference asset kind must be scene_reference_image, got {}.",
            asset.kind
        ));
    }

    let reference_id = create_id("asset_ref");
    let entry = json!({
        "assetId": asset.asset_id,
        "referenceId": reference_id,
        "role": reference_role,
        "imageKind": "scene_reference",
        "assetKind": asset.kind,
        "relativePath": asset.relative_path,
        "source": asset.source_kind
    });
    let reference = AssetRepository::new(database).create_reference_and_append_to_location_bible(
        &NewAssetReferenceRecord {
            reference_id,
            asset_id: asset.asset_id,
            owner_kind: "location_bible".to_string(),
            owner_id: location.location_id.clone(),
            usage_kind: "location_reference".to_string(),
        },
        &entry,
    )?;
    let location_bible = LocationRepository::new(database)
        .get_project_location_bible(&request.project_id, &request.location_id)?
        .ok_or_else(|| format!("Location Bible not found: {}", request.location_id))?;

    Ok(BindLocationReferenceAssetResponse {
        location_bible,
        reference,
    })
}

pub fn validate_storyboard_location_id(
    database: &Database,
    project_id: &str,
    location_id: Option<&str>,
) -> Result<(), String> {
    let Some(location_id) = location_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    validate_location_id(location_id)?;
    if LocationRepository::new(database)
        .get_project_location_bible(project_id, location_id)?
        .is_some()
    {
        Ok(())
    } else {
        Err(format!(
            "Storyboard item references missing Location Bible id: {location_id}."
        ))
    }
}

pub fn location_scene_description_for_id(
    database: &Database,
    project_id: &str,
    location_id: Option<&str>,
) -> Result<Option<String>, String> {
    let Some(location_id) = location_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let location = LocationRepository::new(database)
        .get_project_location_bible(project_id, location_id)?
        .ok_or_else(|| {
            format!("Storyboard item references missing Location Bible id: {location_id}.")
        })?;
    Ok(Some(location.name))
}

fn location_data_from_request(
    request: &UpsertProjectLocationBibleRequest,
    existing: Option<&LocationBibleDto>,
    location_id: &str,
) -> Result<Value, String> {
    let mut data = normalize_location_data(
        existing
            .map(|location| location.data.clone())
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
                &request.space_description,
                &request.lighting,
                &request.time_of_day,
                &request.props,
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
            "location_id".to_string(),
            Value::String(location_id.to_string()),
        );
        object.insert(
            "space_description".to_string(),
            Value::String(request.space_description.trim().to_string()),
        );
        object.insert(
            "lighting".to_string(),
            Value::String(request.lighting.trim().to_string()),
        );
        object.insert(
            "time_of_day".to_string(),
            Value::String(request.time_of_day.trim().to_string()),
        );
        object.insert(
            "props".to_string(),
            json!(normalize_string_list(&request.props)),
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
        if let Some(variants) = &request.variants {
            object.insert("variants".to_string(), Value::Array(variants.clone()));
        }
    }
    Ok(data)
}

fn build_visual_prompt(
    name: &str,
    space_description: &str,
    lighting: &str,
    time_of_day: &str,
    props: &[String],
) -> String {
    [
        normalize_name(name),
        space_description.trim().to_string(),
        lighting.trim().to_string(),
        time_of_day.trim().to_string(),
        normalize_string_list(props).join(", "),
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
            return Err("location reference image must include relativePath.".to_string());
        };
        if !path.starts_with("assets/") {
            return Err("location reference image must use assets/ relativePath.".to_string());
        }
        if let Some(role) = image.get("role").and_then(Value::as_str) {
            validate_reference_role(role)?;
        }
    }
    Ok(())
}

fn validate_reference_role(role: &str) -> Result<(), String> {
    let allowed = [
        "scene_reference",
        "scene_wide_view",
        "scene_layout_view",
        "scene_detail_view",
        "scene_day_variant",
        "scene_night_variant",
    ];
    if allowed.contains(&role) {
        Ok(())
    } else {
        Err(format!("Unsupported location reference role: {role}."))
    }
}

fn validate_project_id(project_id: &str) -> Result<(), String> {
    if project_id.trim().is_empty() {
        Err("project_id is required.".to_string())
    } else {
        Ok(())
    }
}

fn validate_location_id(location_id: &str) -> Result<(), String> {
    let trimmed = location_id.trim();
    if trimmed.is_empty() {
        return Err("location_id is required.".to_string());
    }
    if trimmed.len() > LOCATION_ID_MAX_LENGTH {
        return Err(format!(
            "location_id cannot exceed {} characters.",
            LOCATION_ID_MAX_LENGTH
        ));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
    {
        return Err(
            "location_id may only contain lowercase letters, digits, hyphen, and underscore."
                .to_string(),
        );
    }
    Ok(())
}

fn create_location_id(name: &str) -> String {
    let slug = slugify(name);
    let prefix = if slug.is_empty() { "location" } else { &slug };
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
    fn location_id_is_stable_and_not_name() {
        let root = test_root("location_id");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_test_project(&database, "project_location_id");

        let location = upsert_project_location_bible(
            &database,
            UpsertProjectLocationBibleRequest {
                project_id: "project_location_id".to_string(),
                location_id: None,
                name: "安静书房".to_string(),
                space_description: "wooden desk near the window".to_string(),
                lighting: "warm desk lamp".to_string(),
                time_of_day: "evening".to_string(),
                props: vec!["bookshelf".to_string(), "desk lamp".to_string()],
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                variants: None,
            },
        )
        .expect("location should save");

        assert_ne!(location.location_id, "安静书房");
        assert!(location.location_id.starts_with("location_"));
        assert!(location.visual_prompt.contains("wooden desk"));
        cleanup(root);
    }

    #[test]
    fn storyboard_reference_requires_existing_location_id() {
        let root = test_root("storyboard_location_reference");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_test_project(&database, "project_location_ref");
        upsert_project_location_bible(
            &database,
            UpsertProjectLocationBibleRequest {
                project_id: "project_location_ref".to_string(),
                location_id: Some("study_room".to_string()),
                name: "Study room".to_string(),
                space_description: "desk and bookshelf".to_string(),
                lighting: "soft light".to_string(),
                time_of_day: "morning".to_string(),
                props: vec![],
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                variants: None,
            },
        )
        .expect("location should save");

        validate_storyboard_location_id(&database, "project_location_ref", Some("study_room"))
            .expect("existing location should pass");
        let error =
            validate_storyboard_location_id(&database, "project_location_ref", Some("missing"))
                .expect_err("missing location should fail");
        assert!(error.contains("missing"));
        cleanup(root);
    }

    #[test]
    fn deleting_referenced_location_is_blocked() {
        let root = test_root("delete_referenced_location");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_test_project(&database, "project_delete_location");
        upsert_project_location_bible(
            &database,
            UpsertProjectLocationBibleRequest {
                project_id: "project_delete_location".to_string(),
                location_id: Some("study_room".to_string()),
                name: "Study room".to_string(),
                space_description: "desk and bookshelf".to_string(),
                lighting: "soft light".to_string(),
                time_of_day: "morning".to_string(),
                props: vec![],
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                variants: None,
            },
        )
        .expect("location should save");
        let mut item = test_item("project_delete_location");
        item.location_id = Some("study_room".to_string());
        SceneRepository::new(&database)
            .upsert_storyboard_item(&item)
            .expect("item should save");

        let error = delete_project_location_bible(
            &database,
            LocationBibleIdRequest {
                location_id: "study_room".to_string(),
            },
        )
        .expect_err("referenced location delete should fail");

        assert!(error.contains("storyboard_items=1"));
        cleanup(root);
    }

    #[test]
    fn binding_location_reference_asset_updates_location_bible() {
        let root = test_root("location_reference_asset");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        fs::create_dir_all(workspace_root.join("assets/scene_reference_image"))
            .expect("asset dir should exist");
        fs::write(
            workspace_root.join("assets/scene_reference_image/study.png"),
            "png",
        )
        .expect("asset should write");
        create_test_project(&database, "project_location_asset");
        upsert_project_location_bible(
            &database,
            UpsertProjectLocationBibleRequest {
                project_id: "project_location_asset".to_string(),
                location_id: Some("study_room".to_string()),
                name: "Study room".to_string(),
                space_description: "desk and bookshelf".to_string(),
                lighting: "soft light".to_string(),
                time_of_day: "morning".to_string(),
                props: vec![],
                visual_prompt: None,
                negative_prompt: None,
                reference_image_path: None,
                reference_images: None,
                variants: None,
            },
        )
        .expect("location should save");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_study_ref".to_string(),
                kind: "scene_reference_image".to_string(),
                relative_path: "assets/scene_reference_image/study.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(3),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "mediaKind": "image" }),
            })
            .expect("asset should save");

        let bound = bind_location_reference_asset(
            &database,
            BindLocationReferenceAssetRequest {
                project_id: "project_location_asset".to_string(),
                location_id: "study_room".to_string(),
                asset_id: asset.asset_id,
                reference_role: Some("scene_wide_view".to_string()),
            },
        )
        .expect("reference should bind");

        assert_eq!(bound.reference.owner_kind, "location_bible");
        assert_eq!(bound.reference.usage_kind, "location_reference");
        assert_eq!(
            bound.location_bible.reference_image_path.as_deref(),
            Some("assets/scene_reference_image/study.png")
        );
        assert_eq!(bound.location_bible.reference_images.len(), 1);
        cleanup(root);
    }

    fn create_test_project(database: &Database, project_id: &str) {
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "场景测试".to_string(),
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

    fn test_item(project_id: &str) -> SceneDto {
        SceneDto {
            item_id: format!("{project_id}_item_1"),
            project_id: project_id.to_string(),
            index: 1,
            source_text: "source".to_string(),
            narration_text: "source".to_string(),
            visual_goal: "goal".to_string(),
            visual_description: "visual".to_string(),
            characters: vec![],
            character_ids: vec![],
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
            "vt-ai-short-video-maker-location-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
