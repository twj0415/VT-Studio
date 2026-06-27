use crate::db::asset_repository::{AssetRepository, NewAssetRecord, NewAssetReferenceRecord};
use crate::db::provider_repository::{
    ProviderModelRecord, ProviderRecord, ProviderRepository, WorkflowPresetRecord,
};
use crate::db::Database;
use crate::domain::media::{
    AssetDto, AssetPreviewDto, AssetPreviewRequest, AssetReferenceDto, CreateAssetReferenceRequest,
    DeleteAssetReferenceRequest, DeleteAssetRequest, ExecutableMediaOptionDto, ImportAssetRequest,
    ListAssetsRequest, MediaInputPlanDto, MediaInputRequirementDto,
};
use crate::services::storage_service::{FileAccessPolicy, FileBucket, StorageService};
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const ASSET_PREVIEW_MAX_BYTES: u64 = 12 * 1024 * 1024;
const FFMPEG_BINARY: &str = "ffmpeg.exe";

pub fn list_executable_media_options(
    database: &Database,
) -> Result<Vec<ExecutableMediaOptionDto>, String> {
    let repository = ProviderRepository::new(database);
    let providers = repository.list_providers()?;
    let mut options = Vec::new();

    for model in repository.list_provider_models(None)? {
        let provider = providers
            .iter()
            .find(|provider| provider.provider_id == model.provider_id);
        options.extend(provider_model_to_executable_options(model, provider));
    }

    for preset in repository.list_workflow_presets_by_provider(None)? {
        let provider_id = workflow_provider_id(&preset);
        let provider = provider_id.as_deref().and_then(|provider_id| {
            providers
                .iter()
                .find(|provider| provider.provider_id == provider_id)
        });
        options.extend(workflow_preset_to_executable_options(preset, provider));
    }

    options.sort_by(|left, right| {
        left.source_type
            .cmp(&right.source_type)
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    Ok(options)
}

fn provider_model_to_executable_options(
    model: ProviderModelRecord,
    provider: Option<&ProviderRecord>,
) -> Vec<ExecutableMediaOptionDto> {
    let config = &model.config_json;
    let provider_kind = read_string(config, &["providerKind", "provider_kind"])
        .or_else(|| provider.map(|provider| provider.kind.clone()))
        .unwrap_or_else(|| "unknown".to_string());
    let vendor = read_string(config, &["vendor"])
        .or_else(|| provider.map(|provider| provider.vendor.clone()))
        .unwrap_or_else(|| "unknown".to_string());
    let capabilities = read_string_array(config, &["abilityTypes", "ability_types"])
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec![model.capability.clone()]);
    capabilities
        .iter()
        .map(|capability| {
            provider_model_to_executable_option_for_ability(
                &model,
                provider,
                &provider_kind,
                &vendor,
                &capabilities,
                capability,
            )
        })
        .collect()
}

fn provider_model_to_executable_option_for_ability(
    model: &ProviderModelRecord,
    provider: Option<&ProviderRecord>,
    provider_kind: &str,
    vendor: &str,
    capabilities: &[String],
    capability: &str,
) -> ExecutableMediaOptionDto {
    let config = &model.config_json;
    let limits = ability_scoped_value(config, "limits", capability);
    let input_requirements = ability_scoped_value(config, "inputRequirements", capability);
    let param_schema = ability_scoped_value(config, "paramSchema", capability);
    let default_params = ability_scoped_value(config, "defaultParams", capability);
    let image_kind = read_string(config, &["imageKind", "image_kind"]);
    let asset_kind = read_string(config, &["assetKind", "asset_kind"]);
    let status = read_string(config, &["status"]).unwrap_or_else(|| {
        if model.enabled {
            "ready".to_string()
        } else {
            "disabled".to_string()
        }
    });
    let source_id = model.model_id.clone();
    let disabled_reason =
        provider_model_disabled_reason(model, provider, provider_kind, vendor, &status)
            .or_else(|| ability_disabled_reason(config, capability));
    let enabled = disabled_reason.is_none();
    let input_plan = build_media_input_plan(
        capability,
        provider_kind,
        &input_requirements,
        &limits,
        Some(&param_schema),
        Some(&default_params),
        image_kind.as_deref(),
        asset_kind.as_deref(),
    );

    ExecutableMediaOptionDto {
        option_id: format!("provider_model:{source_id}:{capability}"),
        source_type: "provider_model".to_string(),
        source_id: source_id.clone(),
        label: model.display_name.clone(),
        provider_id: model.provider_id.clone(),
        provider_kind: provider_kind.to_string(),
        vendor: vendor.to_string(),
        kind: "provider_model".to_string(),
        capability: capability.to_string(),
        capabilities: capabilities.to_vec(),
        constraints: json!({
            "limits": limits,
            "inputRequirements": input_requirements,
            "paramSchema": param_schema,
            "defaultParams": default_params,
            "selectedAbilityType": capability,
            "inputModalities": read_string_array(config, &["inputModalities", "input_modalities"]).unwrap_or_default(),
            "outputModalities": read_string_array(config, &["outputModalities", "output_modalities"]).unwrap_or_default(),
            "featureFlags": read_string_array(config, &["featureFlags", "feature_flags"]).unwrap_or_default(),
            "apiContractVerified": read_bool(config, &["apiContractVerified", "api_contract_verified"]).unwrap_or(false),
            "modelName": read_string(config, &["modelName", "model_name"]).unwrap_or_else(|| model.provider_model_id.clone()),
            "vendorModelId": model.provider_model_id,
        }),
        input_plan,
        status,
        provider_model_id: Some(source_id.clone()),
        workflow_preset_id: None,
        enabled,
        disabled_reason,
        normalized_params: json!({
            "sourceType": "provider_model",
            "providerId": model.provider_id,
            "providerKind": provider_kind,
            "providerModelId": source_id,
            "abilityType": capability,
        }),
    }
}

fn workflow_preset_to_executable_options(
    preset: WorkflowPresetRecord,
    provider: Option<&ProviderRecord>,
) -> Vec<ExecutableMediaOptionDto> {
    let config = &preset.config_json;
    let capabilities = read_string_array(config, &["abilityTypes", "ability_types"])
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec![preset.capability.clone()]);
    capabilities
        .iter()
        .map(|capability| {
            workflow_preset_to_executable_option_for_ability(
                &preset,
                provider,
                &capabilities,
                capability,
            )
        })
        .collect()
}

fn workflow_preset_to_executable_option_for_ability(
    preset: &WorkflowPresetRecord,
    provider: Option<&ProviderRecord>,
    capabilities: &[String],
    capability: &str,
) -> ExecutableMediaOptionDto {
    let config = &preset.config_json;
    let source_id = preset.preset_id.clone();
    let provider_id = workflow_provider_id(preset).unwrap_or_default();
    let vendor = read_string(config, &["vendor"]).unwrap_or_else(|| preset.kind.clone());
    let limits = ability_scoped_value(config, "limits", capability);
    let param_schema = ability_scoped_value(config, "paramSchema", capability);
    let node_map = read_value(config, &["nodeMap", "node_map"]).unwrap_or_else(|| json!({}));
    let output_map = read_value(config, &["outputMap", "output_map"]).unwrap_or_else(|| json!({}));
    let default_params = ability_scoped_value(config, "defaultParams", capability);
    let image_kind = read_string(config, &["imageKind", "image_kind"]);
    let asset_kind = read_string(config, &["assetKind", "asset_kind"]);
    let status = read_string(config, &["status"]).unwrap_or_else(|| {
        if preset.enabled {
            "ready".to_string()
        } else {
            "disabled".to_string()
        }
    });
    let label =
        read_string(config, &["displayName", "display_name"]).unwrap_or(preset.name.clone());
    let disabled_reason = workflow_preset_disabled_reason(
        preset,
        provider,
        &provider_id,
        &vendor,
        &status,
        &node_map,
        &output_map,
    )
    .or_else(|| ability_disabled_reason(config, capability));
    let enabled = disabled_reason.is_none();
    let input_plan = build_media_input_plan(
        capability,
        "workflow",
        &json!({}),
        &limits,
        Some(&param_schema),
        Some(&default_params),
        image_kind.as_deref(),
        asset_kind.as_deref(),
    );

    ExecutableMediaOptionDto {
        option_id: format!("workflow_preset:{source_id}:{capability}"),
        source_type: "workflow_preset".to_string(),
        source_id: source_id.clone(),
        label,
        provider_id,
        provider_kind: "workflow".to_string(),
        vendor: vendor.clone(),
        kind: "workflow_preset".to_string(),
        capability: capability.to_string(),
        capabilities: capabilities.to_vec(),
        constraints: json!({
            "limits": limits,
            "paramSchema": param_schema,
            "nodeMap": node_map,
            "outputMap": output_map,
            "selectedAbilityType": capability,
            "workflowKey": read_string(config, &["workflowKey", "workflow_key"]).unwrap_or_default(),
            "workflowId": read_string(config, &["workflowId", "workflow_id"]),
            "workflowVersion": read_string(config, &["workflowVersion", "workflow_version"]).unwrap_or_else(|| "1.0.0".to_string()),
            "inputModalities": read_string_array(config, &["inputModalities", "input_modalities"]).unwrap_or_default(),
            "outputModalities": read_string_array(config, &["outputModalities", "output_modalities"]).unwrap_or_default(),
        }),
        input_plan,
        status,
        provider_model_id: None,
        workflow_preset_id: Some(source_id.clone()),
        enabled,
        disabled_reason,
        normalized_params: json!({
            "sourceType": "workflow_preset",
            "providerId": workflow_provider_id(&preset),
            "providerKind": "workflow",
            "workflowPresetId": source_id,
            "workflowVendor": vendor,
            "defaultParams": default_params,
            "abilityType": capability,
        }),
    }
}

