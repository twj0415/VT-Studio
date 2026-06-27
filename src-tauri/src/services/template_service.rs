use crate::domain::template::{
    ListTemplateManifestsRequest, PreviewTemplateRequest, PreviewTemplateResponseDto,
    RenderTemplateRequest, RenderTemplateResponseDto, TemplateManifestDto, TemplateParamSchemaDto,
    TemplateParamValidationResultDto, TemplateRenderDataDto, TemplateSidecarBinaryStatusDto,
    TemplateSidecarStatusDto, TemplateViewportDto, ValidateTemplateParamsRequest,
};
use crate::security::path_guard::PathGuard;
use crate::security::secret_guard::redact_text;
use crate::services::storage_service::{FileBucket, StorageService};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TEMPLATE_TYPES: &[&str] = &["frame", "cover", "subtitle", "transition", "layout"];
const TEMPLATE_PARAM_TYPES: &[&str] = &[
    "text", "number", "color", "bool", "select", "image", "font", "range", "json",
];
const BUILTIN_PARAMS: &[&str] = &["title", "text", "image", "index"];
const NODE_BINARY: &str = "node.exe";
const CHROMIUM_BINARY: &str = "chromium.exe";
const CHROMIUM_PACKAGE_BINARY: &str = "chromium/chrome.exe";
const PLAYWRIGHT_DRIVER: &str = "playwright-driver.js";
const TEMPLATE_SIDECAR_MISSING: &str = "template.sidecar_missing";
const TEMPLATE_RENDER_FAILED: &str = "template.render_failed";
const TEMPLATE_BROWSER_CRASHED: &str = "template.browser_crashed";
const TEMPLATE_OUTPUT_MISSING: &str = "template.output_missing";
const TEMPLATE_RENDER_TIMEOUT: Duration = Duration::from_secs(45);
const PROCESS_TEXT_MAX_LINES: usize = 120;
const PROCESS_TEXT_MAX_BYTES: usize = 24 * 1024;

pub fn list_template_manifests(
    workspace_root: &Path,
    request: ListTemplateManifestsRequest,
) -> Result<Vec<TemplateManifestDto>, String> {
    StorageService::new(workspace_root).initialize_workspace()?;
    seed_builtin_templates(workspace_root)?;

    let aspect_filter = request
        .aspect_ratio
        .as_deref()
        .map(normalize_aspect_ratio)
        .transpose()?;
    let template_type_filter = request
        .template_type
        .as_deref()
        .map(normalize_template_type)
        .transpose()?;
    let source_type_filter = request
        .source_type
        .as_deref()
        .map(normalize_source_type)
        .transpose()?;

    let mut manifests = Vec::new();
    for source_type in ["builtin", "user"] {
        if source_type_filter
            .as_deref()
            .is_some_and(|expected| expected != source_type)
        {
            continue;
        }

        for template_type in TEMPLATE_TYPES {
            if template_type_filter
                .as_deref()
                .is_some_and(|expected| expected != *template_type)
            {
                continue;
            }

            for aspect_ratio in ["vertical_9_16", "horizontal_16_9", "square_1_1"] {
                if aspect_filter
                    .as_deref()
                    .is_some_and(|expected| expected != aspect_ratio)
                {
                    continue;
                }

                let dir = workspace_root
                    .join("templates")
                    .join(source_type)
                    .join(template_type)
                    .join(aspect_ratio);
                if !dir.is_dir() {
                    continue;
                }

                for entry in fs::read_dir(&dir).map_err(|error| error.to_string())? {
                    let path = entry.map_err(|error| error.to_string())?.path();
                    if path.extension().and_then(|value| value.to_str()) != Some("html") {
                        continue;
                    }
                    manifests.push(read_template_manifest(
                        workspace_root,
                        source_type,
                        template_type,
                        aspect_ratio,
                        &path,
                    )?);
                }
            }
        }
    }

    manifests.sort_by(|left, right| {
        left.template_type
            .cmp(&right.template_type)
            .then(left.aspect_ratio.cmp(&right.aspect_ratio))
            .then(left.template_id.cmp(&right.template_id))
    });
    Ok(manifests)
}

pub fn validate_template_params(
    request: ValidateTemplateParamsRequest,
) -> Result<TemplateParamValidationResultDto, String> {
    validate_template_manifest(&request.manifest)?;
    let input = request
        .params
        .as_object()
        .ok_or_else(|| "template.param_invalid: params must be an object.".to_string())?;

    let mut normalized = Map::new();
    let mut errors = Vec::new();
    for schema in &request.manifest.params {
        let value = input
            .get(&schema.name)
            .cloned()
            .unwrap_or_else(|| schema.default_value.clone());
        match normalize_param_value(schema, value) {
            Ok(value) => {
                normalized.insert(schema.name.clone(), value);
            }
            Err(error) => errors.push(error),
        }
    }

    Ok(TemplateParamValidationResultDto {
        valid: errors.is_empty(),
        normalized_params: Value::Object(normalized),
        errors,
    })
}

pub fn preview_template(
    workspace_root: &Path,
    request: PreviewTemplateRequest,
) -> Result<PreviewTemplateResponseDto, String> {
    let manifest = find_template_manifest(
        workspace_root,
        &request.template_id,
        &request.aspect_ratio,
        &request.template_type,
    )?;
    let output_path = format!(
        "cache/template_preview_{}_{}.png",
        manifest.template_type, manifest.template_id
    );
    let response = render_template(
        workspace_root,
        RenderTemplateRequest {
            template_id: request.template_id,
            aspect_ratio: request.aspect_ratio,
            template_type: request.template_type,
            params: request.params,
            data: request.data,
            output_path,
        },
    )?;
    Ok(PreviewTemplateResponseDto {
        preview_path: response.rendered_frame_path,
        width: response.width,
        height: response.height,
    })
}

pub fn render_template(
    workspace_root: &Path,
    request: RenderTemplateRequest,
) -> Result<RenderTemplateResponseDto, String> {
    render_template_with_runner(workspace_root, request, &ProcessTemplateRenderRunner)
}

pub fn check_template_sidecars(workspace_root: &Path) -> Result<TemplateSidecarStatusDto, String> {
    check_template_sidecars_with_runner(workspace_root, &ProcessTemplateVersionRunner)
}

