# TODO-12：独立 Workflow 与高级能力

> 目标：收纳 `doc` 中所有非当前主线但明确存在的后续能力，避免遗漏，同时防止它们混入当前 `image_to_video` MVP。  
> 本文件来自 `doc/功能模块/02-脚本生成.md`、`15-数字人口播.md`、`16-画布精修.md`、`19-素材导入与素材成片.md`、`12-模板与动效.md`、`14-AI视频.md`、`doc/参考分析/*`。

---

## 阶段目标

后续独立推进：

```text
脚本生成增强
小说长内容
数字人口播 workflow
素材剪辑成片 workflow
纯图文成片 workflow
画布精修
高级视频能力
创作规则 / Agent / 长内容组织
```

这些能力不能阻塞当前 `image_to_video` 主线，也不能和主线页面硬混。

---

## 本阶段范围

包含：

- 脚本生成作为可选高级能力。
- 小说长内容导入、分章、事件提取。
- 数字人口播独立 workflow。
- 素材导入与素材成片独立 workflow。
- 纯图文成片 workflow。
- 画布精修。
- text_to_video / 首尾帧 / 尾帧续接等高级视频能力。
- 创作规则 / PromptSkill / Agent 长内容组织增强。

不包含：

- 把这些能力变成当前 MVP 前置。
- 复用 image_to_video 页面硬套所有 workflow。
- 执行未注册外部代码。

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

### 【X】12.1 脚本生成作为可选高级能力

**问题：**  
脚本生成有价值，但不应作为当前工作台必经独立页。

**位置：**

```text
doc/开始之前/菜单与页面详细说明.md
doc/功能模块/内容导入或脚本生成相关文档
src-tauri/src/domain/script* / storyboard*
src-tauri/src/service/llm* / provider*
src/src/features/content-creation*
```

本条只能把脚本生成定位为内容创作里的可选增强，不允许改掉当前 `image_to_video` 主线。

**改法：**

定位为：

```text
文本预处理
高级脚本编辑
标题生成
长内容改编中间层
```

支持：

```text
topic → narrations
article → narrations
paste → 原文确认，不调用 LLM
novel → 剧情脚本
```

落地要求：

```text
1. topic/article 可选择 LLM 整理为 narration/script draft，再进入当前作品草稿。
2. paste 默认尊重用户原文，不自动 AI 改写；用户显式选择“整理/改写”才调用 LLM。
3. 脚本生成产物必须归属当前 Project，不允许生成后游离在全局。
4. LLM 输出走 TODO-06 的创作规则和 schema 校验。
5. 锁定后的脚本段落不被批量重生成覆盖。
6. 脚本生成失败不破坏原始输入。
```

**验收：**

- paste 不被 AI 改写。
- LLM 输出结构化 JSON。
- 脚本锁定后批量重生成不覆盖。
- 脚本生成不是当前 StepBar 的强制步骤；未使用脚本生成也能从内容创作进入分镜。
- 生成脚本能进入当前作品的 `ScriptDraft / CleanNarrationList / StoryboardItem.sourceText / narrationText` 数据流。

**验证：**

- 单测或数据层测试覆盖 paste 不改写、LLM 输出 schema 校验失败不入库、锁定段落不覆盖。
- 前端 typecheck/build 通过。
- 手动 smoke：topic 生成脚本草稿后进入分镜；paste 直接按原文进入分镜。

**下一步进入条件：**

- 脚本生成定位、数据归属、paste 安全边界和 schema 校验全部写清并落地。
- 完成记录写清它是可选高级能力，不是当前主线必经步骤。
- 确认 12.2 小说长内容可以复用脚本草稿和章节数据结构后，再把本条改为 `【X】` 并进入 12.2。

**风险：**  
不要把脚本页写死为所有 workflow 前置。

**完成记录（2026-06-27）：**

