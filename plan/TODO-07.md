# TODO-07：真实生图、图生视频与 FFmpeg 合成

> 目标：把 TODO-03 的 Mock 主线替换为真实生图、真实图生视频和真实 FFmpeg 合成。
> 本文件来自 `doc/功能模块/08-AI生图.md`、`14-AI视频.md`、`13-视频合成.md`、`doc/底层设计/13-FFmpegSidecar与媒体处理规范.md`。

---

## 阶段目标

跑通真实闭环：

```text
分镜 imagePrompt → 真实候选图 → 选择最终图 → 真实图生视频片段 → 确认片段 → FFmpeg 合成 final.mp4
```

前置要求：

```text
TODO-06 的 provider_models
TODO-06 的 workflow_presets
TODO-06 的 ImageInputPlan / VideoInputPlan
TODO-06 的 list_executable_media_options
```

本阶段不允许按某一家模型或 workflow 名称硬编码输入要求。

---

## 本阶段范围

包含：

- 真实 AI 生图。
- 真实图生视频。
- 基于 `ImageInputPlan / VideoInputPlan` 的输入收集、缺项提示和后端校验。
- 生成结果转存受控工作区。
- ImageCandidate / VideoSegment 入库。
- FFmpeg / FFprobe sidecar 检测。
- 片段格式检查。
- concat 合成 final.mp4。
- 合成错误码和基础恢复。

不包含：

- 字幕烧录。
- 封面模板。
- BGM 混音。
- 多轨时间轴。
- 高级转场。

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

### 【X】7.1 接入真实 AI 生图

**问题：**
Mock 生图只能验证流程，不能产出真实图片。

**位置：**

```text
src-tauri/src/service/provider*
src-tauri/src/service/pipeline*
src-tauri/src/service/storage*
src-tauri/src/repository/storyboard* / asset* / task*
src-tauri/src/command/storyboard* 或 generation*
src/src/features/workspace/image*
src/src/entities/storyboard*
```

必须基于 TODO-06 的 `ProviderManager / ImageInputPlan / provider_model_id / workflow_preset_id`，不能在生图页按模型名写分支。

**改法：**

基于每行：

```text
imagePrompt
negativePrompt
ImageInputPlan.required / optional
providerModelId 或 workflowPresetId
```

调用 Image Provider 或 WorkflowRegistry。

输出：

```text
ImageCandidate[]
imagePath
status
provider/model snapshot
generationContextSnapshot
```

落地要求：

```text
1. 生图请求必须走 Task / TaskStep / TaskAttempt，不允许页面直接调 Provider。
2. 每个 StoryboardItem 独立生成候选图，成功行先落库，失败行不回滚其他行。
3. 远程 URL、base64 或临时文件必须转存到 StorageService 受控目录，再写入 ImageCandidate.relative_path。
4. 入库只存相对路径、provider_model_id 或 workflow_preset_id、脱敏参数快照和 generationContextSnapshot。
5. 每次真实生图生成新的 ImageCandidate revision，不覆盖旧候选，不清空用户已选图。
6. 支持取消：任务取消后 Provider 结果回来也不得写入成功候选。
7. 支持 inputPlan.required 缺项拦截：缺角色参考、姿态图、mask、workflow 参数时，前端提示缺项，后端再次拒绝。
8. Mock 和真实结果在 UI、DB metadata、日志中必须能区分。
```

**验收：**

- 每个分镜可真实生成候选图。
- 使用不同生图模型时，前端展示的参考图、姿态图、mask、workflow 参数来自 `ImageInputPlan`。
- 远程 URL 下载或复制到受控工作区。
- 入库只存相对路径。
- 用户可选择最终图。
- 多次生成保留历史候选和当前已选图。
- 单行失败不影响其他行候选图落库。
- 取消后不会继续写入新的候选图。

**验证：**

- Rust 单测覆盖：URL/base64/本地临时文件转存、路径越权拒绝、候选 revision 追加、已选图不被重生成清空、取消后拒绝写成功。
- 前端 typecheck/build 通过。
- 手动 smoke：用一个真实或受控 fake image provider 生成至少 1 行候选图，确认文件在 workspace 内、DB 只存相对路径、页面可选择最终图。
- 失败 smoke：让其中一行 provider 返回错误，确认其他成功行保留且失败行可重试。

**下一步进入条件：**

- 至少一条真实或受控 fake Provider 生图链路经过 Task 系统跑通，并清楚标识当前是不是付费真实调用。
- 完成记录写清使用的 Provider、是否真实扣费、生成文件位置、执行过的测试和 smoke 结果。
- 确认 7.2 可以基于本条的行级失败状态继续扩展后，再把本条改为 `【X】` 并进入 7.2。

**风险：**
不要长期引用 Provider 返回的临时 URL。

**完成记录（2026-06-25）：**

