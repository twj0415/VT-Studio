# Provider 接口契约与能力矩阵 Schema

> 这篇定义 LLM、TTS、Image、Video、VLM、Workflow 的统一请求响应协议。所有模型调用必须走 ProviderManager，业务代码不得直接调用具体供应商 SDK。

## 一、核心原则

```text
1. ProviderKind 只表示能力大类：llm / tts / image / video / vlm / workflow。
2. ComfyUI / RunningHub 是 workflow vendor，不是 ProviderKind。
3. 真实密钥只由 ProviderManager 通过 key_alias 从 keyring 读取。
4. Provider 请求和响应必须可序列化、可记录摘要、可脱敏。
5. 能力矩阵由后端校验，前端禁用只做体验优化。
6. Provider 返回远程 URL 时，必须下载转存本地后再入库。
```

---

## 二、统一 Provider 元数据

```ts
interface ProviderConfigDto {
  providerId: string
  providerKind: ProviderKind
  vendor: ProviderVendor
  displayName: string
  baseUrl?: string
  authType: ProviderAuthType
  keyAlias?: string
  status: ProviderStatus
  isEnabled: boolean
  config: Record<string, unknown>
}
```

禁止：

```text
config.api_key
config.token
config.secret
```

---

## 三、通用请求上下文

所有 Provider 请求都带上下文：

```ts
interface ProviderRequestContext {
  taskId?: string
  taskStepId?: string
  projectId?: string
  sceneId?: string
  providerId: string
  providerModelId?: string
  workflowPresetId?: string
  modelName?: string
  timeoutSeconds?: number
  idempotencyKey?: string
}
```

规则：

```text
1. API 模型调用优先使用 providerModelId，由 ModelRegistry 解析 modelName 和能力矩阵。
2. ComfyUI / RunningHub 调用使用 workflowPresetId，由 WorkflowRegistry 解析 workflow_key / workflow_id。
3. modelName 只允许作为快照字段或 ProviderAdapter 内部协议字段，不作为业务分支依据。
```

日志只能记录：

```text
task_id / step_id / item_id / image_id / segment_id / provider_id / vendor / model_name / timeout / error_code
```

---

## 四、LLM 契约

### LlmChatRequest

```ts
interface LlmChatRequest {
  context: ProviderRequestContext
  messages: Array<{
    role: 'system' | 'user' | 'assistant'
    content: string
  }>
  temperature?: number
  maxTokens?: number
  responseFormat?: 'text' | 'json_object' | 'json_schema'
  jsonSchema?: Record<string, unknown>
}
```

### LlmChatResponse

```ts
interface LlmChatResponse {
  content: string
  parsedJson?: unknown
  usage?: {
    promptTokens?: number
    completionTokens?: number
    totalTokens?: number
  }
  rawResponseSummary?: Record<string, unknown>
}
```

规则：

```text
1. 生成脚本、分镜、提示词时优先使用 json_schema。
2. 不支持 json_schema 的模型，进入 repair loop，最多 2 次。
3. LLM 输出不得直接入核心表，必须先 schema validate。
```

---

## 五、TTS 契约

### TtsRequest

```ts
interface TtsRequest {
  context: ProviderRequestContext
  text: string
  contentLanguage: ContentLanguage
  voiceId: string
  speed?: number
  pitch?: number
  volume?: number
  format: 'mp3' | 'wav' | 'm4a'
  sampleRate?: number
  outputPath: string
}
```

### TtsResponse

```ts
interface TtsResponse {
  audioPath: string
  audioDurationSeconds: number
  format: string
  sampleRate?: number
  fileSize?: number
}
```

规则：

```text
1. TTS 完成后必须 ffprobe 读取真实时长。
2. Provider 返回的时长只能作为参考。
3. audioPath 必须是任务目录相对路径。
```

---

## 六、Image 契约

### ImageRequest

