use crate::core::error::TaskError;
use crate::db::provider_repository::{ProviderModelRecord, ProviderRepository};
use crate::db::Database;
use crate::domain::provider::{
    ImageProviderRequest, ImageProviderResponse, LlmChatRequest, LlmChatResponse,
    ProviderDryRunRequest, ProviderDryRunResponse, ProviderGenerationTestRequest,
    ProviderGenerationTestResponse, ProviderRequestContext, TtsProviderRequest,
    TtsProviderResponse, VideoProviderRequest, VideoProviderResponse, VlmAnalyzeRequest,
    VlmAnalyzeResponse, WorkflowProviderRequest, WorkflowProviderResponse,
};
use crate::security::secret_guard::redact_text;
use crate::services::keyring_service::KeyringService;
use crate::services::task_cancellation::CancellationToken;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ProviderManager<'a> {
    database: &'a Database,
    keyring_service: &'a KeyringService,
}

impl<'a> ProviderManager<'a> {
    pub fn new(database: &'a Database, keyring_service: &'a KeyringService) -> Self {
        Self {
            database,
            keyring_service,
        }
    }

    pub fn call_llm(
        &self,
        request: LlmChatRequest,
        token: &CancellationToken,
    ) -> Result<LlmChatResponse, TaskError> {
        let provider = self.prepare_provider(&request.context, "llm", None, token)?;
        ProviderAdapterPlaceholder::new(provider).call_llm(request, token)
    }

    pub fn generate_image(
        &self,
        request: ImageProviderRequest,
        token: &CancellationToken,
    ) -> Result<ImageProviderResponse, TaskError> {
        let provider = self.prepare_provider(
            &request.context,
            "image",
            Some(ProviderRequestValidation::Image {
                ability_type: "text_to_image",
                aspect_ratio: &request.aspect_ratio,
                resolution: None,
                duration_seconds: None,
                fps: None,
                reference_image_count: request.reference_images.len(),
            }),
            token,
        )?;
        ProviderAdapterPlaceholder::new(provider).generate_image(request, token)
    }

    pub fn generate_video(
        &self,
        request: VideoProviderRequest,
        token: &CancellationToken,
    ) -> Result<VideoProviderResponse, TaskError> {
        let provider = self.prepare_provider(
            &request.context,
            "video",
            Some(ProviderRequestValidation::Video {
                ability_type: &request.ability_type,
                aspect_ratio: &request.aspect_ratio,
                resolution: request.resolution.as_deref(),
                duration_seconds: Some(request.duration_seconds),
                fps: request.fps,
                reference_image_count: request.input_images.len(),
            }),
            token,
        )?;
        ProviderAdapterPlaceholder::new(provider).generate_video(request, token)
    }

    pub fn generate_tts(
        &self,
        request: TtsProviderRequest,
        token: &CancellationToken,
    ) -> Result<TtsProviderResponse, TaskError> {
        let provider = self.prepare_provider(&request.context, "tts", None, token)?;
        ProviderAdapterPlaceholder::new(provider).generate_tts(request, token)
    }

    pub fn analyze_asset(
        &self,
        request: VlmAnalyzeRequest,
        token: &CancellationToken,
    ) -> Result<VlmAnalyzeResponse, TaskError> {
        let provider = self.prepare_provider(&request.context, "vlm", None, token)?;
        ProviderAdapterPlaceholder::new(provider).analyze_asset(request, token)
    }

    pub fn run_workflow(
        &self,
        request: WorkflowProviderRequest,
        token: &CancellationToken,
    ) -> Result<WorkflowProviderResponse, TaskError> {
        let provider = self.prepare_provider(&request.context, "workflow", None, token)?;
        if provider.kind != "workflow" {
            return Err(provider_error(
                "provider.capability_unsupported",
                "Workflow calls must use ProviderKind=workflow.",
                &request.context,
                Some(json!({ "providerKind": provider.kind })),
            ));
        }
        validate_workflow_preset_for_request(
            self.database,
            &provider,
            &request.workflow_preset_id,
            &request.workflow_vendor,
            &request.context,
        )?;
        ProviderAdapterPlaceholder::new(provider).run_workflow(request, token)
    }

    pub fn dry_run(
        &self,
        request: ProviderDryRunRequest,
    ) -> Result<ProviderDryRunResponse, TaskError> {
        let response = self.generation_test(ProviderGenerationTestRequest {
            provider_id: request.provider_id,
            provider_kind: request.provider_kind,
            provider_model_id: request.provider_model_id,
            workflow_preset_id: request.workflow_preset_id,
            test_mode: "dry_run".to_string(),
            real_generate_confirmed: Some(false),
            confirm_token: None,
            simulate_failure: request.simulate_failure,
            simulate_cancelled: request.simulate_cancelled,
        })?;

        Ok(ProviderDryRunResponse {
            trace_id: response.trace_id,
            provider_id: response.provider_id,
            provider_kind: response.provider_kind,
            status: response.status,
            message: response.message,
            output_summary: response.output_summary,
        })
    }