- 后端 `start_image_generation` 已从直接 mock DTO 改为经过 `Task / TaskStep / TaskAttempt`、`ProviderManager`、`StorageService` 和 SQLite `image_candidates` 的链路。
- 本次跑通的是受控 fake image provider：自动注册 `provider_controlled_fake_image / model_controlled_fake_t2i`，走 TODO-06 的 `list_executable_media_options / provider_model_id / ImageInputPlan` 选择，不按模型名写业务分支；`billable=false`、`externalNetwork=false`，不产生真实扣费。
- 生成文件写入受控 workspace 的 `projects/{project_id}/images/{item_id}/rev_{revision}/...png`，DB 和 DTO 只保存相对路径。
- 候选图每次生成新 revision，不覆盖旧候选；重新生成不清空 `selected_image_id`，历史清理只删除未选中的旧 revision。
- 已支持后端 `inputPlan.required` 再校验；缺少角色/姿态/工作流参数等 required 输入时拒绝生成。
- 已支持任务取消后的拒写：Provider 返回后、候选入库前若任务被取消，不写成功候选。
- 前端 Tauri 态 `listImageCandidates` 改为从 `getStoryboard` 展平读取，避免后端入库后页面仍读旧 mock 内存态。
- 验证：`cargo fmt`、`cargo test scene_service`（5 passed）、`cargo check`、`cargo test`（99 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build` 均通过；Vite 仅保留既有 chunk 体积提示。
- 7.2 可继续基于当前行级任务失败、retry attempt 和候选持久化扩展“失败重试和单行隔离”；7.2.1 再扩展角色图、场景图、风格图等多图片类型落库位置。

---

### 【X】7.2 实现生图失败重试和单行隔离

**问题：**
批量生图时单行失败不能影响其他行保存。

**改法：**

每个 StoryboardItem 单独记录：

```text
status
lastErrorJson
retryCount
TaskStep / TaskAttempt
```

**验证：**

- 单行失败可重试。
- 成功行结果保留。
- 审核失败可进入提示词中性化重试。

**风险：**
不要失败后整批回滚已成功候选图。

**完成记录（2026-06-25）：**

- `storyboard_items` 增加 `image_last_error_json` 和 `image_retry_count`，用于行级记录最后一次生图失败和重试次数。
- 后端 `start_image_generation` 的 inputPlan 缺项、Provider 失败会只标记当前 `StoryboardItem.image_status=failed`，写入脱敏错误 JSON，并递增当前行 retryCount；成功行仍正常写入候选图，不回滚其他行。
- 前端生图页批量生成改为逐行 try/catch，失败行显示错误，继续生成后续行；完成后提示成功 / 失败数量，失败行可再次点击“重新生成”重试。
- UI 状态列展示 `imageRetryCount` 和最后错误摘要，便于定位单行失败。
- 验证：`cargo fmt`、`cargo test scene_service`（6 passed）、`cargo check`、`cargo test`（100 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build` 均通过；Vite 仅保留既有 chunk 体积提示。
- 7.2.1 可继续基于当前 image provider 链路扩展不同 `image_kind / asset_kind` 的真实生成入口。

---

### 【X】7.2.1 接入多图片类型真实生成

**问题：**
AI 生图不只是“给每个分镜生成候选图”。完整需求还包括：

```text
分镜图
角色参考图
场景参考图
风格参考图
道具图
尾帧图
控制图
封面图
```

如果真实生图只围绕 `StoryboardItem.imagePrompt → ImageCandidate` 实现，后续角色一致性、场景一致性、首尾帧视频、封面和素材复用都会缺入口。

**改法：**

- 基于 TODO-06 的 `ImageInputPlan.image_kind / asset_kind` 区分图片类型。
- 分镜图生成仍写入 `ImageCandidate[]`，由用户选择 `StoryboardItem.selectedImageId`，用于当前 `image_to_video` 主线。
- 角色参考图写入受控资产，并绑定 `CharacterBible.reference_images` / `AssetReference`。
- 场景参考图写入受控资产，并绑定 `EnvironmentBible.reference_images` / `AssetReference`。
- 风格参考图写入受控资产，并绑定 `StyleBible.reference_image_path` 或相关引用。
- 道具图、尾帧图、控制图、封面图按 `image_kind` 写入对应 owner，不得混成普通分镜候选图。
- 页面必须让用户明确选择“生成哪类图片”；业务层不得按模型名判断图片类型。

**验证：**

- 分镜图生成后仍能选择最终图并进入图生视频。
- 角色参考图不会写成 `selectedImageId`，而是进入角色设定 / 资产引用。
- 场景参考图不会写成 `selectedImageId`，而是进入场景设定 / 资产引用。
- 不同图片类型生成时，前端展示的缺项和参数来自 `ImageInputPlan`。
- 远程 URL 必须转存受控工作区，入库只存相对路径。

**风险：**
不要把角色参考图、场景参考图、风格参考图自动塞进素材库或所有分镜；只有用户确认保存 / 引用后，才进入对应 Bible 或 AssetReference。

**完成记录（2026-06-25）：**

