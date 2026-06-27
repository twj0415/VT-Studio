use crate::core::error::TaskError;
use crate::db::asset_repository::{AssetRepository, NewAssetReferenceRecord};
use crate::db::character_repository::CharacterRepository;
use crate::db::location_repository::LocationRepository;
use crate::db::provider_repository::{ProviderRecord, ProviderRepository};
use crate::db::scene_repository::SceneRepository;
use crate::db::style_repository::{normalize_style_data, StyleBibleRecordInput, StyleRepository};
use crate::db::Database;
use crate::domain::character::CharacterBibleDto;
use crate::domain::location::LocationBibleDto;
use crate::domain::provider::{ProviderMediaInputDto, ProviderRequestContext, VlmAnalyzeRequest};
use crate::domain::style::{
    AnalyzeStyleReferenceRequest, ApplyStylePresetRequest, BindStyleReferenceAssetRequest,
    BindStyleReferenceAssetResponse, BuildImagePromptPreviewRequest, ImagePromptPreviewDto,
    PromptSectionDto, StyleBibleDto, StylePresetDto, StyleReferenceAnalysisDto,
    UpsertProjectStyleBibleRequest,
};
use crate::services::keyring_service::KeyringService;
use crate::services::provider_service::ProviderManager;
use crate::services::task_cancellation::CancellationToken;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

pub const NEGATIVE_PROMPT_MAX_LENGTH: usize = 800;
const CONTROLLED_DUMMY_VLM_PROVIDER_ID: &str = "provider_controlled_fake_vlm";

pub fn get_project_style_bible(
    database: &Database,
    project_id: String,
) -> Result<StyleBibleDto, String> {
    ensure_project_style_bible(database, &project_id)
}

pub fn list_style_presets(database: &Database) -> Result<Vec<StylePresetDto>, String> {
    let mut presets = builtin_style_presets();
    presets.extend(StyleRepository::new(database).list_user_style_presets()?);
    Ok(presets)
}

pub fn upsert_project_style_bible(
    database: &Database,
    request: UpsertProjectStyleBibleRequest,
) -> Result<StyleBibleDto, String> {
    validate_project_id(&request.project_id)?;
    let style_bible_id = request
        .style_bible_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| project_style_bible_id(&request.project_id));
    let existing = StyleRepository::new(database).get_style_bible(&style_bible_id)?;
    if let Some(existing) = &existing {
        if existing.project_id != request.project_id {
            return Err(format!(
                "Style Bible {} does not belong to project {}.",
                style_bible_id, request.project_id
            ));
        }
    }
    let data = style_data_from_request(&request, existing.as_ref())?;
    let saved = StyleRepository::new(database).upsert_style_bible(StyleBibleRecordInput {
        style_bible_id,
        project_id: request.project_id.clone(),
        name: normalize_name(&request.name),
        data,
    })?;

    if request.save_as_preset.unwrap_or(false) {
        let preset_id = create_id("style_preset");
        let preset_name = request
            .preset_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&saved.name);
        StyleRepository::new(database).upsert_user_style_preset(StylePresetDto {
            preset_id,
            source_type: "user".to_string(),
            name: normalize_name(preset_name),
            style_prompt: saved.style_prompt.clone(),
            color_palette: saved.color_palette.clone(),
            lighting: saved.lighting.clone(),
            composition: saved.composition.clone(),
            negative_prompt: saved.negative_prompt.clone(),
            reference_image_path: saved.reference_image_path.clone(),
        })?;
    }

    Ok(saved)
}

pub fn apply_style_preset(
    database: &Database,
    request: ApplyStylePresetRequest,
) -> Result<StyleBibleDto, String> {
    validate_project_id(&request.project_id)?;
    let presets = list_style_presets(database)?;
    let preset = presets
        .into_iter()
        .find(|item| item.preset_id == request.preset_id)
        .ok_or_else(|| format!("Style preset not found: {}", request.preset_id))?;
    let existing = StyleRepository::new(database).get_project_style_bible(&request.project_id)?;
    let style_bible_id = existing
        .as_ref()
        .map(|item| item.style_bible_id.clone())
        .unwrap_or_else(|| project_style_bible_id(&request.project_id));
    let mut data = style_data_from_preset(&preset);
    if let Some(existing) = existing {
        let object = data
            .as_object_mut()
            .ok_or_else(|| "Style preset data must be an object.".to_string())?;
        object.insert(
            "reference_images_json".to_string(),
            json!(existing.reference_images),
        );
    }
    StyleRepository::new(database).upsert_style_bible(StyleBibleRecordInput {
        style_bible_id,
        project_id: request.project_id,
        name: preset.name,
        data,
    })
}

pub fn bind_style_reference_asset(
    database: &Database,
    request: BindStyleReferenceAssetRequest,
) -> Result<BindStyleReferenceAssetResponse, String> {
    validate_project_id(&request.project_id)?;
    let mut style_bible = match request.style_bible_id.as_deref() {
        Some(style_bible_id) if !style_bible_id.trim().is_empty() => StyleRepository::new(database)
            .get_style_bible(style_bible_id)?
            .ok_or_else(|| format!("Style Bible not found: {style_bible_id}"))?,
        _ => ensure_project_style_bible(database, &request.project_id)?,
    };
    if style_bible.project_id != request.project_id {
        return Err(format!(
            "Style Bible {} does not belong to project {}.",
            style_bible.style_bible_id, request.project_id
        ));
    }
    let asset = AssetRepository::new(database)
        .get_asset(&request.asset_id)?
        .ok_or_else(|| format!("Asset not found: {}", request.asset_id))?;
    if asset.lifecycle == "deleted" {
        return Err("deleted asset cannot be used as a style reference.".to_string());
    }
    if !asset.relative_path.starts_with("assets/") {
        return Err("style reference asset must be in the controlled assets bucket.".to_string());
    }

    let reference_id = create_id("asset_ref");
    let entry = json!({
        "assetId": asset.asset_id,
        "referenceId": reference_id,
        "role": "style_reference",
        "imageKind": "style_reference",
        "assetKind": asset.kind,
        "relativePath": asset.relative_path,
        "source": asset.source_kind
    });
    AssetRepository::new(database).create_reference_and_append_to_style_bible(
        &NewAssetReferenceRecord {
            reference_id: reference_id.clone(),
            asset_id: asset.asset_id.clone(),
            owner_kind: "style_bible".to_string(),
            owner_id: style_bible.style_bible_id.clone(),
            usage_kind: "style_reference".to_string(),
        },
        &entry,
    )?;
    style_bible = StyleRepository::new(database)
        .get_style_bible(&style_bible.style_bible_id)?
        .ok_or_else(|| format!("Style Bible not found: {}", style_bible.style_bible_id))?;

    Ok(BindStyleReferenceAssetResponse {
        style_bible,
        reference_id,
    })
}