fn provider_model_disabled_reason(
    model: &ProviderModelRecord,
    provider: Option<&ProviderRecord>,
    provider_kind: &str,
    vendor: &str,
    status: &str,
) -> Option<String> {
    let provider = match provider {
        Some(provider) => provider,
        None => return Some("provider.not_found".to_string()),
    };
    if !provider.enabled || provider.status == "disabled" {
        return Some("provider.disabled".to_string());
    }
    if provider.status != "ready" {
        return Some(format!(
            "provider.status.{status}",
            status = provider.status
        ));
    }
    if provider.kind == "workflow" {
        return Some("provider_models.cannot_use_workflow_provider".to_string());
    }
    if provider.kind != provider_kind {
        return Some("provider.kind_mismatch".to_string());
    }
    if provider.vendor != vendor {
        return Some("provider.vendor_mismatch".to_string());
    }
    if !model.enabled || status == "disabled" {
        return Some("provider.model_disabled".to_string());
    }
    if status != "ready" {
        return Some(format!("provider.model_status.{status}"));
    }
    None
}

fn workflow_preset_disabled_reason(
    preset: &WorkflowPresetRecord,
    provider: Option<&ProviderRecord>,
    provider_id: &str,
    vendor: &str,
    status: &str,
    node_map: &Value,
    output_map: &Value,
) -> Option<String> {
    let provider = match provider {
        Some(provider) => provider,
        None => return Some("provider.not_found".to_string()),
    };
    if provider.provider_id != provider_id {
        return Some("workflow.provider_unavailable".to_string());
    }
    if !provider.enabled || provider.status == "disabled" {
        return Some("provider.disabled".to_string());
    }
    if provider.status != "ready" {
        return Some(format!(
            "provider.status.{status}",
            status = provider.status
        ));
    }
    if provider.kind != "workflow" {
        return Some("workflow.provider_kind_mismatch".to_string());
    }
    if provider.vendor != vendor {
        return Some("workflow.provider_unavailable".to_string());
    }
    if !preset.enabled || status == "disabled" {
        return Some("workflow.preset_disabled".to_string());
    }
    if status != "ready" {
        return Some(format!("workflow.preset_status.{status}"));
    }
    if !node_map
        .as_object()
        .is_some_and(|object| !object.is_empty())
    {
        return Some("workflow.invalid_node_map".to_string());
    }
    if !output_map
        .as_object()
        .is_some_and(|object| !object.is_empty())
    {
        return Some("workflow.output_missing".to_string());
    }
    None
}

fn build_media_input_plan(
    ability_type: &str,
    provider_kind: &str,
    input_requirements: &Value,
    limits: &Value,
    param_schema: Option<&Value>,
    default_params: Option<&Value>,
    image_kind: Option<&str>,
    asset_kind: Option<&str>,
) -> MediaInputPlanDto {
    let plan_kind = plan_kind_for(provider_kind, ability_type);
    let resolved_image_kind = resolve_image_kind(ability_type, image_kind);
    let resolved_asset_kind = resolve_asset_kind(resolved_image_kind.as_deref(), asset_kind);
    let mut items = Vec::new();

    match ability_type {
        "text_to_image" | "image_to_image" => {
            items.push(input_item(
                "prompt",
                "text",
                "project",
                requirement_for(input_requirements, "prompt", "required"),
                vec!["generate"],
                "prompt.required",
                json!({ "component": "textarea", "rows": 4 }),
                json!({}),
                json!({ "field": "prompt" }),
            ));
            items.push(input_item(
                "negativePrompt",
                "text",
                "project",
                requirement_for(input_requirements, "negativePrompt", "optional"),
                vec!["generate"],
                "negative_prompt.optional",
                json!({ "component": "textarea", "rows": 2 }),
                json!({}),
                json!({ "field": "negativePrompt" }),
            ));
            if ability_type == "image_to_image" {
                items.push(input_item(
                    "referenceAsset",
                    "image",
                    "project",
                    requirement_for(input_requirements, "referenceAsset", "required"),
                    vec!["upload", "select_existing"],
                    "reference_asset.required",
                    json!({ "component": "asset-picker", "accept": "image/*" }),
                    reference_constraints(limits),
                    json!({ "field": "referenceAsset" }),
                ));
            }
        }
        "text_to_video" => {
            items.push(input_item(
                "videoPrompt",
                "text",
                "storyboard_item",
                requirement_for(input_requirements, "videoPrompt", "required"),
                vec!["generate"],
                "video_prompt.required",
                json!({ "component": "textarea", "rows": 4 }),
                json!({}),
                json!({ "field": "videoPrompt" }),
            ));
        }
        "image_to_video" | "first_frame_i2v" | "reference_to_video" => {
            items.push(input_item(
                "startFrame",
                "image",
                "storyboard_item",
                requirement_for(input_requirements, "startFrame", "required"),
                vec!["select_existing", "upload", "generate"],
                "start_frame.required",
                json!({ "component": "asset-picker", "accept": "image/*" }),
                reference_constraints(limits),
                json!({ "field": "startFrame" }),
            ));
            items.push(input_item(
                "videoPrompt",
                "text",
                "storyboard_item",
                requirement_for(input_requirements, "videoPrompt", "required"),
                vec!["generate"],
                "video_prompt.required",
                json!({ "component": "textarea", "rows": 4 }),
                json!({}),
                json!({ "field": "videoPrompt" }),
            ));
            if ability_type == "reference_to_video" {
                items.push(input_item(
                    "referenceAsset",
                    "image",
                    "project",
                    requirement_for(input_requirements, "referenceAsset", "required"),
                    vec!["select_existing", "upload"],
                    "reference_asset.required",
                    json!({ "component": "asset-picker", "accept": "image/*" }),
                    reference_constraints(limits),
                    json!({ "field": "referenceAsset" }),
                ));
            }
        }
        "start_end_frame_i2v" => {
            items.push(input_item(
                "startFrame",
                "image",
                "storyboard_item",
                requirement_for(input_requirements, "startFrame", "required"),
                vec!["select_existing", "upload", "generate"],
                "start_frame.required",
                json!({ "component": "asset-picker", "accept": "image/*" }),
                reference_constraints(limits),
                json!({ "field": "startFrame" }),
            ));
            items.push(input_item(
                "endFrame",
                "image",
                "storyboard_item",
                requirement_for(input_requirements, "endFrame", "required"),
                vec!["select_existing", "upload", "derive_from_selected_image"],
                "end_frame.required",
                json!({ "component": "asset-picker", "accept": "image/*" }),
                reference_constraints(limits),
                json!({ "field": "endFrame" }),
            ));
            items.push(input_item(
                "videoPrompt",
                "text",
                "storyboard_item",
                requirement_for(input_requirements, "videoPrompt", "required"),
                vec!["generate"],
                "video_prompt.required",
                json!({ "component": "textarea", "rows": 4 }),
                json!({}),
                json!({ "field": "videoPrompt" }),
            ));
        }
        "video_continuation" => {
            items.push(input_item(
                "sourceVideo",
                "video",
                "storyboard_item",
                requirement_for(input_requirements, "sourceVideo", "required"),
                vec!["select_existing", "upload"],
                "source_video.required",
                json!({ "component": "asset-picker", "accept": "video/*" }),
                video_constraints(limits),
                json!({ "field": "sourceVideo" }),
            ));
            items.push(input_item(
                "tailFrame",
                "image",
                "storyboard_item",
                requirement_for(input_requirements, "tailFrame", "optional"),
                vec!["extract_from_source_video", "select_existing", "upload"],
                "tail_frame.optional",
                json!({ "component": "asset-picker", "accept": "image/*" }),
                reference_constraints(limits),
                json!({ "field": "tailFrame" }),
            ));
            items.push(input_item(
                "videoPrompt",
                "text",
                "storyboard_item",
                requirement_for(input_requirements, "videoPrompt", "required"),
                vec!["generate"],
                "video_prompt.required",
                json!({ "component": "textarea", "rows": 4 }),
                json!({}),
                json!({ "field": "videoPrompt" }),
            ));
        }
        _ => {
            items.push(input_item(
                "prompt",
                "text",
                "project",
                requirement_for(input_requirements, "prompt", "optional"),
                vec!["generate"],
                "prompt.optional",
                json!({ "component": "textarea" }),
                json!({}),
                json!({ "field": "prompt" }),
            ));
        }
    }

    append_common_params(&mut items, ability_type, limits, input_requirements);
    append_character_inputs(&mut items, input_requirements);
    if let Some(param_schema) = param_schema {
        append_workflow_params(&mut items, param_schema, default_params);
    }

    let required_count = items
        .iter()
        .filter(|item| item.requirement == "required")
        .count();
    let optional_count = items
        .iter()
        .filter(|item| item.requirement == "optional")
        .count();
    let unused_count = items
        .iter()
        .filter(|item| item.requirement == "unused")
        .count();

    MediaInputPlanDto {
        plan_kind,
        ability_type: ability_type.to_string(),
        image_kind: resolved_image_kind,
        asset_kind: resolved_asset_kind,
        items,
        required_count,
        optional_count,
        unused_count,
    }
}