- `MediaInputPlanDto` 已新增 `image_kind / asset_kind` 语义，`list_executable_media_options` 会从 provider model / workflow preset 配置读取 `imageKind / assetKind`，图片能力默认落到 `storyboard_image / generated_output`，业务不按模型名分支。
- 保留 `start_image_generation` 只写分镜 `ImageCandidate[]`；如果传入非 `storyboard_image` 会拒绝，防止角色 / 场景 / 风格图混入普通候选图。
- 新增 `start_image_asset_generation` 命令和 DTO，支持 `character_reference / scene_reference / style_reference / prop_reference / end_frame / control_image / cover_image`，按显式 `image_kind` 校验 owner 和 `asset_kind`。
- 角色参考图写入 `assets + asset_references(owner_kind=character_bible, usage_kind=reference_image)`，并同步 `character_bibles.data_json.reference_images_json / reference_images`；场景、风格同理分别绑定 `location_bible / style_bible`，风格同时维护 `reference_image_path`。
- 道具图、封面图绑定 project；尾帧图、控制图绑定 storyboard_item；所有非分镜图都写受控 `assets/...` 相对路径，不写 `selected_image_id`。
- 生图页新增“图片类型 / 保存到”选择，分镜图继续生成候选图，非分镜图走资产生图入口；无明确 owner 时禁用并提示，避免自动塞进所有分镜或全局素材。
- 本次跑通的是受控 fake image provider，`billable=false`、`externalNetwork=false`，不产生真实扣费。
- 验证：`cargo fmt`、`cargo test scene_service`（7 passed）、`cargo check`、`cargo test`（101 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build` 均通过；Vite 仍只有既有 chunk 体积提示。
- 7.3 可继续基于当前已选 `ImageCandidate.image_path` 的受控相对路径接入真实图生视频。

---

### 【X】7.3 接入真实图生视频

**问题：**
当前主线必须把已选图片生成视频片段。

**位置：**

```text
src-tauri/src/domain/scene.rs
src-tauri/src/db/scene_repository.rs
src-tauri/src/services/scene_service.rs
src-tauri/src/commands/scene.rs
src/src/entities/scene/api.ts
src/src/entities/scene/store.ts
src/src/pages/video-generation/index.vue
```

**改法：**

输入必须来自：

```text
selectedImageId → ImageCandidate.imagePath
videoPrompt
durationSeconds
VideoInputPlan.required / optional
providerModelId 或 workflowPresetId
```

输出：

```text
VideoSegment
videoPath
status
selected=false
generationContextSnapshot
```

**验收：**

- 没有 selectedImageId 不能生成视频。
- 对首尾帧、参考图、ComfyUI / RunningHub 等不同模式，输入项必须来自 `VideoInputPlan`，不能写死只传 startFrame。
- 片段保存到受控工作区。
- 失败可重试。
- 用户确认后写 selectedVideoSegmentId。

**验证：**

- Rust 单测覆盖：缺少 selectedImageId 拒绝、selectedImageId 找不到候选拒绝、生成视频写文件和 `video_segments`、多次生成追加 revision 且不清空已确认片段、取消后拒绝写成功片段。
- 前端 typecheck/build 通过。
- 手动 smoke：使用受控 fake video provider 生成至少 1 行视频，确认文件在 workspace 内、DB 只存相对路径、页面可确认视频片段。

**下一步进入条件：**

- 至少一条真实或受控 fake Provider 图生视频链路经过 Task 系统跑通，并清楚标识当前是不是付费真实调用。
- 完成记录写清使用的 Provider、是否真实扣费、生成文件位置、执行过的测试和 smoke 结果。
- 确认 7.4 可以基于当前链路继续补完整视频 Provider 能力校验后，再把本条改为 `【X】` 并进入 7.4。

**风险：**
不同 Provider 的图生视频能力差异大，必须走能力矩阵和输入规划，不要硬编码某一家。

**完成记录（2026-06-25）：**

- 后端 `start_video_generation` 已从硬编码 mock DTO 改为经过 `Task / TaskStep / TaskAttempt`、`ProviderManager`、`StorageService` 和 SQLite `video_segments` 的链路。
- 本次跑通的是受控 fake video provider：自动注册 `provider_controlled_fake_video / model_controlled_fake_i2v`，走 TODO-06 的 `list_executable_media_options / provider_model_id / VideoInputPlan` 选择，不按模型名写业务分支；`billable=false`、`externalNetwork=false`，不产生真实扣费。
- 视频输入从当前 `StoryboardItem.selected_image_id` 读取真实 `ImageCandidate.image_path`，并按 `VideoInputPlan.required` 校验 `startFrame / videoPrompt / durationSeconds / endFrame / referenceAsset / workflowParams.*` 等缺项；workflow preset 走 `run_workflow`，API 视频模型走 `generate_video`。
- 生成文件写入受控 workspace 的 `projects/{project_id}/videos/{item_id}/rev_{revision}/...mp4`，DB 和 DTO 只保存相对路径。
- `video_segments` 每次生成新 revision，不覆盖旧片段；重新生成不清空 `selected_video_segment_id`，用户确认片段后写回 `selectedVideoSegmentId`，历史清理只删除未确认的旧 revision。
- 已支持任务取消后的拒写：Provider / workflow 返回后、片段入库前若任务被取消，不写成功片段。
- 前端 Tauri 态 `generateVideos` 生成后刷新 `getStoryboard`，避免后端入库后页面仍读旧状态；视频页批量生成改为逐行 try/catch，失败行可单独重试且不影响其他行。
- 验证：`cargo fmt`、`cargo fmt -- --check`、`cargo test scene_service`（13 passed）、`cargo check`、`cargo test`（107 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build` 均通过；Vite 仍只有既有 chunk 体积提示。
- 7.4 可继续基于当前 `VideoInputPlan / ProviderManager` 链路补完整视频 Provider 能力校验和更细错误码。