pub fn analyze_style_reference_image(
    database: &Database,
    keyring_service: &KeyringService,
    request: AnalyzeStyleReferenceRequest,
) -> Result<StyleReferenceAnalysisDto, TaskError> {
    validate_project_id(&request.project_id)
        .map_err(|message| task_validation_error("validation.invalid_input", message))?;
    let style_bible = match request.style_bible_id.as_deref() {
        Some(style_bible_id) if !style_bible_id.trim().is_empty() => StyleRepository::new(database)
            .get_style_bible(style_bible_id)
            .map_err(task_config_error)?
            .ok_or_else(|| {
                task_validation_error(
                    "validation.not_found",
                    format!("Style Bible not found: {style_bible_id}"),
                )
            })?,
        _ => {
            ensure_project_style_bible(database, &request.project_id).map_err(task_config_error)?
        }
    };
    if style_bible.project_id != request.project_id {
        return Err(task_validation_error(
            "validation.invalid_input",
            format!(
                "Style Bible {} does not belong to project {}.",
                style_bible.style_bible_id, request.project_id
            ),
        ));
    }

    let reference_image_path = select_style_reference_path(&style_bible)?;
    let provider_id = match request.provider_id.as_deref().map(str::trim) {
        Some(provider_id) if !provider_id.is_empty() => provider_id.to_string(),
        _ => ensure_controlled_dummy_vlm_provider(database)?,
    };
    let provider_model_id = request
        .provider_model_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let trace_id = create_id("trace_style_vlm");
    let context = ProviderRequestContext {
        trace_id: trace_id.clone(),
        task_id: None,
        task_step_id: None,
        project_id: Some(request.project_id.clone()),
        item_id: None,
        provider_id: provider_id.clone(),
        provider_model_id: provider_model_id.clone(),
        workflow_preset_id: None,
        timeout_seconds: Some(60),
        idempotency_key: Some(format!(
            "style_reference_analysis_{}_{}",
            style_bible.style_bible_id, reference_image_path
        )),
    };
    let response = ProviderManager::new(database, keyring_service).analyze_asset(
        VlmAnalyzeRequest {
            context,
            input_path: reference_image_path.clone(),
            prompt: style_reference_analysis_prompt(),
            output_schema: Some(style_reference_analysis_schema()),
        },
        &CancellationToken::new(format!("style_reference_analysis_{trace_id}")),
    )?;

    Ok(style_analysis_from_provider_response(
        &style_bible,
        &reference_image_path,
        &provider_id,
        provider_model_id.as_deref(),
        &trace_id,
        response.description,
        response.parsed_json,
    ))
}

pub fn build_image_prompt_preview(
    database: &Database,
    request: BuildImagePromptPreviewRequest,
) -> Result<ImagePromptPreviewDto, String> {
    let item = SceneRepository::new(database)
        .get_storyboard_item(&request.item_id)?
        .ok_or_else(|| format!("Storyboard item not found: {}", request.item_id))?;
    if item.project_id != request.project_id {
        return Err(format!(
            "Storyboard item {} does not belong to project {}.",
            request.item_id, request.project_id
        ));
    }
    build_image_prompt_preview_for_item(database, &request.project_id, &item)
}

pub fn build_image_prompt_preview_for_item(
    database: &Database,
    project_id: &str,
    item: &crate::domain::scene::SceneDto,
) -> Result<ImagePromptPreviewDto, String> {
    let style_bible = StyleRepository::new(database).get_project_style_bible(project_id)?;
    let character_bibles = resolve_prompt_characters(database, project_id, &item.character_ids)?;
    let location_bible =
        resolve_prompt_location(database, project_id, item.location_id.as_deref())?;
    let mut sections = Vec::new();
    push_section(
        &mut sections,
        "visualDescription",
        "Visual description",
        &item.visual_description,
    );
    push_section(
        &mut sections,
        "imagePrompt",
        "Storyboard image prompt",
        &item.image_prompt,
    );
    let character_prompt = if character_bibles.is_empty() {
        item.characters.join(", ")
    } else {
        character_bibles
            .iter()
            .map(character_prompt_section)
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>()
            .join("; ")
    };
    push_section(&mut sections, "characters", "Characters", &character_prompt);
    let scene_prompt = if let Some(location) = &location_bible {
        location_prompt_section(location)
    } else {
        item.scene_description.clone()
    };
    push_section(&mut sections, "scene", "Scene", &scene_prompt);
    if let Some(style) = &style_bible {
        push_section(
            &mut sections,
            "stylePrompt",
            "Style Bible",
            &style.style_prompt,
        );
        push_section(
            &mut sections,
            "colorPalette",
            "Color palette",
            &style.color_palette.join(", "),
        );
        push_section(&mut sections, "lighting", "Lighting", &style.lighting);
        push_section(
            &mut sections,
            "composition",
            "Composition",
            &style.composition,
        );
    }

    let final_prompt = sections
        .iter()
        .filter(|section| !section.content.trim().is_empty())
        .map(|section| section.content.trim())
        .collect::<Vec<_>>()
        .join(", ");
    let negative_prompts = std::iter::once(item.negative_prompt.as_str())
        .chain(
            style_bible
                .as_ref()
                .map(|style| style.negative_prompt.as_str())
                .into_iter(),
        )
        .chain(
            character_bibles
                .iter()
                .map(|character| character.negative_prompt.as_str()),
        );
    let negative_prompts = negative_prompts.chain(
        location_bible
            .as_ref()
            .map(|location| location.negative_prompt.as_str())
            .into_iter(),
    );
    let (final_negative_prompt, negative_prompt_truncated) =
        merge_negative_prompts(negative_prompts, NEGATIVE_PROMPT_MAX_LENGTH);
    let mut reference_images = style_bible
        .as_ref()
        .map(style_reference_inputs)
        .unwrap_or_default();
    append_character_reference_inputs(&mut reference_images, &character_bibles);
    if let Some(location) = &location_bible {
        append_location_reference_inputs(&mut reference_images, location);
    }

    Ok(ImagePromptPreviewDto {
        project_id: project_id.to_string(),
        item_id: item.item_id.clone(),
        final_prompt,
        final_negative_prompt,
        sections,
        reference_images,
        style_bible,
        character_bibles,
        location_bible,
        negative_prompt_truncated,
        negative_prompt_max_length: NEGATIVE_PROMPT_MAX_LENGTH,
    })
}