    pub fn generation_test(
        &self,
        request: ProviderGenerationTestRequest,
    ) -> Result<ProviderGenerationTestResponse, TaskError> {
        let test_mode = request.test_mode.trim().to_string();
        let trace_id = create_trace_id();
        let context = ProviderRequestContext {
            trace_id: trace_id.clone(),
            task_id: None,
            task_step_id: None,
            project_id: None,
            item_id: None,
            provider_id: request.provider_id.clone(),
            provider_model_id: request.provider_model_id.clone(),
            workflow_preset_id: request.workflow_preset_id.clone(),
            timeout_seconds: Some(10),
            idempotency_key: Some(format!("{test_mode}_{trace_id}")),
        };

        if test_mode != "dry_run" && test_mode != "real_generate" {
            return Err(provider_error(
                "provider.test_mode_unsupported",
                "Provider test mode must be dry_run or real_generate.",
                &context,
                Some(json!({ "testMode": test_mode })),
            ));
        }
        if request.provider_model_id.is_some() && request.workflow_preset_id.is_some() {
            return Err(provider_error(
                "provider.source_conflict",
                "provider_model_id and workflow_preset_id cannot both be set.",
                &context,
                Some(json!({ "providerKind": request.provider_kind })),
            ));
        }
        if request.provider_kind != "workflow" && request.workflow_preset_id.is_some() {
            return Err(provider_error(
                "workflow.provider_unavailable",
                "workflow_preset_id requires ProviderKind=workflow.",
                &context,
                Some(json!({ "providerKind": request.provider_kind })),
            ));
        }
        if request.provider_kind == "workflow" && request.provider_model_id.is_some() {
            return Err(provider_error(
                "provider.capability_unsupported",
                "workflow dry_run cannot use provider_model_id.",
                &context,
                Some(json!({ "providerKind": request.provider_kind })),
            ));
        }
        let real_generate_confirmed = request.real_generate_confirmed.unwrap_or(false);
        if test_mode == "real_generate" && !real_generate_confirmed {
            return Err(provider_error(
                "provider.real_generate_confirmation_required",
                "real_generate requires explicit user confirmation.",
                &context,
                Some(json!({ "testMode": test_mode })),
            ));
        }
        if test_mode == "real_generate"
            && request.provider_kind == "video"
            && request.confirm_token.as_deref() != Some("REAL_GENERATE_VIDEO")
        {
            return Err(provider_error(
                "provider.video_real_generate_confirmation_required",
                "video real_generate requires the REAL_GENERATE_VIDEO confirmation token.",
                &context,
                Some(json!({ "providerKind": request.provider_kind, "testMode": test_mode })),
            ));
        }
        if test_mode == "real_generate"
            && request.provider_kind == "workflow"
            && request.confirm_token.as_deref() != Some("REAL_GENERATE_VIDEO")
            && workflow_preset_is_video(self.database, request.workflow_preset_id.as_deref())
                .map_err(|message| {
                    provider_error("workflow.config_error", &message, &context, None)
                })?
        {
            return Err(provider_error(
                "provider.video_real_generate_confirmation_required",
                "video workflow real_generate requires the REAL_GENERATE_VIDEO confirmation token.",
                &context,
                Some(json!({ "providerKind": request.provider_kind, "testMode": test_mode })),
            ));
        }

        let output_prefix = if test_mode == "real_generate" {
            "real-generate-test"
        } else {
            "dry-run"
        };
        let token = CancellationToken::new(format!("provider_{test_mode}_{trace_id}"));
        if request.simulate_cancelled.unwrap_or(false) {
            token.cancel();
        }
        if request.simulate_failure.unwrap_or(false) {
            return Err(provider_error(
                "provider.server_error",
                "Provider test simulated failure.",
                &context,
                Some(json!({ "mode": test_mode, "providerId": request.provider_id })),
            ));
        }

        let raw_output_summary = match request.provider_kind.as_str() {
            "llm" => {
                let response = self.call_llm(
                    LlmChatRequest {
                        context: context.clone(),
                        messages: vec![],
                        temperature: Some(0.1),
                        max_tokens: Some(16),
                        response_format: Some("text".to_string()),
                        json_schema: None,
                    },
                    &token,
                )?;
                json!({ "contentLength": response.content.len(), "rawResponseSummary": response.raw_response_summary })
            }
            "image" => {
                let response = self.generate_image(
                    ImageProviderRequest {
                        context: context.clone(),
                        prompt: "dry run image prompt".to_string(),
                        negative_prompt: None,
                        aspect_ratio: "9:16".to_string(),
                        width: Some(720),
                        height: Some(1280),
                        seed: Some(1),
                        reference_images: vec![],
                        output_path: format!("{output_prefix}/provider/image.png"),
                    },
                    &token,
                )?;
                response.provider_output_summary
            }
            "video" => {
                let response = self.generate_video(
                    VideoProviderRequest {
                        context: context.clone(),
                        ability_type: "image_to_video".to_string(),
                        prompt: "dry run video prompt".to_string(),
                        negative_prompt: None,
                        aspect_ratio: "9:16".to_string(),
                        duration_seconds: 3.0,
                        resolution: Some("720p".to_string()),
                        fps: Some(24),
                        seed: Some(1),
                        input_images: vec![],
                        input_video_path: None,
                        input_audio_path: None,
                        output_path: format!("{output_prefix}/provider/video.mp4"),
                    },
                    &token,
                )?;
                response.provider_output_summary
            }
            "tts" => {
                let response = self.generate_tts(
                    TtsProviderRequest {
                        context: context.clone(),
                        text: "dry run".to_string(),
                        content_language: "en-US".to_string(),
                        voice_id: "dummy".to_string(),
                        speed: Some(1.0),
                        pitch: None,
                        volume: None,
                        format: "mp3".to_string(),
                        sample_rate: Some(24000),
                        output_path: format!("{output_prefix}/provider/audio.mp3"),
                    },
                    &token,
                )?;
                json!({ "audioPath": response.audio_path, "durationSeconds": response.audio_duration_seconds })
            }
            "vlm" => {
                let response = self.analyze_asset(
                    VlmAnalyzeRequest {
                        context: context.clone(),
                        input_path: "dry-run/provider/input.png".to_string(),
                        prompt: "describe".to_string(),
                        output_schema: None,
                    },
                    &token,
                )?;
                json!({ "description": response.description, "tags": response.tags })
            }
            "workflow" => {
                let response = self.run_workflow(
                    WorkflowProviderRequest {
                        context: context.clone(),
                        workflow_preset_id: request.workflow_preset_id.clone().ok_or_else(|| {
                            provider_error(
                                "workflow.preset_required",
                                "workflow provider tests require workflow_preset_id.",
                                &context,
                                Some(json!({ "providerKind": request.provider_kind })),
                            )
                        })?,
                        workflow_vendor: "comfyui".to_string(),
                        params: json!({ "mode": test_mode.as_str() }),
                        output_path: format!("{output_prefix}/provider/workflow-output.json"),
                    },
                    &token,
                )?;
                response.metadata
            }
            _ => {
                return Err(provider_error(
                    "provider.capability_unsupported",
                    "Unsupported provider kind for dry run.",
                    &context,
                    Some(json!({ "providerKind": request.provider_kind })),
                ))
            }
        };

        let output_summary = with_test_summary_metadata(
            raw_output_summary,
            &test_mode,
            real_generate_confirmed,
            false,
        );

        Ok(ProviderGenerationTestResponse {
            trace_id,
            test_mode: test_mode.clone(),
            provider_id: request.provider_id,
            provider_kind: request.provider_kind,
            status: "succeeded".to_string(),
            message: if test_mode == "real_generate" {
                "Provider real_generate test completed.".to_string()
            } else {
                "Provider dry_run completed.".to_string()
            },
            output_summary,
            billable: false,
            real_generate_confirmed: test_mode == "real_generate" && real_generate_confirmed,
        })
    }

