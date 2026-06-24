# 配置 Schema 与默认值规范

> 这篇定义应用配置分层、默认值、存储位置、导入导出和任务快照。真实密钥不属于配置，只能进系统钥匙串。

## 一、配置分层

```text
AppConfig          软件偏好
SystemConfig       工作区和 sidecar
ProviderConfig     Provider 元数据
PipelineConfig     生产线默认参数
UiConfig           UI 行为偏好
ExportConfig       导出参数
SecretConfig       只存 key_alias，不存 secret
```

---

## 二、存储位置

| 配置 | 存储位置 | 是否进入任务快照 | 是否允许导出 |
|---|---|---|---|
| AppConfig | SQLite/app_configs | 否 | 是 |
| SystemConfig | SQLite/app_configs | 部分 | 是 |
| ProviderConfig | SQLite/providers | 是，脱敏快照 | 是，不含密钥 |
| PipelineConfig | SQLite/app_configs | 是 | 是 |
| UiConfig | SQLite/app_configs | 否 | 是 |
| ExportConfig | SQLite/app_configs | 是 | 是 |
| SecretConfig | keyring + key_alias | 否 | 只导出 key_alias |

---

## 三、AppConfig

```json
{
  "app_locale": "zh-CN",
  "theme_preset": "graphite",
  "layout_density": "comfortable"
}
```

---

## 四、SystemConfig

```json
{
  "workspace_dir": null,
  "max_log_files": 7,
  "enable_diagnostics": true,
  "ffmpeg_sidecar_path": "sidecars/ffmpeg.exe",
  "ffprobe_sidecar_path": "sidecars/ffprobe.exe",
  "chromium_sidecar_path": null
}
```

规则：

```text
1. workspace_dir 首次启动初始化。
2. sidecar path 只能指向应用资源目录或受控 sidecars bucket。
3. 用户自定义外部 sidecar 第一版不开放。
```

---

## 五、PipelineConfig

```json
{
  "default_content_category": "knowledge",
  "default_aspect_ratio": "vertical_9_16",
  "default_content_language": "zh-CN",
  "default_target_duration_seconds": 60,
  "default_target_scene_count": 8,
  "max_concurrent_provider_calls": 2,
  "max_concurrent_ffmpeg_jobs": 2,
  "retry_max_attempts": 3,
  "retry_backoff_seconds": [2, 5, 10],
  "image_prompt_batch_size": 10,
  "max_prompt_length": 1000,
  "review_required": {
    "script": true,
    "storyboard": true,
    "image": true,
    "cover": true
  }
}
```

---

## 六、ExportConfig

`ExportConfig` 是导出默认值；`export_video` 请求参数可以覆盖本次导出，并写入任务快照。

```json
{
  "format": "mp4",
  "fps": 30,
  "video_codec": "libx264",
  "audio_codec": "aac",
  "audio_bitrate": "192k",
  "crf": 18,
  "preset": "veryfast",
  "pix_fmt": "yuv420p",
  "include_cover": true,
  "include_subtitles_json": true
}
```

---

## 七、UiConfig

```json
{
  "show_advanced_options": false,
  "auto_open_task_detail": true,
  "confirm_before_costly_generation": true,
  "default_project_view": "grid",
  "timeline_density": "comfortable"
}
```

---

## 八、ProviderConfig

```json
{
  "provider_id": "provider_xxx",
  "provider_kind": "llm",
  "vendor": "openai_compatible",
  "display_name": "DeepSeek",
  "base_url": "https://api.deepseek.com/v1",
  "auth_type": "api_key",
  "key_alias": "deepseek_main",
  "status": "ready",
  "is_enabled": true,
  "config": {}
}
```

禁止：

```text
config.api_key
config.secret
config.token
```

---

## 九、任务快照

任务创建时冻结：

```json
{
  "provider_snapshot": {},
  "model_snapshot": {},
  "pipeline_config_snapshot": {},
  "export_config_snapshot": {},
  "template_snapshot": {},
  "prompt_skill_snapshot": {},
  "bible_snapshot": {},
  "retry_policy_snapshot": {}
}
```

规则：

```text
1. snapshot 不包含真实密钥。
2. snapshot 中保留 key_alias。
3. 运行中修改设置不影响已创建任务。
```

---

## 十、配置迁移

```text
1. app_configs 每条配置有 version。
2. 配置 schema 变更必须写 migration。
3. 读取配置失败时不能静默使用空对象，必须报 config.invalid。
4. 缺失配置可以用 defaults.rs 补齐并写回。
```