pub fn merge_negative_prompts<'a>(
    prompts: impl IntoIterator<Item = &'a str>,
    max_length: usize,
) -> (String, bool) {
    let mut seen = HashSet::new();
    let mut parts = Vec::new();
    for prompt in prompts {
        for part in split_prompt_terms(prompt) {
            let key = part.to_ascii_lowercase();
            if seen.insert(key) {
                parts.push(part);
            }
        }
    }
    let mut value = parts.join(", ");
    let truncated = value.chars().count() > max_length;
    if truncated {
        value = value.chars().take(max_length).collect::<String>();
        value = value.trim_end_matches([',', ' ']).to_string();
    }
    (value, truncated)
}

pub fn style_reference_inputs(style: &StyleBibleDto) -> Vec<ProviderMediaInputDto> {
    let mut inputs = Vec::new();
    if let Some(path) = style.reference_image_path.as_deref() {
        if path.starts_with("assets/") {
            inputs.push(ProviderMediaInputDto {
                path: path.to_string(),
                role: "style_reference".to_string(),
                weight: Some(0.65),
            });
        }
    }
    for image in &style.reference_images {
        let Some(path) = image
            .get("relativePath")
            .or_else(|| image.get("relative_path"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        if !path.starts_with("assets/") || inputs.iter().any(|input| input.path == path) {
            continue;
        }
        inputs.push(ProviderMediaInputDto {
            path: path.to_string(),
            role: image
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or("style_reference")
                .to_string(),
            weight: Some(0.65),
        });
    }
    inputs
}

pub fn character_reference_inputs(character: &CharacterBibleDto) -> Vec<ProviderMediaInputDto> {
    let mut inputs = Vec::new();
    if let Some(path) = character.reference_image_path.as_deref() {
        if path.starts_with("assets/") {
            inputs.push(ProviderMediaInputDto {
                path: path.to_string(),
                role: format!("character_front_view:{}", character.character_id),
                weight: Some(0.8),
            });
        }
    }
    for image in &character.reference_images {
        let Some(path) = image
            .get("relativePath")
            .or_else(|| image.get("relative_path"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        if !path.starts_with("assets/") || inputs.iter().any(|input| input.path == path) {
            continue;
        }
        let role = image
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("character_reference");
        inputs.push(ProviderMediaInputDto {
            path: path.to_string(),
            role: format!("{role}:{}", character.character_id),
            weight: Some(character_reference_weight(role)),
        });
    }
    inputs
}

pub fn location_reference_inputs(location: &LocationBibleDto) -> Vec<ProviderMediaInputDto> {
    let mut inputs = Vec::new();
    if let Some(path) = location.reference_image_path.as_deref() {
        if path.starts_with("assets/") {
            inputs.push(ProviderMediaInputDto {
                path: path.to_string(),
                role: format!("scene_wide_view:{}", location.location_id),
                weight: Some(0.72),
            });
        }
    }
    for image in &location.reference_images {
        let Some(path) = image
            .get("relativePath")
            .or_else(|| image.get("relative_path"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        if !path.starts_with("assets/") || inputs.iter().any(|input| input.path == path) {
            continue;
        }
        let role = image
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("scene_reference");
        inputs.push(ProviderMediaInputDto {
            path: path.to_string(),
            role: format!("{role}:{}", location.location_id),
            weight: Some(location_reference_weight(role)),
        });
    }
    inputs
}

fn append_character_reference_inputs(
    inputs: &mut Vec<ProviderMediaInputDto>,
    characters: &[CharacterBibleDto],
) {
    for character in characters {
        for reference in character_reference_inputs(character) {
            if inputs.iter().any(|input| input.path == reference.path) {
                continue;
            }
            inputs.push(reference);
        }
    }
}

fn append_location_reference_inputs(
    inputs: &mut Vec<ProviderMediaInputDto>,
    location: &LocationBibleDto,
) {
    for reference in location_reference_inputs(location) {
        if inputs.iter().any(|input| input.path == reference.path) {
            continue;
        }
        inputs.push(reference);
    }
}

fn character_reference_weight(role: &str) -> f64 {
    match role {
        "character_front_view" | "character_full_body" => 0.8,
        "character_face_closeup" => 0.75,
        "character_side_view" | "character_back_view" | "character_outfit" => 0.7,
        "character_expression_sheet" | "character_pose" | "character_mood" => 0.6,
        _ => 0.65,
    }
}

fn location_reference_weight(role: &str) -> f64 {
    match role {
        "scene_wide_view" | "scene_layout_view" => 0.72,
        "scene_detail_view" => 0.65,
        "scene_day_variant" | "scene_night_variant" => 0.62,
        _ => 0.6,
    }
}

fn resolve_prompt_location(
    database: &Database,
    project_id: &str,
    location_id: Option<&str>,
) -> Result<Option<LocationBibleDto>, String> {
    let Some(location_id) = location_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    LocationRepository::new(database)
        .get_project_location_bible(project_id, location_id)?
        .map(Some)
        .ok_or_else(|| {
            format!("Storyboard item references missing Location Bible id: {location_id}.")
        })
}

fn location_prompt_section(location: &LocationBibleDto) -> String {
    [
        format!("{}({})", location.name, location.location_id),
        location.visual_prompt.clone(),
        location.space_description.clone(),
        location.lighting.clone(),
        location.time_of_day.clone(),
        location.props.join(", "),
    ]
    .into_iter()
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join("; ")
}

pub fn ensure_project_style_bible(
    database: &Database,
    project_id: &str,
) -> Result<StyleBibleDto, String> {
    validate_project_id(project_id)?;
    if let Some(style) = StyleRepository::new(database).get_project_style_bible(project_id)? {
        return Ok(style);
    }
    let project_style_prompt = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT style_prompt FROM projects WHERE project_id = ?1",
                [project_id],
                |row| row.get::<_, Option<String>>(0),
            )
        })
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    let data = json!({
        "style_prompt": project_style_prompt,
        "color_palette": [],
        "lighting": "",
        "composition": "",
        "negative_prompt": "",
        "reference_image_path": null,
        "reference_images_json": [],
        "reference_images": []
    });
    StyleRepository::new(database).upsert_style_bible(StyleBibleRecordInput {
        style_bible_id: project_style_bible_id(project_id),
        project_id: project_id.to_string(),
        name: "默认画风".to_string(),
        data,
    })
}

fn style_data_from_request(
    request: &UpsertProjectStyleBibleRequest,
    existing: Option<&StyleBibleDto>,
) -> Result<Value, String> {
    let mut data = normalize_style_data(
        existing
            .map(|style| style.data.clone())
            .unwrap_or_else(|| json!({})),
    );
    if let Some(object) = data.as_object_mut() {
        object.insert(
            "style_prompt".to_string(),
            Value::String(request.style_prompt.trim().to_string()),
        );
        object.insert(
            "color_palette".to_string(),
            Value::Array(
                request
                    .color_palette
                    .iter()
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .map(|value| Value::String(value.to_string()))
                    .collect(),
            ),
        );
        object.insert(
            "lighting".to_string(),
            Value::String(request.lighting.trim().to_string()),
        );
        object.insert(
            "composition".to_string(),
            Value::String(request.composition.trim().to_string()),
        );
        let (negative_prompt, _) = merge_negative_prompts(
            [request.negative_prompt.as_str()],
            NEGATIVE_PROMPT_MAX_LENGTH,
        );
        object.insert(
            "negative_prompt".to_string(),
            Value::String(negative_prompt),
        );
        if let Some(path) = request
            .reference_image_path
            .as_deref()
            .filter(|value| !value.trim().is_empty())
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
        } else {
            object.insert("reference_image_path".to_string(), Value::Null);
        }
        if let Some(reference_images) = &request.reference_images {
            object.insert(
                "reference_images_json".to_string(),
                Value::Array(reference_images.clone()),
            );
            object.insert(
                "reference_images".to_string(),
                Value::Array(reference_images.clone()),
            );
        }
    }
    Ok(data)
}

fn style_data_from_preset(preset: &StylePresetDto) -> Value {
    json!({
        "style_prompt": preset.style_prompt,
        "color_palette": preset.color_palette,
        "lighting": preset.lighting,
        "composition": preset.composition,
        "negative_prompt": preset.negative_prompt,
        "reference_image_path": preset.reference_image_path,
        "reference_images_json": [],
        "reference_images": []
    })
}

fn builtin_style_presets() -> Vec<StylePresetDto> {
    vec![
        StylePresetDto {
            preset_id: "builtin.clean_realistic".to_string(),
            source_type: "builtin".to_string(),
            name: "干净真实短视频".to_string(),
            style_prompt: "clean realistic short-video frame, natural human-scale detail, sharp subject, subtle background depth".to_string(),
            color_palette: vec!["neutral white".to_string(), "soft gray".to_string(), "muted teal accent".to_string()],
            lighting: "soft natural light, gentle highlights, no harsh studio glare".to_string(),
            composition: "vertical composition, clear subject, balanced negative space".to_string(),
            negative_prompt: "low resolution, distorted face, extra fingers, watermark, heavy filter".to_string(),
            reference_image_path: None,
        },
        StylePresetDto {
            preset_id: "builtin.minimal_line_art".to_string(),
            source_type: "builtin".to_string(),
            name: "极简线稿".to_string(),
            style_prompt: "minimal black and white line art, clean hand-drawn strokes, simple shapes, white background".to_string(),
            color_palette: vec!["black".to_string(), "white".to_string(), "light gray".to_string()],
            lighting: "flat clean lighting".to_string(),
            composition: "centered composition, clear empty space".to_string(),
            negative_prompt: "photorealistic, colorful, 3d render, complex background".to_string(),
            reference_image_path: None,
        },
        StylePresetDto {
            preset_id: "builtin.cinematic_warm".to_string(),
            source_type: "builtin".to_string(),
            name: "暖调电影感".to_string(),
            style_prompt: "cinematic realistic frame, warm restrained color grading, tactile detail, shallow depth of field".to_string(),
            color_palette: vec!["warm amber".to_string(), "deep green".to_string(), "charcoal".to_string()],
            lighting: "warm side light, soft shadow, practical indoor glow".to_string(),
            composition: "rule-of-thirds vertical frame, foreground depth, stable camera feel".to_string(),
            negative_prompt: "oversaturated, plastic skin, blurry subject, text watermark, noisy artifacts".to_string(),
            reference_image_path: None,
        },
    ]
}

fn select_style_reference_path(style: &StyleBibleDto) -> Result<String, TaskError> {
    let path = style
        .reference_image_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or_else(|| {
            style.reference_images.iter().find_map(|image| {
                image
                    .get("relativePath")
                    .or_else(|| image.get("relative_path"))
                    .and_then(Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_string)
            })
        })
        .ok_or_else(|| {
            task_validation_error(
                "validation.missing_input",
                "Style reference image is required before analysis.".to_string(),
            )
        })?;
    if !path.starts_with("assets/") {
        return Err(task_validation_error(
            "storage.path_denied",
            "Style reference analysis only accepts controlled assets/ paths.".to_string(),
        ));
    }
    Ok(path)
}

fn ensure_controlled_dummy_vlm_provider(database: &Database) -> Result<String, TaskError> {
    let repository = ProviderRepository::new(database);
    let providers = repository.list_providers().map_err(provider_config_error)?;
    if providers.iter().any(|provider| {
        provider.provider_id == CONTROLLED_DUMMY_VLM_PROVIDER_ID
            && provider.kind == "vlm"
            && provider.enabled
            && provider.status != "disabled"
    }) {
        return Ok(CONTROLLED_DUMMY_VLM_PROVIDER_ID.to_string());
    }

    repository
        .upsert_provider(&ProviderRecord {
            provider_id: CONTROLLED_DUMMY_VLM_PROVIDER_ID.to_string(),
            vendor: "dummy".to_string(),
            kind: "vlm".to_string(),
            display_name: "Controlled dummy VLM".to_string(),
            auth_type: "none".to_string(),
            key_alias: None,
            base_url: None,
            status: "ready".to_string(),
            enabled: true,
            config_json: json!({
                "adapter": "dummy",
                "controlled": true,
                "externalNetwork": false
            }),
        })
        .map_err(provider_config_error)?;
    Ok(CONTROLLED_DUMMY_VLM_PROVIDER_ID.to_string())
}

fn style_reference_analysis_prompt() -> String {
    [
        "Analyze this style reference image for a short-video Style Bible.",
        "Return only reusable visual style attributes.",
        "Do not include people identity, face details, names, brands, logos, watermarks, addresses, phone numbers, email, license plates, or other private details.",
        "Fields: style_prompt, color_palette, lighting, composition, negative_prompt_suggestion, warnings.",
    ]
    .join("\n")
}

fn style_reference_analysis_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "style_prompt",
            "color_palette",
            "lighting",
            "composition",
            "negative_prompt_suggestion",
            "warnings"
        ],
        "properties": {
            "style_prompt": { "type": "string" },
            "color_palette": {
                "type": "array",
                "items": { "type": "string" },
                "maxItems": 8
            },
            "lighting": { "type": "string" },
            "composition": { "type": "string" },
            "negative_prompt_suggestion": { "type": "string" },
            "warnings": {
                "type": "array",
                "items": { "type": "string" }
            }
        }
    })
}

