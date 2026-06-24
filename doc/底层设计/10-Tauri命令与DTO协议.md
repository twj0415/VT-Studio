# Tauri 命令与 DTO 协议

> 这篇定义前后端通信协议。前端页面不得直接 `invoke`，必须通过 `shared/api` 封装；后端 command 只做参数接收、调用 service、返回 DTO。

## 一、核心原则

```text
1. Command 名统一 snake_case。
2. Rust 内部和 SQLite 使用 snake_case。
3. 前端 TypeScript 使用 camelCase。
4. Tauri 边界通过 serde rename_all 显式映射。
5. 所有 command 返回 Result<T, AppErrorDto>。
6. 页面不直接 invoke，必须走 API Client。
7. DTO 不包含真实 API Key。
```

---

## 二、通用返回结构

后端：

```rust
pub type CommandResult<T> = Result<T, AppErrorDto>;
```

前端 API 封装统一处理错误：

```ts
export async function invokeCommand<TResponse, TPayload = void>(
  command: string,
  payload?: TPayload,
): Promise<TResponse>
```

错误结构见：

```text
底层设计/09-错误日志与事件规范.md
底层设计/15-错误码总表与恢复策略.md
```

---

## 三、DTO 命名规则

```text
CreateProjectRequest / ProjectDetailDto
UpdateStoryboardItemRequest / StoryboardItemDto
StartTaskRequest / TaskDetailDto
ProviderConfigDto / ProviderTestResultDto
```

规则：

```text
1. Request 表示前端传入。
2. Dto 表示后端返回给前端。
3. Patch 表示局部更新。
4. ListXxxRequest 必须包含分页参数。
```

分页：

```ts
interface PageRequest {
  page: number
  pageSize: number
}

interface PageResult<T> {
  items: T[]
  total: number
  page: number
  pageSize: number
}
```

---

## 四、Project Commands

### create_project

用途：创建项目和默认设定集。

```ts
interface CreateProjectRequest {
  title: string
  workflowType: WorkflowType
  inputType: InputType
  topic?: string
  sourceText?: string
  sourceTextPath?: string
  contentCategory?: string
  contentLanguage: ContentLanguage
  tone?: string
  aspectRatio: AspectRatio
  targetDurationSeconds: number
  targetSceneCount: number
  segmentDurationSeconds: number
  stylePrompt?: string
  inputProcessMode: InputProcessMode
  inputOptions?: Record<string, unknown>
}
```

返回：

```ts
interface ProjectDetailDto {
  project: ProjectDto
  projectBible: ProjectBibleDto
  styleBible?: StyleBibleDto
  characterBibles: CharacterBibleDto[]
  locationBibles: LocationBibleDto[]
  latestTask?: TaskSummaryDto
}
```

后端流程：

```text
ProjectCommand → ProjectService.create_project → Repository + StorageService
```

---

### list_projects

```ts
interface ListProjectsRequest extends PageRequest {
  keyword?: string
  lifecycle?: ProjectLifecycle
  sortBy?: 'updated_at' | 'created_at' | 'title'
  sortOrder?: 'asc' | 'desc'
}
```

返回：

```ts
type ListProjectsResponse = PageResult<ProjectSummaryDto>
```

`ProjectSummaryDto` 中“生成中/失败/成功”从 latestTask 读取，不从 Project 读取。

---

### get_project_detail

```ts
interface GetProjectDetailRequest {
  projectId: string
}
```

返回 `ProjectDetailDto`。

---

### update_project

```ts
interface UpdateProjectRequest {
  projectId: string
  patch: Partial<{
    title: string
    topic: string
    sourceText: string
    sourceTextPath: string
    inputOptions: Record<string, unknown>
    contentCategory: string
    contentLanguage: ContentLanguage
    tone: string
    aspectRatio: AspectRatio
    targetDurationSeconds: number
    targetSceneCount: number
    segmentDurationSeconds: number
    stylePrompt: string
    coverTitle: string
  }>
}
```

