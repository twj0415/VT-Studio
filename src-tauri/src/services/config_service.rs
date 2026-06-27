use crate::db::config_repository::ConfigRepository;
use crate::db::provider_repository::{
    ProviderModelRecord, ProviderRecord, ProviderRepository, WorkflowPresetRecord,
};
use crate::db::Database;
use crate::domain::config::AppConfigDto;
use crate::domain::config::{
    DeleteProviderConfigRequest, ListProviderConfigsRequest, ProviderConfigDto,
};
use crate::domain::config::{
    DeleteProviderModelRequest, ListProviderModelsRequest, ProviderModelDto,
};
use crate::domain::config::{
    DeleteWorkflowPresetRequest, ListWorkflowPresetsRequest, WorkflowPresetDto,
};
use crate::security::secret_guard::reject_json_secrets;
use serde_json::{json, Value};

pub fn get_app_config(database: &Database) -> Result<AppConfigDto, String> {
    let repository = ConfigRepository::new(database);
    repository.ensure_defaults()?;
    let value = repository
        .get_config("app")?
        .ok_or_else(|| "config.missing: app config is missing.".to_string())?;
    app_config_from_value(value)
}

pub fn update_app_config(
    database: &Database,
    config: AppConfigDto,
) -> Result<AppConfigDto, String> {
    validate_app_config(&config)?;
    ConfigRepository::new(database).upsert_config(
        "app",
        &json!({
            "app_locale": config.app_locale,
            "theme_preset": config.theme_preset,
            "layout_density": config.layout_density
        }),
        1,
    )?;
    Ok(config)
}

pub fn list_provider_configs(
    database: &Database,
    request: ListProviderConfigsRequest,
) -> Result<Vec<ProviderConfigDto>, String> {
    let providers = ProviderRepository::new(database).list_providers()?;
    let mut providers = providers
        .into_iter()
        .map(provider_record_to_dto)
        .collect::<Vec<_>>();

    if let Some(provider_kind) = request.provider_kind.as_deref() {
        providers.retain(|provider| provider.provider_kind == provider_kind);
    }

    Ok(providers)
}

pub fn upsert_provider_config(
    database: &Database,
    config: ProviderConfigDto,
) -> Result<ProviderConfigDto, String> {
    validate_provider_config(&config)?;
    reject_json_secrets(&config.config)?;
    ProviderRepository::new(database).upsert_provider(&ProviderRecord {
        provider_id: config.provider_id.clone(),
        vendor: config.vendor.clone(),
        kind: config.provider_kind.clone(),
        display_name: config.display_name.clone(),
        auth_type: config.auth_type.clone(),
        key_alias: config.key_alias.clone(),
        base_url: config.base_url.clone(),
        status: config.status.clone(),
        enabled: config.is_enabled,
        config_json: config.config.clone(),
    })?;

    Ok(config)
}

pub fn delete_provider_config(
    database: &Database,
    request: DeleteProviderConfigRequest,
) -> Result<ProviderConfigDto, String> {
    validate_identifier("provider_id", &request.provider_id)?;
    let repository = ProviderRepository::new(database);
    let provider = repository
        .list_providers()?
        .into_iter()
        .find(|provider| provider.provider_id == request.provider_id)
        .ok_or_else(|| "provider.not_found: provider config was not found.".to_string())?;

    repository.delete_provider(&request.provider_id)?;
    Ok(provider_record_to_dto(provider))
}