fn resolve_image_kind(ability_type: &str, image_kind: Option<&str>) -> Option<String> {
    image_kind
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or_else(|| {
            if matches!(ability_type, "text_to_image" | "image_to_image") {
                Some("storyboard_image".to_string())
            } else {
                None
            }
        })
}

fn resolve_asset_kind(image_kind: Option<&str>, asset_kind: Option<&str>) -> Option<String> {
    asset_kind
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or_else(|| image_kind.map(default_asset_kind_for_image_kind))
}

fn default_asset_kind_for_image_kind(image_kind: &str) -> String {
    match image_kind {
        "character_reference" => "character_reference",
        "scene_reference" => "scene_reference",
        "style_reference" => "style_reference",
        "cover_image" => "cover_source",
        "storyboard_image" | "prop_reference" | "end_frame" | "control_image" => "generated_output",
        _ => "generated_output",
    }
    .to_string()
}

fn append_common_params(
    items: &mut Vec<MediaInputRequirementDto>,
    ability_type: &str,
    limits: &Value,
    input_requirements: &Value,
) {
    items.push(input_item(
        "aspectRatio",
        "workflow_param",
        "project",
        requirement_for(input_requirements, "aspectRatio", "optional"),
        vec!["generate"],
        "aspect_ratio.optional",
        json!({ "component": "select" }),
        json!({ "options": read_string_array(limits, &["supportedAspectRatios", "aspectRatios", "supported_aspect_ratios", "aspect_ratios"]).unwrap_or_default() }),
        json!({ "field": "aspectRatio" }),
    ));
    items.push(input_item(
        "resolution",
        "workflow_param",
        "project",
        requirement_for(input_requirements, "resolution", "optional"),
        vec!["generate"],
        "resolution.optional",
        json!({ "component": "select" }),
        json!({ "options": read_string_array(limits, &["resolutions"]).unwrap_or_default() }),
        json!({ "field": "resolution" }),
    ));
    items.push(input_item(
        "seed",
        "workflow_param",
        "project",
        requirement_for(input_requirements, "seed", "optional"),
        vec!["generate"],
        "seed.optional",
        json!({ "component": "number-input" }),
        json!({ "min": 0 }),
        json!({ "field": "seed" }),
    ));

    if ability_type.contains("video") || ability_type.contains("i2v") {
        items.push(input_item(
            "durationSeconds",
            "workflow_param",
            "storyboard_item",
            requirement_for(input_requirements, "durationSeconds", "optional"),
            vec!["generate"],
            "duration.optional",
            json!({ "component": "number-input" }),
            read_value(
                limits,
                &[
                    "durationSeconds",
                    "duration_seconds",
                    "durationRange",
                    "duration_range",
                ],
            )
            .unwrap_or_else(|| json!({})),
            json!({ "field": "durationSeconds" }),
        ));
        items.push(input_item(
            "fps",
            "workflow_param",
            "storyboard_item",
            requirement_for(input_requirements, "fps", "optional"),
            vec!["generate"],
            "fps.optional",
            json!({ "component": "select" }),
            json!({
                "values": read_number_array(limits, &["fps", "fps_values"]).unwrap_or_default(),
                "range": read_value(limits, &["fpsRange", "fps_range"])
            }),
            json!({ "field": "fps" }),
        ));
    }
}

fn append_character_inputs(items: &mut Vec<MediaInputRequirementDto>, input_requirements: &Value) {
    let character_inputs = [
        ("character_front_view", "角色正面图"),
        ("character_side_view", "角色侧面图"),
        ("character_back_view", "角色背面图"),
        ("character_full_body", "角色全身图"),
        ("character_face_closeup", "面部特写"),
        ("character_expression_sheet", "表情表"),
        ("character_outfit", "服装细节"),
        ("character_pose", "姿态参考"),
        ("character_mood", "情绪状态参考"),
    ];
    for (input_key, label) in character_inputs {
        let default_requirement = if input_key == "character_front_view" {
            "optional"
        } else {
            "unused"
        };
        let requirement = requirement_for(input_requirements, input_key, default_requirement);
        items.push(input_item(
            input_key,
            "image",
            "character_bible",
            requirement,
            vec!["generate", "upload", "select_existing"],
            &format!("{input_key}.missing"),
            json!({ "component": "asset-picker", "accept": "image/*", "label": label }),
            json!({ "assetKind": input_key, "maxCount": 1 }),
            json!({ "imageKind": input_key }),
        ));
    }
}

fn append_workflow_params(
    items: &mut Vec<MediaInputRequirementDto>,
    param_schema: &Value,
    default_params: Option<&Value>,
) {
    let Some(object) = param_schema.as_object() else {
        return;
    };
    let schema_required = param_schema
        .get("required")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    for (param_key, schema) in object {
        if param_key == "required" || param_key == "properties" {
            continue;
        }
        let is_required = schema
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or_else(|| schema_required.iter().any(|item| item == param_key));
        let schema_type = schema
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("string");
        let input_group = workflow_param_group(schema_type);
        let default_value = default_params
            .and_then(|params| params.get(param_key))
            .cloned();
        items.push(input_item(
            &format!("workflowParams.{param_key}"),
            input_group,
            "project",
            if is_required { "required" } else { "optional" }.to_string(),
            workflow_param_source_options(input_group),
            &format!("workflow_param.{param_key}.missing"),
            json!({
                "component": workflow_param_component(schema_type),
                "schema": schema,
            }),
            schema.clone(),
            json!({
                "paramKey": param_key,
                "defaultValue": default_value,
            }),
        ));
    }

    if let Some(properties) = param_schema.get("properties").and_then(Value::as_object) {
        for (param_key, schema) in properties {
            let is_required = schema_required.iter().any(|item| item == param_key);
            let schema_type = schema
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("string");
            let input_group = workflow_param_group(schema_type);
            let default_value = default_params
                .and_then(|params| params.get(param_key))
                .cloned();
            items.push(input_item(
                &format!("workflowParams.{param_key}"),
                input_group,
                "project",
                if is_required { "required" } else { "optional" }.to_string(),
                workflow_param_source_options(input_group),
                &format!("workflow_param.{param_key}.missing"),
                json!({
                    "component": workflow_param_component(schema_type),
                    "schema": schema,
                }),
                schema.clone(),
                json!({
                    "paramKey": param_key,
                    "defaultValue": default_value,
                }),
            ));
        }
    }
}