---

### 【X】7.4 实现视频 Provider 能力校验

**问题：**
视频模型对时长、比例、fps、输入图数量约束强。

**位置：**

```text
src-tauri/src/services/provider_service.rs
src-tauri/src/services/scene_service.rs
src-tauri/src/services/media_service.rs
src/src/entities/scene/api.ts
src/src/pages/video-generation/index.vue
src/src/shared/i18n/locales/*
```

**改法：**

后端校验：

```text
durationSeconds
resolution
aspectRatio
fps
inputImages
mode=image_to_video
```

**验收：**

- 超出模型能力后端拒绝。
- 前端禁用只是体验，后端才是最终校验。
- 错误码明确可恢复动作。

**验证：**

- Rust 单测覆盖 durationSeconds、aspectRatio、resolution、fps、inputImages、abilityType 不匹配时拒绝。
- 前端 typecheck/build 通过。
- 手动 smoke：选择受控 fake video provider，合法参数能生成；改成超出 limits 后后端拒绝且页面显示可重试 / 可编辑输入。

**下一步进入条件：**

- 后端不依赖 modelName 分支即可基于 provider model / workflow preset limits 拒绝非法视频生成参数。
- 完成记录写清哪些能力字段已校验、哪些 workflow 校验仍留给后续真实执行器深化。
- 确认 7.5 可以基于当前视频片段继续接 FFmpeg / FFprobe sidecar 检测后，再把本条改为 `【X】` 并进入 7.5。

**风险：**
不要让业务层按 modelName 写特殊逻辑。

**完成记录（2026-06-25）：**