```ts
interface ImageRequest {
  context: ProviderRequestContext
  providerModelId?: string
  workflowPresetId?: string
  prompt: string
  negativePrompt?: string
  aspectRatio: AspectRatio
  width?: number
  height?: number
  seed?: number
  stylePreset?: string
  referenceImages?: Array<{
    path: string
    weight?: number
    role?: 'character' | 'style' | 'scene' | 'composition' | 'other'
  }>
  outputPath: string
}
```

规则：

```text
1. providerModelId 和 workflowPresetId 不能同时为空。
2. providerModelId 和 workflowPresetId 默认不能同时有值。
3. 需要 workflow 内部绑定 API 模型时，只能由 workflow preset 声明，功能模块不得临时传入。
4. 业务代码不得按具体模型名判断生图逻辑。
```

### ImageResponse

```ts
interface ImageResponse {
  imagePath: string
  seed?: number
  width?: number
  height?: number
  fileSize?: number
  providerOutputSummary?: Record<string, unknown>
}
```

规则：

```text
1. referenceImages 必须是 SafePath。
2. 远程图片结果必须下载到 outputPath。
3. 不允许远程 URL 直接写入 ImageCandidate.image_path 或 VideoSegment.video_path；必须先下载/复制到受控工作区。
```

---

## 七、Video 契约

### VideoAbilityType

视频能力 code：

```text
text_to_video
image_to_video
first_frame_i2v
start_end_frame_i2v
reference_to_video
video_continuation
video_editing
action_transfer
digital_human
native_audio
voice_reference
multi_shot
```

### VideoRequest

```ts
interface VideoRequest {
  context: ProviderRequestContext
  providerModelId?: string
  workflowPresetId?: string
  abilityType: VideoAbilityType
  prompt: string
  negativePrompt?: string
  aspectRatio: AspectRatio
  durationSeconds: number
  resolution?: string
  fps?: number
  seed?: number
  inputImages?: Array<{
    path: string
    role: 'first_frame' | 'last_frame' | 'reference' | 'character' | 'style'
  }>
  inputVideoPath?: string
  inputAudioPath?: string
  outputPath: string
}
```

规则：

```text
1. providerModelId 和 workflowPresetId 不能同时为空。
2. providerModelId 和 workflowPresetId 默认不能同时有值。
3. durationSeconds / resolution / fps / inputImages 必须由后端按能力矩阵或 workflow preset limits 校验。
4. 业务代码不得按具体模型名判断视频生成逻辑。
```

### VideoResponse

```ts
interface VideoResponse {
  videoPath: string
  durationSeconds: number
  fps?: number
  width?: number
  height?: number
  fileSize?: number
  providerOutputSummary?: Record<string, unknown>
}
```

规则：

```text
1. durationSeconds 必须先通过能力矩阵校验。
2. AI 视频结果仍需经过 ffprobe 校验真实时长。
3. 进入最终合成前必须和旁白音频做时长对齐。
```

---

## 八、VLM 契约

```ts
interface VlmAnalyzeRequest {
  context: ProviderRequestContext
  inputPath: string
  prompt: string
  outputSchema?: Record<string, unknown>
}

interface VlmAnalyzeResponse {
  description: string
  parsedJson?: unknown
  tags?: string[]
}
```

用途：

```text
素材分析
参考图风格反推
角色图描述
视频首帧/尾帧分析
```

---

## 九、Workflow 契约

```ts
interface WorkflowRequest {
  context: ProviderRequestContext
  workflowPresetId: string
  workflowKey?: string
  workflowVendor: 'comfyui' | 'runninghub'
  params: Record<string, unknown>
  outputPath: string
}

interface WorkflowResponse {
  outputPath: string
  outputKind: MediaKind
  metadata: Record<string, unknown>
}
```

规则：