fn input_item(
    input_key: &str,
    input_group: &str,
    owner_type: &str,
    requirement: String,
    source_options: Vec<&str>,
    missing_reason: &str,
    ui_schema: Value,
    constraints: Value,
    normalized_params: Value,
) -> MediaInputRequirementDto {
    MediaInputRequirementDto {
        input_key: input_key.to_string(),
        input_group: input_group.to_string(),
        owner_type: Some(owner_type.to_string()),
        owner_id: None,
        missing_reason: if requirement == "required" {
            Some(missing_reason.to_string())
        } else {
            None
        },
        requirement,
        source_options: source_options.into_iter().map(str::to_string).collect(),
        ui_schema,
        constraints,
        normalized_params,
    }
}

fn requirement_for(input_requirements: &Value, input_key: &str, fallback: &str) -> String {
    if contains_input_key(
        input_requirements,
        &["requiredInputs", "required_inputs"],
        input_key,
    ) {
        return "required".to_string();
    }
    if contains_input_key(
        input_requirements,
        &["optionalInputs", "optional_inputs"],
        input_key,
    ) {
        return "optional".to_string();
    }
    if contains_input_key(
        input_requirements,
        &["unusedInputs", "unused_inputs"],
        input_key,
    ) {
        return "unused".to_string();
    }
    fallback.to_string()
}

fn contains_input_key(input_requirements: &Value, keys: &[&str], input_key: &str) -> bool {
    keys.iter().any(|key| {
        input_requirements
            .get(*key)
            .and_then(Value::as_array)
            .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(input_key)))
    })
}

fn workflow_param_group(schema_type: &str) -> &'static str {
    match schema_type {
        "asset_path" | "image" | "image_asset" => "image",
        "video" | "video_asset" => "video",
        "audio" | "audio_asset" => "audio",
        "string" | "text" => "text",
        _ => "workflow_param",
    }
}

fn workflow_param_component(schema_type: &str) -> &'static str {
    match schema_type {
        "asset_path" | "image" | "image_asset" => "asset-picker",
        "video" | "video_asset" => "asset-picker",
        "audio" | "audio_asset" => "asset-picker",
        "number" | "integer" => "number-input",
        "boolean" => "switch",
        _ => "input",
    }
}

fn workflow_param_source_options(input_group: &str) -> Vec<&'static str> {
    match input_group {
        "image" | "video" | "audio" => vec!["upload", "select_existing"],
        _ => vec!["generate"],
    }
}

fn plan_kind_for(provider_kind: &str, ability_type: &str) -> String {
    if ability_type.contains("video") || ability_type.contains("i2v") || provider_kind == "video" {
        "video".to_string()
    } else if ability_type.contains("image")
        || provider_kind == "image"
        || provider_kind == "workflow"
    {
        "image".to_string()
    } else {
        provider_kind.to_string()
    }
}

fn reference_constraints(limits: &Value) -> Value {
    json!({
        "maxReferenceImages": read_u64(limits, &["maxReferenceImages", "max_reference_images"]).unwrap_or(1),
        "supportedAspectRatios": read_string_array(limits, &["supportedAspectRatios", "aspectRatios", "supported_aspect_ratios", "aspect_ratios"]).unwrap_or_default(),
        "resolutions": read_string_array(limits, &["resolutions"]).unwrap_or_default(),
    })
}

fn video_constraints(limits: &Value) -> Value {
    json!({
        "durationSeconds": read_value(limits, &["durationSeconds", "duration_seconds", "durationRange", "duration_range"]).unwrap_or_else(|| json!({})),
        "resolutions": read_string_array(limits, &["resolutions"]).unwrap_or_default(),
        "fps": read_number_array(limits, &["fps", "fps_values"]).unwrap_or_default(),
    })
}

fn workflow_provider_id(preset: &WorkflowPresetRecord) -> Option<String> {
    read_string(&preset.config_json, &["providerId", "provider_id"])
        .or_else(|| preset.provider_id.clone())
}

fn ability_disabled_reason(config: &Value, ability_type: &str) -> Option<String> {
    if contains_input_key(
        config,
        &[
            "disabledAbilityTypes",
            "disabled_ability_types",
            "disabledAdvancedAbilities",
            "disabled_advanced_abilities",
        ],
        ability_type,
    ) {
        return Some(format!("ability.disabled.{ability_type}"));
    }
    if ability_type == "video_continuation"
        && read_bool(
            config,
            &[
                "disableVideoContinuation",
                "disable_video_continuation",
                "disableTailFrameContinuation",
                "disable_tail_frame_continuation",
            ],
        )
        .unwrap_or(false)
    {
        return Some("ability.disabled.video_continuation".to_string());
    }
    None
}

fn ability_scoped_value(config: &Value, base_key: &str, ability_type: &str) -> Value {
    for (key, scoped_by_ability) in ability_scoped_keys(base_key) {
        if let Some(value) = config.get(key) {
            if let Some(scoped) = value.get(ability_type) {
                return scoped.clone();
            }
            if !scoped_by_ability {
                return value.clone();
            }
        }
    }
    json!({})
}

fn ability_scoped_keys(base_key: &str) -> Vec<(&'static str, bool)> {
    match base_key {
        "limits" => vec![
            ("abilityLimits", true),
            ("ability_limits", true),
            ("limits", false),
        ],
        "inputRequirements" => vec![
            ("abilityInputRequirements", true),
            ("ability_input_requirements", true),
            ("inputRequirements", false),
            ("input_requirements", false),
        ],
        "paramSchema" => vec![
            ("abilityParamSchema", true),
            ("ability_param_schema", true),
            ("paramSchema", false),
            ("param_schema", false),
        ],
        "defaultParams" => vec![
            ("abilityDefaultParams", true),
            ("ability_default_params", true),
            ("defaultParams", false),
            ("default_params", false),
        ],
        _ => vec![],
    }
}

fn read_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str).map(str::to_string))
}

fn read_bool(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_bool))
}

fn read_value(value: &Value, keys: &[&str]) -> Option<Value> {
    keys.iter().find_map(|key| value.get(*key).cloned())
}

fn read_string_array(value: &Value, keys: &[&str]) -> Option<Vec<String>> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(Value::as_array).map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
    })
}

fn read_number_array(value: &Value, keys: &[&str]) -> Option<Vec<f64>> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
    })
}

fn read_u64(value: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_u64))
}