规则：

```text
1. 不允许通过 update_project 修改 project_lifecycle。
2. update_project 可用于草稿自动保存，但只能修改项目基础字段。
3. 修改影响任务的字段时，只影响新任务。
4. 正在运行的任务继续使用 snapshot。
5. 前端离开页面前如果存在 dirty 表单，必须调用对应 update command 或提示用户。
```

---

### archive_project / restore_project / delete_project

```ts
interface ProjectIdRequest {
  projectId: string
}
```

规则：

```text
1. archive_project 软归档。
2. restore_project 从 archived 恢复 active/draft。
3. delete_project 默认软删除；物理删除必须走单独危险操作。
```

---

## 五、Storyboard / StoryboardItem Commands

### get_storyboard

```ts
interface GetStoryboardRequest {
  projectId: string
  storyboardId?: string
}
```

返回：

```ts
interface StoryboardDto {
  storyboardId: string
  projectId: string
  title: string
  workflowType: WorkflowType
  contentLanguage: ContentLanguage
  aspectRatio: AspectRatio
  totalDurationSeconds: number
  itemCount: number
  items: StoryboardItemDto[]
}
```

---

### update_storyboard_item

```ts
interface UpdateStoryboardItemRequest {
  projectId: string
  storyboardId: string
  itemId: string
  patch: StoryboardItemPatchDto
}
```

规则：

```text
1. patch 只允许改白名单字段。
2. 修改 narration/sourceText 必须 reset audio/subtitle/video_prompt_generation/final_composition 后续状态。
3. 修改 imagePrompt 必须 reset image_status/video_status/final_composition 后续状态。
4. 修改 selectedImageId 必须 reset video_status/final_composition。
5. 修改 selectedVideoSegmentId 必须 reset final_composition。
```

---

### batch_update_storyboard_items

```ts
interface BatchUpdateStoryboardItemsRequest {
  projectId: string
  storyboardId: string
  patches: Array<{
    itemId: string
    patch: StoryboardItemPatchDto
  }>
}
```

必须事务化。

---

### reorder_items

```ts
interface ReorderStoryboardItemsRequest {
  projectId: string
  storyboardId: string
  orderedStoryboardItemIds: string[]
}
```

规则：

```text
1. 后端重新写 item_index。
2. item_index 从 1 开始。
3. 不允许前端只改本地顺序不保存。
```

---

## 六、Task Commands

### create_task

```ts
interface CreateTaskRequest {
  projectId: string
  taskKind: TaskKind
  startImmediately: boolean
  options?: Record<string, unknown>
}
```

返回：

```ts
interface CreateTaskResponse {
  taskId: string
  taskStatus: TaskStatus
  steps: TaskStepDto[]
}
```

规则：

```text
1. 创建任务时冻结 snapshot_json。
2. 创建任务时一次性生成 task_steps。
3. startImmediately=true 时交给 PipelineEngine。
4. 从 storyboard_generation 开始，所有 LLM/TTS/生图/图生视频/FFmpeg/Chromium 操作都必须进入 TaskStep；create_project 只创建项目基础数据。
5. script_generation 仅在启用脚本增强或 script_only 任务时创建，不是 image_to_video 主线必经步骤。
```

---

### start_task / cancel_task / resume_task

```ts
interface TaskIdRequest {
  taskId: string
}
```

规则：

```text
1. cancel_task 先写 cancel_requested=true。
2. Worker 在安全点退出并写 cancelled。
3. resume_task 只恢复 failed/cancelled/running 异常中断的任务。
```

---

### retry_task_step

```ts
interface RetryTaskStepRequest {
  taskId: string
  taskStepId: string
  resetDownstream: boolean
}
```

规则：

```text
1. 默认 resetDownstream=true。
2. 后端根据 step_kind 清理后续 StoryboardItem 状态和产物引用。
3. 不直接删除历史 artifact。
```

---

### approve_task_step

用于 `waiting_user` 节点。