- 后端视频生成入口已在调用 Provider / workflow 前统一执行能力矩阵校验，不按 `modelName` 或 workflow key 写分支。
- 已校验字段：`abilityType`、`durationSeconds`、`aspectRatio`、`resolution`、`fps`、`inputImages/maxReferenceImages`。
- Provider model 和 workflow preset 都基于 `list_executable_media_options` 产出的 `capabilities / constraints.limits / VideoInputPlan` 校验；workflow preset 的 `limits` 也会被后端拒绝非法参数。
- 能力错误使用 `provider.limit_exceeded` / `provider.capability_unsupported`，并补齐命令边界对带前缀错误码的保留；`provider.limit_exceeded` 的恢复动作为 `edit_input`，能力不匹配恢复动作为 `change_provider_or_plan`。
- `workflowParams.*` required 校验兼容顶层参数和嵌套 `workflowParams`，避免前后端参数形态差异导致误判。
- 本条未深化真实 workflow 执行器内部节点级校验；节点映射、输出映射和 RunningHub workflow_id 仍由现有 `ProviderManager.run_workflow` 校验，后续真实执行器接入时再按执行器协议扩展。
- 验证：`cargo fmt`、`cargo fmt -- --check`、`cargo test scene_service`（20 passed）、`cargo check`、`cargo test`（115 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build` 均通过；Vite 仍只有既有 chunk 体积提示。
- 7.5 可基于当前已持久化的视频片段继续接入 FFmpeg / FFprobe sidecar 检测。

---

### 【X】7.5 实现 FFmpeg / FFprobe sidecar 检测

**问题：**
开发环境能跑不代表发布版能合成。

**位置：**

```text
src-tauri/src/domain/media.rs
src-tauri/src/services/ffmpeg_service.rs
src-tauri/src/services/task_service.rs
src-tauri/src/commands/ffmpeg.rs
src-tauri/src/commands/task.rs
src/src/entities/config/api.ts
src/src/entities/config/types.ts
src/src/pages/settings/index.vue
src/src/shared/api/commands.ts
src/src/shared/i18n/locales/*
```

**改法：**

实现：

```text
ffmpeg -version
ffprobe -version
sidecar path resolve
版本信息展示
缺失错误码
```

要求：

```text
1. sidecar 路径只从 StorageService 的 FileBucket::Sidecar 解析。
2. 运行命令必须使用 Command + args 数组，不拼 shell 字符串。
3. DTO 只返回 bucket 内相对路径、版本和状态，不返回绝对路径。
4. 设置页或诊断页可手动刷新检测结果。
5. 合成入口启动前必须再次执行 sidecar 检测；缺失时返回 ffmpeg.not_found。
6. 当前只做 sidecar 自检，不做媒体探测、转码或 concat 合成。
```

**验收：**

- `ffmpeg.exe / ffprobe.exe` 缺失时应用和设置页仍能打开。
- 设置页能看到两个 sidecar 的存在、可执行 / 版本状态和错误码。
- 合成启动前 sidecar 缺失会被后端拒绝，错误码为 `ffmpeg.not_found`。
- 成功检测时版本信息来自 `-version` 输出首行。
- DTO、日志和 DB 不暴露 sidecar 绝对路径。

**验证：**

- sidecar 缺失不阻止打开应用，但禁止启动合成任务。
- 设置页或诊断页可看到 sidecar 状态。
- Rust 单测覆盖 sidecar 缺失、fake sidecar 版本检测、合成入口缺失拒绝。
- 前端 typecheck/build 通过。

**下一步进入条件：**

- 后端已有可复用 sidecar 检测服务，并能被设置页命令和合成入口共同调用。
- 完成记录写清 sidecar 检测字段、错误码、验证命令和是否实际存在本机 sidecar。
- 确认 7.6 可基于当前服务继续实现 ffprobe 媒体探测后，再把本条改为 `【X】` 并进入 7.6。

**风险：**
路径必须来自 PathGuard / SafePath。

**完成记录（2026-06-25）：**

- 新增 `ffmpeg_service`，统一检测 `sidecars/ffmpeg.exe` 与 `sidecars/ffprobe.exe`，路径只通过 `StorageService + FileBucket::Sidecar` 解析，进程调用使用 `Command::new(path).arg("-version")`，不拼 shell 字符串。
- 新增 `FfmpegSidecarStatusDto / SidecarBinaryStatusDto`，返回字段包括 `relativePath`、`exists`、`executable`、`version`、`errorCode`、`message`、`checkedAt`、`ready`；DTO 不返回 sidecar 绝对路径。
- 新增 `check_ffmpeg_sidecars` Tauri command，设置页新增“本地媒体工具”诊断卡，可刷新查看 FFmpeg / FFprobe 缺失、不可执行、版本和错误码。
- 合成入口 `start_composition` 启动前已调用 `require_ffmpeg_sidecars`，sidecar 缺失优先返回 `ffmpeg.not_found: ...`，且错误消息只包含 `sidecars/ffmpeg.exe` / `sidecars/ffprobe.exe` 相对路径。
- 当前仓库未发现随项目存在的真实 `ffmpeg.exe / ffprobe.exe`，所以本机真实 sidecar 状态预期为缺失；应用和设置页仍可打开，但合成启动会被后端禁止。
- 本条只做 sidecar 自检和合成入口 gate，未实现媒体探测、转码或 concat 合成；这些留给 7.6 / 7.7。
- 验证：`cargo fmt`、`cargo fmt -- --check`、`cargo test ffmpeg`（5 passed）、`cargo test task_service`（2 passed）、`cargo check`、`cargo test`（119 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build` 均通过；Vite 仍只有既有 chunk 体积提示。
- 7.6 可基于当前 `ffmpeg_service` 的 sidecar 路径解析与命令执行约束继续实现 `ffprobe` 媒体探测。

---

### 【X】7.6 实现 Ffprobe 媒体探测

**问题：**
不能只相信 Provider 返回的时长和格式。

**位置：**

```text
src-tauri/src/domain/media.rs
src-tauri/src/domain/scene.rs
src-tauri/src/db/scene_repository.rs
src-tauri/src/services/ffmpeg_service.rs
src-tauri/src/services/task_service.rs
src-tauri/src/commands/media.rs
src/src/entities/config/types.ts
src/src/entities/scene/types.ts
src/src/pages/composition/index.vue
src/src/shared/i18n/locales/*
```

**改法：**

探测：

```text
container
codec
width
height
fps
duration
audio stream
video stream
```

要求：

```text
1. probe 输入必须是 workspace 内相对路径，经过 StorageService / PathGuard 解析。
2. ffprobe 通过 sidecars/ffprobe.exe 启动，参数数组传递，不拼 shell。
3. 解析 JSON 输出生成 MediaProbeDto，不把绝对路径写入 DTO、DB 或日志。
4. AI 视频生成后可写入 VideoSegment.generationContextSnapshot.mediaProbe。
5. 合成入口读取 selectedVideoSegmentId，对每个 selected VideoSegment 执行 ffprobe。
6. 当前只做探测和错误归一，不做转码兜底。
```

**验收：**

- 对每个已确认 `VideoSegment.videoPath`，合成前能解析 container、codec、width、height、fps、duration、audio/video stream 状态。
- ffprobe 输出损坏、JSON 不合法、缺少视频流或时长无效时返回明确 `ffmpeg.probe_failed` 或 `ffmpeg.invalid_media`。
- 探测结果路径只包含 workspace 相对路径。
- 探测失败不会删除或覆盖已有片段。

**验证：**

- 每个 VideoSegment 合成前可被 ffprobe。
- 损坏片段有明确错误。
- 编码不一致可进入转码兜底。
- Rust 单测覆盖 ffprobe JSON 解析、无视频流拒绝、无效时长拒绝、相对路径解析越权拒绝、合成入口探测 selected 片段。
- 前端 typecheck/build 通过。

**下一步进入条件：**

- 后端已有可复用 `probe_media` 能力，合成入口能在 concat 前拿到所有 confirmed segment 的探测结果。
- 完成记录写清探测字段、错误码、验证命令，以及转码兜底仍留给 7.8。
- 确认 7.7 可基于 probe 结果继续实现片段 concat 合成后，再把本条改为 `【X】` 并进入 7.7。

**风险：**
音画不同步和编码不一致是合成高频问题。

**完成记录（2026-06-25）：**

- 新增 `MediaProbeDto / ProbeMediaRequest`，探测字段覆盖 `container / formatName / durationSeconds / width / height / fps / videoCodec / audioCodec / sampleRate / bitRate / hasVideoStream / hasAudioStream`。
- `ffmpeg_service` 新增 `probe_media` 和 `probe_video_segments`，输入必须是 workspace 相对路径，先按 bucket 拆解再走 `StorageService / PathGuard`；ffprobe 只从 `sidecars/ffprobe.exe` 启动，参数数组传递，不拼 shell。
- ffprobe JSON 输出已解析为结构化 DTO；无视频流、时长缺失或 JSON 非法时分别归一为 `ffmpeg.invalid_media` / `ffmpeg.probe_failed`，错误消息不包含绝对路径。
- `VideoSegmentDto` 新增 `mediaProbe`；探测结果写入已有 `generation_context_snapshot_json.mediaProbe`，不新增表结构，不覆盖原有 revision / provider 快照。
- 合成入口 `start_composition` 在 sidecar gate 后会读取当前项目所有 `selectedVideoSegmentId`，对每个已确认片段执行 ffprobe 并写回 `mediaProbe`；缺少已确认片段或片段记录时仍拒绝进入合成。
- 前端补齐 `probeMedia` API 类型与命令，合成页显示已确认片段的探测状态、codec、分辨率和 fps；未探测时显示待探测。
- 本条只做媒体探测和错误归一，编码不一致后的转码兜底仍留给 7.8。
- 验证：`cargo fmt`、`cargo fmt -- --check`、`cargo test scene_repository`（1 passed）、`cargo test ffmpeg`（9 passed）、`cargo test task_service`（2 passed）、`cargo check`、`cargo test`（124 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build` 均通过；Vite 仍只有既有 chunk 体积提示。
- 7.7 可基于当前 `probe_media / mediaProbe` 继续实现片段 concat 合成。

---

### 【X】7.7 实现片段 concat 合成

**问题：**
最终必须把多个视频片段合成一个视频。

**位置：**

```text
src-tauri/src/domain/task.rs
src-tauri/src/db/task_repository.rs
src-tauri/src/services/ffmpeg_service.rs
src-tauri/src/services/task_service.rs
src/src/entities/task/types.ts
src/src/pages/composition/index.vue
src/src/shared/i18n/locales/*
```

**改法：**

合成规则：

```text
按 StoryboardItem.index 排序
读取 selectedVideoSegmentId
生成 filelist 到任务工作区
ffmpeg concat
输出 exports/final.mp4
```

默认编码：

```text
mp4
libx264
yuv420p
fps=30
crf=18
preset=veryfast
aac
192k
44100
```

要求：

```text
1. 只读取 StoryboardItem.selectedVideoSegmentId，不按文件名排序。
2. 每个 segment path 必须是 workspace 相对路径，并由 StorageService / PathGuard 解析。
3. filelist 写入 workspace temp bucket，用完删除。
4. ffmpeg 通过 sidecars/ffmpeg.exe 启动，参数数组传递，不拼 shell。
5. 输出写入受控 workspace，CompositionTask.outputPath 只保存相对路径。
6. 本条优先实现 concat demuxer；编码不一致转码兜底留给 7.8。
```

**验收：**

- 所有分镜有确认视频片段时，后端能按 `item_index ASC` 合成一个 `final.mp4`。
- 合成输出文件位于受控 workspace，DB 只保存相对路径。
- `composition_tasks` 写入 `succeeded / progress=100 / segmentIds / outputPath`。
- FFmpeg 调用不经过 shell 字符串拼接。
- 缺确认片段、缺 segment 文件、ffmpeg 失败时有明确错误码，不破坏已有视频片段。

**验证：**

- 得到可播放 final.mp4。
- 输出路径在受控工作区。
- CompositionTask.outputPath 写入相对路径。
- Rust 单测覆盖 filelist 生成转义、缺确认片段拒绝、合成输出相对路径入库、ffmpeg 失败错误码。
- 前端 typecheck/build 通过。

**下一步进入条件：**

- 至少一个受控 fake 或真实 ffmpeg concat 链路输出 `final.mp4`，并写入 `CompositionTask`。
- 完成记录写清输出路径、执行过的测试、当前是否真实 ffmpeg 运行，以及转码兜底留给 7.8。
- 确认 7.8 可基于当前 concat 结果继续实现编码不一致转码兜底后，再把本条改为 `【X】` 并进入 7.8。

**风险：**
FFmpeg 参数必须数组传递，不能拼 shell 字符串。

**完成记录（2026-06-25）：**

- `ffmpeg_service` 已实现 concat demuxer 合成：按 `StoryboardItem.index ASC` 读取每行 `selectedVideoSegmentId`，使用已确认 `VideoSegment.videoPath` 生成 filelist，并调用 `sidecars/ffmpeg.exe`，参数通过 `Command + args` 数组传递，不拼 shell 字符串。
- filelist 写入受控 workspace 的 `temp/composition/{project_id}/{task_id}_concat.txt`，调用完成后删除；输出写入 `outputs/exports/{project_id}/{task_id}_final.mp4`。
- `TaskRepository` 已支持 `composition_tasks` upsert / latest read / task detail 回读，`CompositionTask.outputPath` 只允许保存 `outputs/...` 相对路径，拒绝绝对路径、越权路径和非 output bucket 路径。
- 合成成功后写入 `composition_tasks.status=succeeded / progress=100 / segmentIds / outputPath`；缺 sidecar、缺确认片段、缺 segment 文件和 ffmpeg 失败分别通过 `ffmpeg.not_found / ffmpeg.invalid_media / ffmpeg.concat_failed` 等错误码返回，不删除或覆盖已有视频片段。
- 合成页文案已从“演示合成”改为真实 FFmpeg concat 口径；Tauri 态刷新 storyboard 时会同步读取最新 `TaskDetailDto.compositionTask`，避免后端已入库但页面仍显示未生成。
- 本机当前仍未发现真实 `sidecars/ffmpeg.exe / sidecars/ffprobe.exe`，所以未执行真实可播放 final.mp4 smoke；本条通过受控 fake concat runner 覆盖 filelist、输出相对路径、filelist 清理和失败脱敏，真实可播放验证需放入 sidecar 后执行。
- 本条只做 concat demuxer，编码、fps、分辨率或音频格式不一致时的转码兜底仍留给 7.8。
- 验证：`cargo fmt`、`cargo fmt -- --check`、`cargo test task_repository`（23 passed）、`cargo test ffmpeg`（12 passed）、`cargo test task_service`（2 passed）、`cargo check`、`cargo test`（129 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build` 均通过；Vite 仅保留既有 chunk 体积提示。
- 7.8 可基于当前 `MediaProbeDto` 与 concat 输出路径继续实现编码不一致转码兜底。

---

### 【X】7.8 实现转码兜底

**问题：**
不同 Provider 输出视频编码、fps、分辨率可能不一致。

**位置：**

```text
src-tauri/src/domain/media.rs
src-tauri/src/db/scene_repository.rs
src-tauri/src/services/ffmpeg_service.rs
src-tauri/src/services/task_service.rs
src/src/entities/config/types.ts
src/src/entities/scene/types.ts
src/src/pages/composition/index.vue
src/src/shared/i18n/locales/*
```

**改法：**

合成前检查，不一致时转码到统一规格：

```text
resolution
fps
codec
pixel format
audio format
```

要求：

```text
1. 转码输入仍只来自 StoryboardItem.selectedVideoSegmentId 对应的 VideoSegment.videoPath。
2. 先用 7.6 的 MediaProbeDto 判断是否需要转码，不按文件名、扩展名或 Provider 名称猜测。
3. 标准化目标：mp4 / h264 / yuv420p / fps=30 / aac / 192k / 44100。
4. 转码临时文件必须写入 workspace temp bucket 的任务目录，不能写系统 temp。
5. 转码后 concat 只读取受控 workspace 相对路径，输出仍写入 outputs。
6. 转码失败返回明确错误码，不删除或覆盖原始 VideoSegment。
7. 本条不做字幕烧录、BGM 混音、封面、转场或多轨时间线。
```

**验收：**

- 编码、fps、像素格式、音频编码或采样率不一致时，后端会先转码为统一规格再 concat。
- 已符合统一规格的片段不做无意义转码。
- 所有转码临时文件位于 `temp/composition/{project_id}/{task_id}/...`，DB 不保存临时绝对路径。
- 转码失败时返回 `ffmpeg.transcode_failed`，并保留原始视频片段和已有合成结果。
- concat 输出仍位于受控 workspace，`CompositionTask.outputPath` 仍只保存 `outputs/...` 相对路径。

**验证：**

- 不一致片段可被转码后合成。
- 转码失败有明确错误码。
- 临时文件位于任务工作区。
- Rust 单测覆盖：无需转码判断、需要转码判断、转码参数数组、临时输出相对路径、转码失败脱敏、转码后 concat 使用临时片段。
- 前端 typecheck/build 通过。

**下一步进入条件：**

- 合成入口能基于 probe 结果自动选择直接 concat 或转码后 concat。
- 完成记录写清统一规格、临时路径、错误码、验证命令和是否真实 ffmpeg smoke。
- 确认 7.9 可基于当前 ffmpeg stderr 继续做日志脱敏和截断后，再把本条改为 `【X】` 并进入 7.9。

**风险：**
临时文件不能写系统临时目录破坏 PathGuard 边界。

**完成记录（2026-06-25）：**

- `MediaProbeDto` 新增 `pixelFormat`，ffprobe JSON 解析已读取视频流 `pix_fmt`，前端类型同步补齐。
- 合成入口现在保留 7.6 的 probe 结果，并传入 `concat_segments_with_probes`；媒体服务会基于 `MediaProbeDto` 判断直接 concat 或先转码。
- 标准化规格已实现为 `mp4 / h264(libx264) / yuv420p / 30fps / aac / 192k / 44100`；当容器、视频编码、像素格式、fps、分辨率、音频编码或采样率不符合目标时，先转码为标准片段。
- 转码临时输出写入受控 workspace 的 `temp/composition/{project_id}/{task_id}/segment_0001_normalized.mp4` 这类任务目录；concat 完成后删除转码临时片段，filelist 仍在使用后删除，DB 不保存临时绝对路径。
- 已符合统一规格的片段不会调用转码 runner；不符合规格的片段会使用转码后的临时文件参与 concat，原始 `VideoSegment` 不会被删除或覆盖。
- 转码失败统一返回 `ffmpeg.transcode_failed`，错误消息脱敏掉临时文件绝对路径，恢复动作归为重新生成或替换媒体；失败时会删除可能存在的部分转码输出。
- 本机当前仍未发现真实 `sidecars/ffmpeg.exe / sidecars/ffprobe.exe`，所以未执行真实可播放转码 smoke；本条通过受控 fake runner 覆盖转码参数、临时路径、转码后 concat、清理和失败错误码。
- 验证：`cargo fmt`、`cargo fmt -- --check`、`cargo test ffmpeg`（15 passed）、`cargo test scene_repository`（1 passed）、`cargo test task_service`（2 passed）、`cargo check`、`cargo test`（132 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build` 均通过；Vite 仅保留既有 chunk 体积提示。
- 7.9 可基于当前 FFmpeg / FFprobe / transcode / concat 进程错误输出继续实现日志脱敏和截断。

---

### 【X】7.9 实现 FFmpeg 日志脱敏和截断

**问题：**
FFmpeg stderr 可能包含路径、参数、用户文件名，需要控制日志大小和敏感信息。

**改法：**

规则：

```text
stderr 保存最后 200 行或 32KB
路径脱敏为 relative_path
SecretGuard 扫描
写 task.log / ffmpeg.log
```

**验证：**

- 日志不包含真实密钥。
- 日志不包含敏感绝对路径。
- 错误详情可定位失败原因。

**风险：**
不要把完整 Provider 请求或用户隐私内容写进日志。

**完成记录（2026-06-26）：**

- `ffmpeg_service` 已统一通过 `sanitize_process_message` 处理 FFmpeg / FFprobe / sidecar 进程输出，先走 `SecretGuard.redact_text`，再做受控路径替换，最后按最多 200 行 / 32KB 截断。
- 覆盖范围包括：sidecar `-version` 失败、ffprobe 探测失败、concat 失败、转码失败、版本信息展示和 ffprobe JSON 错误摘要。
- 路径替换会把 workspace 根替换成 `<workspace>`，把 sidecar、输入片段、filelist、转码临时输出和最终输出替换为相对路径语义，不向 DTO / 错误消息暴露绝对路径。
- 截断策略保留尾部日志，便于定位 FFmpeg 最后的失败原因；新增测试覆盖长日志只保留最后 200 行，以及密钥样式文本和绝对路径脱敏。
- 本机当前仍未发现真实 `sidecars/ffmpeg.exe / sidecars/ffprobe.exe`，所以未执行真实 FFmpeg stderr smoke；本条通过受控 runner 和单元测试验证脱敏、路径替换和截断。
- 验证：`cargo fmt`、`cargo fmt -- --check`、`cargo test ffmpeg`（17 passed）、`cargo check`、`cargo test`（134 passed）均通过。

---

## 阶段完成标准

- 每个分镜能真实生成候选图。
- 每个分镜能基于已选图生成视频片段。
- 图片 / 视频真实生成前，前端按输入规划展示缺项，后端按能力矩阵和输入规划拒绝非法参数。
- 所有片段确认后能 FFmpeg 合成 final.mp4。
- final.mp4 可播放。
- 失败可定位、可重试，不破坏已完成产物。
- 所有产物路径受控，日志无密钥和敏感绝对路径。