    fn prepare_provider(
        &self,
        context: &ProviderRequestContext,
        expected_kind: &str,
        validation: Option<ProviderRequestValidation<'_>>,
        token: &CancellationToken,
    ) -> Result<ResolvedProvider, TaskError> {
        token.throw_if_cancelled().map_err(|message| {
            provider_error("provider.cancelled", &message, context, Some(json!({})))
        })?;

        let provider = ProviderRepository::new(self.database)
            .list_providers()
            .map_err(|message| provider_error("provider.config_error", &message, context, None))?
            .into_iter()
            .find(|provider| provider.provider_id == context.provider_id)
            .ok_or_else(|| {
                provider_error(
                    "provider.not_found",
                    "Provider config was not found.",
                    context,
                    None,
                )
            })?;

        if !provider.enabled || provider.status == "disabled" {
            return Err(provider_error(
                "provider.disabled",
                "Provider is disabled.",
                context,
                Some(json!({ "providerId": provider.provider_id })),
            ));
        }

        if provider.kind != expected_kind {
            return Err(provider_error(
                "provider.capability_unsupported",
                "Provider kind does not match requested capability.",
                context,
                Some(json!({ "expectedKind": expected_kind, "actualKind": provider.kind })),
            ));
        }

        let resolved_model = if let Some(provider_model_id) = context.provider_model_id.as_deref() {
            let model = resolve_provider_model(
                self.database,
                &provider.provider_id,
                provider_model_id,
                context,
            )?;
            validate_provider_model_for_request(
                &provider,
                &model,
                expected_kind,
                validation,
                context,
            )?;
            Some(ResolvedProviderModel {
                model_id: model.model_id,
                provider_model_id: model.provider_model_id,
                capability: model.capability,
            })
        } else {
            None
        };

        let has_secret = match provider.auth_type.as_str() {
            "none" => false,
            _ => {
                let key_alias = provider.key_alias.as_deref().ok_or_else(|| {
                    provider_error(
                        "provider.auth_failed",
                        "Provider requires key_alias.",
                        context,
                        Some(json!({ "providerId": provider.provider_id })),
                    )
                })?;
                self.keyring_service
                    .read_provider_secret(key_alias)
                    .map_err(|message| {
                        provider_error("provider.auth_failed", &message, context, None)
                    })?
                    .is_some()
            }
        };

        if provider.auth_type != "none" && !has_secret {
            return Err(provider_error(
                "provider.auth_failed",
                "Provider secret was not found in keyring.",
                context,
                Some(json!({ "keyAlias": provider.key_alias })),
            ));
        }

        Ok(ResolvedProvider {
            provider_id: provider.provider_id,
            vendor: provider.vendor.clone(),
            kind: provider.kind,
            display_name: provider.display_name,
            auth_type: provider.auth_type,
            adapter: read_model_string(&provider.config_json, &["adapter"])
                .unwrap_or_else(|| provider.vendor.clone()),
            has_secret,
            model: resolved_model,
        })
    }
}

#[derive(Debug, Clone)]
struct ResolvedProvider {
    provider_id: String,
    vendor: String,
    kind: String,
    display_name: String,
    auth_type: String,
    adapter: String,
    has_secret: bool,
    model: Option<ResolvedProviderModel>,
}

#[derive(Debug, Clone)]
struct ResolvedProviderModel {
    model_id: String,
    provider_model_id: String,
    capability: String,
}

#[derive(Debug, Clone, Copy)]
enum ProviderRequestValidation<'a> {
    Image {
        ability_type: &'a str,
        aspect_ratio: &'a str,
        resolution: Option<&'a str>,
        duration_seconds: Option<f64>,
        fps: Option<u32>,
        reference_image_count: usize,
    },
    Video {
        ability_type: &'a str,
        aspect_ratio: &'a str,
        resolution: Option<&'a str>,
        duration_seconds: Option<f64>,
        fps: Option<u32>,
        reference_image_count: usize,
    },
}

struct ProviderAdapterPlaceholder {
    provider: ResolvedProvider,
}

impl ProviderAdapterPlaceholder {
    fn new(provider: ResolvedProvider) -> Self {
        Self { provider }
    }

    fn call_llm(
        &self,
        request: LlmChatRequest,
        token: &CancellationToken,
    ) -> Result<LlmChatResponse, TaskError> {
        self.reject_unimplemented_adapter("llm", &request.context)?;
        self.check(token, &request.context)?;
        Ok(LlmChatResponse {
            content: format!("unimplemented llm adapter for {}", self.provider.display_name),
            parsed_json: None,
            usage: Some(json!({ "promptTokens": 0, "completionTokens": 0, "totalTokens": 0 })),
            raw_response_summary: Some(self.summary("llm")),
        })
    }

    fn generate_image(
        &self,
        request: ImageProviderRequest,
        token: &CancellationToken,
    ) -> Result<ImageProviderResponse, TaskError> {
        self.reject_unimplemented_adapter("image", &request.context)?;
        self.check(token, &request.context)?;
        Ok(ImageProviderResponse {
            image_path: request.output_path,
            seed: request.seed,
            width: request.width,
            height: request.height,
            file_size: Some(0),
            provider_output_summary: self.summary("image"),
        })
    }

    fn generate_video(
        &self,
        request: VideoProviderRequest,
        token: &CancellationToken,
    ) -> Result<VideoProviderResponse, TaskError> {
        self.reject_unimplemented_adapter("video", &request.context)?;
        self.check(token, &request.context)?;
        Ok(VideoProviderResponse {
            video_path: request.output_path,
            duration_seconds: request.duration_seconds,
            fps: request.fps,
            width: None,
            height: None,
            file_size: Some(0),
            provider_output_summary: self.summary("video"),
        })
    }

    fn generate_tts(
        &self,
        request: TtsProviderRequest,
        token: &CancellationToken,
    ) -> Result<TtsProviderResponse, TaskError> {
        self.reject_unimplemented_adapter("tts", &request.context)?;
        self.check(token, &request.context)?;
        Ok(TtsProviderResponse {
            audio_path: request.output_path,
            audio_duration_seconds: 1.0,
            format: request.format,
            sample_rate: request.sample_rate,
            file_size: Some(0),
        })
    }

    fn analyze_asset(
        &self,
        request: VlmAnalyzeRequest,
        token: &CancellationToken,
    ) -> Result<VlmAnalyzeResponse, TaskError> {
        self.reject_unimplemented_adapter("vlm", &request.context)?;
        self.check(token, &request.context)?;
        if is_style_reference_analysis_request(&request) {
            return Ok(VlmAnalyzeResponse {
                description: "Style reference analysis focused on reusable visual treatment; identity, logo, and private details were not returned.".to_string(),
                parsed_json: Some(json!({
                    "style_prompt": "clean cinematic short-video still, refined texture, consistent subject-background separation",
                    "color_palette": ["warm neutral", "muted green", "charcoal accent"],
                    "lighting": "soft directional light with gentle shadow falloff",
                    "composition": "vertical frame, clear focal zone, balanced negative space",
                    "negative_prompt_suggestion": "low resolution, distorted geometry, watermark, logo, private details",
                    "warnings": ["style_reference.analysis_scope_only"]
                })),
                tags: vec!["style_reference_analysis".to_string()],
            });
        }
        Ok(VlmAnalyzeResponse {
            description: "unimplemented visual analysis".to_string(),
            parsed_json: None,
            tags: vec![],
        })
    }

    fn run_workflow(
        &self,
        request: WorkflowProviderRequest,
        token: &CancellationToken,
    ) -> Result<WorkflowProviderResponse, TaskError> {
        self.reject_unimplemented_adapter("workflow", &request.context)?;
        self.check(token, &request.context)?;
        Ok(WorkflowProviderResponse {
            output_path: request.output_path,
            output_kind: "metadata".to_string(),
            metadata: self.summary("workflow"),
        })
    }

    fn check(
        &self,
        token: &CancellationToken,
        context: &ProviderRequestContext,
    ) -> Result<(), TaskError> {
        token.throw_if_cancelled().map_err(|message| {
            provider_error("provider.cancelled", &message, context, Some(json!({})))
        })
    }

