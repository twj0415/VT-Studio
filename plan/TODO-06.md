# TODO-06：Provider、模型能力与 Workflow 注册

> 目标：统一所有模型、工作流和 PromptSkill 的注册、选择、调用与校验。  
> 本文件来自 `doc/底层设计/12`、`23`、`18`、`Provider与安全`、`doc/功能模块/18-设置与模型配置.md`。

---

## 阶段目标

建立：

```text
ProviderManager
ModelRegistry
WorkflowRegistry
provider_models
workflow_presets
list_executable_media_options
ImageInputPlan / VideoInputPlan
PromptSkill
结构化输出校验
```

业务层只能选择 `provider_model_id` 或 `workflow_preset_id`，不能按具体模型名、供应商、workflow key 写分支。

---

## 本阶段范围

包含：

- Provider 配置模型。
- API 模型能力矩阵。
- ComfyUI / RunningHub workflow preset 注册。
- 统一媒体能力选择器。
- Provider 连通性测试。
- PromptSkill 文件化和版本化。
- LLM 结构化输出校验。
- “模型 / 工作流”页 Provider / 模型 / workflow preset 基础表单。
- 基于模型能力生成图片 / 视频输入规划，供前端动态渲染缺项和参数表单。

不包含：

- 所有真实供应商一次性接完。
- 执行用户上传 JS/TS/Python Provider。
- 未注册 workflow 执行。

---

## TODO

> 本文件的每条 TODO 按以下口径执行：
> - 顺序：只做本文件中第一条未完成 TODO；本文件未完成前不得跳到后续 TODO 文件。
> - 规范：先遵守本阶段范围、底层设计、安全红线、命名规则和 `plan/阶段路线图.md` 的完成判定。
> - 做什么：落地到具体文件、接口、页面、表结构、DTO、Store、Service、组件、样式、i18n、测试或文档。
> - 做到怎么样：UI、逻辑、样式、组件封装、多语言、状态、错误处理、安全、验证全部满足，才算完成。
> - 怎么做：按“改法”小步实现；不要引入本阶段明确排除的能力。
> - 打标：完成一条后必须立刻把 `【】` 改成 `【X】`，并写完成记录；不需要用户提醒。
> - 停止：如果需求不清、验收不清、文档冲突、数据结构不明或风险高，停下来说明，不靠猜测继续。

### 【】6.1 实现 ProviderManager

**问题：**  
如果页面或业务服务直接调模型，后续能力矩阵、密钥安全、日志脱敏、重试都会失控。

**改法：**

ProviderKind：

```text
llm
tts
image
video
vlm
workflow
```

ProviderManager 统一入口：

```text
call_llm
generate_image
generate_video
generate_tts
analyze_asset
run_workflow
```

**验证：**

- 业务层不直接调用供应商 SDK。
- Provider adapter 不写数据库。
- 日志不出现 API Key。

**风险：**  
不要把 ComfyUI / RunningHub 当普通 ProviderKind，它们是 workflow vendor。

---

### 【】6.2 实现模型 / 工作流页的 Provider 设置与密钥引用

**问题：**  
Provider 连接配置、模型能力、workflow preset 不能混在一个表单。

**改法：**

“模型 / 工作流”页拆三层：

```text
Provider 连接配置
API 模型能力配置
Workflow Preset 配置
```

ProviderConfig 只保存连接信息和 `key_alias`，不保存具体 model_name。

**验证：**

- 保存 Provider 不返回 secret。
- 删除 Provider 不自动删除 keyring secret，需明确策略。
- 测试 Provider 能返回成功/失败。

**风险：**  
真实密钥不能进 SQLite、日志、DTO、导出配置。

---

### 【】6.3 实现 provider_models 能力矩阵

**问题：**  
不同模型支持的输入、比例、时长、分辨率不同，不能靠前端口头限制。

**改法：**

`provider_models` 记录：

```text
provider_id
provider_kind
vendor
model_name
capabilities
aspect_ratios
resolutions
duration_range
fps_range
input_requirements
status
```

**验证：**

- 参数超过能力矩阵时后端拒绝。
- disabled model 不可执行。
- 前端模型选择来自后端能力列表。

**风险：**  
业务层不得按 `model_name` 写 if/else。

---

### 【】6.4 实现 workflow_presets 注册

**问题：**  
ComfyUI / RunningHub workflow 不能裸 key 执行，否则参数、节点、输出不可控。

**改法：**

workflow preset 包含：

```text
workflow_preset_id
vendor
name
capabilities
param_schema
node_map
output_map
default_params
status
```

执行前必须校验：

```text
param_schema
node_map
output_map
```

**验证：**

- 未注册 workflow 执行失败。
- 缺节点返回 `workflow.invalid_node_map`。
- 缺输出返回 `workflow.output_missing`。
- RunningHub 远程 URL 下载到任务目录后再入库。

**风险：**  
不允许执行用户上传的任意 JS/TS/Python Provider 代码。