pub fn list_provider_models(
    database: &Database,
    request: ListProviderModelsRequest,
) -> Result<Vec<ProviderModelDto>, String> {
    let repository = ProviderRepository::new(database);
    let providers = repository.list_providers()?;
    let mut models = repository
        .list_provider_models(request.provider_id.as_deref())?
        .into_iter()
        .filter_map(|record| {
            providers
                .iter()
                .find(|provider| provider.provider_id == record.provider_id)
                .map(|provider| provider_model_record_to_dto(record, provider))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(provider_kind) = request.provider_kind.as_deref() {
        models.retain(|model| model.provider_kind == provider_kind);
    }

    Ok(models)
}

pub fn upsert_provider_model(
    database: &Database,
    model: ProviderModelDto,
) -> Result<ProviderModelDto, String> {
    let repository = ProviderRepository::new(database);
    let provider = repository
        .list_providers()?
        .into_iter()
        .find(|provider| provider.provider_id == model.provider_id)
        .ok_or_else(|| "provider.not_found: provider config was not found.".to_string())?;

    validate_provider_model(&model, &provider)?;
    let record = provider_model_dto_to_record(&model, &provider);
    repository.upsert_provider_model(&record)?;
    provider_model_record_to_dto(record, &provider)
}

pub fn delete_provider_model(
    database: &Database,
    request: DeleteProviderModelRequest,
) -> Result<ProviderModelDto, String> {
    validate_identifier("model_id", &request.model_id)?;
    let repository = ProviderRepository::new(database);
    let model = repository
        .get_provider_model(&request.model_id)?
        .ok_or_else(|| "provider.model_not_found: provider model was not found.".to_string())?;
    let provider = repository
        .list_providers()?
        .into_iter()
        .find(|provider| provider.provider_id == model.provider_id)
        .ok_or_else(|| "provider.not_found: provider config was not found.".to_string())?;
    repository.delete_provider_model(&request.model_id)?;
    provider_model_record_to_dto(model, &provider)
}

pub fn list_workflow_presets(
    database: &Database,
    request: ListWorkflowPresetsRequest,
) -> Result<Vec<WorkflowPresetDto>, String> {
    let repository = ProviderRepository::new(database);
    let providers = repository.list_providers()?;
    let mut presets = repository
        .list_workflow_presets_by_provider(request.provider_id.as_deref())?
        .into_iter()
        .map(|record| {
            let provider = record.provider_id.as_deref().and_then(|provider_id| {
                providers
                    .iter()
                    .find(|provider| provider.provider_id == provider_id)
            });
            workflow_preset_record_to_dto(record, provider)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(vendor) = request.vendor.as_deref() {
        presets.retain(|preset| preset.vendor == vendor);
    }
    if let Some(ability_type) = request.ability_type.as_deref() {
        presets.retain(|preset| preset.ability_types.iter().any(|item| item == ability_type));
    }

    Ok(presets)
}

pub fn upsert_workflow_preset(
    database: &Database,
    preset: WorkflowPresetDto,
) -> Result<WorkflowPresetDto, String> {
    let repository = ProviderRepository::new(database);
    let provider = repository
        .list_providers()?
        .into_iter()
        .find(|provider| provider.provider_id == preset.provider_id)
        .ok_or_else(|| "provider.not_found: workflow provider config was not found.".to_string())?;

    validate_workflow_preset(&preset, &provider)?;
    let record = workflow_preset_dto_to_record(&preset);
    repository.upsert_workflow_preset(&record)?;
    workflow_preset_record_to_dto(record, Some(&provider))
}

pub fn delete_workflow_preset(
    database: &Database,
    request: DeleteWorkflowPresetRequest,
) -> Result<WorkflowPresetDto, String> {
    validate_identifier("workflow_preset_id", &request.workflow_preset_id)?;
    let repository = ProviderRepository::new(database);
    let preset = repository
        .get_workflow_preset(&request.workflow_preset_id)?
        .ok_or_else(|| "workflow.preset_not_found: workflow preset was not found.".to_string())?;
    let providers = repository.list_providers()?;
    let provider = preset.provider_id.as_deref().and_then(|provider_id| {
        providers
            .iter()
            .find(|provider| provider.provider_id == provider_id)
    });
    repository.delete_workflow_preset(&request.workflow_preset_id)?;
    workflow_preset_record_to_dto(preset, provider)
}

fn app_config_from_value(value: Value) -> Result<AppConfigDto, String> {
    let app_locale = read_string(&value, "app_locale")?;
    let theme_preset = read_string(&value, "theme_preset")?;
    let layout_density = read_string(&value, "layout_density")?;
    let config = AppConfigDto {
        app_locale,
        theme_preset,
        layout_density,
    };
    validate_app_config(&config)?;
    Ok(config)
}

fn validate_app_config(config: &AppConfigDto) -> Result<(), String> {
    validate_one_of("app_locale", &config.app_locale, &["zh-CN", "en-US"])?;
    validate_one_of(
        "theme_preset",
        &config.theme_preset,
        &["graphite", "aurora", "ember", "porcelain", "sandstone"],
    )?;
    validate_one_of(
        "layout_density",
        &config.layout_density,
        &["compact", "comfortable"],
    )?;
    Ok(())
}

fn validate_provider_config(config: &ProviderConfigDto) -> Result<(), String> {
    validate_identifier("provider_id", &config.provider_id)?;
    validate_one_of(
        "provider_kind",
        &config.provider_kind,
        &["llm", "tts", "image", "video", "vlm", "workflow"],
    )?;
    validate_one_of(
        "auth_type",
        &config.auth_type,
        &["none", "api_key", "bearer_token", "custom_header", "oauth"],
    )?;
    validate_one_of(
        "status",
        &config.status,
        &["unconfigured", "ready", "testing", "failed", "disabled"],
    )?;
    validate_identifier("vendor", &config.vendor)?;
    if config.display_name.trim().is_empty() {
        return Err("display_name cannot be empty.".to_string());
    }
    if config.auth_type != "none" && config.key_alias.as_deref().unwrap_or("").is_empty() {
        return Err("key_alias is required when auth_type needs a secret.".to_string());
    }
    if config.config.get("model_name").is_some() || config.config.get("modelName").is_some() {
        return Err("ProviderConfig cannot store model_name; use provider_models.".to_string());
    }
    Ok(())
}

fn validate_provider_model(
    model: &ProviderModelDto,
    provider: &ProviderRecord,
) -> Result<(), String> {
    validate_identifier("model_id", &model.model_id)?;
    validate_identifier("provider_id", &model.provider_id)?;
    validate_model_name(&model.provider_model_id)?;
    validate_one_of(
        "provider_kind",
        &model.provider_kind,
        &["llm", "tts", "image", "video", "vlm"],
    )?;
    validate_identifier("vendor", &model.vendor)?;
    validate_one_of(
        "status",
        &model.status,
        &["unconfigured", "ready", "testing", "failed", "disabled"],
    )?;

    if model.provider_id != provider.provider_id {
        return Err("provider_id does not match provider config.".to_string());
    }
    if provider.kind == "workflow" {
        return Err(
            "provider_models cannot register workflow providers; use workflow_presets.".to_string(),
        );
    }
    if model.provider_kind != provider.kind {
        return Err("provider_kind must match the selected provider.".to_string());
    }
    if model.vendor != provider.vendor {
        return Err("vendor must match the selected provider.".to_string());
    }
    validate_model_name(&model.model_name)?;
    if model.display_name.trim().is_empty() {
        return Err("display_name cannot be empty.".to_string());
    }
    validate_string_array(
        "ability_types",
        &model.ability_types,
        allowed_model_ability_types(),
    )?;
    validate_string_array(
        "input_modalities",
        &model.input_modalities,
        &["text", "image", "audio", "video"],
    )?;
    validate_string_array(
        "output_modalities",
        &model.output_modalities,
        &["text", "image", "audio", "video"],
    )?;
    for feature_flag in &model.feature_flags {
        validate_identifier("feature_flag", feature_flag)?;
    }
    validate_json_object("limits", &model.limits)?;
    validate_json_container("input_requirements", &model.input_requirements)?;
    validate_json_object("config", &model.config)?;
    reject_json_secrets(&model.limits)?;
    reject_json_secrets(&model.input_requirements)?;
    reject_json_secrets(&model.config)?;
    validate_model_limits(&model.limits)?;
    Ok(())
}

fn validate_workflow_preset(
    preset: &WorkflowPresetDto,
    provider: &ProviderRecord,
) -> Result<(), String> {
    validate_identifier("workflow_preset_id", &preset.workflow_preset_id)?;
    validate_identifier("provider_id", &preset.provider_id)?;
    validate_one_of("vendor", &preset.vendor, &["comfyui", "runninghub"])?;
    validate_one_of(
        "status",
        &preset.status,
        &["unconfigured", "ready", "testing", "failed", "disabled"],
    )?;
    validate_model_name(&preset.workflow_key)?;
    if let Some(workflow_id) = preset.workflow_id.as_deref() {
        validate_model_name(workflow_id)?;
    }
    validate_model_name(&preset.workflow_version)?;
    validate_string_array(
        "ability_types",
        &preset.ability_types,
        allowed_workflow_ability_types(),
    )?;
    validate_string_array(
        "input_modalities",
        &preset.input_modalities,
        &["text", "image", "audio", "video"],
    )?;
    validate_string_array(
        "output_modalities",
        &preset.output_modalities,
        &["text", "image", "audio", "video", "metadata"],
    )?;
    if preset.provider_id != provider.provider_id {
        return Err("provider_id does not match provider config.".to_string());
    }
    if provider.kind != "workflow" {
        return Err("workflow_presets require ProviderKind=workflow.".to_string());
    }
    if provider.vendor != preset.vendor {
        return Err("workflow preset vendor must match provider vendor.".to_string());
    }
    if preset.display_name.trim().is_empty() {
        return Err("display_name cannot be empty.".to_string());
    }
    if preset.vendor == "runninghub" && preset.workflow_id.as_deref().unwrap_or("").is_empty() {
        return Err("RunningHub workflow preset requires workflow_id.".to_string());
    }
    validate_json_object("limits", &preset.limits)?;
    validate_json_object("param_schema", &preset.param_schema)?;
    validate_non_empty_json_object("node_map", &preset.node_map)?;
    validate_non_empty_json_object("output_map", &preset.output_map)?;
    validate_json_object("default_params", &preset.default_params)?;
    validate_json_object("config", &preset.config)?;
    reject_json_secrets(&preset.limits)?;
    reject_json_secrets(&preset.param_schema)?;
    reject_json_secrets(&preset.node_map)?;
    reject_json_secrets(&preset.output_map)?;
    reject_json_secrets(&preset.default_params)?;
    reject_json_secrets(&preset.config)?;
    validate_node_map(&preset.vendor, &preset.node_map)?;
    validate_output_map(&preset.output_map)?;
    Ok(())
}

fn provider_record_to_dto(record: ProviderRecord) -> ProviderConfigDto {
    ProviderConfigDto {
        provider_id: record.provider_id,
        provider_kind: record.kind,
        vendor: record.vendor,
        display_name: record.display_name,
        base_url: record.base_url,
        auth_type: record.auth_type,
        key_alias: record.key_alias,
        status: record.status,
        is_enabled: record.enabled,
        config: record.config_json,
    }
}

fn workflow_preset_dto_to_record(preset: &WorkflowPresetDto) -> WorkflowPresetRecord {
    let capability = preset
        .ability_types
        .first()
        .cloned()
        .unwrap_or_else(|| "workflow_execution".to_string());
    WorkflowPresetRecord {
        preset_id: preset.workflow_preset_id.clone(),
        provider_id: Some(preset.provider_id.clone()),
        model_id: None,
        name: preset.display_name.clone(),
        kind: preset.vendor.clone(),
        capability,
        enabled: preset.is_enabled,
        config_json: json!({
            "providerId": preset.provider_id,
            "vendor": preset.vendor,
            "workflowKey": preset.workflow_key,
            "workflowId": preset.workflow_id,
            "displayName": preset.display_name,
            "workflowVersion": preset.workflow_version,
            "abilityTypes": preset.ability_types,
            "inputModalities": preset.input_modalities,
            "outputModalities": preset.output_modalities,
            "limits": preset.limits,
            "paramSchema": preset.param_schema,
            "nodeMap": preset.node_map,
            "outputMap": preset.output_map,
            "defaultParams": preset.default_params,
            "status": preset.status,
            "isBuiltin": preset.is_builtin,
            "config": preset.config
        }),
    }
}

fn workflow_preset_record_to_dto(
    record: WorkflowPresetRecord,
    provider: Option<&ProviderRecord>,
) -> Result<WorkflowPresetDto, String> {
    let config = record.config_json;
    let ability_types = read_string_array_from_value(&config, &["abilityTypes", "ability_types"])
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec![record.capability.clone()]);
    let provider_id = read_optional_string(&config, &["providerId", "provider_id"])
        .or_else(|| record.provider_id.clone())
        .unwrap_or_default();

    Ok(WorkflowPresetDto {
        workflow_preset_id: record.preset_id,
        provider_id,
        vendor: read_optional_string(&config, &["vendor"]).unwrap_or(record.kind),
        workflow_key: read_optional_string(&config, &["workflowKey", "workflow_key"])
            .unwrap_or_default(),
        workflow_id: read_optional_string(&config, &["workflowId", "workflow_id"]),
        display_name: read_optional_string(&config, &["displayName", "display_name"])
            .unwrap_or(record.name),
        workflow_version: read_optional_string(&config, &["workflowVersion", "workflow_version"])
            .unwrap_or_else(|| "1.0.0".to_string()),
        ability_types,
        input_modalities: read_string_array_from_value(
            &config,
            &["inputModalities", "input_modalities"],
        )
        .unwrap_or_default(),
        output_modalities: read_string_array_from_value(
            &config,
            &["outputModalities", "output_modalities"],
        )
        .unwrap_or_default(),
        limits: read_optional_value(&config, &["limits"]).unwrap_or_else(|| json!({})),
        param_schema: read_optional_value(&config, &["paramSchema", "param_schema"])
            .unwrap_or_else(|| json!({})),
        node_map: read_optional_value(&config, &["nodeMap", "node_map"])
            .unwrap_or_else(|| json!({})),
        output_map: read_optional_value(&config, &["outputMap", "output_map"])
            .unwrap_or_else(|| json!({})),
        default_params: read_optional_value(&config, &["defaultParams", "default_params"])
            .unwrap_or_else(|| json!({})),
        status: read_optional_string(&config, &["status"]).unwrap_or_else(|| {
            if record.enabled {
                "ready".to_string()
            } else {
                "disabled".to_string()
            }
        }),
        is_builtin: read_optional_bool(&config, &["isBuiltin", "is_builtin"]).unwrap_or(false),
        is_enabled: record.enabled,
        config: read_optional_value(&config, &["config"]).unwrap_or_else(|| {
            provider
                .map(|provider| json!({ "providerStatus": provider.status }))
                .unwrap_or_else(|| json!({}))
        }),
    })
}

fn provider_model_dto_to_record(
    model: &ProviderModelDto,
    provider: &ProviderRecord,
) -> ProviderModelRecord {
    let capability = model
        .ability_types
        .first()
        .cloned()
        .unwrap_or_else(|| "text_generation".to_string());
    ProviderModelRecord {
        model_id: model.model_id.clone(),
        provider_id: provider.provider_id.clone(),
        provider_model_id: model.provider_model_id.clone(),
        display_name: model.display_name.clone(),
        capability,
        enabled: model.is_enabled,
        config_json: json!({
            "providerKind": provider.kind,
            "vendor": provider.vendor,
            "modelName": model.model_name,
            "abilityTypes": model.ability_types,
            "inputModalities": model.input_modalities,
            "outputModalities": model.output_modalities,
            "featureFlags": model.feature_flags,
            "limits": model.limits,
            "inputRequirements": model.input_requirements,
            "apiContractVerified": model.api_contract_verified,
            "status": model.status,
            "config": model.config
        }),
    }
}

fn provider_model_record_to_dto(
    record: ProviderModelRecord,
    provider: &ProviderRecord,
) -> Result<ProviderModelDto, String> {
    let config = record.config_json;
    let ability_types = read_string_array_from_value(&config, &["abilityTypes", "ability_types"])
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec![record.capability.clone()]);

    Ok(ProviderModelDto {
        model_id: record.model_id,
        provider_id: record.provider_id,
        provider_kind: read_optional_string(&config, &["providerKind", "provider_kind"])
            .unwrap_or_else(|| provider.kind.clone()),
        vendor: read_optional_string(&config, &["vendor"])
            .unwrap_or_else(|| provider.vendor.clone()),
        provider_model_id: record.provider_model_id.clone(),
        model_name: read_optional_string(&config, &["modelName", "model_name"])
            .unwrap_or(record.provider_model_id),
        display_name: record.display_name,
        ability_types,
        input_modalities: read_string_array_from_value(
            &config,
            &["inputModalities", "input_modalities"],
        )
        .unwrap_or_default(),
        output_modalities: read_string_array_from_value(
            &config,
            &["outputModalities", "output_modalities"],
        )
        .unwrap_or_default(),
        feature_flags: read_string_array_from_value(&config, &["featureFlags", "feature_flags"])
            .unwrap_or_default(),
        limits: read_optional_value(&config, &["limits"]).unwrap_or_else(|| json!({})),
        input_requirements: read_optional_value(
            &config,
            &["inputRequirements", "input_requirements"],
        )
        .unwrap_or_else(|| json!({})),
        api_contract_verified: read_optional_bool(
            &config,
            &["apiContractVerified", "api_contract_verified"],
        )
        .unwrap_or(false),
        status: read_optional_string(&config, &["status"]).unwrap_or_else(|| {
            if record.enabled {
                "ready".to_string()
            } else {
                "disabled".to_string()
            }
        }),
        is_enabled: record.enabled,
        config: read_optional_value(&config, &["config"]).unwrap_or_else(|| json!({})),
    })
}

fn read_string(value: &Value, key: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("config.invalid: missing {key}."))
}

fn read_optional_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str).map(str::to_string))
}

