# TODO-02：主线数据模型与 API 协议

> 目标：把 `image_to_video` 主线的数据结构和 Tauri DTO 协议定稳。  
> 本文件来自 `doc/底层设计/数据结构.md`、`10-Tauri命令与DTO协议.md`、`doc/功能模块/01/03/07/08/13/14`。

---

## 阶段目标

定义并落地：

```text
Project
StoryboardItem
ImageCandidate
VideoSegment
CompositionTask
AppErrorDto
主线 API / Store / Tauri Command
```

当前主线必须表达：

```text
文字输入 → 分镜 → 多候选图 → 选择最终图 → 图生视频片段 → 选择最终片段 → 合成 final.mp4
```

---

## 本阶段范围

包含：

- Project 主线字段。
- StoryboardItem 数据结构。
- ImageCandidate 数据结构。
- VideoSegment 数据结构。
- CompositionTask 数据结构。
- AppErrorDto 边界协议。
- 前端 entity types / store / api。
- Rust DTO / Tauri command。
- serde snake_case ↔ camelCase 映射。

不包含：

- 真实 Provider 调用。
- 复杂 UI。
- TTS / 字幕 / 封面字段全量实现。

---

## TODO

> 本文件的每条 TODO 按以下口径执行：
> - 顺序：只做本文件中第一条未完成 TODO；本文件未完成前不得跳到后续 TODO 文件。
> - 规范：先遵守本阶段范围、底层设计、安全红线、命名规则和 `plan/阶段路线图.md` 的完成判定。
> - 问题：必须说清不做会造成什么用户问题、工程问题或后续返工。
> - 位置：必须落到页面、接口、表结构、DTO、Store、Service、组件、样式、i18n、测试或文档；不能只写“相关文件”。
> - 改法：按小步实现，写清数据流、状态流、边界和本阶段不做什么。
> - 验收：写清做到什么客观状态才算完成，不能把验证命令当验收。
> - 验证：写清命令、页面流程、数据库检查、文件检查、日志检查或 smoke test。
> - 下一步：本条必须满足“下一步进入条件”后，才能打勾进入下一条；旧 TODO 缺字段时先补齐再实现。
> - 打标：完成一条后必须立刻把 `【】` 改成 `【X】`，并写完成记录；不需要用户提醒。
> - 停止：如果需求不清、验收不清、文档冲突、数据结构不明或风险高，停下来说明，不靠猜测继续。

### 【X】2.1 定义 Project 主线字段

**问题：**  
旧项目分类把内容来源、制作流程、成片形态混在一起。

**改法：**

Project 至少包含：

```text
projectId
workflowType
inputType
inputProcessMode
sourceText
sourceTextPath
aspectRatio
targetSceneCount
segmentDurationSeconds
stylePrompt
contentLanguage
lifecycle
createdAt
updatedAt
```

**验证：**

- 创建项目请求包含 `workflowType` 和 `inputType`。
- `paste` 使用 fixed 模式，不调用 LLM 改写。
- 长文本使用 `sourceTextPath`。

**风险：**  
不要继续使用 `videoType` 承担 workflowType 职责。

**完成记录：**

- 前端 `entities/project/types.ts` 已定义 `ProjectDto / ProjectSummaryDto / ProjectDetailDto / CreateProjectRequest` 主线字段，覆盖 `projectId / workflowType / inputType / inputProcessMode / sourceText / sourceTextPath / aspectRatio / targetSceneCount / segmentDurationSeconds / stylePrompt / contentLanguage / lifecycle / createdAt / updatedAt`。
- 前端项目 mock API 已按 `inputType=paste` 强制使用 `inputProcessMode=fixed`，短文本进入 `sourceText`，超过 20KB 或已给路径的长文本进入相对 `sourceTextPath=input/source.txt`；首页画幅展示改为读取 `project.aspectRatio`，不再写死。
- Rust `domain/project.rs`、`commands/project.rs`、`services/project_service.rs` 已补齐同一组 DTO 字段，Tauri 边界继续用 `serde(rename_all = "camelCase")` 显式映射，后端字段保持 snake_case。
- 搜索确认源码中无 `videoType / video_type` 承担 workflow 职责。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

### 【X】2.2 定义 StoryboardItem

**问题：**  
旧 Scene 字段偏影视镜头，不适合当前图生视频流水线。

**改法：**