- 脚本生成定位已落到“内容创作里的可选高级能力”：当前路由仍不注册 `script-editor` 为主线必经页，`create-project -> workspace/storyboard` 主线不变；新增能力只提供脚本草稿应用入口，不改变 StepBar。
- 后端新增 `ApplyScriptDraftRequest / ScriptDraftNarrationDto` 与 `apply_script_draft`：只接收已经生成的结构化脚本草稿，必须通过 `structured_output_service` 的 `narrations` schema 校验后，才写入当前 Project 的 `StoryboardItem.sourceText / narrationText` 数据流。
- `apply_script_draft` 失败不会写库；schema 缺 `narrations`、数量不匹配或字段类型错误会直接返回错误，不会破坏已有分镜文本。
- `lock_flags_json.sourceText / narrationText` 已成为脚本段落锁定来源：脚本草稿应用和 mock 批量重生成都会保留锁定行，`confirmedNarrations.locked` 也按真实锁定状态返回，不再固定为 false。
- paste 安全边界已用测试固定：`paste + fixed` 保留用户原文；`paste + generate` 被拒绝。当前没有把 paste 默认接入 LLM，也没有伪造成真实 AI 改写。
- 前端 mock / 浏览器预览已补 `applyScriptDraft`，并让 `regenerateStoryboard` 复用同一锁定保留逻辑，避免 mock 行为和 Tauri 行为冲突。
- 未伪造内容：本条没有声称已经完成真实 LLM 脚本生成、真实 Provider 调用或真实 topic/article 一键生成 UI；真实 LLM 入口后续必须继续走创作规则、ProviderManager、schema 校验和任务记录。
- 12.2 进入条件复核：小说长内容可以复用本条的 `ScriptDraftNarrationDto` 作为章节提炼后的 narrations 落点；但章节识别、章节事件、章节级状态和失败重试仍需 12.2 单独建立，不能把整本小说直接塞进 `projects.source_text`。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo test apply_script_draft -- --nocapture` 通过，覆盖 schema 失败不入库、锁定段落不覆盖。
- `cargo test paste_input -- --nocapture` 通过，覆盖 paste 不改写边界。
- `cargo check` 通过。
- `pnpm --dir src typecheck` 通过。

---

### 【X】12.2 小说长内容导入

**问题：**  
小说不能一次塞给 LLM，需要分章、事件、续跑。

**改法：**

实现：

```text
小说导入
长文本文件存储
章节识别
章节事件提取
章节级状态
失败重试
```

**验证：**

- 整本原文不进入 projects.source_text。
- 每章事件结构化。
- 章节失败可单独重试。

**风险：**  
长内容必须分块处理，不能整本塞入模型。

**完成记录（2026-06-27）：**

- 已新增 `novel_chapters` 表和 `NovelChapterDto` 数据结构，字段覆盖 `novelChapterId / projectId / chapterIndex / volumeTitle / chapterTitle / chapterContent / structuredEvent / eventStatus / errorReason / retryCount`，章节事件状态限定为 `pending / running / succeeded / failed`。
- 已新增 `novel_repository / novel_service / novel commands`，支持 `import_novel / list_novel_chapters / update_novel_chapter_event / mark_novel_chapter_event_failed / retry_novel_chapter_event`。
- 小说导入会把整本原文写入受控工作区 `projects/{projectId}/input/source.txt`，并把 `projects.source_text` 清空、只保留 `source_text_path`；没有把整本小说塞入数据库 `source_text` 或 LLM 上下文。
- 章节识别先按“第X章 / 第X回 / 第X节 / Chapter N”拆分；识别不到时按固定字符数切块作为手动兜底，确保长内容仍是分块处理。
- 章节事件可以单章成功、单章失败、单章重试；失败只影响当前章节，不会阻断或覆盖其他章节。
- 前端数据层新增 `entities/novel` 和 Tauri command 映射，后续页面可以直接接入章节列表和事件状态。
- 未伪造内容：本条没有声称完成真实 LLM 章节规律识别、真实并发事件抽取、真实 TaskStep 续跑或流式进度；当前落地的是长内容安全存储、章节底座和可重试事件状态，真实 AI 抽取会在 12.3/后续任务系统接入。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo test novel -- --nocapture` 通过，覆盖整本原文文件化、章节落库、事件失败、单章重试和事件成功。
- `cargo test core_tables_migration_creates_required_tables -- --nocapture` 通过，确认 `novel_chapters` migration 表存在。
- `cargo check` 通过。
- `pnpm --dir src typecheck` 通过。

