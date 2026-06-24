# Provider 与安全

> 这篇定义两件事：①AI / TTS / 图片 / 视频 / VLM / workflow 能力怎么统一接入；②密钥、路径、重试、限流、安全红线怎么做。枚举取值见 `01-枚举字典与配置规范.md`，后端封装见 `03-后端工程规范.md`。

## 一、Provider 统一抽象

### 设计原则

```text
业务代码绝不直接调某家 API。
业务代码只调用 ProviderManager。
ProviderManager 再选择具体 Provider。
```

ProviderKind 统一为：

```text
llm / tts / image / video / vlm / workflow
```

说明：

```text
llm       文本生成、结构化 JSON 输出
tts       文字转语音
image     文生图、图生图
video     文生视频、图生视频、首尾帧视频
vlm       图像/视频理解、画风反推、一致性检查
workflow  ComfyUI / RunningHub 等工作流运行方式
```

ComfyUI / RunningHub 不作为单独 ProviderKind，而是：

```text
ProviderKind = workflow
ProviderVendor = comfyui / runninghub
```

---

## 二、ProviderManager 职责

ProviderManager 是模型调用唯一入口，负责：

```text
1. 根据 provider_id 选择供应商连接，并根据 provider_model_id / model_name 选择具体模型。
2. 用 key_alias 从系统钥匙串读取真实密钥。
3. 校验模型能力矩阵。
4. 做 Provider 级并发限流。
5. 做指数退避重试。
6. 做错误分类。
7. 做日志脱敏。
8. 把不同 Provider 响应统一成内部 DTO。
```

业务层禁止：

```text
直接 new 具体 Provider
直接读取 API Key
直接拼某家 API 参数
直接把远程 URL 存入数据库
直接执行未注册 workflow
按具体 model_name 写业务分支
```

---

## 三、模型寻址

统一用稳定配置定位模型：

```text
provider_id
provider_model_id（API 模型）
workflow_preset_id（ComfyUI / RunningHub 工作流）
```

展示或日志中可以使用：

```text
vendor:model_name
```

示例：

```text
deepseek:deepseek-chat
dashscope:wanx-v1
openai_compatible:gpt-4o-mini
```

ProviderConfig 只存供应商连接和鉴权元数据，不存具体 model_name：

```json
{
  "provider_id": "provider_xxx",
  "provider_kind": "llm",
  "vendor": "openai_compatible",
  "display_name": "DeepSeek",
  "base_url": "https://api.deepseek.com/v1",
  "key_alias": "deepseek_main",
  "is_enabled": true,
  "config": {}
}
```

真实 API Key 不在 ProviderConfig 里，只存 `key_alias`。具体模型写入 `provider_models`，包括 `model_name`、能力矩阵、默认模型标记。

API 模型和工作流必须分开寻址：

```text
provider_models      只管理 API 模型能力矩阵
workflow_presets     只管理 ComfyUI / RunningHub workflow preset
```

业务层只允许把 `provider_model_id` 或 `workflow_preset_id` 交给 ProviderManager。不得直接使用 `model_name`、`workflow_key`、`workflow_id` 作为业务分支条件。workflow 执行细节见 `23-模型适配与工作流注册规范.md`。

---

## 四、默认接入策略

### LLM

第一版默认接入 OpenAI-compatible Chat Completions。

配置项：

```text
base_url
model_name
key_alias
temperature
max_tokens
```

天然兼容 OpenAI / DeepSeek / 通义兼容模式 / Ollama / 各类中转。

### TTS

第一版默认 Edge TTS，保证无 API Key 也能跑通配音闭环。

后续扩展：

```text
Azure TTS
火山 TTS
阿里云 TTS
ElevenLabs
workflow TTS（ComfyUI / RunningHub）
```

### Image

第一版默认接一个云 API，例如 DashScope Image 或 OpenAI Image。

结果若是远程 URL，必须自动下载转存本地，再入库相对路径。

### Video / VLM / Workflow

作为扩展能力接入，不污染主流程。AI 视频、数字人、画布精修都复用同一套 ProviderManager / TaskStep / Storage 规则。