fn render_template_with_runner(
    workspace_root: &Path,
    request: RenderTemplateRequest,
    runner: &dyn TemplateRenderRunner,
) -> Result<RenderTemplateResponseDto, String> {
    let manifest = find_template_manifest(
        workspace_root,
        &request.template_id,
        &request.aspect_ratio,
        &request.template_type,
    )?;
    let validation = validate_template_params(ValidateTemplateParamsRequest {
        manifest: manifest.clone(),
        params: request.params,
    })?;
    if !validation.valid {
        return Err(format!(
            "template.param_invalid: {}",
            validation.errors.join("; ")
        ));
    }

    validate_render_data_paths(workspace_root, &request.data)?;
    let entry = resolve_render_input_path(workspace_root, &manifest.entry_path)?;
    let output = resolve_render_output_path(workspace_root, &request.output_path)?;
    let render_plan = build_browser_render_plan(
        workspace_root,
        &entry.relative_path,
        &output.relative_path,
        &request.data,
        &validation.normalized_params,
    )?;
    let sidecars = require_template_sidecars(workspace_root)?;
    execute_browser_render_plan(
        workspace_root,
        &manifest.viewport,
        &render_plan,
        &sidecars,
        runner,
    )?;

    let output_path = workspace_root.join(&output.relative_path);
    if !output_path.is_file() {
        return Err(format!(
            "{TEMPLATE_OUTPUT_MISSING}: rendered output {} was not created.",
            output.relative_path
        ));
    }

    Ok(RenderTemplateResponseDto {
        rendered_frame_path: output.relative_path,
        width: manifest.viewport.width,
        height: manifest.viewport.height,
    })
}

pub fn parse_template_params_from_html(html: &str) -> Result<Vec<TemplateParamSchemaDto>, String> {
    let mut params = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut offset = 0;
    while let Some(start) = html[offset..].find("{{") {
        let start_index = offset + start + 2;
        let Some(end) = html[start_index..].find("}}") else {
            return Err("template.param_invalid: unclosed template placeholder.".to_string());
        };
        let raw = html[start_index..start_index + end].trim();
        offset = start_index + end + 2;
        if raw.is_empty() {
            continue;
        }
        let param = parse_template_param_placeholder(raw)?;
        if BUILTIN_PARAMS.contains(&param.name.as_str()) {
            continue;
        }
        if seen.insert(param.name.clone()) {
            params.push(param);
        }
    }
    Ok(params)
}

fn read_template_manifest(
    workspace_root: &Path,
    source_type: &str,
    template_type: &str,
    aspect_ratio: &str,
    path: &Path,
) -> Result<TemplateManifestDto, String> {
    let html = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let template_id = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "template.param_invalid: template file name is invalid.".to_string())?
        .to_string();
    validate_template_id(&template_id)?;
    let entry_path = controlled_template_entry_path(workspace_root, path)?;
    let manifest = TemplateManifestDto {
        template_id: template_id.clone(),
        template_type: template_type.to_string(),
        source_type: source_type.to_string(),
        display_name: template_id.clone(),
        display_name_key: format!("template.{template_type}.{template_id}"),
        version: "1.0.0".to_string(),
        aspect_ratio: aspect_ratio.to_string(),
        entry_path,
        viewport: viewport_for_aspect_ratio(aspect_ratio)?,
        params: parse_template_params_from_html(&html)?,
    };
    validate_template_manifest(&manifest)?;
    Ok(manifest)
}

fn find_template_manifest(
    workspace_root: &Path,
    template_id: &str,
    aspect_ratio: &str,
    template_type: &str,
) -> Result<TemplateManifestDto, String> {
    let aspect_ratio = normalize_aspect_ratio(aspect_ratio)?;
    let template_type = normalize_template_type(template_type)?;
    let manifests = list_template_manifests(
        workspace_root,
        ListTemplateManifestsRequest {
            aspect_ratio: Some(aspect_ratio),
            template_type: Some(template_type),
            source_type: None,
        },
    )?;
    manifests
        .into_iter()
        .find(|manifest| manifest.template_id == template_id)
        .ok_or_else(|| format!("template.not_found: template {template_id} was not found."))
}

fn parse_template_param_placeholder(raw: &str) -> Result<TemplateParamSchemaDto, String> {
    let (declaration, default_raw) = split_once(raw, '=');
    let (name_raw, type_raw) = split_once(declaration, ':');
    let name = name_raw.trim();
    validate_param_name(name)?;
    let param_type = type_raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("text");
    validate_param_type(param_type)?;
    let dictionary_code = dictionary_code_for_select(name, param_type)?;
    let default_value = default_value_for_param(param_type, default_raw.map(str::trim))?;

    Ok(TemplateParamSchemaDto {
        name: name.to_string(),
        param_type: param_type.to_string(),
        default_value,
        required: default_raw.is_none(),
        dictionary_code,
        min: None,
        max: None,
    })
}

fn validate_template_manifest(manifest: &TemplateManifestDto) -> Result<(), String> {
    validate_template_id(&manifest.template_id)?;
    normalize_template_type(&manifest.template_type)?;
    normalize_source_type(&manifest.source_type)?;
    normalize_aspect_ratio(&manifest.aspect_ratio)?;
    PathGuard::validate_relative_path(&manifest.entry_path)?;
    if !manifest.entry_path.starts_with("templates/") || !manifest.entry_path.ends_with(".html") {
        return Err(
            "template.resource_denied: entryPath must be a templates/*.html path.".to_string(),
        );
    }
    let expected_viewport = viewport_for_aspect_ratio(&manifest.aspect_ratio)?;
    if manifest.viewport != expected_viewport {
        return Err("template.param_invalid: viewport does not match aspectRatio.".to_string());
    }
    for param in &manifest.params {
        validate_param_name(&param.name)?;
        validate_param_type(&param.param_type)?;
        if param.param_type == "select" && param.dictionary_code.is_none() {
            return Err(format!(
                "template.param_invalid: select param {} must reference a dictionary.",
                param.name
            ));
        }
        normalize_param_value(param, param.default_value.clone())?;
    }
    Ok(())
}