fn style_analysis_from_provider_response(
    style_bible: &StyleBibleDto,
    reference_image_path: &str,
    provider_id: &str,
    provider_model_id: Option<&str>,
    provider_trace_id: &str,
    raw_description: String,
    parsed_json: Option<Value>,
) -> StyleReferenceAnalysisDto {
    let parsed = parsed_json.unwrap_or_else(|| json!({}));
    let mut warnings = read_style_string_array(&parsed, &["warnings"]).unwrap_or_default();
    let mut sensitive_omitted = false;

    let style_prompt =
        sanitize_positive_style_text(&read_style_string(&parsed, &["style_prompt", "stylePrompt"]))
            .map(|(value, changed)| {
                sensitive_omitted |= changed;
                value
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                warnings.push("style_reference.dummy_fallback".to_string());
                "consistent visual style from reference image, medium detail, balanced subject-background separation".to_string()
            });
    let color_palette = read_style_string_array(&parsed, &["color_palette", "colorPalette"])
        .map(|items| {
            items
                .into_iter()
                .filter_map(|item| {
                    sanitize_positive_style_text(&Some(item)).map(|(value, changed)| {
                        sensitive_omitted |= changed;
                        value
                    })
                })
                .filter(|value| !value.is_empty())
                .take(8)
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| {
            vec![
                "muted neutrals".to_string(),
                "soft accent color".to_string(),
                "controlled contrast".to_string(),
            ]
        });
    let lighting = sanitize_positive_style_text(&read_style_string(&parsed, &["lighting"]))
        .map(|(value, changed)| {
            sensitive_omitted |= changed;
            value
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "soft natural lighting with gentle shadow transitions".to_string());
    let composition = sanitize_positive_style_text(&read_style_string(&parsed, &["composition"]))
        .map(|(value, changed)| {
            sensitive_omitted |= changed;
            value
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            "vertical short-video frame, clear focal area, balanced negative space".to_string()
        });
    let negative_prompt_suggestion = read_style_string(
        &parsed,
        &[
            "negative_prompt_suggestion",
            "negativePromptSuggestion",
            "negative_prompt",
            "negativePrompt",
        ],
    )
    .map(|value| merge_negative_prompts([value.as_str()], NEGATIVE_PROMPT_MAX_LENGTH).0)
    .filter(|value| !value.is_empty())
    .unwrap_or_else(|| {
        "low resolution, distorted geometry, text artifacts, brand marks, private details"
            .to_string()
    });
    let raw_description = sanitize_raw_style_description(&raw_description);
    if sensitive_omitted {
        warnings.push("style_reference.sensitive_content_omitted".to_string());
    }
    if warnings.is_empty() {
        warnings.push("style_reference.analysis_scope_only".to_string());
    }
    warnings.sort();
    warnings.dedup();

    StyleReferenceAnalysisDto {
        project_id: style_bible.project_id.clone(),
        style_bible_id: style_bible.style_bible_id.clone(),
        reference_image_path: reference_image_path.to_string(),
        style_prompt,
        color_palette,
        lighting,
        composition,
        negative_prompt_suggestion,
        warnings,
        raw_description,
        provider_trace_id: Some(provider_trace_id.to_string()),
        provider_id: Some(provider_id.to_string()),
        provider_model_id: provider_model_id.map(str::to_string),
    }
}

fn sanitize_positive_style_text(value: &Option<String>) -> Option<(String, bool)> {
    let value = value.as_deref()?.trim();
    if value.is_empty() {
        return None;
    }
    let mut changed = false;
    let parts = value
        .split([',', '，', ';', '；', '\n'])
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            if contains_sensitive_positive_term(part) {
                changed = true;
                None
            } else {
                Some(part.to_string())
            }
        })
        .collect::<Vec<_>>();
    let sanitized = parts.join(", ");
    if sanitized != value {
        changed = true;
    }
    Some((sanitized, changed))
}