```ts
interface ApproveTaskStepRequest {
  taskId: string
  taskStepId: string
  approved: boolean
  comment?: string
}
```

规则：

```text
approved=true  → step_status=succeeded，继续任务
approved=false → step_status=cancelled 或 failed，按业务决定是否终止
review_required=false → 对应 review step 创建后直接 step_status=skipped
```

---

### get_task_detail / list_tasks

```ts
interface GetTaskDetailRequest {
  taskId: string
}
```

返回：

```ts
interface TaskDetailDto {
  task: TaskDto
  steps: TaskStepDto[]
  attempts: TaskAttemptDto[]
  artifacts: ArtifactDto[]
  result?: TaskResultDto // 来自 tasks.result_json
}
```

---

## 七、Provider Commands

### list_providers

返回 Provider 配置摘要，不返回真实密钥。

```ts
interface ProviderDto {
  providerId: string
  providerKind: ProviderKind
  vendor: ProviderVendor
  displayName: string
  baseUrl?: string
  keyAlias?: string
  status: ProviderStatus
  isEnabled: boolean
  models: ProviderModelDto[]
}
```

---

### save_provider

```ts
interface SaveProviderRequest {
  providerId?: string
  providerKind: ProviderKind
  vendor: ProviderVendor
  displayName: string
  baseUrl?: string
  authType: ProviderAuthType
  keyAlias?: string
  secretValue?: string
  config?: Record<string, unknown>
}
```

规则：

```text
1. secretValue 只用于写 keyring，绝不进 SQLite。
2. 返回时不带 secretValue。
3. keyAlias 必须稳定。
```

---

### test_provider

```ts
interface TestProviderRequest {
  providerId?: string
  providerDraft?: SaveProviderRequest
  modelName?: string
  providerKind: ProviderKind
}
```

返回：

```ts
interface ProviderTestResultDto {
  ok: boolean
  latencyMs?: number
  error?: AppError
  modelInfo?: Record<string, unknown>
}
```

---

### list_provider_models / save_provider_model

模型能力矩阵由后端校验：

```ts
interface ProviderModelDto {
  providerModelId: string
  providerId: string
  modelName: string
  providerKind: ProviderKind
  abilityTypes: ModelAbilityType[]
  inputModalities: Array<'text' | 'image' | 'audio' | 'video'>
  outputModalities: Array<'text' | 'image' | 'audio' | 'video'>
  featureFlags?: ModelFeatureFlag[]
  limits: ModelLimitsDto
  isDefault: boolean
  isEnabled: boolean
}
```

### list_workflow_presets / save_workflow_preset

Workflow preset 用于注册 ComfyUI / RunningHub 工作流。API 模型仍使用 `provider_models`，不得混用。

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

interface ListWorkflowPresetsRequest {
  providerId?: string
  vendor?: 'comfyui' | 'runninghub'
  abilityType?: string
  includeDisabled?: boolean
}

interface SaveWorkflowPresetRequest {
  preset: WorkflowPresetDto
}
```

规则：

```text
1. save_workflow_preset 必须校验 paramSchema / nodeMap / outputMap。
2. isBuiltin=true 的 preset 不允许被前端删除，只允许启用/禁用。
3. RunningHub preset 必须有 workflowId；ComfyUI selfhost preset 必须有 workflowKey。
```

### test_workflow_preset / scan_builtin_workflow_presets

```ts
interface TestWorkflowPresetRequest {
  workflowPresetId: string
  mode: 'dry_run' | 'real_generate'
  sampleParams?: Record<string, unknown>
}

interface TestWorkflowPresetResponse {
  ok: boolean
  mode: 'dry_run' | 'real_generate'
  errors: string[]
  outputArtifactPath?: string
  metadata?: Record<string, unknown>
}