StoryboardItem 至少包含：

```text
itemId
projectId
index
sourceText
narrationText
visualDescription
characters
characterIds
sceneDescription
locationId
imagePrompt
negativePrompt
videoPrompt
durationSeconds
selectedImageId
selectedVideoSegmentId
status
lockFlagsJson
```

景别、运镜、转场可作为高级可选字段，不作为主线必填。

**验证：**

- 不填写景别/运镜/转场也能进入生图。
- 进入生图前校验 imagePrompt、visualDescription、durationSeconds。
- `itemId` 稳定，`index` 仅表示顺序。

**风险：**  
如果保留旧 `Scene` 命名，必须明确它承载 StoryboardItem 语义。

**完成记录：**

- 前端 `entities/scene/types.ts` 已定义 `StoryboardItemDto`，覆盖 `itemId / projectId / index / sourceText / narrationText / visualDescription / characters / characterIds / sceneDescription / locationId / imagePrompt / negativePrompt / videoPrompt / durationSeconds / selectedImageId / selectedVideoSegmentId / status / lockFlagsJson`；`shotSize / cameraMotion / composition / pace / transitionType` 仅作为高级可选字段保留。
- 当前 `entities/scene` 目录和 `SceneDto` 仅保留为兼容别名，文件内已明确其语义是 StoryboardItem；页面和 store 主数据已改为 `storyboard.items`、`itemId`、`index`、`narrationText`。
- 新增 `entities/scene/validation.ts`，进入生图前校验每条分镜的原文或旁白、`visualDescription`、`imagePrompt`、`durationSeconds > 0`；景别、运镜、转场不参与必填校验。
- 分镜页“进入生图”不再直接跳转，先执行校验；失败时用 i18n + Naive Message 明确提示第几条缺什么，并定位到该条。
- Rust `domain/scene.rs` 与 `services/scene_service.rs` 已同步 `StoryboardItemDto` 字段，Tauri 边界继续用 `serde(rename_all = "camelCase")` 显式映射，`reorder_scenes` 只更新 `index`，不改 `itemId`。
- 搜索确认源码中无旧 `sceneId / sceneIndex / sceneRole / estimatedDurationSeconds / storyboard.scenes` 主字段残留。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

### 【X】2.3 定义 ImageCandidate

**问题：**  
每个分镜只有一个 `imagePath` 无法支撑多候选图和用户选择。

**改法：**

ImageCandidate 至少包含：

```text
imageId
itemId
imagePath
prompt
negativePrompt
model
providerModelId
workflowPresetId
status
selected
createdAt
derivedFromImageId
generationContextSnapshot
```

**验证：**

- 每个 StoryboardItem 可有 0~N 张候选图。
- 同一 item 只能选中一张图。
- 重生成保留旧候选图。
- 远程 URL 已转存本地后入库。

**风险：**  
不要把候选图只存在前端临时状态。

**完成记录：**

- 前端 `entities/scene/types.ts` 已定义 `ImageCandidateDto`，覆盖 `imageId / itemId / imagePath / prompt / negativePrompt / model / providerModelId / workflowPresetId / status / selected / createdAt / derivedFromImageId / generationContextSnapshot`。
- `StoryboardItemDto` 已挂载 `imageCandidates: ImageCandidateDto[]`，每条分镜可表达 0~N 张候选图；候选图数据收口在 `entities/scene/api.ts` 与 `useSceneStore`，不放在页面临时数组里。
- `startImageGeneration` 会追加新候选图，不删除旧候选图；`selectImageCandidate` 会把同一 `itemId` 下其他候选图置为 `selected=false`，并回写 `StoryboardItem.selectedImageId`。
- mock Provider 输出先模拟为远程 URL，再统一转换为相对 `imagePath=images/{itemId}/{imageId}.png` 后进入候选图数据，表达“远程 URL 转存本地后入库”的协议边界；真实下载和存储留到 TODO-04/07。
- Rust `domain/scene.rs` 已同步 `ImageCandidateDto`，并在 `StoryboardItemDto.image_candidates` 表达候选图关系，Tauri 边界继续用 camelCase serde 映射。
- 搜索确认候选图逻辑集中在 entity 数据层，页面层没有本地 `ImageCandidate` mock 数组。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

### 【X】2.4 定义 VideoSegment

**问题：**  
视频片段和最终合成视频不能混成一个字段。