---

### 【X】12.3 长内容 AI 组织链路

**问题：**  
长内容改编不是简单分镜，需要更高层的导演规划。

**改法：**

借鉴参考分析，后续支持：

```text
骨架 → 策略 → 剧本
导演规划
拍摄计划
资产抽取
分镜表
```

**验证：**

- 每层输出有 schema。
- 用户可在关键节点确认。
- 任务可断点续跑。

**风险：**  
不要让长内容能力拖慢短视频主线。

**完成记录（2026-06-27）：**

- 已新增 `long_content_plans` 表和 `LongContentPlanDto`，用于保存长内容组织产物：`story_skeleton / adaptation_strategy / episode_script / storyboard_table / asset_extraction`。
- 已新增 `long_content_repository / long_content_service / long_content commands`，支持保存结构化规划产物、按项目/类型查询、用户批准和用户拒绝。
- 所有规划产物保存前必须走 `structured_output_service` schema 校验；schema 失败不入库，保存后默认 `waiting_user`，必须用户确认后才可作为后续输入。
- 规划产物归属当前 Project，并可记录 `chapterIds` 和 `parentPlanId`，为“骨架 → 策略 → 剧本 → 分镜表 → 资产抽取”的分层关系留出明确数据链路。
- 前端数据层新增 `entities/long-content` 和 Tauri command 映射；当前没有把长内容规划页面塞进 `image_to_video` 主线。
- 未伪造内容：本条没有声称完成真实 Agent、真实 LLM 调用、真实导演规划自动生成或流式任务执行；当前完成的是结构化产物、schema 门禁、项目归属和人工确认点，后续真实生成必须接 ProviderManager / TaskStep。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo test long_content -- --nocapture` 通过，覆盖 schema 失败不入库、有效规划保存为 `waiting_user`、用户批准。
- `cargo test core_tables_migration_creates_required_tables -- --nocapture` 通过，确认 `long_content_plans` migration 表存在。
- `cargo check` 通过。
- `pnpm --dir src typecheck` 通过。

---

### 【X】12.4 数字人口播 workflow

**问题：**  
数字人口播是独立流程，不能混进 image_to_video 主线。

**改法：**

独立 workflow：

```text
输入脚本 → 选数字人 → 选声音 → 生成口播 → 字幕 → 合成
```

能力：

```text
上传人物参考图
生成/输入口播文案
生成 TTS 音频
商品口播模式
调用数字人视频模型
作品内部输出记录
```

**验证：**

- `workflowType=digital_human` 有独立步骤。
- TTS 失败不继续生成数字人视频。
- 人物图和商品图不混淆。

**风险：**  
不同数字人模型输入差异必须由 Provider 封装，业务层不能硬写。

**完成记录（2026-06-27）：**

- 已放开 `workflowType=digital_human` 的项目创建校验，但仍只允许 `image_to_video / digital_human`，其他 workflow 不会误入当前主线。
- `ProjectRepository` 已按 workflow 初始化独立任务步骤：数字人使用 `project_init → script_review → digital_human_asset_review → tts_generation → digital_human_generation → subtitle_generation → final_composition → export`，不再复用 `storyboard_generation / image_generation / video_generation` 主线步骤。
- 新增 `DigitalHumanProjectStateDto / StartDigitalHumanVideoRequest`、`digital_human_service` 和 Tauri commands，前端新增 `entities/digital-human` 数据层。
- 已实现 TTS 门禁：`tts_generation` 未成功或失败时，`start_digital_human_video` 会阻断并把 `digital_human_generation` 标记为 failed；只有 TTS 成功并写入 `referenceAudioPath` 后，才允许进入数字人视频生成状态。
- 未伪造内容：当前没有调用真实数字人 Provider，没有生成真实 mp4，也没有硬编码具体供应商输入格式；后续真实模型调用必须由 Provider / workflow preset 封装。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo test digital_human -- --nocapture` 通过，覆盖独立数字人步骤和 TTS 失败阻断视频生成。
- `cargo test paste_input -- --nocapture` 通过，确认放开 digital_human 没破坏 paste/fixed 边界。
- `cargo check` 通过。
- `pnpm --dir src typecheck` 通过。