fn normalize_param_value(schema: &TemplateParamSchemaDto, value: Value) -> Result<Value, String> {
    match schema.param_type.as_str() {
        "text" | "font" | "image" => value
            .as_str()
            .map(|value| Value::String(value.to_string()))
            .ok_or_else(|| format!("template.param_invalid: {} must be text.", schema.name)),
        "color" => {
            let Some(value) = value.as_str() else {
                return Err(format!(
                    "template.param_invalid: {} must be a color.",
                    schema.name
                ));
            };
            if is_hex_color(value) {
                Ok(Value::String(value.to_string()))
            } else {
                Err(format!(
                    "template.param_invalid: {} must be #RGB or #RRGGBB.",
                    schema.name
                ))
            }
        }
        "number" | "range" => {
            let Some(number) = value.as_f64() else {
                return Err(format!(
                    "template.param_invalid: {} must be a number.",
                    schema.name
                ));
            };
            if schema.min.is_some_and(|min| number < min)
                || schema.max.is_some_and(|max| number > max)
            {
                return Err(format!(
                    "template.param_invalid: {} is outside the allowed range.",
                    schema.name
                ));
            }
            Ok(json!(number))
        }
        "bool" => value
            .as_bool()
            .map(Value::Bool)
            .ok_or_else(|| format!("template.param_invalid: {} must be boolean.", schema.name)),
        "select" => value
            .as_str()
            .map(|value| Value::String(value.to_string()))
            .ok_or_else(|| {
                format!(
                    "template.param_invalid: {} must be a select value.",
                    schema.name
                )
            }),
        "json" => Ok(value),
        _ => Err(format!(
            "template.param_invalid: unsupported param type {}.",
            schema.param_type
        )),
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ResolvedRenderPath {
    relative_path: String,
}

#[derive(Debug, Clone, PartialEq)]
struct BrowserRenderPlan {
    entry_path: String,
    output_path: String,
    workspace_root: String,
    chromium_path: String,
    allowed_resource_roots: Vec<String>,
    injection_script: String,
    payload: Value,
}

fn build_browser_render_plan(
    workspace_root: &Path,
    entry_path: &str,
    output_path: &str,
    data: &TemplateRenderDataDto,
    params: &Value,
) -> Result<BrowserRenderPlan, String> {
    let payload = json!({
        "data": data,
        "params": params,
    });
    let payload_text = serde_json::to_string(&payload).map_err(|error| error.to_string())?;
    Ok(BrowserRenderPlan {
        entry_path: entry_path.to_string(),
        output_path: output_path.to_string(),
        workspace_root: workspace_root.to_string_lossy().replace('\\', "/"),
        chromium_path: sidecar_relative_path(CHROMIUM_BINARY),
        allowed_resource_roots: vec![
            "templates/".to_string(),
            "assets/".to_string(),
            "projects/".to_string(),
            "cache/fonts/".to_string(),
        ],
        injection_script: format!(
            "window.__VT_TEMPLATE_PAYLOAD__ = {payload_text}; document.dispatchEvent(new CustomEvent('vt-template-data'));"
        ),
        payload,
    })
}

fn validate_render_data_paths(
    workspace_root: &Path,
    data: &TemplateRenderDataDto,
) -> Result<(), String> {
    if let Some(path) = data.image_path.as_deref() {
        resolve_render_input_path(workspace_root, path)?;
    }
    if let Some(path) = data.video_frame_path.as_deref() {
        resolve_render_input_path(workspace_root, path)?;
    }
    Ok(())
}

fn resolve_render_input_path(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<ResolvedRenderPath, String> {
    let normalized = PathGuard::validate_relative_path(relative_path)
        .map_err(|error| format!("template.resource_denied: {error}"))?;
    if !is_allowed_render_input_path(&normalized) {
        return Err("template.resource_denied: template render input must be under templates, assets, projects, or cache/fonts.".to_string());
    }
    let guard = PathGuard::new(workspace_root);
    guard
        .safe_chromium_file(&normalized)
        .map_err(|error| format!("template.resource_denied: {error}"))?;
    Ok(ResolvedRenderPath {
        relative_path: normalized,
    })
}

fn resolve_render_output_path(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<ResolvedRenderPath, String> {
    let normalized = PathGuard::validate_relative_path(relative_path)
        .map_err(|error| format!("template.resource_denied: {error}"))?;
    if !(normalized.starts_with("cache/")
        || normalized.starts_with("projects/")
        || normalized.starts_with("outputs/"))
    {
        return Err(
            "template.resource_denied: template output must be under cache, projects, or outputs."
                .to_string(),
        );
    }
    let guard = PathGuard::new(workspace_root);
    guard
        .safe_join_for_write(&normalized)
        .map_err(|error| format!("template.resource_denied: {error}"))?;
    Ok(ResolvedRenderPath {
        relative_path: normalized,
    })
}

fn is_allowed_render_input_path(path: &str) -> bool {
    path.starts_with("templates/")
        || path.starts_with("assets/")
        || path.starts_with("projects/")
        || path.starts_with("cache/fonts/")
}

#[derive(Debug, Clone)]
struct TemplateSidecars {
    node_path: PathBuf,
    chromium_path: PathBuf,
    driver_path: PathBuf,
}

trait TemplateRenderRunner {
    fn run(
        &self,
        node_path: &Path,
        driver_path: &Path,
        request_path: &Path,
        replacements: &[(PathBuf, String)],
    ) -> Result<(), String>;
}

struct ProcessTemplateRenderRunner;
struct ProcessTemplateVersionRunner;

trait TemplateSidecarVersionRunner {
    fn run_version(&self, binary_path: &Path, binary_name: &str) -> Result<String, String>;
}

impl TemplateSidecarVersionRunner for ProcessTemplateVersionRunner {
    fn run_version(&self, binary_path: &Path, binary_name: &str) -> Result<String, String> {
        if binary_name == PLAYWRIGHT_DRIVER {
            return Ok("driver file present".to_string());
        }

        let output = Command::new(binary_path)
            .arg(if binary_name == NODE_BINARY {
                "--version"
            } else {
                "--version"
            })
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(if stderr.trim().is_empty() {
                "process exited with a non-zero status.".to_string()
            } else {
                stderr.to_string()
            });
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(first_non_empty_line(&stdout)
            .or_else(|| first_non_empty_line(&stderr))
            .unwrap_or_else(|| "version output was empty.".to_string()))
    }
}

impl TemplateRenderRunner for ProcessTemplateRenderRunner {
    fn run(
        &self,
        node_path: &Path,
        driver_path: &Path,
        request_path: &Path,
        replacements: &[(PathBuf, String)],
    ) -> Result<(), String> {
        let mut child = Command::new(node_path)
            .arg(driver_path)
            .arg(request_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| sanitize_process_message(&error.to_string(), replacements))?;

        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => {
                    let output = child.wait_with_output().map_err(|error| {
                        sanitize_process_message(&error.to_string(), replacements)
                    })?;
                    if output.status.success() {
                        return Ok(());
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let combined = if stdout.trim().is_empty() {
                        stderr.to_string()
                    } else if stderr.trim().is_empty() {
                        stdout.to_string()
                    } else {
                        format!("{stdout}\n{stderr}")
                    };
                    let message = sanitize_process_message(&combined, replacements);
                    let code = if message.contains("browser_crashed") {
                        TEMPLATE_BROWSER_CRASHED
                    } else {
                        TEMPLATE_RENDER_FAILED
                    };
                    return Err(format!("{code}: {message}"));
                }
                Ok(None) => {
                    if start.elapsed() >= TEMPLATE_RENDER_TIMEOUT {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(format!(
                            "{TEMPLATE_RENDER_FAILED}: browser render timed out after {} seconds.",
                            TEMPLATE_RENDER_TIMEOUT.as_secs()
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "{TEMPLATE_RENDER_FAILED}: {}",
                        sanitize_process_message(&error.to_string(), replacements)
                    ));
                }
            }
        }
    }
}

fn require_template_sidecars(workspace_root: &Path) -> Result<TemplateSidecars, String> {
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let node_path = resolve_template_sidecar(&storage, NODE_BINARY)?;
    let chromium_path = resolve_template_chromium_sidecar(&storage)?;
    let driver_path = resolve_template_sidecar(&storage, PLAYWRIGHT_DRIVER)?;
    Ok(TemplateSidecars {
        node_path,
        chromium_path,
        driver_path,
    })
}

fn resolve_template_sidecar(storage: &StorageService, binary: &str) -> Result<PathBuf, String> {
    storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Sidecar, binary)
        .map_err(|error| {
            format!(
                "{TEMPLATE_SIDECAR_MISSING}: {} is missing or unavailable: {}",
                sidecar_relative_path(binary),
                sanitize_sidecar_error(&error)
            )
        })
}

fn resolve_template_chromium_sidecar(storage: &StorageService) -> Result<PathBuf, String> {
    resolve_template_sidecar(storage, CHROMIUM_PACKAGE_BINARY)
        .or_else(|_| resolve_template_sidecar(storage, CHROMIUM_BINARY))
}

fn check_template_sidecars_with_runner(
    workspace_root: &Path,
    runner: &dyn TemplateSidecarVersionRunner,
) -> Result<TemplateSidecarStatusDto, String> {
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let node = check_template_sidecar_binary(&storage, NODE_BINARY, runner);
    let chromium = check_template_chromium_binary(&storage, runner);
    let playwright_driver = check_template_sidecar_binary(&storage, PLAYWRIGHT_DRIVER, runner);
    let ready = node.executable && chromium.executable && playwright_driver.executable;
    Ok(TemplateSidecarStatusDto {
        node,
        chromium,
        playwright_driver,
        ready,
        checked_at: current_timestamp_string(),
    })
}

fn check_template_chromium_binary(
    storage: &StorageService,
    runner: &dyn TemplateSidecarVersionRunner,
) -> TemplateSidecarBinaryStatusDto {
    let packaged = check_template_sidecar_binary(storage, CHROMIUM_PACKAGE_BINARY, runner);
    if packaged.exists {
        return TemplateSidecarBinaryStatusDto {
            name: CHROMIUM_BINARY.to_string(),
            relative_path: sidecar_relative_path(CHROMIUM_PACKAGE_BINARY),
            exists: packaged.exists,
            executable: packaged.executable,
            version: packaged.version,
            error_code: packaged.error_code,
            message: packaged.message,
        };
    }
    check_template_sidecar_binary(storage, CHROMIUM_BINARY, runner)
}

fn check_template_sidecar_binary(
    storage: &StorageService,
    binary: &str,
    runner: &dyn TemplateSidecarVersionRunner,
) -> TemplateSidecarBinaryStatusDto {
    let relative_path = sidecar_relative_path(binary);
    let resolved = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Sidecar, binary);
    let absolute_path = match resolved {
        Ok(path) => path,
        Err(error) => {
            return TemplateSidecarBinaryStatusDto {
                name: binary.to_string(),
                relative_path,
                exists: false,
                executable: false,
                version: None,
                error_code: Some(TEMPLATE_SIDECAR_MISSING.to_string()),
                message: Some(sanitize_sidecar_error(&error)),
            };
        }
    };

    if !absolute_path.is_file() {
        return TemplateSidecarBinaryStatusDto {
            name: binary.to_string(),
            relative_path,
            exists: false,
            executable: false,
            version: None,
            error_code: Some(TEMPLATE_SIDECAR_MISSING.to_string()),
            message: Some("sidecar file is missing.".to_string()),
        };
    }

    match runner.run_version(&absolute_path, binary) {
        Ok(version) => TemplateSidecarBinaryStatusDto {
            name: binary.to_string(),
            relative_path,
            exists: true,
            executable: true,
            version: Some(sanitize_process_message(&version, &[])),
            error_code: None,
            message: None,
        },
        Err(error) => TemplateSidecarBinaryStatusDto {
            name: binary.to_string(),
            relative_path,
            exists: true,
            executable: false,
            version: None,
            error_code: Some(TEMPLATE_RENDER_FAILED.to_string()),
            message: Some(sanitize_process_message(
                &error,
                &[(absolute_path, sidecar_relative_path(binary))],
            )),
        },
    }
}

fn sidecar_relative_path(binary: &str) -> String {
    format!("sidecars/{binary}")
}

fn sanitize_sidecar_error(error: &str) -> String {
    error.replace('\\', "/")
}

fn execute_browser_render_plan(
    workspace_root: &Path,
    viewport: &TemplateViewportDto,
    plan: &BrowserRenderPlan,
    sidecars: &TemplateSidecars,
    runner: &dyn TemplateRenderRunner,
) -> Result<(), String> {
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let request_relative_path = format!("template_render/request_{}.json", unique_render_id());
    let request_path = storage
        .resolver()
        .resolve_bucket_path_for_write(FileBucket::Temp, &request_relative_path)?;
    let request = json!({
        "workspaceRoot": workspace_root.to_string_lossy().replace('\\', "/"),
        "entryPath": plan.entry_path,
        "outputPath": plan.output_path,
        "chromiumPath": sidecars.chromium_path.to_string_lossy().replace('\\', "/"),
        "viewport": viewport,
        "allowedResourceRoots": plan.allowed_resource_roots,
        "payload": plan.payload,
        "injectionScript": plan.injection_script,
        "policy": {
            "blockNetwork": true,
            "blockDownloads": true,
            "blockPopups": true,
            "blockClipboard": true,
            "persistentUserData": false,
            "retryOnBrowserCrash": true
        }
    });
    let request_text = serde_json::to_string_pretty(&request).map_err(|error| error.to_string())?;
    fs::write(&request_path, request_text).map_err(|error| error.to_string())?;

    let replacements = vec![
        (workspace_root.to_path_buf(), "<workspace>".to_string()),
        (
            sidecars.node_path.clone(),
            sidecar_relative_path(NODE_BINARY),
        ),
        (
            sidecars.chromium_path.clone(),
            sidecar_relative_path(CHROMIUM_BINARY),
        ),
        (
            sidecars.driver_path.clone(),
            sidecar_relative_path(PLAYWRIGHT_DRIVER),
        ),
        (
            request_path.clone(),
            format!("temp/{request_relative_path}"),
        ),
        (
            workspace_root.join(&plan.output_path),
            plan.output_path.clone(),
        ),
        (
            workspace_root.join(&plan.entry_path),
            plan.entry_path.clone(),
        ),
    ];

    let first = runner.run(
        &sidecars.node_path,
        &sidecars.driver_path,
        &request_path,
        &replacements,
    );
    let result = match first {
        Ok(()) => Ok(()),
        Err(error) if error.starts_with(TEMPLATE_BROWSER_CRASHED) => runner.run(
            &sidecars.node_path,
            &sidecars.driver_path,
            &request_path,
            &replacements,
        ),
        Err(error) => Err(error),
    };
    let _ = fs::remove_file(&request_path);
    result
}

fn sanitize_process_message(message: &str, replacements: &[(PathBuf, String)]) -> String {
    let mut sanitized = redact_text(message);
    for (absolute, relative) in replacements {
        let absolute_text = absolute.to_string_lossy();
        sanitized = sanitized.replace(absolute_text.as_ref(), relative);
        sanitized = sanitized.replace(&absolute_text.replace('\\', "/"), relative);
    }
    limit_process_message(&sanitized)
}

fn limit_process_message(message: &str) -> String {
    let lines = message.lines().collect::<Vec<_>>();
    let line_start = lines.len().saturating_sub(PROCESS_TEXT_MAX_LINES);
    let line_limited = lines[line_start..].join("\n");
    if line_limited.len() <= PROCESS_TEXT_MAX_BYTES {
        return line_limited;
    }
    take_last_utf8_bytes(&line_limited, PROCESS_TEXT_MAX_BYTES)
}

fn take_last_utf8_bytes(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut start = value.len() - max_bytes;
    while !value.is_char_boundary(start) {
        start += 1;
    }
    value[start..].to_string()
}

fn unique_render_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}_{}", std::process::id(), nanos)
}