    fn reject_unimplemented_adapter(
        &self,
        capability: &str,
        context: &ProviderRequestContext,
    ) -> Result<(), TaskError> {
        let adapter = self.provider.adapter.to_ascii_lowercase();
        let vendor = self.provider.vendor.to_ascii_lowercase();
        if adapter == "dummy"
            || adapter == "mock"
            || adapter == "controlled_fake"
            || vendor == "dummy"
            || vendor == "mock"
            || vendor == "controlled_fake"
        {
            return Err(provider_error(
                "provider.config_error",
                "Runtime generation requires a real provider adapter; dummy/mock providers are not allowed.",
                context,
                Some(json!({
                    "capability": capability,
                    "providerId": self.provider.provider_id,
                    "vendor": self.provider.vendor,
                    "adapter": self.provider.adapter
                })),
            ));
        }

        Err(provider_error(
            "provider.config_error",
            "Provider adapter is not implemented yet; configure a real adapter before running generation.",
            context,
            Some(json!({
                "capability": capability,
                "providerId": self.provider.provider_id,
                "vendor": self.provider.vendor,
                "adapter": self.provider.adapter
            })),
        ))
    }

    fn summary(&self, capability: &str) -> Value {
        json!({
            "adapter": "unimplemented",
            "capability": capability,
            "providerId": self.provider.provider_id,
            "vendor": self.provider.vendor,
            "authType": self.provider.auth_type,
            "hasSecret": self.provider.has_secret,
            "model": self.provider.model.as_ref().map(|model| json!({
                "modelId": model.model_id,
                "providerModelId": model.provider_model_id,
                "capability": model.capability
            }))
        })
    }
}

fn is_style_reference_analysis_request(request: &VlmAnalyzeRequest) -> bool {
    let prompt = request.prompt.to_ascii_lowercase();
    prompt.contains("style bible")
        || prompt.contains("style reference")
        || request
            .output_schema
            .as_ref()
            .is_some_and(schema_mentions_style_prompt)
}

fn schema_mentions_style_prompt(value: &Value) -> bool {
    match value {
        Value::String(text) => text == "style_prompt" || text == "stylePrompt",
        Value::Array(items) => items.iter().any(schema_mentions_style_prompt),
        Value::Object(object) => object.iter().any(|(key, value)| {
            key == "style_prompt" || key == "stylePrompt" || schema_mentions_style_prompt(value)
        }),
        _ => false,
    }
}

fn resolve_provider_model(
    database: &Database,
    provider_id: &str,
    requested_model_id: &str,
    context: &ProviderRequestContext,
) -> Result<ProviderModelRecord, TaskError> {
    let models = ProviderRepository::new(database)
        .list_provider_models(Some(provider_id))
        .map_err(|message| provider_error("provider.config_error", &message, context, None))?;
    models
        .into_iter()
        .find(|model| {
            model.model_id == requested_model_id || model.provider_model_id == requested_model_id
        })
        .ok_or_else(|| {
            provider_error(
                "provider.model_not_found",
                "Provider model was not found in provider_models.",
                context,
                Some(json!({ "providerModelId": requested_model_id })),
            )
        })
}

fn validate_workflow_preset_for_request(
    database: &Database,
    provider: &ResolvedProvider,
    workflow_preset_id: &str,
    workflow_vendor: &str,
    context: &ProviderRequestContext,
) -> Result<(), TaskError> {
    let preset = ProviderRepository::new(database)
        .get_workflow_preset(workflow_preset_id)
        .map_err(|message| provider_error("workflow.config_error", &message, context, None))?
        .ok_or_else(|| {
            provider_error(
                "workflow.preset_not_found",
                "Workflow preset was not registered.",
                context,
                Some(json!({ "workflowPresetId": workflow_preset_id })),
            )
        })?;

    if !preset.enabled {
        return Err(provider_error(
            "workflow.preset_disabled",
            "Workflow preset is disabled.",
            context,
            Some(json!({ "workflowPresetId": workflow_preset_id })),
        ));
    }

    let config = &preset.config_json;
    let provider_id = read_model_string(config, &["providerId", "provider_id"])
        .or_else(|| preset.provider_id.clone())
        .unwrap_or_default();
    if provider_id != provider.provider_id {
        return Err(provider_error(
            "workflow.provider_unavailable",
            "Workflow preset belongs to a different provider.",
            context,
            Some(json!({ "workflowProviderId": provider_id })),
        ));
    }

    let vendor = read_model_string(config, &["vendor"]).unwrap_or(preset.kind);
    if vendor != workflow_vendor || vendor != provider.vendor {
        return Err(provider_error(
            "workflow.provider_unavailable",
            "Workflow preset vendor does not match workflow request.",
            context,
            Some(json!({
                "presetVendor": vendor,
                "requestVendor": workflow_vendor,
                "providerVendor": provider.vendor
            })),
        ));
    }

    let status = read_model_string(config, &["status"]).unwrap_or_else(|| {
        if preset.enabled {
            "ready".to_string()
        } else {
            "disabled".to_string()
        }
    });
    if status == "disabled" {
        return Err(provider_error(
            "workflow.preset_disabled",
            "Workflow preset status is disabled.",
            context,
            Some(json!({ "workflowPresetId": workflow_preset_id })),
        ));
    }

    let param_schema =
        read_model_value(config, &["paramSchema", "param_schema"]).unwrap_or_else(|| json!({}));
    if !param_schema.is_object() {
        return Err(provider_error(
            "workflow.invalid_param_schema",
            "Workflow param_schema must be a JSON object.",
            context,
            Some(json!({ "workflowPresetId": workflow_preset_id })),
        ));
    }

    let node_map = read_model_value(config, &["nodeMap", "node_map"]).unwrap_or_else(|| json!({}));
    if !node_map
        .as_object()
        .is_some_and(|object| !object.is_empty())
    {
        return Err(provider_error(
            "workflow.invalid_node_map",
            "Workflow node_map is missing or empty.",
            context,
            Some(json!({ "workflowPresetId": workflow_preset_id })),
        ));
    }

    let output_map =
        read_model_value(config, &["outputMap", "output_map"]).unwrap_or_else(|| json!({}));
    if !output_map
        .as_object()
        .is_some_and(|object| !object.is_empty())
    {
        return Err(provider_error(
            "workflow.output_missing",
            "Workflow output_map is missing or empty.",
            context,
            Some(json!({ "workflowPresetId": workflow_preset_id })),
        ));
    }

    if vendor == "runninghub" {
        let workflow_id = read_model_string(config, &["workflowId", "workflow_id"]);
        if workflow_id.as_deref().unwrap_or("").is_empty() {
            return Err(provider_error(
                "workflow.workflow_id_missing",
                "RunningHub workflow preset requires workflow_id.",
                context,
                Some(json!({ "workflowPresetId": workflow_preset_id })),
            ));
        }
    }

    Ok(())
}