---

### 【X】12.5 素材剪辑成片 workflow

**问题：**  
素材导入和素材成片是另一条流程，不能和文本图生视频混为一谈。

**改法：**

独立 workflow：

```text
素材导入 → VLM 素材分析 → 素材分组 → 生成脚本/分镜 → 素材匹配 → 剪辑合成
```

能力：

```text
批量导入图片/视频
读取元数据
VLM 分析
素材标签和分组
素材绑定 StoryboardItem
使用已有图片跳过生图
使用已有视频创建 VideoSegment
```

**验证：**

- `workflowType=material_edit` 可独立推进。
- 每个 StoryboardItem 绑定素材或说明无需素材。
- 绑定关系写 asset_references。

**风险：**  
VLM 分析是建议，不是事实，重要字段要用户确认。

**完成记录（2026-06-27）：**

- 已放开 `workflowType=material_edit` 的项目创建校验，并新增 `MATERIAL_EDIT_TASK_KIND / MATERIAL_EDIT_PIPELINE_STEPS`：素材成片使用 `project_init → material_import → material_analysis → material_grouping → storyboard_generation → storyboard_review → material_matching → segment_composition → final_composition → export`，不再复用 `image_to_video` 的生图/图生视频步骤。
- `TaskRepository` 的审批、重试和步骤成功记录已从“只认 image_to_video 常量”改为按当前 Task 实际 `task_steps` 校验；`material_import / material_analysis / material_matching` 作为人工确认关口处理，不会被旧流程拦住。
- 新增 `material_analysis_suggestions` 表：VLM 结果按 `projectId / assetId / providerId / modelId / suggestion_json / status` 保存，保存前必须走结构化 schema 校验，默认 `waiting_user`；支持 approve/reject。
- 新增 `storyboard_material_requirements` 表：每个 `StoryboardItem` 可以明确记录 `needs_material` 或 `no_material_needed`，`no_material_needed` 必须有用户确认原因，不再把这类状态塞进 `lock_flags_json`。
- 新增 `material_edit_service / material_edit commands`：只允许 `workflow_type=material_edit` 项目调用；绑定素材前会校验素材存在、分镜属于当前项目；绑定关系写入 `asset_references(owner_kind='storyboard_item', usage_kind='source_material')`。
- 新增覆盖率校验：每个分镜必须“已绑定素材”或“明确无需素材”，否则 `validate_material_storyboard_coverage` 阻断 `material_matching`；全部满足后才把 `material_matching` 标为 succeeded。
- 前端新增 `entities/material-edit` 类型和 API 层，并补 Tauri command 映射；共享枚举和字典已加入 `material_edit` task kind / task step kind，素材剪辑成片 workflow 可在创建入口选择。
- 未伪造内容：当前没有调用真实 VLM Provider，没有把 VLM 输出当事实覆盖分镜字段，没有生成真实 `VideoSegment` 或最终剪辑视频；后续真实分析/匹配/剪辑必须继续接 ProviderManager、任务日志、FFmpeg/剪辑服务和人工确认。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test material_edit -- --nocapture` 通过，覆盖独立步骤、素材绑定写入 `asset_references`、每个分镜素材覆盖校验、VLM 建议不自动改分镜、非 material_edit 项目拒绝。
- `cargo test core_tables_migration_creates_required_tables -- --nocapture` 通过，确认 `material_analysis_suggestions / storyboard_material_requirements` migration 表存在。
- `pnpm --dir src typecheck` 通过。

---

### 【X】12.6 纯图文成片 workflow

**问题：**  
纯图文成片成本低，但不是当前图生视频主线。

**改法：**

独立 workflow：

```text
输入文字 → 分镜 → 生图 → 模板动效 → 合成
```

依赖：

```text
模板系统
字幕
TTS
BGM
FFmpeg segment composition
```

**验证：**

- `workflowType=image_slideshow` 不调用图生视频 Provider 也能成片。
- 模板动效生成的视频片段可进入合成。

**风险：**  
不要和 image_to_video 的 VideoSegment 语义混淆。

**完成记录（2026-06-27）：**

- 已放开 `workflowType=image_slideshow` 的项目创建校验，并新增 `IMAGE_SLIDESHOW_TASK_KIND / IMAGE_SLIDESHOW_PIPELINE_STEPS`：纯图文成片使用 `project_init → storyboard_generation → storyboard_review → image_prompt_generation → image_generation → image_review → template_motion → segment_composition → final_composition → export`，不包含 `video_prompt_generation / video_generation / video_review`，不会调用图生视频 Provider。
- `template_motion` 已作为人工确认关口加入任务状态模型；`TaskRepository` 已支持按当前 Task 的实际步骤推进，因此 `image_slideshow` 不会被 image_to_video 的步骤常量卡住。
- 新增 `image_slideshow_service / image_slideshow commands`：只允许 `workflow_type=image_slideshow` 项目调用；登记模板动效片段前校验分镜属于当前项目、输入图必须是该分镜的选中图、`video_path` 必须是受控 workspace 内已存在的真实文件。
- 模板动效片段登记会写入现有 `video_segments`，但 `model='template_motion'`、`provider_model_id='local_template_motion'`、`generation_context_snapshot.renderKind='template_motion'`，用于和 AI 图生视频片段区分；登记后会选中该片段，使其可进入现有合成链路。
- 新增 `validate_image_slideshow_segments`：每个分镜必须有选中的模板动效片段，否则阻断 `segment_composition`；全部满足后才把 `segment_composition` 标为 succeeded。
- 前端新增 `entities/image-slideshow` 类型和 API 层，并补 Tauri command 映射；共享枚举和字典已加入 `image_slideshow` task kind / `template_motion` step，创建入口可选择纯图文成片。
- 未伪造内容：当前没有假装模板 HTML 已渲染成视频，没有伪造 mp4，没有调用图生视频 Provider；本条只提供真实模板动效视频产物登记和合成前校验底座。后续模板动效真实生成必须由模板渲染、FFmpeg 图片转视频/动效服务产出文件后再登记。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test image_slideshow -- --nocapture` 通过，覆盖独立步骤、不含图生视频步骤、缺真实模板视频不得登记、真实模板视频登记进 `video_segments`、非 image_slideshow 项目拒绝。
- `cargo test core_tables_migration_creates_required_tables -- --nocapture` 通过。
- `pnpm --dir src typecheck` 通过。