---

## 五、Provider 能力矩阵

每个模型必须声明能力，后端负责校验，不能只靠前端禁用。

示例：

```json
{
  "provider_kind": "video",
  "vendor": "dashscope",
  "model_name": "wanx-video",
  "ability_types": ["text_to_video", "image_to_video"],
  "input_modalities": ["text", "image"],
  "output_modalities": ["video"],
  "feature_flags": [
    "reference_image",
    "duration",
    "resolution",
    "aspect_ratio"
  ],
  "limits": {
    "supported_aspect_ratios": ["vertical_9_16", "horizontal_16_9", "square_1_1"],
    "duration_seconds": { "min": 2, "max": 15, "integer": true },
    "resolutions": ["720p", "1080p"]
  }
}
```

能力枚举和接口契约见 `01-枚举字典与配置规范.md`、`12-Provider接口契约与能力矩阵Schema.md`。

---

## 六、调用可靠性（重试 / 退避 / 限流）

这是相对参考项目最重要的架构级超越，必须做：

```text
1. 每次 Provider 调用统一封装重试。
2. 失败按指数退避，例如 1s / 2s / 4s。
3. 最多重试次数来自 PipelineConfig 或任务快照。
4. 按 Provider 限流，用信号量/令牌桶控制并发。
5. 大文件流式下载落盘，不在内存里 base64 整个媒体。
6. 失败要分类：可重试 vs 不可重试。
7. 鉴权失败、参数非法这类不可重试错误直接停并提示，不空转重试。
```

---

## 七、密钥存储（安全红线）

```text
真实 API Key  →  系统钥匙串（keyring crate）
                 Windows Credential Manager / macOS Keychain / Linux Secret Service
SQLite 只存   →  provider_id / provider_kind / vendor / base_url / key_alias；具体模型存 provider_models.model_name
```

**绝不**把明文 API Key 存进 SQLite、yaml、json 配置文件、导出文件或日志。

导出配置时：

```text
只导出 key_alias 占位
导入后提示用户重新录入真实 Key
```

---

## 八、路径安全（PathGuard）

完整文件目录规范见 `08-文件存储与路径规范.md`。

所有文件读写必经 StorageService + PathGuard：

```text
1. 读写只允许落在白名单 FileBucket 内。
2. 所有路径先 canonicalize / resolve，再判断是否在白名单内。
3. 不允许用字符串 startsWith 判断边界。
4. 前端不允许直接传任意系统路径给 FFmpeg / Chromium / Provider 使用。
5. ImageCandidate.image_path、VideoSegment.video_path、CompositionTask.output_path，以及后续音频/字幕/封面路径都必须过 PathGuard。
6. 入库一律相对路径，绝不存绝对路径。
7. 远程 URL 先转存本地。
```

---

## 九、错误分类

错误枚举和错误码见：

```text
01-枚举字典与配置规范.md
09-错误日志与事件规范.md
```

Provider 层至少要区分：

```text
鉴权失败
限流
超时
网络错误
参数非法
内容审核失败
服务端错误
未知错误
```

上层根据 `is_retryable` 决定重试、停止或提示用户处理。

---

## 十、可编程供应商风险

Toonflow 有“用户写 TS 代码定义 vendor、热执行不重启即生效”的能力，但用 vm2 沙箱。vm2 已停止维护且有沙箱逃逸 CVE，又注入了 fetch/axios/crypto，恶意 vendor 可任意联网，等于 RCE 风险。

本项目第一版处理：

```text
默认只支持内置 Provider 适配器 + 声明式能力矩阵。
不开放任意用户代码执行。
不执行用户上传 JS / TS / Python Provider。
不执行未注册 workflow。
workflow 执行前必须校验 param_schema / node_map / output_map。
API Key 只进入 keyring，不进入 SQLite / JSON / YAML / 日志。
```

如果以后做可编程供应商，必须满足：

```text
isolated-vm 或更强隔离
出站域名白名单
资源限制
编译缓存
权限审计
禁用任意文件系统访问
```

不要照搬 vm2 方案。