```text
1. workflowPresetId 必须来自 workflow_presets。
2. workflowKey 只能由 WorkflowRegistry 从 workflow preset 解析，前端不得直接传任意 workflow 文件。
3. workflow params 必须按 param_schema 校验。
4. RunningHub workflow_id 可以存配置，但真实密钥仍走 keyring。
5. node_map / output_map 必须在执行前校验。
```

---

## 十、ProviderError

```ts
interface ProviderError {
  code: string
  providerId: string
  vendor: ProviderVendor
  modelName?: string
  httpStatus?: number
  isRetryable: boolean
  retryAfterSeconds?: number
  rawErrorSummary?: string
}
```

映射规则：

| 原始问题 | AppError code | 可重试 |
|---|---|---|
| 401/403 | provider.auth_failed | 否 |
| 429 | provider.rate_limited | 是 |
| timeout | provider.timeout | 是 |
| 内容审核 | provider.content_policy | 可中性化后重试 |
| JSON 解析失败 | provider.invalid_response | 是 |
| 不支持能力 | provider.capability_unsupported | 否 |

---

## 十一、能力矩阵 Schema

```ts
interface ProviderModelAbilityDto {
  providerId: string
  providerKind: ProviderKind
  vendor: ProviderVendor
  modelName: string
  abilityTypes: ModelAbilityType[]
  inputModalities: Array<'text' | 'image' | 'audio' | 'video'>
  outputModalities: Array<'text' | 'image' | 'audio' | 'video'>
  featureFlags?: ModelFeatureFlag[]
  limits: {
    maxPromptLength?: number
    supportedAspectRatios?: AspectRatio[]
    resolutions?: string[]
    durationSeconds?: {
      min: number
      max: number
      integer: boolean
    }
    fps?: number[]
    maxReferenceImages?: number
    supportedFormats?: string[]
  }
  apiContractVerified: boolean
  notes?: string[]
}
```

Workflow preset 和统一可执行候选项见 `23-模型适配与工作流注册规范.md`，核心 DTO：

```ts
interface WorkflowPresetDto {
  workflowPresetId: string
  providerId: string
  vendor: 'comfyui' | 'runninghub'
  workflowKey: string
  workflowId?: string
  displayName: string
  workflowVersion: string
  abilityTypes: string[]
  inputModalities: string[]
  outputModalities: string[]
  limits: Record<string, unknown>
  paramSchema: Record<string, unknown>
  nodeMap: Record<string, string>
  outputMap: Record<string, string>
  isBuiltin: boolean
  isEnabled: boolean
}

interface ExecutableModelOptionDto {
  id: string
  sourceType: 'provider_model' | 'workflow_preset'
  providerId: string
  providerKind: ProviderKind
  displayName: string
  vendor: string
  abilityTypes: string[]
  inputModalities: string[]
  outputModalities: string[]
  limits: Record<string, unknown>
  isAvailable: boolean
  unavailableReason?: string
}
```

---

## 十二、内容审核中性化

仅对内容审核失败执行：

```text
provider.content_policy
```

流程：

```text
原 prompt
→ LLM neutralize
→ 保存 original_prompt / neutralized_prompt 到 step output
→ 重试一次
```

规则：

```text
1. 不对违法规避做优化。
2. 只做普通商业平台审核中性化，例如降低暴力、敏感、夸张描述。
3. 不隐藏原始失败原因。
```

---

## 十三、禁止事项

```text
1. 业务代码直接 reqwest 调模型。
2. Provider 直接读写数据库。
3. Provider 返回真实 API Key 到前端。
4. Provider 接收未经 PathGuard 的系统路径。
5. Provider 把远程 URL 直接写入数据库。
6. 第一版不支持用户在设置页执行自定义 JS/TS Provider 代码。
7. 业务代码按具体 modelName 判断能力或拼参数。
8. 功能模块绕过 WorkflowRegistry 直接执行 workflowKey。
9. workflow 执行前跳过 param_schema / node_map / output_map 校验。
```