pub fn import_asset(
    database: &Database,
    workspace_root: &Path,
    request: ImportAssetRequest,
) -> Result<AssetDto, String> {
    validate_asset_kind(&request.kind)?;
    let source_path = PathBuf::from(&request.source_path);
    if !source_path.is_absolute() {
        return Err("source_path must be an absolute path selected by the user.".to_string());
    }
    if !source_path.is_file() {
        return Err("source_path must point to an existing file.".to_string());
    }

    let asset_id = create_id("asset");
    let extension = source_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{}", sanitize_file_segment(value)))
        .unwrap_or_default();
    let raw_display_name = request
        .display_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or_else(|| {
            source_path
                .file_stem()
                .and_then(|value| value.to_str())
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| asset_id.clone());
    let display_name = non_empty_file_segment(&raw_display_name);
    let relative_target = if !extension.is_empty()
        && display_name
            .to_ascii_lowercase()
            .ends_with(&extension.to_ascii_lowercase())
    {
        format!("{}/{}", request.kind, display_name)
    } else {
        format!("{}/{}{}", request.kind, display_name, extension)
    };
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let stored = storage.copy_into_bucket(
        &source_path,
        FileBucket::Asset,
        &relative_target,
        FileAccessPolicy::WriteProject,
    )?;
    let file_metadata = fs::metadata(&source_path).map_err(|error| error.to_string())?;
    let mime_type = request
        .mime_type
        .clone()
        .or_else(|| guess_mime_type(&source_path));
    let metadata = normalize_import_metadata(
        request.metadata.unwrap_or_else(|| json!({})),
        &source_path,
        &stored.relative_path,
        &request.kind,
        mime_type.as_deref(),
        file_metadata.len(),
        &raw_display_name,
    )?;

    AssetRepository::new(database).insert_asset(&NewAssetRecord {
        asset_id,
        kind: request.kind,
        relative_path: stored.relative_path,
        source_kind: "user_import".to_string(),
        mime_type,
        size_bytes: Some(file_metadata.len() as i64),
        checksum: Some(simple_checksum(&source_path)?),
        is_builtin: false,
        metadata,
    })
}

pub fn list_assets(
    database: &Database,
    request: ListAssetsRequest,
) -> Result<Vec<AssetDto>, String> {
    if let Some(kind) = request.kind.as_deref() {
        validate_asset_kind(kind)?;
    }
    AssetRepository::new(database).list_assets(
        request.kind.as_deref(),
        request.include_deleted.unwrap_or(false),
    )
}

pub fn delete_asset(
    database: &Database,
    workspace_root: &Path,
    request: DeleteAssetRequest,
) -> Result<AssetDto, String> {
    let repository = AssetRepository::new(database);
    let asset = repository
        .get_asset(&request.asset_id)?
        .ok_or_else(|| format!("Asset not found: {}", request.asset_id))?;
    if asset.is_builtin {
        return Err("builtin assets cannot be deleted; hide them instead.".to_string());
    }

    let reference_count = repository.count_references(&request.asset_id)?;
    if reference_count > 0 {
        let references = repository.list_references(&request.asset_id)?;
        return Err(format!(
            "asset is still referenced and cannot be deleted: {}",
            format_reference_summary(&references)
        ));
    }

    let deleted = repository.mark_deleted(&request.asset_id)?;
    if request.physical.unwrap_or(false) {
        remove_asset_file(workspace_root, &asset.relative_path)?;
    }

    Ok(deleted)
}

pub fn create_asset_reference(
    database: &Database,
    request: CreateAssetReferenceRequest,
) -> Result<AssetReferenceDto, String> {
    validate_owner_kind(&request.owner_kind)?;
    validate_usage_kind(&request.usage_kind)?;
    if AssetRepository::new(database)
        .get_asset(&request.asset_id)?
        .is_none()
    {
        return Err(format!("Asset not found: {}", request.asset_id));
    }

    AssetRepository::new(database).create_reference(&NewAssetReferenceRecord {
        reference_id: create_id("asset_ref"),
        asset_id: request.asset_id,
        owner_kind: request.owner_kind,
        owner_id: request.owner_id,
        usage_kind: request.usage_kind,
    })
}

pub fn list_asset_references(
    database: &Database,
    asset_id: String,
) -> Result<Vec<AssetReferenceDto>, String> {
    AssetRepository::new(database).list_references(&asset_id)
}

pub fn delete_asset_reference(
    database: &Database,
    request: DeleteAssetReferenceRequest,
) -> Result<AssetReferenceDto, String> {
    AssetRepository::new(database).delete_reference(&request.reference_id)
}

pub fn collect_project_asset_paths(
    database: &Database,
    project_id: String,
) -> Result<Vec<String>, String> {
    AssetRepository::new(database).collect_project_asset_paths(&project_id)
}

pub fn read_asset_preview(
    database: &Database,
    workspace_root: &Path,
    request: AssetPreviewRequest,
) -> Result<AssetPreviewDto, String> {
    let asset = AssetRepository::new(database)
        .get_asset(&request.asset_id)?
        .ok_or_else(|| format!("Asset not found: {}", request.asset_id))?;
    if asset.lifecycle == "deleted" {
        return Err("deleted asset cannot be previewed.".to_string());
    }

    let media_kind = asset_media_kind(&asset);
    match media_kind.as_str() {
        "image" => read_image_asset_preview(workspace_root, &asset, &media_kind),
        "video" => render_video_thumbnail_preview(workspace_root, &asset, &media_kind),
        _ => Err(format!(
            "asset preview is not supported for {media_kind} media."
        )),
    }
}

fn remove_asset_file(workspace_root: &Path, relative_path: &str) -> Result<(), String> {
    let prefix = "assets/";
    if !relative_path.starts_with(prefix) {
        return Err("asset relative_path must point to the assets bucket.".to_string());
    }
    let storage = StorageService::new(workspace_root);
    let asset_relative_path = &relative_path[prefix.len()..];
    let absolute_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Asset, asset_relative_path)?;
    fs::remove_file(absolute_path).map_err(|error| error.to_string())
}

fn read_image_asset_preview(
    workspace_root: &Path,
    asset: &AssetDto,
    media_kind: &str,
) -> Result<AssetPreviewDto, String> {
    let asset_relative_path = asset_bucket_relative_path(&asset.relative_path)?;
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let absolute_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Asset, asset_relative_path)?;
    let metadata = fs::metadata(&absolute_path).map_err(|error| error.to_string())?;
    if metadata.len() > ASSET_PREVIEW_MAX_BYTES {
        return Err("asset preview file is too large.".to_string());
    }
    let bytes = fs::read(&absolute_path).map_err(|error| error.to_string())?;
    Ok(AssetPreviewDto {
        asset_id: asset.asset_id.clone(),
        relative_path: asset.relative_path.clone(),
        media_kind: media_kind.to_string(),
        mime_type: asset
            .mime_type
            .clone()
            .or_else(|| guess_mime_type(Path::new(&asset.relative_path)))
            .unwrap_or_else(|| "image/png".to_string()),
        preview_kind: "image".to_string(),
        bytes,
    })
}

fn render_video_thumbnail_preview(
    workspace_root: &Path,
    asset: &AssetDto,
    media_kind: &str,
) -> Result<AssetPreviewDto, String> {
    let asset_relative_path = asset_bucket_relative_path(&asset.relative_path)?;
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let ffmpeg_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Sidecar, FFMPEG_BINARY)
        .map_err(|error| format!("ffmpeg.not_found: {error}"))?;
    let input_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Asset, asset_relative_path)?;
    let preview_relative_path = format!(
        "asset_previews/{}_{}.jpg",
        sanitize_file_segment(&asset.asset_id),
        create_id("frame")
    );
    let output_path = storage
        .resolver()
        .resolve_bucket_path_for_write(FileBucket::Temp, &preview_relative_path)?;
    let output = Command::new(&ffmpeg_path)
        .args(["-y", "-ss", "0", "-i"])
        .arg(&input_path)
        .args(["-frames:v", "1", "-q:v", "3"])
        .arg(&output_path)
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        let _ = fs::remove_file(&output_path);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "ffmpeg.preview_failed: {}",
            sanitize_preview_error(&stderr, workspace_root, &input_path, &output_path)
        ));
    }
    let metadata = fs::metadata(&output_path).map_err(|error| error.to_string())?;
    if metadata.len() > ASSET_PREVIEW_MAX_BYTES {
        let _ = fs::remove_file(&output_path);
        return Err("asset preview file is too large.".to_string());
    }
    let bytes = fs::read(&output_path).map_err(|error| error.to_string())?;
    let _ = fs::remove_file(&output_path);

    Ok(AssetPreviewDto {
        asset_id: asset.asset_id.clone(),
        relative_path: asset.relative_path.clone(),
        media_kind: media_kind.to_string(),
        mime_type: "image/jpeg".to_string(),
        preview_kind: "video_frame".to_string(),
        bytes,
    })
}

fn asset_bucket_relative_path(relative_path: &str) -> Result<&str, String> {
    let prefix = "assets/";
    relative_path
        .strip_prefix(prefix)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "asset relative_path must point to the assets bucket.".to_string())
}

fn validate_asset_kind(kind: &str) -> Result<(), String> {
    validate_one_of(
        "asset kind",
        kind,
        &[
            "bgm",
            "font",
            "reference_image",
            "character_reference",
            "character_reference_image",
            "style_reference",
            "style_reference_image",
            "scene_reference",
            "scene_reference_image",
            "pose_reference",
            "depth_reference",
            "mask_reference",
            "source_video",
            "source_audio",
            "generated_audio",
            "template_resource",
            "generated_image_candidate",
            "generated_video_segment",
            "final_export",
            "task_artifact",
            "user_image",
            "user_video",
            "cover_source",
            "source_material",
            "generated_output",
        ],
    )
}

fn validate_owner_kind(owner_kind: &str) -> Result<(), String> {
    validate_one_of(
        "owner kind",
        owner_kind,
        &[
            "project",
            "storyboard_item",
            "image_candidate",
            "video_segment",
            "character_bible",
            "style_bible",
            "location_bible",
            "video_pack",
            "template",
            "task",
            "composition_task",
            "export",
        ],
    )
}