fn sanitize_raw_style_description(value: &str) -> String {
    if value.trim().is_empty() || contains_sensitive_positive_term(value) {
        "Visual style analysis completed; identity, brand, and private details were omitted."
            .to_string()
    } else {
        value.trim().to_string()
    }
}

fn contains_sensitive_positive_term(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    [
        "portrait",
        "person",
        "people",
        "man",
        "woman",
        "girl",
        "boy",
        "face",
        "identity",
        "name",
        "logo",
        "brand",
        "watermark",
        "license plate",
        "phone",
        "email",
        "address",
        "private",
        "privacy",
        "alice",
        "acme",
    ]
    .iter()
    .any(|term| normalized.contains(term))
        || [
            "肖像", "人物", "人脸", "男人", "女人", "男孩", "女孩", "姓名", "名字", "品牌", "水印",
            "车牌", "电话", "邮箱", "地址", "隐私",
        ]
        .iter()
        .any(|term| value.contains(term))
}

fn read_style_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn read_style_string_array(value: &Value, keys: &[&str]) -> Option<Vec<String>> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(|value| {
            if let Some(items) = value.as_array() {
                return Some(
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::trim)
                        .filter(|item| !item.is_empty())
                        .map(str::to_string)
                        .collect::<Vec<_>>(),
                );
            }
            value.as_str().map(|text| split_prompt_terms(text))
        })
    })
}

fn task_validation_error(code: &str, message: String) -> TaskError {
    TaskError::from_code(code, message)
}

fn task_config_error(message: String) -> TaskError {
    TaskError::from_code("db.query_failed", message)
}

fn provider_config_error(message: String) -> TaskError {
    TaskError::from_code("provider.config_error", message)
}