fn validate_provider_model_for_request(
    provider: &crate::db::provider_repository::ProviderRecord,
    model: &ProviderModelRecord,
    expected_kind: &str,
    validation: Option<ProviderRequestValidation<'_>>,
    context: &ProviderRequestContext,
) -> Result<(), TaskError> {
    if !model.enabled {
        return Err(provider_error(
            "provider.model_disabled",
            "Provider model is disabled.",
            context,
            Some(json!({ "modelId": model.model_id })),
        ));
    }

    let config = &model.config_json;
    let provider_kind = read_model_string(config, &["providerKind", "provider_kind"])
        .unwrap_or_else(|| provider.kind.clone());
    if provider_kind != expected_kind || provider_kind != provider.kind {
        return Err(provider_error(
            "provider.capability_unsupported",
            "Provider model kind does not match requested provider kind.",
            context,
            Some(json!({
                "expectedKind": expected_kind,
                "providerKind": provider.kind,
                "modelKind": provider_kind
            })),
        ));
    }

    let status = read_model_string(config, &["status"]).unwrap_or_else(|| {
        if model.enabled {
            "ready".to_string()
        } else {
            "disabled".to_string()
        }
    });
    if status == "disabled" {
        return Err(provider_error(
            "provider.model_disabled",
            "Provider model status is disabled.",
            context,
            Some(json!({ "modelId": model.model_id })),
        ));
    }

    if let Some(validation) = validation {
        let limits = read_model_value(config, &["limits"]).unwrap_or_else(|| json!({}));
        let ability_types = read_model_string_array(config, &["abilityTypes", "ability_types"])
            .filter(|items| !items.is_empty())
            .unwrap_or_else(|| vec![model.capability.clone()]);

        match validation {
            ProviderRequestValidation::Image {
                ability_type,
                aspect_ratio,
                resolution,
                duration_seconds,
                fps,
                reference_image_count,
            }
            | ProviderRequestValidation::Video {
                ability_type,
                aspect_ratio,
                resolution,
                duration_seconds,
                fps,
                reference_image_count,
            } => {
                validate_ability_type(&ability_types, ability_type, context)?;
                validate_aspect_ratio(&limits, aspect_ratio, context)?;
                validate_resolution(&limits, resolution, context)?;
                validate_duration(&limits, duration_seconds, context)?;
                validate_fps(&limits, fps, context)?;
                validate_reference_image_count(&limits, reference_image_count, context)?;
            }
        }
    }

    Ok(())
}

fn validate_ability_type(
    ability_types: &[String],
    ability_type: &str,
    context: &ProviderRequestContext,
) -> Result<(), TaskError> {
    if ability_types.iter().any(|item| item == ability_type) {
        return Ok(());
    }

    Err(provider_error(
        "provider.capability_unsupported",
        "Provider model does not support requested ability type.",
        context,
        Some(json!({ "abilityType": ability_type, "abilityTypes": ability_types })),
    ))
}

fn validate_aspect_ratio(
    limits: &Value,
    aspect_ratio: &str,
    context: &ProviderRequestContext,
) -> Result<(), TaskError> {
    if let Some(aspect_ratios) = read_model_string_array(
        limits,
        &[
            "aspectRatios",
            "aspect_ratios",
            "supportedAspectRatios",
            "supported_aspect_ratios",
        ],
    ) {
        if !aspect_ratios.is_empty() && !aspect_ratios.iter().any(|item| item == aspect_ratio) {
            return Err(provider_error(
                "provider.limit_exceeded",
                "Aspect ratio is not supported by provider model.",
                context,
                Some(
                    json!({ "aspectRatio": aspect_ratio, "supportedAspectRatios": aspect_ratios }),
                ),
            ));
        }
    }
    Ok(())
}

fn validate_resolution(
    limits: &Value,
    resolution: Option<&str>,
    context: &ProviderRequestContext,
) -> Result<(), TaskError> {
    let Some(resolution) = resolution else {
        return Ok(());
    };
    if let Some(resolutions) = read_model_string_array(limits, &["resolutions"]) {
        if !resolutions.is_empty() && !resolutions.iter().any(|item| item == resolution) {
            return Err(provider_error(
                "provider.limit_exceeded",
                "Resolution is not supported by provider model.",
                context,
                Some(json!({ "resolution": resolution, "resolutions": resolutions })),
            ));
        }
    }
    Ok(())
}

fn validate_duration(
    limits: &Value,
    duration_seconds: Option<f64>,
    context: &ProviderRequestContext,
) -> Result<(), TaskError> {
    let Some(duration_seconds) = duration_seconds else {
        return Ok(());
    };
    if let Some(range) = read_model_value(
        limits,
        &[
            "durationSeconds",
            "duration_seconds",
            "durationRange",
            "duration_range",
        ],
    ) {
        let min = range.get("min").and_then(Value::as_f64).unwrap_or(0.0);
        let max = range.get("max").and_then(Value::as_f64).unwrap_or(f64::MAX);
        let integer = range
            .get("integer")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if duration_seconds < min
            || duration_seconds > max
            || (integer && duration_seconds.fract() != 0.0)
        {
            return Err(provider_error(
                "provider.limit_exceeded",
                "Duration is outside provider model limits.",
                context,
                Some(json!({ "durationSeconds": duration_seconds, "limits": range })),
            ));
        }
    }
    Ok(())
}

fn validate_fps(
    limits: &Value,
    fps: Option<u32>,
    context: &ProviderRequestContext,
) -> Result<(), TaskError> {
    let Some(fps) = fps else {
        return Ok(());
    };
    if let Some(range) = read_model_value(limits, &["fpsRange", "fps_range"]) {
        let min = range.get("min").and_then(Value::as_f64).unwrap_or(0.0);
        let max = range.get("max").and_then(Value::as_f64).unwrap_or(f64::MAX);
        if (fps as f64) < min || (fps as f64) > max {
            return Err(provider_error(
                "provider.limit_exceeded",
                "FPS is outside provider model limits.",
                context,
                Some(json!({ "fps": fps, "limits": range })),
            ));
        }
    }
    if let Some(fps_values) = read_model_number_array(limits, &["fps", "fpsValues", "fps_values"]) {
        if !fps_values.is_empty()
            && !fps_values
                .iter()
                .any(|value| (*value - fps as f64).abs() < f64::EPSILON)
        {
            return Err(provider_error(
                "provider.limit_exceeded",
                "FPS is not supported by provider model.",
                context,
                Some(json!({ "fps": fps, "supportedFps": fps_values })),
            ));
        }
    }
    Ok(())
}

fn validate_reference_image_count(
    limits: &Value,
    reference_image_count: usize,
    context: &ProviderRequestContext,
) -> Result<(), TaskError> {
    let Some(max_reference_images) =
        read_model_u64(limits, &["maxReferenceImages", "max_reference_images"])
    else {
        return Ok(());
    };
    if reference_image_count as u64 > max_reference_images {
        return Err(provider_error(
            "provider.limit_exceeded",
            "Too many reference images for provider model.",
            context,
            Some(json!({
                "referenceImageCount": reference_image_count,
                "maxReferenceImages": max_reference_images
            })),
        ));
    }
    Ok(())
}