fn read_optional_bool(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_bool))
}

fn read_optional_value(value: &Value, keys: &[&str]) -> Option<Value> {
    keys.iter().find_map(|key| value.get(*key).cloned())
}

fn read_string_array_from_value(value: &Value, keys: &[&str]) -> Option<Vec<String>> {
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

fn validate_one_of(name: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        return Ok(());
    }

    Err(format!("config.invalid: {name} has unsupported value."))
}

fn validate_string_array(name: &str, values: &[String], allowed: &[&str]) -> Result<(), String> {
    if values.is_empty() {
        return Err(format!("{name} cannot be empty."));
    }

    for value in values {
        validate_one_of(name, value, allowed)?;
    }

    Ok(())
}

fn validate_identifier(name: &str, value: &str) -> Result<(), String> {
    if value.trim() != value || value.is_empty() {
        return Err(format!("{name} cannot be empty or padded."));
    }

    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "_-.:".contains(character))
    {
        return Ok(());
    }

    Err(format!(
        "{name} may only contain ASCII letters, numbers, underscore, hyphen, dot, or colon."
    ))
}

fn validate_model_name(value: &str) -> Result<(), String> {
    if value.trim() != value || value.is_empty() {
        return Err("model_name cannot be empty or padded.".to_string());
    }

    if value.len() > 160 {
        return Err("model_name is too long.".to_string());
    }

    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "_-.:/".contains(character))
    {
        return Ok(());
    }

    Err("model_name may only contain ASCII letters, numbers, underscore, hyphen, dot, colon, or slash.".to_string())
}