fn split_prompt_terms(prompt: &str) -> Vec<String> {
    prompt
        .split([',', '，', ';', '；', '\n'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn push_section(sections: &mut Vec<PromptSectionDto>, key: &str, label: &str, content: &str) {
    if content.trim().is_empty() {
        return;
    }
    sections.push(PromptSectionDto {
        key: key.to_string(),
        label: label.to_string(),
        content: content.trim().to_string(),
    });
}

fn resolve_prompt_characters(
    database: &Database,
    project_id: &str,
    character_ids: &[String],
) -> Result<Vec<CharacterBibleDto>, String> {
    let repository = CharacterRepository::new(database);
    let mut characters = Vec::new();
    for character_id in character_ids {
        let character_id = character_id.trim();
        if character_id.is_empty() {
            continue;
        }
        let character = repository
            .get_project_character_bible(project_id, character_id)?
            .ok_or_else(|| format!("Character Bible not found: {character_id}"))?;
        characters.push(character);
    }
    Ok(characters)
}

fn character_prompt_section(character: &CharacterBibleDto) -> String {
    [
        format!("{}({})", character.name, character.character_id),
        character.visual_prompt.clone(),
        character.appearance.clone(),
        character.clothing.clone(),
        character.personality.clone(),
    ]
    .into_iter()
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(", ")
}

fn validate_project_id(project_id: &str) -> Result<(), String> {
    if project_id.trim().is_empty() {
        Err("project_id is required.".to_string())
    } else {
        Ok(())
    }
}

fn normalize_name(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "默认画风".to_string()
    } else {
        name.to_string()
    }
}

fn project_style_bible_id(project_id: &str) -> String {
    format!("style_{project_id}")
}

fn create_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::asset_repository::NewAssetRecord;
    use crate::db::project_repository::ProjectRepository;
    use crate::db::provider_repository::{ProviderRecord, ProviderRepository};
    use crate::domain::project::CreateProjectRequest;
    use crate::services::keyring_service::KeyringService;

    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn merge_negative_prompt_deduplicates_and_limits() {
        let (merged, truncated) = merge_negative_prompts(
            [
                "low quality, watermark, distorted face",
                "watermark, extra fingers",
            ],
            64,
        );

        assert!(!truncated);
        assert_eq!(
            merged,
            "low quality, watermark, distorted face, extra fingers"
        );

        let (_, truncated) = merge_negative_prompts(["a, b, c, d, e, f"], 5);
        assert!(truncated);
    }

    #[test]
    fn style_bible_can_bind_asset_and_build_prompt_preview() {
        let root = test_root("style_bible_preview");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        fs::create_dir_all(workspace_root.join("assets/style_reference_image"))
            .expect("asset dir should exist");
        fs::write(
            workspace_root.join("assets/style_reference_image/ref.png"),
            "png",
        )
        .expect("asset should write");
        let detail = ProjectRepository::new(&database)
            .create_with_id(
                "project_style".to_string(),
                CreateProjectRequest {
                    title: "画风测试".to_string(),
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
                    style_prompt: Some("clean documentary".to_string()),
                    active_pack_id: None,
                    rule_refs: None,
                    executable_refs: None,
                    input_process_mode: "generate".to_string(),
                    input_options: Some(json!({})),
                },
            )
            .expect("project should create");
        let item = crate::domain::scene::SceneDto {
            item_id: "item_style".to_string(),
            project_id: "project_style".to_string(),
            index: 1,
            source_text: "source".to_string(),
            narration_text: "source".to_string(),
            visual_goal: "goal".to_string(),
            visual_description: "visual desc".to_string(),
            characters: vec!["主角".to_string()],
            character_ids: vec![],
            location_id: None,
            scene_description: "morning room".to_string(),
            image_prompt: "person waking up".to_string(),
            negative_prompt: "watermark, low quality".to_string(),
            video_prompt: "slow push".to_string(),
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
        };
        SceneRepository::new(&database)
            .upsert_storyboard_item(&item)
            .expect("item should save");
        let style = upsert_project_style_bible(
            &database,
            UpsertProjectStyleBibleRequest {
                project_id: detail.project.project_id.clone(),
                style_bible_id: None,
                name: "统一画风".to_string(),
                style_prompt: "clean documentary".to_string(),
                color_palette: vec!["white".to_string(), "teal".to_string()],
                lighting: "soft morning light".to_string(),
                composition: "vertical frame".to_string(),
                negative_prompt: "watermark, distorted face".to_string(),
                reference_image_path: None,
                reference_images: None,
                save_as_preset: None,
                preset_name: None,
            },
        )
        .expect("style should save");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_style_ref".to_string(),
                kind: "style_reference_image".to_string(),
                relative_path: "assets/style_reference_image/ref.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(3),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "mediaKind": "image" }),
            })
            .expect("asset should save");

        let bound = bind_style_reference_asset(
            &database,
            BindStyleReferenceAssetRequest {
                project_id: "project_style".to_string(),
                style_bible_id: Some(style.style_bible_id),
                asset_id: asset.asset_id,
            },
        )
        .expect("asset should bind");
        assert_eq!(
            bound.style_bible.reference_image_path.as_deref(),
            Some("assets/style_reference_image/ref.png")
        );

        let preview = build_image_prompt_preview(
            &database,
            BuildImagePromptPreviewRequest {
                project_id: "project_style".to_string(),
                item_id: "item_style".to_string(),
            },
        )
        .expect("preview should build");

        assert!(preview.final_prompt.contains("person waking up"));
        assert!(preview.final_prompt.contains("clean documentary"));
        assert_eq!(
            preview.final_negative_prompt.matches("watermark").count(),
            1
        );
        assert_eq!(preview.reference_images.len(), 1);
        assert_eq!(
            preview.reference_images[0].path,
            "assets/style_reference_image/ref.png"
        );

        cleanup(root);
    }

    #[test]
    fn character_bible_reference_images_are_injected_into_prompt_preview() {
        let root = test_root("character_bible_prompt_preview");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        fs::create_dir_all(workspace_root.join("assets/character_reference_image"))
            .expect("asset dir should exist");
        fs::write(
            workspace_root.join("assets/character_reference_image/hero_front.png"),
            "png",
        )
        .expect("asset should write");
        create_test_project(&database, "project_character_prompt");
        crate::services::character_service::upsert_project_character_bible(
            &database,
            crate::domain::character::UpsertProjectCharacterBibleRequest {
                project_id: "project_character_prompt".to_string(),
                character_id: Some("hero".to_string()),
                name: "Hero".to_string(),
                alias: vec![],
                age: "28".to_string(),
                gender: "female".to_string(),
                appearance: "silver hair, sharp green eyes".to_string(),
                clothing: "red pilot jacket".to_string(),
                personality: "calm and brave".to_string(),
                visual_prompt: None,
                negative_prompt: Some("wrong outfit, face drift".to_string()),
                reference_image_path: None,
                reference_images: None,
                lock_flags: None,
            },
        )
        .expect("character should save");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_hero_front".to_string(),
                kind: "character_reference_image".to_string(),
                relative_path: "assets/character_reference_image/hero_front.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(3),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "mediaKind": "image" }),
            })
            .expect("asset should save");
        crate::services::character_service::bind_character_reference_asset(
            &database,
            crate::domain::character::BindCharacterReferenceAssetRequest {
                project_id: "project_character_prompt".to_string(),
                character_id: "hero".to_string(),
                asset_id: asset.asset_id,
                reference_role: Some("character_front_view".to_string()),
            },
        )
        .expect("character asset should bind");

        let mut item = crate::domain::scene::SceneDto {
            item_id: "item_character_prompt".to_string(),
            project_id: "project_character_prompt".to_string(),
            index: 1,
            source_text: "source".to_string(),
            narration_text: "source".to_string(),
            visual_goal: "goal".to_string(),
            visual_description: "visual desc".to_string(),
            characters: vec![],
            character_ids: vec!["hero".to_string()],
            location_id: None,
            scene_description: "cockpit".to_string(),
            image_prompt: "hero checks the controls".to_string(),
            negative_prompt: "watermark".to_string(),
            video_prompt: "slow push".to_string(),
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
        };
        item = crate::services::scene_service::update_storyboard_item(&database, item)
            .expect("item should save");

        let preview = build_image_prompt_preview(
            &database,
            BuildImagePromptPreviewRequest {
                project_id: "project_character_prompt".to_string(),
                item_id: item.item_id,
            },
        )
        .expect("preview should build");

        assert!(preview.final_prompt.contains("Hero(hero)"));
        assert!(preview.final_prompt.contains("silver hair"));
        assert!(preview.final_prompt.contains("red pilot jacket"));
        assert!(preview.final_negative_prompt.contains("wrong outfit"));
        assert!(preview.final_negative_prompt.contains("face drift"));
        assert!(preview.final_negative_prompt.contains("watermark"));
        assert_eq!(preview.reference_images.len(), 1);
        assert_eq!(
            preview.reference_images[0].path,
            "assets/character_reference_image/hero_front.png"
        );
        assert_eq!(
            preview.reference_images[0].role,
            "character_front_view:hero"
        );

        cleanup(root);
    }

    #[test]
    fn location_bible_reference_images_are_injected_into_prompt_preview() {
        let root = test_root("location_bible_prompt_preview");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        fs::create_dir_all(workspace_root.join("assets/scene_reference_image"))
            .expect("asset dir should exist");
        fs::write(
            workspace_root.join("assets/scene_reference_image/study_wide.png"),
            "png",
        )
        .expect("asset should write");
        create_test_project(&database, "project_location_prompt");
        crate::services::location_service::upsert_project_location_bible(
            &database,
            crate::domain::location::UpsertProjectLocationBibleRequest {
                project_id: "project_location_prompt".to_string(),
                location_id: Some("study_room".to_string()),
                name: "Study room".to_string(),
                space_description: "wooden desk near a tall window, bookshelf on the left wall"
                    .to_string(),
                lighting: "warm morning side light".to_string(),
                time_of_day: "morning".to_string(),
                props: vec!["desk lamp".to_string(), "notebook".to_string()],
                visual_prompt: None,
                negative_prompt: Some("empty white background, floating furniture".to_string()),
                reference_image_path: None,
                reference_images: None,
                variants: None,
            },
        )
        .expect("location should save");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_study_wide".to_string(),
                kind: "scene_reference_image".to_string(),
                relative_path: "assets/scene_reference_image/study_wide.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(3),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "mediaKind": "image" }),
            })
            .expect("asset should save");
        crate::services::location_service::bind_location_reference_asset(
            &database,
            crate::domain::location::BindLocationReferenceAssetRequest {
                project_id: "project_location_prompt".to_string(),
                location_id: "study_room".to_string(),
                asset_id: asset.asset_id,
                reference_role: Some("scene_wide_view".to_string()),
            },
        )
        .expect("location asset should bind");
        let item = crate::domain::scene::SceneDto {
            item_id: "item_location_prompt".to_string(),
            project_id: "project_location_prompt".to_string(),
            index: 1,
            source_text: "source".to_string(),
            narration_text: "source".to_string(),
            visual_goal: "goal".to_string(),
            visual_description: "主角推开书房门".to_string(),
            characters: vec![],
            character_ids: vec![],
            location_id: Some("study_room".to_string()),
            scene_description: "free text should not drive prompt when location id is set"
                .to_string(),
            image_prompt: "cinematic vertical frame".to_string(),
            negative_prompt: "watermark".to_string(),
            video_prompt: "slow push".to_string(),
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
        };
        crate::services::scene_service::update_storyboard_item(&database, item)
            .expect("item should save");

        let preview = build_image_prompt_preview(
            &database,
            BuildImagePromptPreviewRequest {
                project_id: "project_location_prompt".to_string(),
                item_id: "item_location_prompt".to_string(),
            },
        )
        .expect("preview should build");

        assert!(preview.final_prompt.contains("Study room(study_room)"));
        assert!(preview
            .final_prompt
            .contains("wooden desk near a tall window"));
        assert!(preview.final_prompt.contains("desk lamp, notebook"));
        assert!(!preview
            .final_prompt
            .contains("free text should not drive prompt"));
        assert!(preview.final_negative_prompt.contains("floating furniture"));
        assert_eq!(preview.reference_images.len(), 1);
        assert_eq!(
            preview.reference_images[0].path,
            "assets/scene_reference_image/study_wide.png"
        );
        assert_eq!(
            preview.reference_images[0].role,
            "scene_wide_view:study_room"
        );
        cleanup(root);
    }

    #[test]
    fn analyze_style_reference_requires_controlled_reference_image() {
        let root = test_root("style_reference_requires_asset");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_test_project(&database, "project_no_style_ref");

        let missing = analyze_style_reference_image(
            &database,
            &KeyringService::memory(),
            AnalyzeStyleReferenceRequest {
                project_id: "project_no_style_ref".to_string(),
                style_bible_id: None,
                provider_id: None,
                provider_model_id: None,
            },
        )
        .expect_err("analysis without reference image should fail");
        assert_eq!(missing.error_code, "validation.missing_input");

        let style = upsert_project_style_bible(
            &database,
            UpsertProjectStyleBibleRequest {
                project_id: "project_no_style_ref".to_string(),
                style_bible_id: None,
                name: "bad path".to_string(),
                style_prompt: "".to_string(),
                color_palette: vec![],
                lighting: "".to_string(),
                composition: "".to_string(),
                negative_prompt: "".to_string(),
                reference_image_path: Some("../outside.png".to_string()),
                reference_images: None,
                save_as_preset: None,
                preset_name: None,
            },
        );
        assert!(style.is_err());

        cleanup(root);
    }

    #[test]
    fn analyze_style_reference_returns_editable_style_suggestions() {
        let root = test_root("style_reference_analysis");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        fs::create_dir_all(root.join("workspace/assets/style_reference_image"))
            .expect("asset dir should exist");
        fs::write(
            root.join("workspace/assets/style_reference_image/ref.png"),
            "png",
        )
        .expect("asset should write");
        create_test_project(&database, "project_style_analysis");
        let style = upsert_project_style_bible(
            &database,
            UpsertProjectStyleBibleRequest {
                project_id: "project_style_analysis".to_string(),
                style_bible_id: None,
                name: "analysis style".to_string(),
                style_prompt: "".to_string(),
                color_palette: vec![],
                lighting: "".to_string(),
                composition: "".to_string(),
                negative_prompt: "".to_string(),
                reference_image_path: None,
                reference_images: None,
                save_as_preset: None,
                preset_name: None,
            },
        )
        .expect("style should save");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_style_analysis_ref".to_string(),
                kind: "style_reference_image".to_string(),
                relative_path: "assets/style_reference_image/ref.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(3),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "mediaKind": "image" }),
            })
            .expect("asset should save");
        bind_style_reference_asset(
            &database,
            BindStyleReferenceAssetRequest {
                project_id: "project_style_analysis".to_string(),
                style_bible_id: Some(style.style_bible_id.clone()),
                asset_id: asset.asset_id,
            },
        )
        .expect("asset should bind");

        let analysis = analyze_style_reference_image(
            &database,
            &KeyringService::memory(),
            AnalyzeStyleReferenceRequest {
                project_id: "project_style_analysis".to_string(),
                style_bible_id: Some(style.style_bible_id),
                provider_id: None,
                provider_model_id: None,
            },
        )
        .expect("analysis should succeed");

        assert_eq!(
            analysis.reference_image_path,
            "assets/style_reference_image/ref.png"
        );
        assert!(!analysis.style_prompt.trim().is_empty());
        assert!(!analysis.color_palette.is_empty());
        assert!(!analysis.lighting.trim().is_empty());
        assert!(!analysis.composition.trim().is_empty());
        assert!(analysis
            .provider_id
            .as_deref()
            .is_some_and(|provider_id| { provider_id == CONTROLLED_DUMMY_VLM_PROVIDER_ID }));

        let serialized = serde_json::to_string(&analysis).expect("analysis should serialize");
        for forbidden in [
            "Alice",
            "ACME",
            "portrait of a woman",
            "logo on wall",
            "phone",
            "email",
            "address",
        ] {
            assert!(
                !serialized
                    .to_ascii_lowercase()
                    .contains(&forbidden.to_ascii_lowercase()),
                "analysis leaked forbidden term: {forbidden}"
            );
        }
        let saved = StyleRepository::new(&database)
            .get_style_bible(&analysis.style_bible_id)
            .expect("style should read")
            .expect("style should exist");
        assert!(saved.style_prompt.is_empty());

        cleanup(root);
    }

    #[test]
    fn style_reference_analysis_sanitizes_sensitive_provider_output() {
        let style = StyleBibleDto {
            style_bible_id: "style_safe".to_string(),
            project_id: "project_safe".to_string(),
            name: "safe".to_string(),
            style_prompt: "".to_string(),
            color_palette: vec![],
            lighting: "".to_string(),
            composition: "".to_string(),
            negative_prompt: "".to_string(),
            reference_image_path: Some("assets/style_reference_image/ref.png".to_string()),
            reference_images: vec![],
            data: json!({}),
            created_at: None,
            updated_at: None,
        };

        let analysis = style_analysis_from_provider_response(
            &style,
            "assets/style_reference_image/ref.png",
            "provider_test",
            None,
            "trace_test",
            "portrait of Alice beside ACME logo and phone number".to_string(),
            Some(json!({
                "style_prompt": "portrait of a woman, clean cinematic color, ACME logo on wall",
                "color_palette": ["warm beige", "brand red logo"],
                "lighting": "soft face lighting, gentle contrast",
                "composition": "centered portrait composition, vertical frame",
                "negative_prompt_suggestion": "watermark, logo, private details",
                "warnings": []
            })),
        );

        let serialized = serde_json::to_string(&analysis).expect("analysis should serialize");
        assert!(analysis
            .warnings
            .contains(&"style_reference.sensitive_content_omitted".to_string()));
        assert!(!serialized.to_ascii_lowercase().contains("alice"));
        assert!(!serialized.to_ascii_lowercase().contains("acme"));
        assert!(!serialized
            .to_ascii_lowercase()
            .contains("portrait of a woman"));
        assert!(!serialized.to_ascii_lowercase().contains("logo on wall"));
        assert!(!serialized.to_ascii_lowercase().contains("phone number"));
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-style-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }

    fn create_test_project(database: &Database, project_id: &str) {
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "画风测试".to_string(),
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

    #[allow(dead_code)]
    fn seed_vlm_provider(database: &Database, provider_id: &str) {
        ProviderRepository::new(database)
            .upsert_provider(&ProviderRecord {
                provider_id: provider_id.to_string(),
                vendor: "dummy".to_string(),
                kind: "vlm".to_string(),
                display_name: "Dummy VLM".to_string(),
                auth_type: "none".to_string(),
                key_alias: None,
                base_url: None,
                status: "ready".to_string(),
                enabled: true,
                config_json: json!({ "adapter": "dummy" }),
            })
            .expect("provider should save");
    }
}