fn read_model_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str).map(str::to_string))
}

fn read_model_value(value: &Value, keys: &[&str]) -> Option<Value> {
    keys.iter().find_map(|key| value.get(*key).cloned())
}

fn read_model_u64(value: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_u64))
}

fn read_model_string_array(value: &Value, keys: &[&str]) -> Option<Vec<String>> {
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

fn read_model_number_array(value: &Value, keys: &[&str]) -> Option<Vec<f64>> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
    })
}

fn with_test_summary_metadata(
    summary: Value,
    test_mode: &str,
    real_generate_confirmed: bool,
    billable: bool,
) -> Value {
    let mut object = match summary {
        Value::Object(map) => map,
        other => {
            let mut map = serde_json::Map::new();
            map.insert("summary".to_string(), other);
            map
        }
    };
    object.insert("testMode".to_string(), json!(test_mode));
    object.insert(
        "realGenerateConfirmed".to_string(),
        json!(test_mode == "real_generate" && real_generate_confirmed),
    );
    object.insert("billable".to_string(), json!(billable));
    object.insert("externalNetwork".to_string(), json!(false));
    Value::Object(object)
}

fn workflow_preset_is_video(
    database: &Database,
    workflow_preset_id: Option<&str>,
) -> Result<bool, String> {
    let Some(workflow_preset_id) = workflow_preset_id else {
        return Ok(false);
    };
    let Some(preset) = ProviderRepository::new(database).get_workflow_preset(workflow_preset_id)?
    else {
        return Ok(false);
    };
    let ability_types = read_model_string_array(
        &preset.config_json,
        &[
            "abilityTypes",
            "ability_types",
            "outputModalities",
            "output_modalities",
        ],
    )
    .unwrap_or_default();
    Ok(ability_types.iter().any(|value| {
        let normalized = value.to_ascii_lowercase();
        normalized.contains("video") || normalized.contains("i2v")
    }))
}

pub fn provider_error(
    code: &str,
    message: &str,
    context: &ProviderRequestContext,
    detail: Option<Value>,
) -> TaskError {
    let detail = detail.unwrap_or_else(|| json!({}));
    TaskError::from_code_with_detail(
        code,
        redact_text(message),
        Some(json!({
            "traceId": context.trace_id,
            "providerId": context.provider_id,
            "providerModelId": context.provider_model_id,
            "workflowPresetId": context.workflow_preset_id,
            "detail": detail
        })),
    )
}

fn create_trace_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("trace_{nanos}")
}

#[cfg(test)]
mod tests {
    use super::ProviderManager;
    use crate::db::provider_repository::{ProviderRecord, ProviderRepository};
    use crate::db::Database;
    use crate::domain::provider::{ProviderDryRunRequest, ProviderGenerationTestRequest};
    use crate::services::keyring_service::KeyringService;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn provider_dry_run_requires_implemented_adapter() {
        let path = test_database_path("provider_dry_run_success");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_dummy_image", "image", "none", None);
        let keyring = KeyringService::memory();
        let manager = ProviderManager::new(&database, &keyring);

        let error = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_dummy_image".to_string(),
                provider_kind: "image".to_string(),
                provider_model_id: None,
                workflow_preset_id: None,
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("dry run cannot fake provider success without a real adapter");

        assert_eq!(error.error_code, "provider.config_error");
        cleanup(path);
    }

    #[test]
    fn generation_test_dry_run_rejects_unimplemented_provider_kinds() {
        let path = test_database_path("provider_generation_test_kinds");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_dummy_llm", "llm", "none", None);
        seed_provider(&database, "provider_dummy_image", "image", "none", None);
        seed_provider(&database, "provider_dummy_video", "video", "none", None);
        seed_provider(&database, "provider_dummy_tts", "tts", "none", None);
        seed_provider(&database, "provider_dummy_vlm", "vlm", "none", None);
        let keyring = KeyringService::memory();
        let manager = ProviderManager::new(&database, &keyring);

        for (provider_id, provider_kind) in [
            ("provider_dummy_llm", "llm"),
            ("provider_dummy_image", "image"),
            ("provider_dummy_video", "video"),
            ("provider_dummy_tts", "tts"),
            ("provider_dummy_vlm", "vlm"),
        ] {
            let error = manager
                .generation_test(ProviderGenerationTestRequest {
                    provider_id: provider_id.to_string(),
                    provider_kind: provider_kind.to_string(),
                    provider_model_id: None,
                    workflow_preset_id: None,
                    test_mode: "dry_run".to_string(),
                    real_generate_confirmed: None,
                    confirm_token: None,
                    simulate_failure: None,
                    simulate_cancelled: None,
                })
                .expect_err("dry_run test cannot fake provider output");

            assert_eq!(error.error_code, "provider.config_error");
        }

        cleanup(path);
    }