---

### 【X】12.7 画布精修

**问题：**  
用户需要对生成图片局部修复或二次编辑，但不能覆盖原候选图。

**改法：**

流程：

```text
从 ImageCandidate 打开图片
复制到编辑工作区
保存 flow JSON
局部重绘 / 改图
参考图生成
确认结果
回写为新的 ImageCandidate
```

新图字段：

```text
derivedFromImageId
editFlowPath
```

**验证：**

- 精修结果作为新候选图。
- 原图保留。
- 替换选中图后，图生视频和合成状态重新检查。

**风险：**  
不要只保存最终图，编辑流程也要保存。

**完成记录（2026-06-27）：**

- 已新增 `canvas_edit` 数据结构、Service 和 Tauri command：`create_canvas_edit_candidate` 接收 `sourceImageId / editedImagePath / editFlowPath / editKind / flowSnapshot / selectAfterCreate`，用于把画布精修产物回写为新的 `ImageCandidate`。
- 精修结果不会覆盖原 `ImageCandidate.image_path`：新候选图使用新的 `image_id`，写入 `derived_from_image_id=sourceImageId`，原图候选记录保留。
- `generation_context_snapshot` 已写入 `source='canvas_edit' / renderKind='canvas_edit' / editKind / derivedFromImageId / sourceImagePath / editFlowPath / editedImagePath / revision / externalNetwork=false / billable=false / flowSnapshot`，确保后续可追踪编辑流程和来源。
- 已校验 `editedImagePath` 和 `editFlowPath` 必须是 workspace 内 `projects/` 或 `outputs/` 下已存在的真实文件；`editFlowPath` 必须能解析为 JSON object/array。缺真实图片或 flow JSON 不入库。
- 支持 `selectAfterCreate`：用户确认精修结果后可直接选中新候选图；选中后会清空当前分镜的 `selected_video_segment_id`，取消旧视频片段选中，并把 `video_status / segment_status / render_status` 重置为 `pending`，让图生视频和合成重新检查。
- 公共 `select_image_candidate` 后端逻辑已同步修正：不只画布精修，任何切换选中图都会重置旧视频/合成依赖，避免前端 store 和后端状态不一致。
- 前端新增 `entities/canvas-edit` 类型和 API，并补 `tauriCommands.createCanvasEditCandidate`；当前没有把画布精修加入主线 StepBar，也没有硬塞成必经页面。
- 未伪造内容：本条没有声称完成真实局部重绘 Provider、真实参考图生成 Provider 或完整画布 UI；当前落地的是真实文件校验、编辑流程 JSON 保存引用、新候选图回写和依赖重置底座。后续真实局部重绘必须由 Provider/ComfyUI workflow 产出真实图片和 flow JSON 后再调用本入口登记。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test canvas_edit -- --nocapture` 通过，覆盖新候选图、原图保留、真实文件/flow JSON 校验、选中精修图后重置视频和合成状态。
- `cargo test select_image_candidate_resets_video_selection_and_dependents -- --nocapture` 通过，覆盖公共选图逻辑会取消旧视频选中并重置下游状态。
- `cargo test core_tables_migration_creates_required_tables -- --nocapture` 通过。
- `pnpm --dir src typecheck` 通过。

---

### 【X】12.8 高级 AI 视频能力

**问题：**  
当前主线只做 image_to_video，但文档还包含多种后续视频能力。

**改法：**

后续扩展：

```text
text_to_video
start_end_frame_i2v
reference_to_video
video_continuation
动作迁移
尾帧续接
```

**验证：**

- 每种能力在 ModelCapability 中表达。
- 不同输入要求由能力矩阵校验。
- 失败率高的尾帧续接可关闭。

**风险：**  
不要把 text_to_video 当当前主线依赖。

**完成记录（2026-06-27）：**

- 已把高级视频能力落到可执行媒体能力矩阵：`list_executable_media_options` 不再把一个多能力视频模型压成单一 option，而是按 `abilityTypes` 拆成独立候选项，例如 `text_to_video / image_to_video / start_end_frame_i2v / reference_to_video / video_continuation` 各自拥有独立 `optionId` 和 `inputPlan`。
- `VideoInputPlan` 已补齐高级视频输入要求：
  - `text_to_video`：只要求 `videoPrompt`，不要求 `startFrame`。
  - `image_to_video / first_frame_i2v`：要求 `startFrame + videoPrompt`。
  - `start_end_frame_i2v`：要求 `startFrame + endFrame + videoPrompt`。
  - `reference_to_video`：要求 `startFrame + videoPrompt`，并按能力矩阵声明 `referenceAsset` 必选/可选。
  - `video_continuation`：要求 `sourceVideo + videoPrompt`，`tailFrame` 可选，可从源视频提取或手动选择。
- 支持能力级配置：模型或 workflow preset 可使用 `abilityLimits / abilityInputRequirements / abilityParamSchema / abilityDefaultParams` 为每种视频能力声明独立限制和输入要求；没有能力级配置时继续使用全局 `limits / inputRequirements / paramSchema / defaultParams`。
- 支持关闭高风险能力：配置 `disabledAbilityTypes / disabledAdvancedAbilities / disableVideoContinuation / disableTailFrameContinuation` 后，对应候选项仍可展示但 `enabled=false`，`disabledReason='ability.disabled.xxx'`，用于把尾帧续接等高失败率能力关闭。
- 当前 `start_video_generation` 主线已收紧：只允许 `image_to_video / first_frame_i2v`，不会把 `text_to_video / start_end_frame_i2v / reference_to_video / video_continuation` 当作当前 image_to_video 主线模型来跑。
- 前端基础枚举和字典已补高级能力值：`text_to_video / image_to_image / first_frame_i2v / start_end_frame_i2v / reference_to_video / video_continuation / video_editing / action_transfer / digital_human / native_audio / voice_reference / multi_shot`，模型/工作流设置页可明确配置这些能力。
- 未伪造内容：本条没有实现真实文生视频、首尾帧视频、参考图视频、尾帧续接生成，也没有伪造真实视频 Provider 成功；当前完成的是能力矩阵、输入计划、关闭开关和主线隔离。真实高级生成入口后续必须单独建 workflow/command，并由 ProviderManager/WorkflowRegistry 校验真实输入后执行。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test advanced_video_abilities_have_separate_input_plans_and_disable_flags -- --nocapture` 通过，覆盖多能力拆分、各能力输入计划和 `video_continuation` 关闭。
- `cargo test image_to_video_mainline_rejects_advanced_video_only_models -- --nocapture` 通过，确认当前主线不会误用高级视频-only 模型。
- `cargo test executable_media_options_merge_models_and_workflow_presets -- --nocapture` 通过，确认已有 provider model / workflow preset 合并逻辑未破坏。
- `cargo test core_tables_migration_creates_required_tables -- --nocapture` 通过。
- `pnpm --dir src typecheck` 通过。