fn validate_json_object(name: &str, value: &Value) -> Result<(), String> {
    if value.is_object() {
        return Ok(());
    }

    Err(format!("{name} must be a JSON object."))
}

fn validate_non_empty_json_object(name: &str, value: &Value) -> Result<(), String> {
    validate_json_object(name, value)?;
    if value.as_object().is_some_and(|object| !object.is_empty()) {
        return Ok(());
    }

    Err(format!("{name} cannot be empty."))
}

fn validate_json_container(name: &str, value: &Value) -> Result<(), String> {
    if value.is_object() || value.is_array() {
        return Ok(());
    }

    Err(format!("{name} must be a JSON object or array."))
}

fn validate_model_limits(limits: &Value) -> Result<(), String> {
    validate_string_list_if_present(
        limits,
        &[
            "aspectRatios",
            "aspect_ratios",
            "supportedAspectRatios",
            "supported_aspect_ratios",
        ],
        "aspect ratio",
    )?;
    validate_string_list_if_present(limits, &["resolutions"], "resolution")?;
    validate_duration_or_fps_range(
        limits,
        &[
            "durationSeconds",
            "duration_seconds",
            "durationRange",
            "duration_range",
        ],
        "duration_seconds",
    )?;
    validate_duration_or_fps_range(limits, &["fpsRange", "fps_range"], "fps_range")?;
    if let Some(fps_values) = read_number_array(limits, &["fps", "fps_values"]) {
        if fps_values.is_empty() || fps_values.iter().any(|value| *value <= 0.0) {
            return Err("fps values must be positive.".to_string());
        }
    }
    if let Some(max_reference_images) =
        read_optional_u64(limits, &["maxReferenceImages", "max_reference_images"])
    {
        if max_reference_images > 20 {
            return Err("max_reference_images cannot exceed 20.".to_string());
        }
    }
    Ok(())
}