interface ScanBuiltinWorkflowPresetsResponse {
  insertedCount: number
  updatedCount: number
  skippedCount: number
  errors: string[]
}
```

规则：

```text
1. dry_run 只检查 Provider 可达、workflow 文件或 workflow_id 存在、schema 完整、node_map/output_map 可解析。
2. real_generate 才允许真实生成样例媒体，必须明确由用户触发。
3. scan_builtin_workflow_presets 只扫描内置 resources/workflows，不执行 workflow。
```

### list_executable_media_options

统一返回 API 模型和 workflow preset，供 AI 生图、AI 视频、数字人、画布精修等模块选择。

```ts
interface ListExecutableMediaOptionsRequest {
  providerKind?: 'image' | 'video' | 'workflow'
  abilityType?: string
  inputModalities?: string[]
  outputModalities?: string[]
  includeUnavailable?: boolean
}

interface ExecutableModelOptionDto {
  id: string
  sourceType: 'provider_model' | 'workflow_preset'
  providerId: string
  providerKind: 'llm' | 'tts' | 'image' | 'video' | 'vlm' | 'workflow'
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

规则：

```text
1. 前端模型选择器只使用 list_executable_media_options。
2. 前端不得自行合并 provider_models 和 workflow_presets。
3. sourceType=workflow_preset 时，后端后续请求使用 workflowPresetId。
4. sourceType=provider_model 时，后端后续请求使用 providerModelId。
```

---

## 八、Asset Commands

### import_asset

```ts
interface ImportAssetRequest {
  projectId?: string
  sourcePath: string
  assetKind: AssetKind
  displayName?: string
  usageKind?: string
}
```

规则：

```text
1. sourcePath 必须由 Tauri 文件选择器产生。
2. 后端复制到 workspace/assets 或 project 目录。
3. 入库只存复制后的 relative_path。
```

---

### list_assets / delete_asset

```ts
interface ListAssetsRequest extends PageRequest {
  projectId?: string
  assetKind?: AssetKind
  keyword?: string
}
```

删除规则：

```text
1. delete_asset 默认软删除。
2. 被引用资产不能物理删除。
3. 不删除用户原始文件。
```

---

## 九、Template Commands

### list_templates

```ts
interface ListTemplatesRequest {
  templateType?: TemplateType
  aspectRatio?: AspectRatio
}
```

### preview_template

```ts
interface PreviewTemplateRequest {
  templateId: string
  aspectRatio: AspectRatio
  params: Record<string, unknown>
  itemId?: string
}
```

返回：

```ts
interface PreviewTemplateResponse {
  previewPath: string
  width: number
  height: number
}
```

---

## 十、Config / Dictionary Commands

```text
get_app_config
update_app_config
get_pipeline_config
update_pipeline_config
get_dictionary
list_dictionaries
```

规则：

```text
1. 字典只返回 code 和 label_key，不返回硬编码中文。
2. 配置更新必须 schema 校验。
3. 修改 Provider/Pipeline 配置不影响已创建任务。
```

---

## 十一、Export Commands

### export_video

```ts
interface ExportVideoRequest {
  projectId: string
  taskId: string
  targetPath?: string // 必须来自 Tauri save dialog 授权，只用于最终复制导出，不入库
  includeCover: boolean
  includeSubtitles: boolean
}
```

### export_project_package

```ts
interface ExportProjectPackageRequest {
  projectId: string
  targetPath: string
  includeTaskArtifacts: boolean
}
```

规则：

```text
1. ExportConfig 是默认值；export_video 请求参数覆盖本次导出，并写入任务快照。
2. targetPath 只能来自 Tauri save dialog 授权路径，只用于最终复制导出，不写入数据库；内部产物仍使用相对路径。
3. 项目包不得包含真实 API Key。
4. 导出前生成 manifest.json。
5. 所有路径重写为包内相对路径。
```

---

## 十二、前端 API 目录

```text
src/shared/api/
  invoke.ts
  project.api.ts
  storyboard.api.ts
  task.api.ts
  provider.api.ts
  asset.api.ts
  template.api.ts
  config.api.ts
  export.api.ts
```

页面只能调用这些 API，不允许直接调用 `@tauri-apps/api/core.invoke`。