**改法：**

VideoSegment 至少包含：

```text
segmentId
itemId
inputImageId
videoPath
videoPrompt
durationSeconds
model
providerModelId
workflowPresetId
status
selected
createdAt
generationContextSnapshot
```

**验证：**

- 图生视频必须读取 `selectedImageId` 对应的 ImageCandidate。
- 同一 item 可有多个 VideoSegment。
- 同一 item 只能确认一个 selectedVideoSegmentId。

**风险：**  
不能绕过已选图直接文生视频进入当前主线。

**完成记录：**

- 前端 `entities/scene/types.ts` 已定义 `VideoSegmentDto`，覆盖 `segmentId / itemId / inputImageId / videoPath / videoPrompt / durationSeconds / model / providerModelId / workflowPresetId / status / selected / createdAt / generationContextSnapshot`。
- `StoryboardItemDto` 已挂载 `videoSegments: VideoSegmentDto[]`，每条分镜可表达多个视频片段，并通过 `selectedVideoSegmentId` 指向最终片段。
- `startVideoGeneration` 必须读取当前 `StoryboardItem.selectedImageId`，并校验能找到对应 `ImageCandidate` 后才生成 `VideoSegment`；没有走文生视频或绕过已选图的逻辑。
- `selectVideoSegment` 会把同一 `itemId` 下其他片段置为 `selected=false`，并回写 `StoryboardItem.selectedVideoSegmentId`。
- mock Provider 输出先模拟为远程 URL，再统一转换为相对 `videoPath=videos/{itemId}/{segmentId}.mp4`；真实下载、文件校验和转存留到 TODO-04/07。
- 新增 `validateStoryboardItemsForVideoGeneration`，进入图生视频前可校验每条 StoryboardItem 是否已有 `selectedImageId`。
- Rust `domain/scene.rs` 已同步 `VideoSegmentDto`，并在 `StoryboardItemDto.video_segments` 表达片段关系，Tauri 边界继续用 camelCase serde 映射。
- 搜索确认源码中无 `text_to_video / textToVideo` 绕过当前 `image_to_video` 主线。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

### 【X】2.5 定义 CompositionTask

**问题：**  
合成任务需要能追踪片段顺序、输出路径、状态、错误。

**改法：**

CompositionTask 至少包含：

```text
taskId
projectId
segmentIds
outputPath
status
progress
errorJson
createdAt
updatedAt
```

**验证：**

- 合成前校验所有 item 有 selectedVideoSegmentId。
- 按 StoryboardItem.index 拼接，而不是按文件名。
- 输出 final.mp4 路径写入 outputPath。

**风险：**  
不要把最终视频路径写在某个分镜或视频片段上。

**完成记录：**

- 前端 `entities/task/types.ts` 已定义 `CompositionTaskDto`，覆盖 `taskId / projectId / segmentIds / outputPath / status / progress / errorJson / createdAt / updatedAt`，并允许 `TaskDetailDto.compositionTask` 承载合成任务。
- 新增 `validateStoryboardItemsForComposition`，合成前可校验每条 StoryboardItem 都有 `selectedVideoSegmentId`。
- `startComposition` 会按 `StoryboardItem.index` 排序收集 `selectedVideoSegmentId`，并校验每个选中片段能在 `VideoSegment` 数据中找到；不按文件名排序。
- mock 合成任务的最终输出只写入 `CompositionTask.outputPath=exports/final.mp4`，没有写到 StoryboardItem 或 VideoSegment。
- `useSceneStore` 已增加 `compositionTasks/currentCompositionTask/startComposition/loadCompositionTask`，合成任务不放在页面临时状态。
- Rust `domain/task.rs` 已同步 `CompositionTaskDto`，`TaskDetailDto` 通过 `composition_task` 可携带合成任务；当前 `approve_task_step` 不伪造空 segmentIds 的合成任务，实际创建留给 TODO-02 的 command 定义和后续任务队列阶段。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

### 【X】2.6 定义主线 Tauri command

**问题：**  
前端需要稳定 API，不能页面直接造数据。

**改法：**

至少实现或声明：

```text
create_project
list_projects
get_project_detail
update_project

get_storyboard
update_storyboard_item
batch_update_storyboard_items
reorder_storyboard_items

generate_image_prompts
start_image_generation
select_image_candidate

start_video_generation
select_video_segment

start_composition
get_task_detail

get_app_config
update_app_config
list_dictionaries
list_executable_media_options
```