fn validate_node_map(vendor: &str, node_map: &Value) -> Result<(), String> {
    let object = node_map
        .as_object()
        .ok_or_else(|| "node_map must be a JSON object.".to_string())?;
    for (key, value) in object {
        validate_identifier("node_map key", key)?;
        let Some(path) = value.as_str() else {
            return Err("node_map values must be strings.".to_string());
        };
        if vendor == "comfyui" {
            let parts = path.split('.').collect::<Vec<_>>();
            if parts.len() != 3 || parts[1] != "inputs" || parts.iter().any(|part| part.is_empty())
            {
                return Err("ComfyUI node_map values must use node.inputs.field paths.".to_string());
            }
        } else {
            validate_model_name(path)?;
        }
    }
    Ok(())
}

fn validate_output_map(output_map: &Value) -> Result<(), String> {
    let object = output_map
        .as_object()
        .ok_or_else(|| "output_map must be a JSON object.".to_string())?;
    for (key, value) in object {
        validate_one_of("output kind", key, &["image", "video", "audio", "metadata"])?;
        let Some(path) = value.as_str() else {
            return Err("output_map values must be strings.".to_string());
        };
        if path.trim().is_empty() {
            return Err("output_map values cannot be empty.".to_string());
        }
    }
    Ok(())
}

fn validate_string_list_if_present(value: &Value, keys: &[&str], name: &str) -> Result<(), String> {
    if let Some(items) = read_string_array_from_value(value, keys) {
        if items.iter().any(|item| item.trim().is_empty()) {
            return Err(format!("{name} values cannot be empty."));
        }
    }
    Ok(())
}

fn validate_duration_or_fps_range(value: &Value, keys: &[&str], name: &str) -> Result<(), String> {
    let Some(range) = keys.iter().find_map(|key| value.get(*key)) else {
        return Ok(());
    };
    validate_json_object(name, range)?;
    let min = range
        .get("min")
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{name}.min is required."))?;
    let max = range
        .get("max")
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{name}.max is required."))?;
    if min <= 0.0 || max < min {
        return Err(format!("{name} range is invalid."));
    }
    Ok(())
}