fn validate_usage_kind(usage_kind: &str) -> Result<(), String> {
    validate_one_of(
        "usage kind",
        usage_kind,
        &[
            "bgm",
            "cover",
            "reference_image",
            "character_reference",
            "style_reference",
            "location_reference",
            "pose_reference",
            "depth_reference",
            "mask_reference",
            "source_material",
            "selected_image",
            "selected_video",
            "generated_image",
            "generated_audio",
            "generated_video",
            "font",
            "template_resource",
            "final_export",
            "task_artifact",
        ],
    )
}

fn format_reference_summary(references: &[AssetReferenceDto]) -> String {
    references
        .iter()
        .map(|reference| {
            format!(
                "{}/{}/{}",
                reference.owner_kind, reference.owner_id, reference.usage_kind
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn validate_one_of(name: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        return Ok(());
    }

    Err(format!("{name} has unsupported value."))
}

fn normalize_import_metadata(
    metadata: Value,
    source_path: &Path,
    relative_path: &str,
    kind: &str,
    mime_type: Option<&str>,
    size_bytes: u64,
    display_name: &str,
) -> Result<Value, String> {
    let mut metadata = if metadata.is_object() {
        metadata
    } else {
        json!({})
    };
    let object = metadata
        .as_object_mut()
        .ok_or_else(|| "asset metadata must be a JSON object.".to_string())?;
    object.remove("sourcePath");
    object.remove("source_path");
    object.remove("absolutePath");
    object.remove("absolute_path");
    object.remove("path");
    object.insert(
        "displayName".to_string(),
        Value::String(display_name.to_string()),
    );
    object.insert(
        "fileName".to_string(),
        Value::String(source_file_name(source_path)),
    );
    object.insert(
        "extension".to_string(),
        Value::String(source_extension(source_path).unwrap_or_default()),
    );
    object.insert("assetKind".to_string(), Value::String(kind.to_string()));
    object.insert(
        "mediaKind".to_string(),
        Value::String(resolve_media_kind(kind, mime_type, source_path)),
    );
    object.insert(
        "sourceType".to_string(),
        Value::String("user_import".to_string()),
    );
    object.insert(
        "relativePath".to_string(),
        Value::String(relative_path.to_string()),
    );
    object.insert(
        "sizeBytes".to_string(),
        Value::Number(serde_json::Number::from(size_bytes)),
    );
    if let Some(mime_type) = mime_type {
        object.insert("mimeType".to_string(), Value::String(mime_type.to_string()));
    }

    Ok(metadata)
}

fn asset_media_kind(asset: &AssetDto) -> String {
    asset
        .metadata
        .get("mediaKind")
        .or_else(|| asset.metadata.get("media_kind"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            resolve_media_kind(
                &asset.kind,
                asset.mime_type.as_deref(),
                Path::new(&asset.relative_path),
            )
        })
}

fn resolve_media_kind(kind: &str, mime_type: Option<&str>, path: &Path) -> String {
    if let Some(mime_type) = mime_type {
        let mime_type = mime_type.to_ascii_lowercase();
        if mime_type.starts_with("image/") {
            return "image".to_string();
        }
        if mime_type.starts_with("video/") {
            return "video".to_string();
        }
        if mime_type.starts_with("audio/") {
            return "audio".to_string();
        }
        if mime_type.starts_with("font/") {
            return "font".to_string();
        }
    }

    match kind {
        "source_video" | "user_video" | "generated_video_segment" => "video".to_string(),
        "source_audio" | "bgm" | "generated_audio" => "audio".to_string(),
        "font" => "font".to_string(),
        "template_resource" => "template".to_string(),
        _ => match source_extension(path).as_deref() {
            Some("jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp") => "image".to_string(),
            Some("mp4" | "mov" | "webm" | "mkv" | "avi") => "video".to_string(),
            Some("mp3" | "wav" | "m4a" | "aac" | "flac" | "ogg") => "audio".to_string(),
            Some("ttf" | "otf" | "woff" | "woff2") => "font".to_string(),
            _ => "unknown".to_string(),
        },
    }
}

fn guess_mime_type(path: &Path) -> Option<String> {
    let mime_type = match source_extension(path).as_deref()? {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "ogg" => "audio/ogg",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "json" => "application/json",
        "html" => "text/html",
        "css" => "text/css",
        "srt" => "application/x-subrip",
        "vtt" => "text/vtt",
        _ => return None,
    };
    Some(mime_type.to_string())
}

fn source_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "asset".to_string())
}

fn source_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim_start_matches('.').to_ascii_lowercase())
}

fn sanitize_file_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    sanitized.trim_matches('.').trim_matches('_').to_string()
}

fn non_empty_file_segment(value: &str) -> String {
    let sanitized = sanitize_file_segment(value);
    if sanitized.is_empty() {
        "asset".to_string()
    } else {
        sanitized
    }
}

fn sanitize_preview_error(
    message: &str,
    workspace_root: &Path,
    input_path: &Path,
    output_path: &Path,
) -> String {
    let mut sanitized = message.trim().to_string();
    for (from, to) in [
        (
            workspace_root.display().to_string(),
            "<workspace>".to_string(),
        ),
        (input_path.display().to_string(), "<asset>".to_string()),
        (output_path.display().to_string(), "<preview>".to_string()),
    ] {
        sanitized = sanitized.replace(&from, &to);
        sanitized = sanitized.replace(&from.replace('\\', "/"), &to);
        sanitized = sanitized.replace(&from.replace('/', "\\"), &to);
    }
    let lines = sanitized.lines().rev().take(40).collect::<Vec<_>>();
    let mut limited = lines.into_iter().rev().collect::<Vec<_>>().join("\n");
    if limited.len() > 4096 {
        limited = limited[limited.len() - 4096..].to_string();
    }
    if limited.is_empty() {
        "failed to render video thumbnail.".to_string()
    } else {
        limited
    }
}