fn current_timestamp_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn first_non_empty_line(value: &str) -> Option<String> {
    value
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

fn default_value_for_param(param_type: &str, raw: Option<&str>) -> Result<Value, String> {
    match param_type {
        "text" | "font" | "image" | "select" => {
            Ok(Value::String(raw.unwrap_or_default().to_string()))
        }
        "color" => {
            let value = raw.unwrap_or("#ffffff");
            if is_hex_color(value) {
                Ok(Value::String(value.to_string()))
            } else {
                Err(format!(
                    "template.param_invalid: invalid color default {value}."
                ))
            }
        }
        "number" | "range" => raw
            .unwrap_or("0")
            .parse::<f64>()
            .map(|number| json!(number))
            .map_err(|_| "template.param_invalid: number default must be numeric.".to_string()),
        "bool" => match raw.unwrap_or("false") {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err("template.param_invalid: bool default must be true or false.".to_string()),
        },
        "json" => {
            let Some(raw) = raw else {
                return Ok(json!({}));
            };
            serde_json::from_str(raw)
                .map_err(|error| format!("template.param_invalid: json default invalid: {error}"))
        }
        _ => Err(format!(
            "template.param_invalid: unsupported param type {param_type}."
        )),
    }
}

fn dictionary_code_for_select(name: &str, param_type: &str) -> Result<Option<String>, String> {
    if param_type != "select" {
        return Ok(None);
    }
    let code = match name {
        "position" | "subtitle_position" => "templatePosition",
        "transition_type" => "transitionType",
        "font_weight" => "fontWeight",
        "template_type" => "templateType",
        _ => {
            return Err(format!(
                "template.param_invalid: select param {name} must map to a unified dictionary."
            ))
        }
    };
    Ok(Some(code.to_string()))
}

fn seed_builtin_templates(workspace_root: &Path) -> Result<(), String> {
    for (template_type, aspect_ratio, template_id, html) in [
        (
            "cover",
            "vertical_9_16",
            "knowledge_bold",
            r##"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      :root { color-scheme: dark; --accent: #FFD54A; }
      * { box-sizing: border-box; }
      body {
        margin: 0;
        width: 1080px;
        height: 1920px;
        overflow: hidden;
        background: #111827;
        color: #fff;
        font-family: "Microsoft YaHei", "PingFang SC", sans-serif;
      }
      .cover {
        position: relative;
        width: 100%;
        height: 100%;
        padding: 132px 92px;
        display: flex;
        align-items: flex-end;
        background:
          linear-gradient(180deg, rgba(17,24,39,.25), rgba(17,24,39,.92)),
          radial-gradient(circle at 72% 18%, rgba(255,213,74,.24), transparent 32%),
          #111827;
      }
      .image {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        object-fit: cover;
        opacity: .72;
      }
      .content { position: relative; z-index: 1; max-width: 860px; }
      .rule { width: 132px; height: 12px; margin-bottom: 36px; background: var(--accent); }
      .title { margin: 0; font-size: 112px; line-height: 1.08; font-weight: 900; letter-spacing: 0; }
      .narration { margin-top: 34px; max-width: 780px; font-size: 38px; line-height: 1.45; color: rgba(255,255,255,.82); }
    </style>
  </head>
  <body data-template="cover" data-position="{{position:select=bottom}}" data-accent="{{accent_color:color=#FFD54A}}">
    <main class="cover">
      <img id="image" class="image" alt="" hidden />
      <section class="content">
        <div class="rule"></div>
        <h1 id="title" class="title">{{cover_title:text}}</h1>
        <div id="narration" class="narration"></div>
      </section>
    </main>
    <script>
      function applyTemplateData() {
        const payload = window.__VT_TEMPLATE_PAYLOAD__ || { data: {}, params: {} };
        const params = payload.params || {};
        const data = payload.data || {};
        document.documentElement.style.setProperty('--accent', String(params.accent_color || '#FFD54A'));
        document.getElementById('title').textContent = String(params.cover_title || data.title || '未命名封面');
        document.getElementById('narration').textContent = String(data.narration || '');
        const image = document.getElementById('image');
        if (data.image_path) {
          image.src = '../../../../../' + String(data.image_path);
          image.hidden = false;
        }
      }
      document.addEventListener('vt-template-data', applyTemplateData);
      applyTemplateData();
    </script>
  </body>
</html>"##,
        ),
        (
            "subtitle",
            "vertical_9_16",
            "karaoke_basic",
            r##"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8" />
    <style>
      body { margin: 0; width: 1080px; height: 1920px; background: transparent; font-family: "Microsoft YaHei", sans-serif; }
      .subtitle {
        position: absolute;
        left: 72px;
        right: 72px;
        bottom: {{subtitle_safe_bottom:number=240}}px;
        color: {{subtitle_color:color=#ffffff}};
        font-size: 56px;
        line-height: 1.28;
        text-align: center;
        font-weight: 800;
        text-shadow: 0 4px 0 rgba(0,0,0,.85), 0 0 18px rgba(0,0,0,.55);
      }
      .highlight { color: #FFD54A; }
    </style>
  </head>
  <body data-template="subtitle" data-position="{{subtitle_position:select=bottom}}">
    <div id="subtitle" class="subtitle"></div>
    <script>
      function applyTemplateData() {
        const payload = window.__VT_TEMPLATE_PAYLOAD__ || { data: {} };
        const chunks = payload.data.subtitle_chunks || [];
        document.getElementById('subtitle').textContent = String(chunks[0] || payload.data.narration || '');
      }
      document.addEventListener('vt-template-data', applyTemplateData);
      applyTemplateData();
    </script>
  </body>
</html>"##,
        ),
        (
            "frame",
            "vertical_9_16",
            "image_soft_zoom",
            r##"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8" />
    <style>
      body { margin: 0; width: 1080px; height: 1920px; overflow: hidden; background: #0f172a; }
      img { width: 100%; height: 100%; object-fit: cover; transform: scale({{zoom_amount:range=1.05}}); }
    </style>
  </head>
  <body data-template="frame" data-transition="{{transition_type:select=fade}}">
    <img id="image" alt="" hidden />
    <script>
      function applyTemplateData() {
        const payload = window.__VT_TEMPLATE_PAYLOAD__ || { data: {} };
        const image = document.getElementById('image');
        if (payload.data.image_path) {
          image.src = '../../../../../' + String(payload.data.image_path);
          image.hidden = false;
        }
      }
      document.addEventListener('vt-template-data', applyTemplateData);
      applyTemplateData();
    </script>
  </body>
</html>"##,
        ),
    ] {
        let path = workspace_root
            .join("templates")
            .join("builtin")
            .join(template_type)
            .join(aspect_ratio)
            .join(format!("{template_id}.html"));
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(path, html).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn controlled_template_entry_path(workspace_root: &Path, path: &Path) -> Result<String, String> {
    let templates_root = workspace_root.join("templates");
    let relative = path
        .strip_prefix(&templates_root)
        .map_err(|_| "template.resource_denied: template path is outside templates.".to_string())?
        .to_string_lossy()
        .replace('\\', "/");
    PathGuard::validate_relative_path(&relative)?;
    Ok(format!("templates/{relative}"))
}

fn normalize_aspect_ratio(value: &str) -> Result<String, String> {
    match value.trim() {
        "vertical_9_16" | "9:16" => Ok("vertical_9_16".to_string()),
        "horizontal_16_9" | "16:9" => Ok("horizontal_16_9".to_string()),
        "square_1_1" | "1:1" => Ok("square_1_1".to_string()),
        other => Err(format!(
            "template.param_invalid: unsupported aspectRatio {other}."
        )),
    }
}

fn normalize_template_type(value: &str) -> Result<String, String> {
    let value = value.trim();
    if TEMPLATE_TYPES.contains(&value) {
        Ok(value.to_string())
    } else {
        Err(format!(
            "template.param_invalid: unsupported templateType {value}."
        ))
    }
}

fn normalize_source_type(value: &str) -> Result<String, String> {
    match value.trim() {
        "builtin" | "user" => Ok(value.trim().to_string()),
        other => Err(format!(
            "template.param_invalid: unsupported sourceType {other}."
        )),
    }
}

fn viewport_for_aspect_ratio(aspect_ratio: &str) -> Result<TemplateViewportDto, String> {
    match aspect_ratio {
        "vertical_9_16" => Ok(TemplateViewportDto {
            width: 1080,
            height: 1920,
        }),
        "horizontal_16_9" => Ok(TemplateViewportDto {
            width: 1920,
            height: 1080,
        }),
        "square_1_1" => Ok(TemplateViewportDto {
            width: 1080,
            height: 1080,
        }),
        other => Err(format!(
            "template.param_invalid: unsupported aspectRatio {other}."
        )),
    }
}

fn validate_template_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
    {
        return Err("template.param_invalid: templateId must use snake_case.".to_string());
    }
    Ok(())
}

fn validate_param_name(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.starts_with('_')
        || value.ends_with('_')
        || !value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
    {
        return Err(format!(
            "template.param_invalid: param name {value} must use snake_case."
        ));
    }
    Ok(())
}

fn validate_param_type(value: &str) -> Result<(), String> {
    if TEMPLATE_PARAM_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "template.param_invalid: unsupported param type {value}."
        ))
    }
}

fn split_once(value: &str, delimiter: char) -> (&str, Option<&str>) {
    if let Some(index) = value.find(delimiter) {
        (&value[..index], Some(&value[index + 1..]))
    } else {
        (value, None)
    }
}

fn is_hex_color(value: &str) -> bool {
    let Some(value) = value.strip_prefix('#') else {
        return false;
    };
    matches!(value.len(), 3 | 6) && value.chars().all(|character| character.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::{
        list_template_manifests, parse_template_params_from_html, validate_template_params,
    };
    use crate::domain::template::{ListTemplateManifestsRequest, ValidateTemplateParamsRequest};
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_template_placeholder_dsl_and_ignores_builtin_params() {
        let params = parse_template_params_from_html(
            "{{title}}{{accent_color:color=#FFD54A}}{{subtitle_safe_bottom:number=240}}{{position:select=bottom}}",
        )
        .expect("params should parse");

        assert_eq!(params.len(), 3);
        assert_eq!(params[0].name, "accent_color");
        assert_eq!(params[0].param_type, "color");
        assert_eq!(params[1].default_value, json!(240.0));
        assert_eq!(
            params[2].dictionary_code.as_deref(),
            Some("templatePosition")
        );
    }

    #[test]
    fn rejects_select_without_unified_dictionary_mapping() {
        let error = parse_template_params_from_html("{{random_choice:select=a}}")
            .expect_err("select without dictionary mapping should fail");
        assert!(error.contains("unified dictionary"));
    }

    #[test]
    fn lists_seeded_builtin_templates_by_aspect_and_type() {
        let root = test_root("template_list");
        let manifests = list_template_manifests(
            &root,
            ListTemplateManifestsRequest {
                aspect_ratio: Some("9:16".to_string()),
                template_type: Some("cover".to_string()),
                source_type: Some("builtin".to_string()),
            },
        )
        .expect("templates should list");

        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].template_id, "knowledge_bold");
        assert_eq!(manifests[0].aspect_ratio, "vertical_9_16");
        assert_eq!(manifests[0].viewport.width, 1080);
        assert!(root
            .join("templates/builtin/cover/vertical_9_16/knowledge_bold.html")
            .is_file());

        cleanup_dir(root);
    }

    #[test]
    fn validates_template_params_and_normalizes_defaults() {
        let root = test_root("template_validate");
        let manifest = list_template_manifests(
            &root,
            ListTemplateManifestsRequest {
                aspect_ratio: Some("vertical_9_16".to_string()),
                template_type: Some("cover".to_string()),
                source_type: Some("builtin".to_string()),
            },
        )
        .expect("templates should list")
        .remove(0);

        let result = validate_template_params(ValidateTemplateParamsRequest {
            manifest,
            params: json!({ "cover_title": "标题", "accent_color": "#00ff99" }),
        })
        .expect("params should validate");

        assert!(result.valid);
        assert_eq!(result.normalized_params["cover_title"], "标题");
        assert_eq!(result.normalized_params["position"], "bottom");

        cleanup_dir(root);
    }

    #[test]
    fn rejects_user_template_with_invalid_param_name() {
        let root = test_root("template_invalid");
        let template_dir = root.join("templates/user/cover/vertical_9_16");
        fs::create_dir_all(&template_dir).expect("template dir");
        fs::write(template_dir.join("bad_name.html"), "{{Color:color=#fff}}")
            .expect("template should write");

        let error = list_template_manifests(
            &root,
            ListTemplateManifestsRequest {
                aspect_ratio: Some("vertical_9_16".to_string()),
                template_type: Some("cover".to_string()),
                source_type: Some("user".to_string()),
            },
        )
        .expect_err("invalid param name should fail");
        assert!(error.contains("snake_case"));

        cleanup_dir(root);
    }

    #[test]
    fn render_template_rejects_external_or_file_url_resources_before_sidecar() {
        let root = test_root("template_resource_denied");
        let error = super::render_template(
            &root,
            crate::domain::template::RenderTemplateRequest {
                template_id: "knowledge_bold".to_string(),
                aspect_ratio: "vertical_9_16".to_string(),
                template_type: "cover".to_string(),
                params: json!({ "cover_title": "标题" }),
                data: crate::domain::template::TemplateRenderDataDto {
                    image_path: Some("file:///C:/Windows/win.ini".to_string()),
                    ..Default::default()
                },
                output_path: "cache/preview.png".to_string(),
            },
        )
        .expect_err("file URL must be rejected");

        assert!(error.starts_with("template.resource_denied:"));
        cleanup_dir(root);
    }

    #[test]
    fn render_template_reports_missing_template_sidecar_without_fake_output() {
        let root = test_root("template_sidecar_missing");
        let image_path = root.join("projects/project_a/images/source.png");
        fs::create_dir_all(image_path.parent().unwrap()).expect("image dir");
        fs::write(&image_path, "png").expect("image should write");

        let error = super::render_template(
            &root,
            crate::domain::template::RenderTemplateRequest {
                template_id: "knowledge_bold".to_string(),
                aspect_ratio: "9:16".to_string(),
                template_type: "cover".to_string(),
                params: json!({ "cover_title": "标题" }),
                data: crate::domain::template::TemplateRenderDataDto {
                    image_path: Some("projects/project_a/images/source.png".to_string()),
                    title: Some("<b>不能执行</b>".to_string()),
                    ..Default::default()
                },
                output_path: "cache/preview.png".to_string(),
            },
        )
        .expect_err("missing sidecar should block real render");

        assert!(error.starts_with("template.sidecar_missing:"));
        assert!(error.contains("sidecars/node.exe"));
        assert!(!root.join("cache/preview.png").exists());
        cleanup_dir(root);
    }

    #[test]
    fn check_template_sidecars_reports_missing_and_ready_without_system_path() {
        let missing_root = test_root("template_check_missing");
        let missing =
            super::check_template_sidecars_with_runner(&missing_root, &FakeTemplateVersionRunner)
                .expect("missing sidecars should return status");
        assert!(!missing.ready);
        assert_eq!(missing.node.relative_path, "sidecars/node.exe");
        assert_eq!(
            missing.node.error_code.as_deref(),
            Some("template.sidecar_missing")
        );
        assert!(!missing
            .node
            .message
            .as_deref()
            .unwrap_or_default()
            .contains(&missing_root.display().to_string()));
        cleanup_dir(missing_root);

        let ready_root = test_root("template_check_ready");
        write_fake_template_sidecars(&ready_root);
        let ready =
            super::check_template_sidecars_with_runner(&ready_root, &FakeTemplateVersionRunner)
                .expect("fake sidecars should be ready");
        assert!(ready.ready);
        assert_eq!(ready.node.version.as_deref(), Some("node.exe fake version"));
        assert_eq!(
            ready.playwright_driver.version.as_deref(),
            Some("playwright-driver.js fake version")
        );
        cleanup_dir(ready_root);
    }

    #[test]
    fn render_template_executes_sidecar_and_returns_real_output_path() {
        let root = test_root("template_render_success");
        write_fake_template_sidecars(&root);
        let runner = FakeTemplateRunner::writes_output();

        let response = super::render_template_with_runner(
            &root,
            crate::domain::template::RenderTemplateRequest {
                template_id: "knowledge_bold".to_string(),
                aspect_ratio: "vertical_9_16".to_string(),
                template_type: "cover".to_string(),
                params: json!({ "cover_title": "标题" }),
                data: crate::domain::template::TemplateRenderDataDto {
                    title: Some("<b>不能执行</b>".to_string()),
                    ..Default::default()
                },
                output_path: "cache/preview.png".to_string(),
            },
            &runner,
        )
        .expect("fake sidecar should render");

        assert_eq!(response.rendered_frame_path, "cache/preview.png");
        assert_eq!(response.width, 1080);
        assert_eq!(response.height, 1920);
        assert_eq!(runner.calls(), 1);
        assert!(root.join("cache/preview.png").is_file());
        assert!(runner
            .last_request_text()
            .expect("request should be captured")
            .contains("\"blockNetwork\": true"));

        cleanup_dir(root);
    }

    #[test]
    fn render_template_retries_once_after_browser_crash() {
        let root = test_root("template_render_retry");
        write_fake_template_sidecars(&root);
        let runner = FakeTemplateRunner::crash_once_then_write_output();

        let response = super::render_template_with_runner(
            &root,
            crate::domain::template::RenderTemplateRequest {
                template_id: "knowledge_bold".to_string(),
                aspect_ratio: "vertical_9_16".to_string(),
                template_type: "cover".to_string(),
                params: json!({ "cover_title": "标题" }),
                data: crate::domain::template::TemplateRenderDataDto::default(),
                output_path: "cache/retry.png".to_string(),
            },
            &runner,
        )
        .expect("browser crash should retry once");

        assert_eq!(response.rendered_frame_path, "cache/retry.png");
        assert_eq!(runner.calls(), 2);
        assert!(root.join("cache/retry.png").is_file());
        cleanup_dir(root);
    }

    #[test]
    fn render_template_reports_output_missing_after_sidecar_success() {
        let root = test_root("template_output_missing");
        write_fake_template_sidecars(&root);
        let runner = FakeTemplateRunner::succeeds_without_output();

        let error = super::render_template_with_runner(
            &root,
            crate::domain::template::RenderTemplateRequest {
                template_id: "knowledge_bold".to_string(),
                aspect_ratio: "vertical_9_16".to_string(),
                template_type: "cover".to_string(),
                params: json!({ "cover_title": "标题" }),
                data: crate::domain::template::TemplateRenderDataDto::default(),
                output_path: "cache/missing.png".to_string(),
            },
            &runner,
        )
        .expect_err("missing output should be explicit");

        assert!(error.starts_with("template.output_missing:"));
        assert_eq!(runner.calls(), 1);
        cleanup_dir(root);
    }

    #[test]
    fn render_template_sanitizes_sidecar_failure_logs() {
        let root = test_root("template_render_fail");
        write_fake_template_sidecars(&root);
        let absolute_secret_path = root.join("projects/project_a/secret.png");
        let runner = FakeTemplateRunner::fails(format!(
            "failed at {} with Authorization: Bearer sk-live-secret-token-123456",
            absolute_secret_path.display()
        ));

        let error = super::render_template_with_runner(
            &root,
            crate::domain::template::RenderTemplateRequest {
                template_id: "knowledge_bold".to_string(),
                aspect_ratio: "vertical_9_16".to_string(),
                template_type: "cover".to_string(),
                params: json!({ "cover_title": "标题" }),
                data: crate::domain::template::TemplateRenderDataDto::default(),
                output_path: "cache/fail.png".to_string(),
            },
            &runner,
        )
        .expect_err("runner failure should surface sanitized message");

        assert!(error.starts_with("template.render_failed:"));
        assert!(!error.contains(&root.display().to_string()));
        assert!(!error.contains("sk-live"));
        assert!(error.contains("***REDACTED***"));
        cleanup_dir(root);
    }

    #[test]
    #[ignore = "requires real sidecars/node.exe, sidecars/chromium.exe, and sidecars/node_modules/playwright-core"]
    fn real_template_sidecar_renders_default_cover_png() {
        let workspace_root = std::env::current_dir()
            .expect("current dir should exist")
            .parent()
            .expect("repo root should be parent of src-tauri")
            .to_path_buf();
        let response = super::render_template(
            &workspace_root,
            crate::domain::template::RenderTemplateRequest {
                template_id: "knowledge_bold".to_string(),
                aspect_ratio: "vertical_9_16".to_string(),
                template_type: "cover".to_string(),
                params: json!({
                    "cover_title": "默认模板截图",
                    "accent_color": "#FFD54A",
                    "position": "bottom"
                }),
                data: crate::domain::template::TemplateRenderDataDto {
                    title: Some("<script>window.__xss = true</script>默认模板截图".to_string()),
                    ..Default::default()
                },
                output_path: "cache/template_smoke/knowledge_bold.png".to_string(),
            },
        )
        .expect("real template sidecar should render default template");

        assert_eq!(
            response.rendered_frame_path,
            "cache/template_smoke/knowledge_bold.png"
        );
        let output = workspace_root.join(&response.rendered_frame_path);
        let bytes = fs::read(output).expect("rendered png should read");
        assert!(bytes.starts_with(&[137, 80, 78, 71]));
    }

    #[test]
    fn browser_render_plan_uses_data_payload_event_not_inner_html() {
        let plan = super::build_browser_render_plan(
            Path::new("."),
            "templates/builtin/cover/vertical_9_16/knowledge_bold.html",
            "cache/preview.png",
            &crate::domain::template::TemplateRenderDataDto {
                title: Some("<script>alert(1)</script>".to_string()),
                ..Default::default()
            },
            &json!({ "cover_title": "<img src=x onerror=alert(1)>" }),
        )
        .expect("render plan should build");

        assert!(plan.injection_script.contains("__VT_TEMPLATE_PAYLOAD__"));
        assert!(plan.injection_script.contains("vt-template-data"));
        assert!(!plan.injection_script.contains("innerHTML"));
        assert!(!plan.injection_script.contains("document.write"));
    }

    #[derive(Clone)]
    struct FakeTemplateRunner {
        mode: FakeTemplateRunnerMode,
        calls: Arc<Mutex<usize>>,
        last_request_text: Arc<Mutex<Option<String>>>,
    }

    #[derive(Clone)]
    enum FakeTemplateRunnerMode {
        WriteOutput,
        CrashOnceThenWriteOutput,
        NoOutput,
        Fail(String),
    }

    impl FakeTemplateRunner {
        fn writes_output() -> Self {
            Self::new(FakeTemplateRunnerMode::WriteOutput)
        }

        fn crash_once_then_write_output() -> Self {
            Self::new(FakeTemplateRunnerMode::CrashOnceThenWriteOutput)
        }

        fn succeeds_without_output() -> Self {
            Self::new(FakeTemplateRunnerMode::NoOutput)
        }

        fn fails(message: String) -> Self {
            Self::new(FakeTemplateRunnerMode::Fail(message))
        }

        fn new(mode: FakeTemplateRunnerMode) -> Self {
            Self {
                mode,
                calls: Arc::new(Mutex::new(0)),
                last_request_text: Arc::new(Mutex::new(None)),
            }
        }

        fn calls(&self) -> usize {
            *self.calls.lock().expect("calls should lock")
        }

        fn last_request_text(&self) -> Option<String> {
            self.last_request_text
                .lock()
                .expect("request text should lock")
                .clone()
        }
    }

    impl super::TemplateRenderRunner for FakeTemplateRunner {
        fn run(
            &self,
            _node_path: &Path,
            _driver_path: &Path,
            request_path: &Path,
            replacements: &[(PathBuf, String)],
        ) -> Result<(), String> {
            let mut calls = self.calls.lock().expect("calls should lock");
            *calls += 1;
            let current_call = *calls;
            drop(calls);

            let request_text = fs::read_to_string(request_path).expect("request should read");
            *self
                .last_request_text
                .lock()
                .expect("request text should lock") = Some(request_text.clone());
            let request: serde_json::Value =
                serde_json::from_str(&request_text).expect("request json should parse");
            let workspace_root = request["workspaceRoot"]
                .as_str()
                .expect("workspaceRoot should exist");
            let output_path = request["outputPath"]
                .as_str()
                .expect("outputPath should exist");

            match &self.mode {
                FakeTemplateRunnerMode::WriteOutput => {
                    write_render_output(workspace_root, output_path);
                    Ok(())
                }
                FakeTemplateRunnerMode::CrashOnceThenWriteOutput if current_call == 1 => {
                    Err("template.browser_crashed: browser_crashed".to_string())
                }
                FakeTemplateRunnerMode::CrashOnceThenWriteOutput => {
                    write_render_output(workspace_root, output_path);
                    Ok(())
                }
                FakeTemplateRunnerMode::NoOutput => Ok(()),
                FakeTemplateRunnerMode::Fail(message) => Err(format!(
                    "template.render_failed: {}",
                    super::sanitize_process_message(message, replacements)
                )),
            }
        }
    }

    struct FakeTemplateVersionRunner;

    impl super::TemplateSidecarVersionRunner for FakeTemplateVersionRunner {
        fn run_version(&self, _binary_path: &Path, binary_name: &str) -> Result<String, String> {
            Ok(format!("{binary_name} fake version"))
        }
    }

    fn write_render_output(workspace_root: &str, output_path: &str) {
        let output = PathBuf::from(workspace_root).join(output_path);
        fs::create_dir_all(output.parent().expect("output parent should exist"))
            .expect("output dir should write");
        fs::write(output, "png").expect("output should write");
    }

    fn write_fake_template_sidecars(root: &Path) {
        let sidecars = root.join("sidecars");
        fs::create_dir_all(&sidecars).expect("sidecars dir should exist");
        for binary in [
            super::NODE_BINARY,
            super::CHROMIUM_BINARY,
            super::PLAYWRIGHT_DRIVER,
        ] {
            fs::write(sidecars.join(binary), "fake").expect("fake sidecar should write");
        }
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

    fn cleanup_dir(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