fn read_number_array(value: &Value, keys: &[&str]) -> Option<Vec<f64>> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
    })
}

fn read_optional_u64(value: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_u64))
}

fn allowed_model_ability_types() -> &'static [&'static str] {
    &[
        "text_generation",
        "structured_output",
        "text_to_image",
        "image_to_image",
        "text_to_video",
        "image_to_video",
        "first_frame_i2v",
        "start_end_frame_i2v",
        "reference_to_video",
        "video_continuation",
        "video_editing",
        "action_transfer",
        "digital_human",
        "native_audio",
        "voice_reference",
        "multi_shot",
        "text_to_speech",
        "vision_analysis",
    ]
}

fn allowed_workflow_ability_types() -> &'static [&'static str] {
    &[
        "text_to_image",
        "image_to_image",
        "text_to_video",
        "image_to_video",
        "first_frame_i2v",
        "start_end_frame_i2v",
        "reference_to_video",
        "video_continuation",
        "video_editing",
        "action_transfer",
        "digital_human",
        "native_audio",
        "voice_reference",
        "multi_shot",
        "text_to_speech",
        "vision_analysis",
        "workflow_execution",
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        delete_provider_config, get_app_config, list_provider_configs, update_app_config,
        upsert_provider_config,
    };
    use super::{delete_provider_model, list_provider_models, upsert_provider_model};
    use super::{delete_workflow_preset, list_workflow_presets, upsert_workflow_preset};
    use crate::db::Database;
    use crate::domain::config::{
        AppConfigDto, DeleteProviderConfigRequest, ListProviderConfigsRequest, ProviderConfigDto,
    };
    use crate::domain::config::{
        DeleteProviderModelRequest, ListProviderModelsRequest, ProviderModelDto,
    };
    use crate::domain::config::{
        DeleteWorkflowPresetRequest, ListWorkflowPresetsRequest, WorkflowPresetDto,
    };
    use crate::services::keyring_service::KeyringService;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn app_config_persists_in_sqlite() {
        let path = test_database_path("app_config");
        let database = Database::open(&path).expect("database should open");

        let default_config = get_app_config(&database).expect("default config should load");
        assert_eq!(default_config.app_locale, "zh-CN");

        update_app_config(
            &database,
            AppConfigDto {
                app_locale: "en-US".to_string(),
                theme_preset: "ember".to_string(),
                layout_density: "compact".to_string(),
            },
        )
        .expect("config should save");

        drop(database);
        let database = Database::open(&path).expect("database should reopen");
        let config = get_app_config(&database).expect("config should reload");
        assert_eq!(config.app_locale, "en-US");
        assert_eq!(config.theme_preset, "ember");
        assert_eq!(config.layout_density, "compact");

        cleanup(path);
    }

    #[test]
    fn invalid_app_config_is_rejected() {
        let path = test_database_path("invalid_app_config");
        let database = Database::open(&path).expect("database should open");

        assert!(update_app_config(
            &database,
            AppConfigDto {
                app_locale: "fr-FR".to_string(),
                theme_preset: "ember".to_string(),
                layout_density: "compact".to_string(),
            },
        )
        .is_err());

        cleanup(path);
    }

    #[test]
    fn provider_config_persists_without_secret_or_model_name() {
        let path = test_database_path("provider_config");
        let database = Database::open(&path).expect("database should open");

        let saved = upsert_provider_config(
            &database,
            ProviderConfigDto {
                provider_id: "provider_deepseek".to_string(),
                provider_kind: "llm".to_string(),
                vendor: "deepseek".to_string(),
                display_name: "DeepSeek".to_string(),
                base_url: Some("https://api.deepseek.com/v1".to_string()),
                auth_type: "api_key".to_string(),
                key_alias: Some("deepseek_main".to_string()),
                status: "disabled".to_string(),
                is_enabled: false,
                config: json!({ "timeoutSeconds": 30 }),
            },
        )
        .expect("provider config should save");

        assert_eq!(saved.key_alias.as_deref(), Some("deepseek_main"));
        let providers = list_provider_configs(
            &database,
            ListProviderConfigsRequest {
                provider_kind: Some("llm".to_string()),
            },
        )
        .expect("providers should list");
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].config["timeoutSeconds"], 30);

        cleanup(path);
    }

    #[test]
    fn provider_config_rejects_secret_and_model_name() {
        let path = test_database_path("provider_config_rejects");
        let database = Database::open(&path).expect("database should open");

        let base = ProviderConfigDto {
            provider_id: "provider_bad".to_string(),
            provider_kind: "llm".to_string(),
            vendor: "deepseek".to_string(),
            display_name: "Bad".to_string(),
            base_url: Some("https://api.example.com/v1".to_string()),
            auth_type: "api_key".to_string(),
            key_alias: Some("bad_alias".to_string()),
            status: "disabled".to_string(),
            is_enabled: false,
            config: json!({}),
        };

        let mut with_secret = base.clone();
        with_secret.config = json!({ "api_key": "sk-abcdefghijklmnopqrstuvwxyz012345" });
        assert!(upsert_provider_config(&database, with_secret).is_err());

        let mut with_model = base;
        with_model.config = json!({ "model_name": "deepseek-chat" });
        assert!(upsert_provider_config(&database, with_model).is_err());

        cleanup(path);
    }

    #[test]
    fn deleting_provider_config_does_not_delete_keyring_secret() {
        let path = test_database_path("provider_delete_keeps_secret");
        let database = Database::open(&path).expect("database should open");
        let keyring = KeyringService::memory();
        keyring
            .save_provider_secret(
                "provider_delete",
                "api_key",
                Some("provider_delete_key"),
                "sk-delete-secret-123456",
            )
            .expect("secret should save");

        upsert_provider_config(
            &database,
            ProviderConfigDto {
                provider_id: "provider_delete".to_string(),
                provider_kind: "llm".to_string(),
                vendor: "dummy".to_string(),
                display_name: "Delete Provider".to_string(),
                base_url: None,
                auth_type: "api_key".to_string(),
                key_alias: Some("provider_delete_key".to_string()),
                status: "ready".to_string(),
                is_enabled: true,
                config: json!({}),
            },
        )
        .expect("provider config should save");

        let deleted = delete_provider_config(
            &database,
            DeleteProviderConfigRequest {
                provider_id: "provider_delete".to_string(),
            },
        )
        .expect("provider config should delete");

        assert_eq!(deleted.key_alias.as_deref(), Some("provider_delete_key"));
        assert!(keyring
            .has_provider_secret("provider_delete_key")
            .expect("secret status should be readable"));
        assert!(list_provider_configs(
            &database,
            ListProviderConfigsRequest {
                provider_kind: None,
            },
        )
        .expect("providers should list")
        .is_empty());

        cleanup(path);
    }

    #[test]
    fn provider_model_persists_structured_capability_matrix() {
        let path = test_database_path("provider_model_matrix");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_image", "image", "dummy");

        let saved = upsert_provider_model(&database, image_model("model_image", "provider_image"))
            .expect("provider model should save");

        assert_eq!(saved.provider_kind, "image");
        assert_eq!(saved.ability_types, vec!["text_to_image"]);
        assert_eq!(saved.limits["supportedAspectRatios"][0], "9:16");
        let models = list_provider_models(
            &database,
            ListProviderModelsRequest {
                provider_id: Some("provider_image".to_string()),
                provider_kind: Some("image".to_string()),
            },
        )
        .expect("models should list");
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].model_name, "dummy-image-v1");

        let deleted = delete_provider_model(
            &database,
            DeleteProviderModelRequest {
                model_id: "model_image".to_string(),
            },
        )
        .expect("provider model should delete");
        assert_eq!(deleted.model_id, "model_image");
        assert!(list_provider_models(
            &database,
            ListProviderModelsRequest {
                provider_id: None,
                provider_kind: None,
            },
        )
        .expect("models should list")
        .is_empty());

        cleanup(path);
    }

    #[test]
    fn provider_model_rejects_workflow_provider_and_invalid_limits() {
        let path = test_database_path("provider_model_rejects");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_workflow", "workflow", "comfyui");
        seed_provider(&database, "provider_video", "video", "dummy");

        let mut workflow_model = image_model("model_workflow", "provider_workflow");
        workflow_model.provider_kind = "workflow".to_string();
        workflow_model.vendor = "comfyui".to_string();
        assert!(upsert_provider_model(&database, workflow_model).is_err());

        let mut invalid_limits = video_model("model_video", "provider_video");
        invalid_limits.limits = json!({
            "durationSeconds": { "min": 8, "max": 3, "integer": true }
        });
        assert!(upsert_provider_model(&database, invalid_limits).is_err());

        let mut secret_config = video_model("model_video_secret", "provider_video");
        secret_config.config = json!({ "api_key": "sk-abcdefghijklmnopqrstuvwxyz012345" });
        assert!(upsert_provider_model(&database, secret_config).is_err());

        cleanup(path);
    }

    #[test]
    fn workflow_preset_persists_structured_registration() {
        let path = test_database_path("workflow_preset_matrix");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_comfyui", "workflow", "comfyui");

        let saved = upsert_workflow_preset(
            &database,
            comfyui_preset("workflow_comfyui_i2v", "provider_comfyui"),
        )
        .expect("workflow preset should save");

        assert_eq!(saved.vendor, "comfyui");
        assert_eq!(saved.ability_types, vec!["image_to_video"]);
        assert_eq!(saved.node_map["prompt"], "12.inputs.text");
        let presets = list_workflow_presets(
            &database,
            ListWorkflowPresetsRequest {
                provider_id: Some("provider_comfyui".to_string()),
                vendor: Some("comfyui".to_string()),
                ability_type: Some("image_to_video".to_string()),
            },
        )
        .expect("presets should list");
        assert_eq!(presets.len(), 1);

        let deleted = delete_workflow_preset(
            &database,
            DeleteWorkflowPresetRequest {
                workflow_preset_id: "workflow_comfyui_i2v".to_string(),
            },
        )
        .expect("workflow preset should delete");
        assert_eq!(deleted.workflow_preset_id, "workflow_comfyui_i2v");
        assert!(list_workflow_presets(
            &database,
            ListWorkflowPresetsRequest {
                provider_id: None,
                vendor: None,
                ability_type: None,
            },
        )
        .expect("presets should list")
        .is_empty());

        cleanup(path);
    }

    #[test]
    fn workflow_preset_rejects_invalid_provider_and_maps() {
        let path = test_database_path("workflow_preset_rejects");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_image", "image", "dummy");
        seed_provider(&database, "provider_runninghub", "workflow", "runninghub");
        seed_provider(&database, "provider_comfyui", "workflow", "comfyui");

        let wrong_provider = comfyui_preset("workflow_wrong_provider", "provider_image");
        assert!(upsert_workflow_preset(&database, wrong_provider).is_err());

        let mut missing_workflow_id =
            runninghub_preset("workflow_missing_id", "provider_runninghub");
        missing_workflow_id.workflow_id = None;
        assert!(upsert_workflow_preset(&database, missing_workflow_id).is_err());

        let mut empty_node_map = comfyui_preset("workflow_empty_node", "provider_comfyui");
        empty_node_map.node_map = json!({});
        assert!(upsert_workflow_preset(&database, empty_node_map).is_err());

        let mut empty_output_map = comfyui_preset("workflow_empty_output", "provider_comfyui");
        empty_output_map.output_map = json!({});
        assert!(upsert_workflow_preset(&database, empty_output_map).is_err());

        let mut secret_config = comfyui_preset("workflow_secret", "provider_comfyui");
        secret_config.config = json!({ "token": "sk-abcdefghijklmnopqrstuvwxyz012345" });
        assert!(upsert_workflow_preset(&database, secret_config).is_err());

        cleanup(path);
    }

    fn seed_provider(database: &Database, provider_id: &str, kind: &str, vendor: &str) {
        upsert_provider_config(
            database,
            ProviderConfigDto {
                provider_id: provider_id.to_string(),
                provider_kind: kind.to_string(),
                vendor: vendor.to_string(),
                display_name: format!("{vendor} {kind}"),
                base_url: None,
                auth_type: "none".to_string(),
                key_alias: None,
                status: "ready".to_string(),
                is_enabled: true,
                config: json!({}),
            },
        )
        .expect("provider should save");
    }

    fn image_model(model_id: &str, provider_id: &str) -> ProviderModelDto {
        ProviderModelDto {
            model_id: model_id.to_string(),
            provider_id: provider_id.to_string(),
            provider_kind: "image".to_string(),
            vendor: "dummy".to_string(),
            provider_model_id: "dummy/image-v1".to_string(),
            model_name: "dummy-image-v1".to_string(),
            display_name: "Dummy image v1".to_string(),
            ability_types: vec!["text_to_image".to_string()],
            input_modalities: vec!["text".to_string()],
            output_modalities: vec!["image".to_string()],
            feature_flags: vec!["aspect_ratio".to_string()],
            limits: json!({
                "supportedAspectRatios": ["9:16", "16:9"],
                "resolutions": ["720p", "1080p"],
                "maxReferenceImages": 1
            }),
            input_requirements: json!({}),
            api_contract_verified: false,
            status: "ready".to_string(),
            is_enabled: true,
            config: json!({ "timeoutSeconds": 30 }),
        }
    }

    fn video_model(model_id: &str, provider_id: &str) -> ProviderModelDto {
        ProviderModelDto {
            model_id: model_id.to_string(),
            provider_id: provider_id.to_string(),
            provider_kind: "video".to_string(),
            vendor: "dummy".to_string(),
            provider_model_id: "dummy/video-v1".to_string(),
            model_name: "dummy-video-v1".to_string(),
            display_name: "Dummy video v1".to_string(),
            ability_types: vec!["image_to_video".to_string()],
            input_modalities: vec!["text".to_string(), "image".to_string()],
            output_modalities: vec!["video".to_string()],
            feature_flags: vec!["duration".to_string(), "fps".to_string()],
            limits: json!({
                "durationSeconds": { "min": 3, "max": 8, "integer": true },
                "fps": [16, 24],
                "supportedAspectRatios": ["9:16"]
            }),
            input_requirements: json!({}),
            api_contract_verified: false,
            status: "ready".to_string(),
            is_enabled: true,
            config: json!({}),
        }
    }

    fn comfyui_preset(preset_id: &str, provider_id: &str) -> WorkflowPresetDto {
        WorkflowPresetDto {
            workflow_preset_id: preset_id.to_string(),
            provider_id: provider_id.to_string(),
            vendor: "comfyui".to_string(),
            workflow_key: "comfyui/video_wan_i2v_v1/workflow_api.json".to_string(),
            workflow_id: None,
            display_name: "ComfyUI Wan I2V".to_string(),
            workflow_version: "1.0.0".to_string(),
            ability_types: vec!["image_to_video".to_string()],
            input_modalities: vec!["text".to_string(), "image".to_string()],
            output_modalities: vec!["video".to_string()],
            limits: json!({
                "durationSeconds": { "min": 3, "max": 8, "integer": true },
                "supportedAspectRatios": ["9:16"]
            }),
            param_schema: json!({
                "prompt": { "type": "string", "required": true },
                "input_image": { "type": "asset_path", "required": true }
            }),
            node_map: json!({
                "prompt": "12.inputs.text",
                "input_image": "7.inputs.image"
            }),
            output_map: json!({ "video": "99.outputs.video" }),
            default_params: json!({ "frames": 81 }),
            status: "ready".to_string(),
            is_builtin: false,
            is_enabled: true,
            config: json!({}),
        }
    }

    fn runninghub_preset(preset_id: &str, provider_id: &str) -> WorkflowPresetDto {
        WorkflowPresetDto {
            vendor: "runninghub".to_string(),
            workflow_key: "runninghub/video_wan_i2v_v1".to_string(),
            workflow_id: Some("rh_workflow_123".to_string()),
            node_map: json!({
                "prompt": "prompt",
                "input_image": "image"
            }),
            output_map: json!({ "video": "video" }),
            ..comfyui_preset(preset_id, provider_id)
        }
    }

    fn test_database_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-config-{name}-{}-{nanos}.sqlite3",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
    }
}