---

### 【】6.5 实现 `list_executable_media_options`

**问题：**  
前端不应该自己合并 API 模型和 workflow preset，也不应该硬编码某个模型需要哪些图片、参数或 ComfyUI 节点。

**改法：**

后端统一返回：

```text
sourceType: provider_model | workflow_preset
sourceId
label
providerKind
capabilities
constraints
inputPlan
status
```

业务调用时只能二选一：

```text
provider_model_id
workflow_preset_id
```

默认规则：不能同时为空，也不能同时有值。

`inputPlan` 至少分为：

```text
ImageInputPlan:
- prompt
- negativePrompt
- aspectRatio
- resolution
- seed
- characterReference[]
- styleReference[]
- pose / depth / mask / referenceAsset[]
- workflowParams

VideoInputPlan:
- startFrame
- endFrame
- characterReference[]
- styleReference[]
- videoPrompt
- durationSeconds
- fps
- resolution
- motionStrength / cameraMotion
- pose / depth / mask / referenceAsset[]
- workflowParams
```

典型能力映射：

```text
普通图生视频：startFrame + videoPrompt + durationSeconds
首尾帧模型：startFrame + endFrame + videoPrompt + durationSeconds
参考图模型：startFrame + characterReference/styleReference + videoPrompt
ComfyUI / RunningHub：根据 workflow_preset.param_schema 返回 pose/depth/mask/workflowParams
无分镜图模型：资产图或参考图 + prompt + 模型参数，不强制 selectedImageId
```

前端只能根据 `inputPlan.required / optional / constraints` 渲染字段、缺项提示和禁用状态，不能在页面里按模型名写分支。

**验证：**

- API 图片模型和 ComfyUI 生图 workflow 能在同一选择器展示。
- 不同图片 / 视频模型返回不同 `ImageInputPlan / VideoInputPlan`。
- 前端能根据输入规划动态展示必填项、可选项、缺失原因和参数范围。
- disabled 项不可选择。
- 后端再次校验能力。

**风险：**  
不能只靠前端禁用来保证安全；后端必须按能力矩阵和输入规划再次校验。

---

### 【】6.6 实现创作规则 / PromptSkill 文件化

**问题：**  
创作规则如果散落在 Vue 页面或 Rust 字符串里，无法版本化、复现和诊断。用户侧统一叫“创作规则”，技术侧可以使用 `PromptSkill / SkillDefinition / SkillSnapshot`。

**改法：**

内置目录：

```text
workspace/prompts/builtin/script
workspace/prompts/builtin/storyboard
workspace/prompts/builtin/image_prompt
workspace/prompts/builtin/video_prompt
workspace/prompts/builtin/subtitle
workspace/prompts/builtin/cover
workspace/prompts/builtin/review
```

frontmatter 必填：

```text
key
name
version
module
provider_kind
output_schema
description
```

任务快照记录：

```text
skill_key
skill_version
content_hash
schema_hash
```

**验证：**

- Vue 页面无大段 prompt。
- Rust service 无不可追踪 prompt。
- 删除或修改用户创作规则不影响历史任务。

**风险：**  
创作规则不版本化会导致历史任务不可复现。

---

### 【】6.7 实现 LLM 结构化输出校验

**问题：**  
LLM 输出不稳定，核心数据不能直接写表。

**改法：**

核心输出必须 schema 校验：

```text
ScriptNarrations
StoryboardItems
ImagePromptList
VideoPromptList
SubtitleChunks
AssetAnalysis
ReviewReport
```

流程：

```text
JSON parse
schema validate
repair loop 最多 2 次
仍失败则 step failed 或 waiting_user
```

**验证：**

- Markdown 包裹 JSON 可被清洗或拒绝。
- 数量不一致会触发修复或失败。
- 不合格输出不能写核心表。

**风险：**  
不要用普通文本解析代替 schema 校验。

---

### 【】6.8 实现 dry_run / real_generate 测试

**问题：**  
Provider 保存成功不代表模型真实可用。

**改法：**

支持：

```text
dry_run：轻量连通性/参数校验
real_generate：真实调用，必须用户明确点击并二次确认
```

**验证：**

- LLM/Image/Video/TTS/VLM 能按能力测试。
- Video real_generate 有二次确认。
- 测试结果不泄露密钥。

**风险：**  
真实生成可能产生费用，不能保存配置时自动触发。

---

## 阶段完成标准

- ProviderManager、ModelRegistry、WorkflowRegistry 可用。
- API 模型与 workflow preset 分离存储。
- 前端统一通过 `list_executable_media_options` 选择，并通过 `ImageInputPlan / VideoInputPlan` 动态渲染模型输入。
- 用户侧创作规则有版本、hash、schema；技术侧 PromptSkill 可追踪。
- LLM 核心输出必须 schema 校验。
- 未注册 workflow、越权参数、disabled 模型都会被后端拒绝。