**验证：**

- command 名使用 snake_case。
- Rust / SQLite / JSON 存 snake_case。
- 前端 TypeScript 使用 camelCase。
- serde 显式映射。

**风险：**  
DTO 不得包含真实密钥、完整 Provider 请求头、用户绝对路径。

**完成记录：**

- Rust 已注册主线 command：`create_project / list_projects / get_project_detail / update_project / get_storyboard / update_storyboard_item / batch_update_storyboard_items / reorder_storyboard_items / generate_image_prompts / start_image_generation / select_image_candidate / start_video_generation / select_video_segment / start_composition / get_task_detail / get_app_config / update_app_config / list_dictionaries / list_executable_media_options`。
- 旧 `update_scene / batch_update_scenes / reorder_scenes` 已从 Tauri handler 移除，改为 StoryboardItem 语义 command。
- 新增并注册 `commands/media.rs`、`domain/media.rs`、`services/media_service.rs`，提供 `list_executable_media_options` stub；当前只返回 mock provider/workflow 能力，不触发真实 Provider。
- `domain/scene.rs` 已补齐 `GenerateImagePromptsRequest / StartImageGenerationRequest / SelectImageCandidateRequest / StartVideoGenerationRequest / SelectVideoSegmentRequest`，`domain/task.rs` 已补 `StartCompositionRequest`，Tauri 边界统一 `serde(rename_all = "camelCase")`。
- 前端新增 `shared/api/commands.ts`，用 camelCase key 映射 snake_case command 名，避免页面散写 command 字符串。
- DTO 中未加入真实密钥、Provider 请求头或用户绝对路径；媒体输出仍使用受控相对路径协议。
- 验证通过：`cargo check`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`。

---

### 【X】2.7 建立前端 entity API / store

**问题：**  
页面不能直接 invoke，也不能直接拼 mock。

**改法：**

建立：

```text
entities/project/types.ts
entities/project/api.ts
entities/project/store.ts
entities/storyboard/types.ts
entities/storyboard/api.ts
entities/storyboard/store.ts
entities/task/types.ts
entities/task/api.ts
shared/api/invoke.ts
```

**验证：**

- 页面调用 store action / entity api。
- shared/api 统一处理 AppErrorDto。
- mock adapter 与 Tauri adapter 可切换。

**风险：**  
不要把页面 UI 状态和业务持久状态混在一起。

**完成记录：**

- `entities/project/types.ts / api.ts / store.ts`、`entities/task/types.ts / api.ts / store.ts` 已存在并接入主线 DTO；项目、任务 API 在 `getApiAdapter()==='tauri'` 时走统一 `callCommand`，默认仍使用当前 mock 数据层。
- 新增 `entities/storyboard/types.ts / api.ts / store.ts / validation.ts / ui.ts` 作为 StoryboardItem 公开业务入口；分镜页和脚本页已改从 `entities/storyboard` 导入，旧 `entities/scene` 仅保留为兼容实现。
- `shared/api/invoke.ts` 已支持 `mock | tauri` adapter 切换，提供 `setApiAdapter / getApiAdapter / registerMockCommand / clearMockCommands`；`shared/api/client.ts` 统一经 `normalizeApiError` 处理 command 错误。
- 新增 `shared/api/commands.ts`，用 camelCase key 集中映射 snake_case Tauri command 名，页面不直接散写 command 字符串。
- Storyboard 数据层集中管理 `StoryboardItem / ImageCandidate / VideoSegment / CompositionTask` mock 状态；页面只调用 store action 或 entity API，不直接持有核心业务数据源。
- 搜索确认 `pages/` 与 `widgets/` 中无直接 `@tauri-apps/api/core.invoke`、`invoke(`、`callCommand(`，也无旧 `@/entities/scene` 页面导入。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

### 【X】2.8 定义进入下一阶段的校验规则

**问题：**  
步骤跳转不能只靠按钮灰掉，后端也要校验。

**改法：**

进入生图前：

```text
item 数量 > 0
sourceText 或 narrationText 非空
visualDescription 非空
imagePrompt 非空
durationSeconds > 0
引用角色/场景 ID 必须存在
```

进入视频前：

```text
每条 StoryboardItem 有 selectedImageId
```

进入合成前：

```text
每条 StoryboardItem 有 selectedVideoSegmentId
```

**验证：**

- 校验失败明确提示第几条缺什么。
- 前端和后端校验一致。

**风险：**  
不要只靠前端 StepBar 禁用来保证流程正确。

**完成记录：**

- 前端 `entities/storyboard/validation.ts` 公开 `validateStoryboardItemsForImageGeneration / validateStoryboardItemsForVideoGeneration / validateStoryboardItemsForComposition`，分别覆盖进入生图、进入视频、进入合成的主线校验。
- 进入生图前校验：item 数量、`sourceText` 或 `narrationText`、`visualDescription`、`imagePrompt`、`durationSeconds > 0`；并支持通过 `validCharacterIds / validLocationIds` 校验角色和场景引用是否存在。
- 进入视频前校验：每条 StoryboardItem 必须有 `selectedImageId`。
- 进入合成前校验：每条 StoryboardItem 必须有 `selectedVideoSegmentId`。
- 分镜页“进入生图”已使用校验 helper；失败时通过 i18n + Naive Message 明确提示第几条缺少哪些字段，并定位到该条，不只依赖 StepBar 禁用。
- Rust `scene_service` 已补同口径校验：生图前检查原文/旁白、画面描述、生图提示词和时长；图生视频前阻断缺少 `selected_image_id`；合成校验 helper 阻断缺少 `selected_video_segment_id`。
- Rust `start_composition` stub 不再伪造合成结果，缺少已确认视频片段时返回明确阻断错误，避免后端被前端状态绕过。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

### 【X】2.9 定义 `AppErrorDto` 与前端错误映射

**问题：**  
错误如果只返回字符串，后续页面无法做 i18n、恢复动作、重试判断和日志关联；如果等到导出诊断阶段再补，会反向影响任务队列、Provider 和 FFmpeg。

**改法：**

统一 Tauri 边界错误结构：

```text
code
kind
message
detail
isRetryable
recoverAction
traceId
```

错误码格式：

```text
domain.specific_error
```

前端建立：

```text
shared/api/errors.ts
shared/i18n error code mapping
```

**验证：**

- Tauri command 失败统一返回 AppErrorDto。
- 前端 shared/api 能识别 code、kind、isRetryable、recoverAction。
- 用户可见错误优先用 code 映射 i18n。
- detail 不直接展示给普通用户。

**风险：**  
错误 detail 可能包含路径、密钥、Provider 原始响应，必须在进入 DTO 前脱敏。

**完成记录：**

- Rust `core/error.rs` 已定义 `AppErrorDto`，覆盖 `code / kind / message / detail / is_retryable / recover_action / trace_id`；`core/result.rs` 已将 `AppResult<T>` 统一为 `Result<T, AppErrorDto>`。
- 所有 Tauri command 已从 `Result<T, String>` 改为 `AppResult<T>`，并在 command 边界把 service 的 `String` 错误统一映射为 `AppErrorDto`；service 内部暂不提前大改，后续错误码细分留给任务队列和错误码阶段。
- `AppErrorDto::from_message` 会把校验类错误映射为 `code=validation.failed / kind=validation / is_retryable=false`，其他 command 失败映射为 `code=app.command_failed / kind=unknown / is_retryable=true / recoverAction=retry`。
- 前端 `shared/api/errors.ts` 已更新为新结构，`AppApiError` 可识别 `code / kind / isRetryable / recoverAction / traceId / detail`；保留旧 `recoverable/details` 兼容读取，但新 DTO 不再以旧字段为主。
- 前端新增 `getApiErrorI18nKey`，`zh-CN/en-US` 已补 `errors.app.unknown / errors.app.command_failed / errors.validation.failed`，用户可见错误可优先用 code 做 i18n 映射；`detail` 不作为普通用户展示字段。
- 当前 DTO 未携带真实密钥、完整 Provider 请求头或用户绝对路径；`detail` 仅写入脱敏的 `source=command_boundary`。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

## 阶段完成标准

- Project / StoryboardItem / ImageCandidate / VideoSegment / CompositionTask 类型稳定。
- AppErrorDto 边界协议稳定，前端 shared/api 可统一处理。
- 前端 entity API / store 可用。
- Tauri command 命名与 DTO 映射符合规范。
- 主线跳转校验完整。
- 页面不直接 invoke，不直接持有核心业务数据源。