fn simple_checksum(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
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
    use super::{
        collect_project_asset_paths, create_asset_reference, delete_asset, delete_asset_reference,
        import_asset, list_assets, list_executable_media_options, read_asset_preview,
    };
    use crate::db::asset_repository::{AssetRepository, NewAssetRecord};
    use crate::db::project_repository::ProjectRepository;
    use crate::db::provider_repository::{
        ProviderModelRecord, ProviderRecord, ProviderRepository, WorkflowPresetRecord,
    };
    use crate::db::Database;
    use crate::domain::media::{
        AssetPreviewRequest, CreateAssetReferenceRequest, DeleteAssetReferenceRequest,
        DeleteAssetRequest, ImportAssetRequest, ListAssetsRequest,
    };
    use crate::domain::project::CreateProjectRequest;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn imports_asset_into_workspace_and_stores_relative_path() {
        let root = test_root("import_asset");
        let source_root = test_root("import_asset_source");
        fs::create_dir_all(&source_root).expect("source dir should exist");
        let source_path = source_root.join("photo.png");
        fs::write(&source_path, "image-bytes").expect("source file should write");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");

        let asset = import_asset(
            &database,
            &workspace_root,
            ImportAssetRequest {
                source_path: source_path.to_string_lossy().to_string(),
                kind: "user_image".to_string(),
                display_name: Some("cover".to_string()),
                mime_type: Some("image/png".to_string()),
                metadata: Some(json!({ "width": 100 })),
            },
        )
        .expect("asset should import");

        assert_eq!(asset.kind, "user_image");
        assert!(asset.relative_path.starts_with("assets/user_image/"));
        assert!(!PathBuf::from(&asset.relative_path).is_absolute());
        assert!(workspace_root.join(&asset.relative_path).is_file());
        assert_eq!(asset.mime_type.as_deref(), Some("image/png"));
        assert_eq!(
            asset
                .metadata
                .get("mediaKind")
                .and_then(|value| value.as_str()),
            Some("image")
        );
        assert_eq!(
            asset
                .metadata
                .get("sizeBytes")
                .and_then(|value| value.as_u64()),
            Some("image-bytes".len() as u64)
        );
        assert!(!asset
            .metadata
            .to_string()
            .contains(&source_root.display().to_string()));

        cleanup(root);
        cleanup(source_root);
    }

    #[test]
    fn image_asset_preview_reads_only_assets_bucket() {
        let root = test_root("image_preview");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        fs::create_dir_all(workspace_root.join("assets/user_image"))
            .expect("asset dir should exist");
        fs::write(workspace_root.join("assets/user_image/preview.png"), b"png")
            .expect("preview should write");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_preview".to_string(),
                kind: "user_image".to_string(),
                relative_path: "assets/user_image/preview.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(3),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "mediaKind": "image" }),
            })
            .expect("asset should save");

        let preview = read_asset_preview(
            &database,
            &workspace_root,
            AssetPreviewRequest {
                asset_id: asset.asset_id,
            },
        )
        .expect("image asset should preview");

        assert_eq!(preview.relative_path, "assets/user_image/preview.png");
        assert_eq!(preview.media_kind, "image");
        assert_eq!(preview.mime_type, "image/png");
        assert_eq!(preview.bytes, b"png");

        let outside_asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_preview_outside".to_string(),
                kind: "user_image".to_string(),
                relative_path: "outputs/not_assets/preview.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(3),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "mediaKind": "image" }),
            })
            .expect("asset should save");
        let error = read_asset_preview(
            &database,
            &workspace_root,
            AssetPreviewRequest {
                asset_id: outside_asset.asset_id,
            },
        )
        .expect_err("preview outside assets bucket should fail");
        assert!(error.contains("assets bucket"));

        cleanup(root);
    }

    #[test]
    fn referenced_asset_cannot_be_physically_deleted() {
        let root = test_root("delete_referenced");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_ref_test".to_string(),
                kind: "user_image".to_string(),
                relative_path: "assets/user_image/ref.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(10),
                checksum: None,
                is_builtin: false,
                metadata: json!({}),
            })
            .expect("asset should save");

        create_asset_reference(
            &database,
            CreateAssetReferenceRequest {
                asset_id: asset.asset_id.clone(),
                owner_kind: "project".to_string(),
                owner_id: "project_a".to_string(),
                usage_kind: "source_material".to_string(),
            },
        )
        .expect("reference should save");

        assert!(delete_asset(
            &database,
            &workspace_root,
            DeleteAssetRequest {
                asset_id: asset.asset_id,
                physical: Some(true),
            },
        )
        .is_err());

        cleanup(root);
    }

    #[test]
    fn deleting_reference_unblocks_asset_deletion() {
        let root = test_root("delete_reference_unblocks");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        fs::create_dir_all(workspace_root.join("assets/user_image"))
            .expect("asset dir should exist");
        fs::write(
            workspace_root.join("assets/user_image/ref.png"),
            "image-bytes",
        )
        .expect("asset file should write");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_reference_unblock".to_string(),
                kind: "user_image".to_string(),
                relative_path: "assets/user_image/ref.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(11),
                checksum: None,
                is_builtin: false,
                metadata: json!({}),
            })
            .expect("asset should save");

        let reference = create_asset_reference(
            &database,
            CreateAssetReferenceRequest {
                asset_id: asset.asset_id.clone(),
                owner_kind: "project".to_string(),
                owner_id: "project_reference_unblock".to_string(),
                usage_kind: "source_material".to_string(),
            },
        )
        .expect("reference should save");

        delete_asset_reference(
            &database,
            DeleteAssetReferenceRequest {
                reference_id: reference.reference_id,
            },
        )
        .expect("reference should delete");

        let deleted = delete_asset(
            &database,
            &workspace_root,
            DeleteAssetRequest {
                asset_id: asset.asset_id,
                physical: Some(true),
            },
        )
        .expect("asset should delete after reference is removed");

        assert_eq!(deleted.lifecycle, "deleted");
        assert!(!workspace_root.join("assets/user_image/ref.png").exists());

        cleanup(root);
    }

    #[test]
    fn physical_delete_rejects_non_asset_bucket_paths() {
        let root = test_root("delete_outside_assets_bucket");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let workspace_root = root.join("workspace");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_outside_bucket".to_string(),
                kind: "user_image".to_string(),
                relative_path: "outputs/project/final.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(10),
                checksum: None,
                is_builtin: false,
                metadata: json!({}),
            })
            .expect("asset should save");

        let error = delete_asset(
            &database,
            &workspace_root,
            DeleteAssetRequest {
                asset_id: asset.asset_id,
                physical: Some(true),
            },
        )
        .expect_err("physical delete outside assets bucket should fail");

        assert!(error.contains("assets bucket"));

        cleanup(root);
    }

    #[test]
    fn builtin_asset_cannot_be_deleted() {
        let root = test_root("builtin_delete");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_builtin".to_string(),
                kind: "font".to_string(),
                relative_path: "assets/font/default.ttf".to_string(),
                source_kind: "builtin".to_string(),
                mime_type: Some("font/ttf".to_string()),
                size_bytes: Some(10),
                checksum: None,
                is_builtin: true,
                metadata: json!({}),
            })
            .expect("asset should save");

        assert!(delete_asset(
            &database,
            &root.join("workspace"),
            DeleteAssetRequest {
                asset_id: asset.asset_id,
                physical: Some(false),
            },
        )
        .is_err());

        cleanup(root);
    }

    #[test]
    fn lists_assets_and_collects_project_asset_paths() {
        let root = test_root("collect_project_assets");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let project = ProjectRepository::new(&database)
            .create_with_id(
                "project_collect".to_string(),
                CreateProjectRequest {
                    title: "收集资产".to_string(),
                    workflow_type: "image_to_video".to_string(),
                    input_type: "topic".to_string(),
                    topic: Some("主题".to_string()),
                    source_text: None,
                    source_text_path: None,
                    content_language: "zh-CN".to_string(),
                    tone: None,
                    aspect_ratio: "9:16".to_string(),
                    target_scene_count: 3,
                    segment_duration_seconds: 4.0,
                    style_prompt: None,
                    active_pack_id: None,
                    rule_refs: None,
                    executable_refs: None,
                    input_process_mode: "generate".to_string(),
                    input_options: Some(json!({})),
                },
            )
            .expect("project should save");
        let asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_collect".to_string(),
                kind: "source_material".to_string(),
                relative_path: "assets/source_material/a.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(10),
                checksum: None,
                is_builtin: false,
                metadata: json!({}),
            })
            .expect("asset should save");

        create_asset_reference(
            &database,
            CreateAssetReferenceRequest {
                asset_id: asset.asset_id.clone(),
                owner_kind: "project".to_string(),
                owner_id: project.project.project_id,
                usage_kind: "source_material".to_string(),
            },
        )
        .expect("reference should save");
        let pack_asset = AssetRepository::new(&database)
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_pack_collect".to_string(),
                kind: "character_reference".to_string(),
                relative_path: "assets/character_reference/hero.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(12),
                checksum: None,
                is_builtin: false,
                metadata: json!({}),
            })
            .expect("pack asset should save");
        create_asset_reference(
            &database,
            CreateAssetReferenceRequest {
                asset_id: pack_asset.asset_id,
                owner_kind: "video_pack".to_string(),
                owner_id: "project_collect:pack_default".to_string(),
                usage_kind: "character_reference".to_string(),
            },
        )
        .expect("video pack reference should save");

        let assets = list_assets(
            &database,
            ListAssetsRequest {
                kind: Some("source_material".to_string()),
                include_deleted: None,
            },
        )
        .expect("assets should list");
        assert_eq!(assets.len(), 1);

        let paths =
            collect_project_asset_paths(&database, "project_collect".to_string()).expect("paths");
        assert_eq!(
            paths,
            vec![
                "assets/character_reference/hero.png".to_string(),
                "assets/source_material/a.png".to_string(),
            ],
        );

        cleanup(root);
    }

    #[test]
    fn executable_media_options_merge_models_and_workflow_presets() {
        let root = test_root("executable_options_merge");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let repository = ProviderRepository::new(&database);
        seed_provider(
            &repository,
            "provider_image",
            "image",
            "dummy",
            true,
            "ready",
        );
        seed_provider(
            &repository,
            "provider_video_disabled",
            "video",
            "dummy",
            false,
            "ready",
        );
        seed_provider(
            &repository,
            "provider_comfyui",
            "workflow",
            "comfyui",
            true,
            "ready",
        );
        seed_provider_model(
            &repository,
            "model_image",
            "provider_image",
            "text_to_image",
            true,
            json!({
                "providerKind": "image",
                "vendor": "dummy",
                "modelName": "dummy-image-v1",
                "abilityTypes": ["text_to_image"],
                "inputRequirements": {
                    "requiredInputs": ["character_front_view"]
                },
                "limits": {
                    "supportedAspectRatios": ["9:16"],
                    "resolutions": ["720p"],
                    "maxReferenceImages": 1
                },
                "status": "ready"
            }),
        );
        seed_provider_model(
            &repository,
            "model_video_disabled_provider",
            "provider_video_disabled",
            "image_to_video",
            true,
            json!({
                "providerKind": "video",
                "vendor": "dummy",
                "modelName": "dummy-video-v1",
                "abilityTypes": ["image_to_video"],
                "limits": {
                    "durationSeconds": { "min": 3, "max": 8, "integer": true }
                },
                "status": "ready"
            }),
        );
        seed_workflow_preset(
            &repository,
            "workflow_comfyui",
            "provider_comfyui",
            true,
            json!({
                "providerId": "provider_comfyui",
                "vendor": "comfyui",
                "workflowKey": "comfyui/workflow_api.json",
                "workflowVersion": "1.0.0",
                "abilityTypes": ["image_to_video"],
                "paramSchema": {
                    "prompt": { "type": "string", "required": true },
                    "pose": { "type": "image", "required": true },
                    "strength": { "type": "number", "required": false }
                },
                "nodeMap": { "prompt": "12.inputs.text", "pose": "9.inputs.image" },
                "outputMap": { "video": "99.outputs.video" },
                "defaultParams": { "strength": 0.7 },
                "limits": { "supportedAspectRatios": ["9:16"] },
                "status": "ready"
            }),
        );

        let options = list_executable_media_options(&database).expect("options should list");
        assert_eq!(options.len(), 3);

        let image = options
            .iter()
            .find(|option| option.source_id == "model_image")
            .expect("image option should exist");
        assert_eq!(image.source_type, "provider_model");
        assert_eq!(image.provider_model_id.as_deref(), Some("model_image"));
        assert_eq!(image.workflow_preset_id, None);
        assert!(image.enabled);
        assert!(image
            .input_plan
            .items
            .iter()
            .any(|item| item.input_key == "prompt" && item.requirement == "required"));
        assert!(image.input_plan.items.iter().any(|item| {
            item.input_key == "character_front_view" && item.requirement == "required"
        }));
        assert!(image.input_plan.items.iter().any(|item| {
            item.input_key == "character_side_view" && item.requirement == "unused"
        }));

        let disabled = options
            .iter()
            .find(|option| option.source_id == "model_video_disabled_provider")
            .expect("disabled provider option should exist");
        assert!(!disabled.enabled);
        assert_eq!(
            disabled.disabled_reason.as_deref(),
            Some("provider.disabled")
        );

        let workflow = options
            .iter()
            .find(|option| option.source_id == "workflow_comfyui")
            .expect("workflow option should exist");
        assert_eq!(workflow.source_type, "workflow_preset");
        assert_eq!(workflow.provider_model_id, None);
        assert_eq!(
            workflow.workflow_preset_id.as_deref(),
            Some("workflow_comfyui")
        );
        assert!(workflow.input_plan.items.iter().any(|item| {
            item.input_key == "workflowParams.pose" && item.requirement == "required"
        }));
        assert!(workflow.input_plan.items.iter().any(|item| {
            item.input_key == "workflowParams.strength" && item.requirement == "optional"
        }));

        cleanup(root);
    }

    #[test]
    fn advanced_video_abilities_have_separate_input_plans_and_disable_flags() {
        let root = test_root("advanced_video_abilities");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let repository = ProviderRepository::new(&database);
        seed_provider(
            &repository,
            "provider_video",
            "video",
            "dummy",
            true,
            "ready",
        );
        seed_provider_model(
            &repository,
            "model_video_advanced",
            "provider_video",
            "image_to_video",
            true,
            json!({
                "providerKind": "video",
                "vendor": "dummy",
                "modelName": "dummy-video-advanced",
                "abilityTypes": [
                    "text_to_video",
                    "image_to_video",
                    "start_end_frame_i2v",
                    "reference_to_video",
                    "video_continuation"
                ],
                "abilityLimits": {
                    "text_to_video": { "durationSeconds": { "min": 2, "max": 8, "integer": true } },
                    "image_to_video": { "durationSeconds": { "min": 2, "max": 6, "integer": true }, "maxReferenceImages": 1 },
                    "start_end_frame_i2v": { "durationSeconds": { "min": 2, "max": 5, "integer": true }, "maxReferenceImages": 2 },
                    "reference_to_video": { "durationSeconds": { "min": 2, "max": 6, "integer": true }, "maxReferenceImages": 4 },
                    "video_continuation": { "durationSeconds": { "min": 2, "max": 4, "integer": true } }
                },
                "abilityInputRequirements": {
                    "reference_to_video": { "optionalInputs": ["referenceAsset"] },
                    "video_continuation": { "requiredInputs": ["sourceVideo"], "optionalInputs": ["tailFrame"] }
                },
                "disabledAdvancedAbilities": ["video_continuation"],
                "status": "ready"
            }),
        );

        let options = list_executable_media_options(&database).expect("options should list");
        let advanced = options
            .iter()
            .filter(|option| option.source_id == "model_video_advanced")
            .collect::<Vec<_>>();
        assert_eq!(advanced.len(), 5);

        let text_to_video = advanced
            .iter()
            .find(|option| option.capability == "text_to_video")
            .expect("text_to_video option");
        assert!(text_to_video.enabled);
        assert!(text_to_video
            .input_plan
            .items
            .iter()
            .any(|item| item.input_key == "videoPrompt" && item.requirement == "required"));
        assert!(!text_to_video
            .input_plan
            .items
            .iter()
            .any(|item| item.input_key == "startFrame"));

        let start_end = advanced
            .iter()
            .find(|option| option.capability == "start_end_frame_i2v")
            .expect("start_end option");
        assert!(start_end
            .input_plan
            .items
            .iter()
            .any(|item| item.input_key == "startFrame" && item.requirement == "required"));
        assert!(start_end
            .input_plan
            .items
            .iter()
            .any(|item| item.input_key == "endFrame" && item.requirement == "required"));

        let reference = advanced
            .iter()
            .find(|option| option.capability == "reference_to_video")
            .expect("reference option");
        assert!(reference
            .input_plan
            .items
            .iter()
            .any(|item| item.input_key == "referenceAsset" && item.requirement == "optional"));

        let continuation = advanced
            .iter()
            .find(|option| option.capability == "video_continuation")
            .expect("continuation option");
        assert!(!continuation.enabled);
        assert_eq!(
            continuation.disabled_reason.as_deref(),
            Some("ability.disabled.video_continuation")
        );
        assert!(continuation
            .input_plan
            .items
            .iter()
            .any(|item| item.input_key == "sourceVideo" && item.requirement == "required"));
        assert!(continuation
            .input_plan
            .items
            .iter()
            .any(|item| item.input_key == "tailFrame" && item.requirement == "optional"));

        cleanup(root);
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-asset-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }

    fn seed_provider(
        repository: &ProviderRepository<'_>,
        provider_id: &str,
        kind: &str,
        vendor: &str,
        enabled: bool,
        status: &str,
    ) {
        repository
            .upsert_provider(&ProviderRecord {
                provider_id: provider_id.to_string(),
                vendor: vendor.to_string(),
                kind: kind.to_string(),
                display_name: format!("{vendor} {kind}"),
                auth_type: "none".to_string(),
                key_alias: None,
                base_url: None,
                status: status.to_string(),
                enabled,
                config_json: json!({}),
            })
            .expect("provider should save");
    }

    fn seed_provider_model(
        repository: &ProviderRepository<'_>,
        model_id: &str,
        provider_id: &str,
        capability: &str,
        enabled: bool,
        config_json: serde_json::Value,
    ) {
        repository
            .upsert_provider_model(&ProviderModelRecord {
                model_id: model_id.to_string(),
                provider_id: provider_id.to_string(),
                provider_model_id: format!("dummy/{model_id}"),
                display_name: format!("Model {model_id}"),
                capability: capability.to_string(),
                config_json,
                enabled,
            })
            .expect("provider model should save");
    }

    fn seed_workflow_preset(
        repository: &ProviderRepository<'_>,
        preset_id: &str,
        provider_id: &str,
        enabled: bool,
        config_json: serde_json::Value,
    ) {
        repository
            .upsert_workflow_preset(&WorkflowPresetRecord {
                preset_id: preset_id.to_string(),
                provider_id: Some(provider_id.to_string()),
                model_id: None,
                name: format!("Workflow {preset_id}"),
                kind: "comfyui".to_string(),
                capability: "image_to_video".to_string(),
                config_json,
                enabled,
            })
            .expect("workflow preset should save");
    }
}