    #[test]
    fn generation_test_real_generate_requires_confirmation_and_video_token() {
        let path = test_database_path("provider_generation_test_confirmation");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_dummy_image", "image", "none", None);
        seed_provider(&database, "provider_dummy_video", "video", "none", None);
        let keyring = KeyringService::memory();
        let manager = ProviderManager::new(&database, &keyring);

        let missing_confirm = manager
            .generation_test(ProviderGenerationTestRequest {
                provider_id: "provider_dummy_image".to_string(),
                provider_kind: "image".to_string(),
                provider_model_id: None,
                workflow_preset_id: None,
                test_mode: "real_generate".to_string(),
                real_generate_confirmed: Some(false),
                confirm_token: None,
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("real_generate must require explicit confirmation");
        assert_eq!(
            missing_confirm.error_code,
            "provider.real_generate_confirmation_required"
        );

        let missing_video_token = manager
            .generation_test(ProviderGenerationTestRequest {
                provider_id: "provider_dummy_video".to_string(),
                provider_kind: "video".to_string(),
                provider_model_id: None,
                workflow_preset_id: None,
                test_mode: "real_generate".to_string(),
                real_generate_confirmed: Some(true),
                confirm_token: None,
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("video real_generate must require token");
        assert_eq!(
            missing_video_token.error_code,
            "provider.video_real_generate_confirmation_required"
        );

        let confirmed = manager
            .generation_test(ProviderGenerationTestRequest {
                provider_id: "provider_dummy_video".to_string(),
                provider_kind: "video".to_string(),
                provider_model_id: None,
                workflow_preset_id: None,
                test_mode: "real_generate".to_string(),
                real_generate_confirmed: Some(true),
                confirm_token: Some("REAL_GENERATE_VIDEO".to_string()),
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("confirmed real_generate cannot fake provider output");

        assert_eq!(confirmed.error_code, "provider.config_error");
        cleanup(path);
    }

    #[test]
    fn generation_test_video_workflow_requires_video_confirmation_token() {
        let path = test_database_path("provider_generation_test_workflow_video_confirmation");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_comfyui", "workflow", "none", None);
        seed_workflow_preset(
            &database,
            "workflow_video_ready",
            "provider_comfyui",
            true,
            json!({
                "providerId": "provider_comfyui",
                "vendor": "comfyui",
                "workflowKey": "dummy/video-workflow",
                "workflowVersion": "1.0.0",
                "abilityTypes": ["image_to_video"],
                "outputModalities": ["video"],
                "paramSchema": {},
                "nodeMap": { "prompt": "prompt" },
                "outputMap": { "video": "video" },
                "status": "ready"
            }),
        );
        let keyring = KeyringService::memory();
        let manager = ProviderManager::new(&database, &keyring);

        let missing_token = manager
            .generation_test(ProviderGenerationTestRequest {
                provider_id: "provider_comfyui".to_string(),
                provider_kind: "workflow".to_string(),
                provider_model_id: None,
                workflow_preset_id: Some("workflow_video_ready".to_string()),
                test_mode: "real_generate".to_string(),
                real_generate_confirmed: Some(true),
                confirm_token: None,
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("video workflow real_generate must require token");
        assert_eq!(
            missing_token.error_code,
            "provider.video_real_generate_confirmation_required"
        );

        let confirmed = manager
            .generation_test(ProviderGenerationTestRequest {
                provider_id: "provider_comfyui".to_string(),
                provider_kind: "workflow".to_string(),
                provider_model_id: None,
                workflow_preset_id: Some("workflow_video_ready".to_string()),
                test_mode: "real_generate".to_string(),
                real_generate_confirmed: Some(true),
                confirm_token: Some("REAL_GENERATE_VIDEO".to_string()),
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("confirmed video workflow cannot fake provider output");
        assert_eq!(confirmed.error_code, "provider.config_error");
        cleanup(path);
    }

    #[test]
    fn generation_test_does_not_return_secret_material() {
        let path = test_database_path("provider_generation_test_redaction");
        let database = Database::open(&path).expect("database should open");
        seed_provider(
            &database,
            "provider_dummy_llm",
            "llm",
            "api_key",
            Some("dummy_llm_key"),
        );
        let keyring = KeyringService::memory();
        keyring
            .save_provider_secret(
                "provider_dummy_llm",
                "api_key",
                Some("dummy_llm_key"),
                "sk-test-secret-should-not-leak",
            )
            .expect("secret should save");
        let manager = ProviderManager::new(&database, &keyring);

        let error = manager
            .generation_test(ProviderGenerationTestRequest {
                provider_id: "provider_dummy_llm".to_string(),
                provider_kind: "llm".to_string(),
                provider_model_id: None,
                workflow_preset_id: None,
                test_mode: "dry_run".to_string(),
                real_generate_confirmed: None,
                confirm_token: None,
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("dry_run cannot fake provider output");

        let serialized = format!("{error:?}");
        assert!(!serialized.contains("sk-test-secret-should-not-leak"));
        assert!(!serialized.contains("Authorization"));
        assert!(!serialized.contains("Bearer"));
        cleanup(path);
    }

    #[test]
    fn provider_manager_requires_secret_in_keyring() {
        let path = test_database_path("provider_dry_run_secret");
        let database = Database::open(&path).expect("database should open");
        seed_provider(
            &database,
            "provider_dummy_llm",
            "llm",
            "api_key",
            Some("dummy_llm_key"),
        );
        let keyring = KeyringService::memory();
        let manager = ProviderManager::new(&database, &keyring);

        let error = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_dummy_llm".to_string(),
                provider_kind: "llm".to_string(),
                provider_model_id: None,
                workflow_preset_id: None,
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("missing secret should fail");

        assert_eq!(error.error_code, "provider.auth_failed");
        assert!(!format!("{error:?}").contains("sk-test"));
        cleanup(path);
    }

    #[test]
    fn provider_manager_maps_failure_and_cancellation_to_task_error() {
        let path = test_database_path("provider_dry_run_errors");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_dummy_video", "video", "none", None);
        let keyring = KeyringService::memory();
        let manager = ProviderManager::new(&database, &keyring);

        let failure = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_dummy_video".to_string(),
                provider_kind: "video".to_string(),
                provider_model_id: Some("dummy/video".to_string()),
                workflow_preset_id: None,
                simulate_failure: Some(true),
                simulate_cancelled: None,
            })
            .expect_err("simulated failure should fail");
        assert_eq!(failure.error_code, "provider.server_error");
        assert!(failure.is_retryable);

        let cancelled = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_dummy_video".to_string(),
                provider_kind: "video".to_string(),
                provider_model_id: Some("dummy/video".to_string()),
                workflow_preset_id: None,
                simulate_failure: None,
                simulate_cancelled: Some(true),
            })
            .expect_err("simulated cancel should fail");
        assert_eq!(cancelled.error_code, "provider.cancelled");
        cleanup(path);
    }

    #[test]
    fn provider_manager_rejects_disabled_model_and_limit_overflow() {
        let path = test_database_path("provider_model_limits");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_dummy_video", "video", "none", None);
        seed_provider_model(
            &database,
            "model_disabled",
            "provider_dummy_video",
            false,
            json!({
                "providerKind": "video",
                "vendor": "comfyui",
                "modelName": "dummy-video-disabled",
                "abilityTypes": ["image_to_video"],
                "limits": {
                    "durationSeconds": { "min": 3, "max": 8, "integer": true },
                    "supportedAspectRatios": ["9:16"],
                    "fps": [16, 24]
                },
                "status": "ready"
            }),
        );
        seed_provider_model(
            &database,
            "model_limited",
            "provider_dummy_video",
            true,
            json!({
                "providerKind": "video",
                "vendor": "comfyui",
                "modelName": "dummy-video-limited",
                "abilityTypes": ["image_to_video"],
                "limits": {
                    "durationSeconds": { "min": 3, "max": 8, "integer": true },
                    "supportedAspectRatios": ["9:16"],
                    "fps": [16, 24],
                    "maxReferenceImages": 1
                },
                "status": "ready"
            }),
        );
        let keyring = KeyringService::memory();
        let manager = ProviderManager::new(&database, &keyring);

        let disabled = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_dummy_video".to_string(),
                provider_kind: "video".to_string(),
                provider_model_id: Some("model_disabled".to_string()),
                workflow_preset_id: None,
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("disabled model should fail");
        assert_eq!(disabled.error_code, "provider.model_disabled");

        let overflow = manager
            .generate_video(
                crate::domain::provider::VideoProviderRequest {
                    context: crate::domain::provider::ProviderRequestContext {
                        trace_id: "trace_test_model_limit".to_string(),
                        task_id: None,
                        task_step_id: None,
                        project_id: None,
                        item_id: None,
                        provider_id: "provider_dummy_video".to_string(),
                        provider_model_id: Some("model_limited".to_string()),
                        workflow_preset_id: None,
                        timeout_seconds: Some(10),
                        idempotency_key: None,
                    },
                    ability_type: "image_to_video".to_string(),
                    prompt: "prompt".to_string(),
                    negative_prompt: None,
                    aspect_ratio: "16:9".to_string(),
                    duration_seconds: 12.0,
                    resolution: None,
                    fps: Some(30),
                    seed: None,
                    input_images: vec![],
                    input_video_path: None,
                    input_audio_path: None,
                    output_path: "dry-run/provider/video.mp4".to_string(),
                },
                &crate::services::task_cancellation::CancellationToken::new("model_limit"),
            )
            .expect_err("limit overflow should fail");
        assert_eq!(overflow.error_code, "provider.limit_exceeded");

        cleanup(path);
    }

    #[test]
    fn provider_manager_validates_registered_workflow_preset() {
        let path = test_database_path("workflow_preset_validation");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_comfyui", "workflow", "none", None);
        seed_workflow_preset(
            &database,
            "workflow_ready",
            "provider_comfyui",
            true,
            json!({
                "providerId": "provider_comfyui",
                "vendor": "comfyui",
                "workflowKey": "dummy/workflow",
                "workflowVersion": "1.0.0",
                "abilityTypes": ["image_to_video"],
                "paramSchema": {},
                "nodeMap": { "prompt": "prompt" },
                "outputMap": { "video": "video" },
                "status": "ready"
            }),
        );
        seed_workflow_preset(
            &database,
            "workflow_disabled",
            "provider_comfyui",
            false,
            json!({
                "providerId": "provider_comfyui",
                "vendor": "comfyui",
                "workflowKey": "dummy/workflow-disabled",
                "workflowVersion": "1.0.0",
                "abilityTypes": ["image_to_video"],
                "paramSchema": {},
                "nodeMap": { "prompt": "prompt" },
                "outputMap": { "video": "video" },
                "status": "ready"
            }),
        );
        seed_workflow_preset(
            &database,
            "workflow_bad_node_map",
            "provider_comfyui",
            true,
            json!({
                "providerId": "provider_comfyui",
                "vendor": "comfyui",
                "workflowKey": "dummy/workflow-bad-node",
                "workflowVersion": "1.0.0",
                "abilityTypes": ["image_to_video"],
                "paramSchema": {},
                "nodeMap": {},
                "outputMap": { "video": "video" },
                "status": "ready"
            }),
        );
        seed_workflow_preset(
            &database,
            "workflow_bad_output_map",
            "provider_comfyui",
            true,
            json!({
                "providerId": "provider_comfyui",
                "vendor": "comfyui",
                "workflowKey": "dummy/workflow-bad-output",
                "workflowVersion": "1.0.0",
                "abilityTypes": ["image_to_video"],
                "paramSchema": {},
                "nodeMap": { "prompt": "prompt" },
                "outputMap": {},
                "status": "ready"
            }),
        );
        let keyring = KeyringService::memory();
        let manager = ProviderManager::new(&database, &keyring);

        let adapter_missing = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_comfyui".to_string(),
                provider_kind: "workflow".to_string(),
                provider_model_id: None,
                workflow_preset_id: Some("workflow_ready".to_string()),
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("registered workflow still requires an implemented adapter");
        assert_eq!(adapter_missing.error_code, "provider.config_error");

        let missing = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_comfyui".to_string(),
                provider_kind: "workflow".to_string(),
                provider_model_id: None,
                workflow_preset_id: Some("workflow_missing".to_string()),
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("missing workflow should fail");
        assert_eq!(missing.error_code, "workflow.preset_not_found");

        let disabled = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_comfyui".to_string(),
                provider_kind: "workflow".to_string(),
                provider_model_id: None,
                workflow_preset_id: Some("workflow_disabled".to_string()),
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("disabled workflow should fail");
        assert_eq!(disabled.error_code, "workflow.preset_disabled");

        let bad_node = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_comfyui".to_string(),
                provider_kind: "workflow".to_string(),
                provider_model_id: None,
                workflow_preset_id: Some("workflow_bad_node_map".to_string()),
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("bad node_map should fail");
        assert_eq!(bad_node.error_code, "workflow.invalid_node_map");

        let bad_output = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_comfyui".to_string(),
                provider_kind: "workflow".to_string(),
                provider_model_id: None,
                workflow_preset_id: Some("workflow_bad_output_map".to_string()),
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("bad output_map should fail");
        assert_eq!(bad_output.error_code, "workflow.output_missing");

        cleanup(path);
    }

    #[test]
    fn provider_manager_rejects_mixed_executable_source_ids() {
        let path = test_database_path("mixed_executable_source_ids");
        let database = Database::open(&path).expect("database should open");
        seed_provider(&database, "provider_comfyui", "workflow", "none", None);
        let keyring = KeyringService::memory();
        let manager = ProviderManager::new(&database, &keyring);

        let mixed = manager
            .dry_run(ProviderDryRunRequest {
                provider_id: "provider_comfyui".to_string(),
                provider_kind: "workflow".to_string(),
                provider_model_id: Some("model_should_not_mix".to_string()),
                workflow_preset_id: Some("workflow_should_not_mix".to_string()),
                simulate_failure: None,
                simulate_cancelled: None,
            })
            .expect_err("mixed source IDs should fail");

        assert_eq!(mixed.error_code, "provider.source_conflict");
        cleanup(path);
    }

    fn seed_provider(
        database: &Database,
        provider_id: &str,
        kind: &str,
        auth_type: &str,
        key_alias: Option<&str>,
    ) {
        ProviderRepository::new(database)
            .upsert_provider(&ProviderRecord {
                provider_id: provider_id.to_string(),
                vendor: if kind == "workflow" {
                    "comfyui".to_string()
                } else {
                    "dummy".to_string()
                },
                kind: kind.to_string(),
                display_name: format!("Dummy {kind}"),
                auth_type: auth_type.to_string(),
                key_alias: key_alias.map(str::to_string),
                base_url: None,
                status: "ready".to_string(),
                enabled: true,
                config_json: json!({ "adapter": "dummy" }),
            })
            .expect("provider should save");
    }

    fn seed_provider_model(
        database: &Database,
        model_id: &str,
        provider_id: &str,
        enabled: bool,
        config_json: serde_json::Value,
    ) {
        ProviderRepository::new(database)
            .upsert_provider_model(&crate::db::provider_repository::ProviderModelRecord {
                model_id: model_id.to_string(),
                provider_id: provider_id.to_string(),
                provider_model_id: format!("dummy/{model_id}"),
                display_name: format!("Dummy {model_id}"),
                capability: "image_to_video".to_string(),
                config_json,
                enabled,
            })
            .expect("provider model should save");
    }

    fn seed_workflow_preset(
        database: &Database,
        preset_id: &str,
        provider_id: &str,
        enabled: bool,
        config_json: serde_json::Value,
    ) {
        ProviderRepository::new(database)
            .upsert_workflow_preset(&crate::db::provider_repository::WorkflowPresetRecord {
                preset_id: preset_id.to_string(),
                provider_id: Some(provider_id.to_string()),
                model_id: None,
                name: format!("Workflow {preset_id}"),
                kind: "dummy".to_string(),
                capability: "image_to_video".to_string(),
                config_json,
                enabled,
            })
            .expect("workflow preset should save");
    }

    fn test_database_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-provider-{name}-{}-{nanos}.sqlite3",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
    }
}