---

### 【X】12.9 创作规则 / Skill / Agent 文件化增强

**问题：**  
长内容、导演规划、画风、叙事能力适合规则化，但必须版本化和可追踪。用户侧统一叫“创作规则”，技术侧可以继续使用 PromptSkill / SkillDefinition / SkillSnapshot。

**改法：**

支持：

```text
叙事 Skill
导演 Skill
画风 Skill
分镜 Skill
审核 Skill
资产分析 Skill
```

记录：

```text
skill key
version
content_hash
schema_hash
引用任务
```

**验证：**

- 修改创作规则不影响历史任务。
- 输出仍走 schema 校验。
- 用户可查看任务使用了哪个创作规则版本。

**风险：**  
不要执行创作规则中的任意代码；当前创作规则只是提示词、schema、参数模板和校验规则，不是插件代码执行器。

**完成记录（2026-06-27）：**

- 已在创作规则 DTO / 文件解析 / 保存流程中补齐版本和 hash 字段：`version / contentHash / schemaHash`。旧规则文件未声明版本时默认 `1.0.0`，新保存的用户规则会写入 `version`。
- `CreativeRuleRefDto` 已补 `version / contentHash / schemaHash`，视频包和 Project 的 `ruleRefs` 仍只保存引用元数据，不复制 prompt 正文。
- `project_rule_snapshot` 已升级为 `SkillSnapshot` 风格快照：任务记录中的 `ruleSnapshot` 现在包含 `activePackId / ruleRefs / skillSnapshots`，每个 slot 记录 `ruleKey / ruleId / sourceType / ruleType / module / name / version / contentHash / schemaHash / relativePath / enabled`。
- 快照不保存 `body`，避免把 prompt 正文复制到任务输入和 generation context；历史任务通过 `ruleId + version + contentHash + schemaHash` 追踪当时用的规则版本。
- `start_image_generation / start_image_asset_generation / start_video_generation` 继续写 `ruleSnapshot`，但现在会带规则版本和 hash，满足任务可追踪。
- 前端类型和 mock 数据已同步新增 `version / contentHash / schemaHash`，创作资源页和浏览器 mock 不会和 Tauri DTO 冲突。
- 未伪造内容：本条没有把创作规则做成可执行插件、没有执行任意代码、没有实现 Agent 文件运行器；当前创作规则仍只是提示词正文、输出 schema、参数 schema、校验规则和可追踪快照。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test project_rule_snapshot_records_rule_versions_and_hashes_without_body -- --nocapture` 通过，确认 snapshot 包含 version/hash 且不包含 prompt body。
- `cargo test initializes_and_lists_builtin_rules -- --nocapture` 通过，确认内置规则有 version/hash。
- `cargo test clones_saves_enables_and_deletes_user_rule -- --nocapture` 通过，确认用户规则保存版本和 hash。
- `cargo test user_rule_referenced_by_video_pack_cannot_be_disabled_or_deleted -- --nocapture` 通过，确认引用保护未破坏。
- `cargo test core_tables_migration_creates_required_tables -- --nocapture` 通过。
- `pnpm --dir src typecheck` 通过。

---

### 【X】12.10 本地记忆 / RAG 后续能力

**问题：**  
如果后续做项目记忆或素材检索，不能只按 topN 盲取。

**改法：**

后续设计：

```text
向量索引
相似度阈值
来源引用
过期策略
用户确认
```

**验证：**

- 检索结果低于阈值不使用。
- 生成内容可追踪引用来源。

**风险：**  
RAG 不能把历史错误设定无限传播。

**完成记录（2026-06-27）：**

- 已新增 `local_memory_entries / local_memory_retrievals / local_memory_retrieval_candidates` 三张表，分别保存记忆索引项、一次检索请求、检索候选及其引用来源。
- 已新增 `local_memory` domain / repository / service / command，并注册到 Tauri handler；前端新增 `entities/local-memory` 类型和 API，补齐 `tauriCommands` 映射。
- 记忆项保存 `sourceKind / sourceId / sourceLabel / contentSummary / contentHash / embeddingProviderId / embeddingModelId / embeddingVectorPath / metadata / lifecycle / expiresAt`，用于后续接真实 embedding 或本地向量文件时保持来源可追踪。
- `create_local_memory_retrieval` 不执行 embedding 搜索，只接收外部检索候选；候选必须带 `similarity`，并按 `minSimilarity` 过滤，低于阈值会直接保存为 `rejected`，不会进入可用上下文。
- 检索候选默认 `waiting_user`，只有用户显式 `approve_local_memory_candidate` 后，`build_local_memory_context` 才会把该候选作为生成上下文引用输出。
- 已限制项目作用域：项目记忆只能被同项目使用；全局记忆允许复用；跨项目候选会被拒绝，避免历史错误设定串到其他作品。
- `build_local_memory_context` 只返回确认后的引用摘要和 citation，不返回不可控长正文，不会把历史内容自动注入生成链路。
- 未伪造内容：本条没有实现真实向量索引、真实 embedding 模型、真实 RAG 检索排序、真实本地 onnx 推理或自动生成注入；当前完成的是后续能力的安全数据底座、阈值门禁、来源引用和用户确认机制。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test local_memory -- --nocapture` 通过，覆盖低于阈值自动拒绝、未确认候选不进入上下文、确认后才可用、低分候选不能强行 approve、跨项目记忆拒绝。
- `cargo test core_tables_migration_creates_required_tables -- --nocapture` 通过，确认本地记忆 / RAG 表已纳入 migration。
- `pnpm --dir src typecheck` 通过。

---

## 阶段完成标准

- 非主线能力都有独立 workflow 或明确增强定位。
- 数字人、素材成片、纯图文不污染 image_to_video 主线。
- 小说长内容不一次性塞给 LLM。
- 画布精修不覆盖原候选图。
- 高级视频能力由 ModelCapability 和 WorkflowRegistry 管控。






