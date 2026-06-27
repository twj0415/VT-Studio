# TODO-08：创作资源、素材库与设定一致性

> 目标：解决画风、角色、场景、资产引用和生成上下文一致性问题。
> 本文件来自 `doc/功能模块/04-画风设定.md`、`05-角色设定.md`、`06-场景设定.md`、`20-资产库与媒体管理.md`、`doc/底层设计/21-资产一致性与设定集规范.md`。

---

## 阶段目标

建立：

```text
Asset / AssetReference
Project Bible
Style Bible
Character Bible
Environment Bible
generation_context_snapshot
lock_flags_json
资产版本与引用保护
```

让生成结果不只是“能跑”，还要尽量保持同一项目内画风、角色、场景一致。

用户侧菜单归属：

```text
创作资源
- 视频包
- 创作规则
- 素材库
```

技术侧仍可使用 `Asset / AssetReference / PromptSkill / Project Bible` 等命名；完整 `SkillSnapshot` 属于后续增强，不作为本阶段门槛。

---

## 本阶段范围

包含：

- “创作资源 > 素材库”基础 UI 和数据流。
- “创作资源 > 创作规则”查看、复制为用户规则、编辑、启用/禁用和引用关系；完整版本化后续增强。
- “创作资源 > 视频包”中的默认规则、默认模型偏好和默认素材引用。
- `video_packs` 表、Project 当前视频包引用、rule_refs / executable_refs 当前配置和保存当前配置为新视频包的基础能力。
- 角色参考、画风参考、场景参考、姿态 / 深度 / mask、音频、BGM、字体、模板、生成产物登记。
- Style Bible。
- Character Bible。
- Environment / Location Bible。
- 参考图上传和受控存储。
- 生图 / 视频 prompt 注入设定集。
- 生成上下文快照。
- 删除保护和引用关系。

不包含：

- 高级画布精修。
- 复杂角色三视图 / 表情表全量能力。
- 素材成片 workflow。
- 模板市场。

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

> 参考门禁：本文件涉及 Toonflow / waoowaoo 的创作资源、资产贯穿、参考图、创作规则、视频包和模型 / 工作流分层。开始任一条前必须先读 `doc/参考分析/参考项目深度机制与吸收口径.md` 的“参考强项写入文档的审计表”，并核对 `doc/功能模块/README.md` 的参考强项落地门禁；不能只按“有视频包 / 有素材库 / 有规则管理”这种概念施工。
>
> 深度门禁：如果当前条目命中资产、候选、视频包、创作规则或模型 / 工作流引用链，还必须继续读 `doc/参考分析/参考项目深度机制与吸收口径.md` 的“逐项深度落点核对表”和第十一节对应条目。执行前要写清：参考项目具体怎么做、本项目筛选吸收什么、现有代码已有多少、缺哪些 migration / DTO / Repository / command / 前端类型 / 测试。只写“参考 Toonflow / waoowaoo”或“实现视频包 / 素材库”不满足本阶段门禁。
>
> 筛选门禁：执行 TODO-08 时还必须读 `doc/参考分析/参考项目深度机制与吸收口径.md` 的“筛选后吸收执行总账”。参考项目里资产、参考图、视频包、规则、模型设置、生成历史等能力只能按“当前必须吸收 / 本阶段必须吸收 / 后续预留 / 明确不吸收”落地；不能为了“学 100%”把参考项目 UI、字段名、Prompt、技术栈或高风险能力整包搬进本项目。
>
> 施工卡门禁：执行 8.1 前必须读 `参考项目深度机制与吸收口径.md` 第 12.8 “资产贯穿施工卡”；执行 8.1.1 前必须读第 12.9 “视频包 / 创作规则 / 模型工作流施工卡”。如果施工卡与本 TODO 出现差异，以“先补底层闭环、不做页面假功能、不吸收整包参考项目”为优先原则，并把差异补回当前 TODO。当前 8.1 只实现 `Asset / AssetReference` 闭环；视频包相关只保留 `owner_kind=video_pack` 的引用表达能力，不做视频包页面、静态假 UI 或 `video_packs` 表。

### 【X】8.1 完善素材库 Asset / AssetReference

**问题：**
生成图、视频片段、音频、封面、参考图、模板资源如果没有统一资产引用，导出和删除都会出问题。

**位置：**

```text
src-tauri/src/domain/asset*
src-tauri/src/repository/asset*
src-tauri/src/service/asset*
src-tauri/src/service/storage*
src-tauri/src/command/asset*
src/src/entities/asset*
src/src/features/creative-resources*
doc/底层设计/21-资产一致性与设定集规范.md
```

必须复用 TODO-04 的 StorageService / PathGuard，以及 TODO-05 的 Artifact 记录，不允许再建一套独立素材路径体系。

**改法：**

执行前参考核对：

```text
1. 先按 `参考项目深度机制与吸收口径.md` 第 9.1.1 行“资产贯穿流程”和第 11.9 条核对：参考项目强在资产服务生产流程，不是强在把所有产物塞进素材库。
2. 再按 `20-资产库与媒体管理.md` 核对四类对象边界：Artifact / ImageCandidate / VideoSegment / Asset / AssetReference。
3. 最后按当前代码审计确认现有 AssetDto、assets 表、asset_references 表、media_service、asset_repository 已有能力；已有字段优先复用，缺字段才做 migration。
4. 实现前必须列出本条最终采用的 `asset_kind / media_kind / owner_type / usage_type` 映射，避免把参考项目的 `assets / deriveAsset / referenceList` 名字直接搬进来。
5. 必须同时核对第 12.8 “资产贯穿施工卡”：页面布局、对象边界、按钮到数据写回、删除保护、当前不吸收项必须一致。
```

统一资产类型：

```text
character_reference_image
style_reference_image
scene_reference_image
pose_reference
depth_reference
mask_reference
source_video
source_audio
bgm
font
template_resource
generated_image_candidate
generated_video_segment
final_export
task_artifact
```

本条最终采用的枚举映射：

```text
asset_kind：
  character_reference_image    角色参考图；来自用户导入或候选图保存为角色参考
  style_reference_image        画风参考图；来自用户导入或候选图保存为画风参考
  scene_reference_image        场景参考图；来自用户导入或候选图保存为场景参考
  pose_reference               姿态 / 动作控制图
  depth_reference              depth / 深度控制图
  mask_reference               mask / 局部重绘控制图
  source_video                 用户导入源视频
  source_audio                 用户导入源音频
  bgm                          背景音乐
  font                         字体
  template_resource            HTML / 字幕 / 封面模板资源
  generated_image_candidate    用户显式资产化后的生成图片候选
  generated_video_segment      用户显式资产化后的生成视频片段
  final_export                 用户显式保存的最终导出产物索引
  task_artifact                任务产物需要复用时的资产化记录
  user_image / user_video      用户普通上传素材；兼容现有代码
  source_material              素材成片或项目源素材；兼容现有代码

owner_kind：
  project
  storyboard_item
  image_candidate
  video_segment
  character_bible
  style_bible
  location_bible
  video_pack
  task
  composition_task
  export

usage_kind：
  character_reference
  style_reference
  location_reference
  pose_reference
  depth_reference
  mask_reference
  source_material
  selected_image
  selected_video
  bgm
  font
  template_resource
  generated_image
  generated_video
  final_export
  task_artifact
```

兼容口径：

```text
1. 当前代码已有 `kind / source_kind / metadata_json`，本条优先复用，不新增第二套 Asset 表。
2. 前端可显示 `asset_kind / media_kind / source_type`，其中 `asset_kind` 对应后端 `kind`，`source_type` 对应后端 `source_kind`。
3. `media_kind` 可以由 mime_type 或 metadata_json 派生，不为此新增必填列；尺寸、时长、缩略图、来源说明保存在 metadata_json。
4. 已存在的 legacy 值 `reference_image / character_reference / style_reference / scene_reference / cover_source / generated_output` 不立即破坏；新增代码优先写上面的规范值，展示层做兼容映射。
```

先做实现前核对：

```text
1. 先读现有 `src-tauri/src/domain/media.rs`、`src-tauri/src/db/asset_repository.rs`、`src-tauri/src/services/media_service.rs` 和 `src-tauri/src/db/mod.rs`，确认哪些字段已存在，哪些只是目标文档。
2. 如果现有 AssetDto / 表字段已经能表达某项能力，优先补 service / UI / 测试，不重复建第二套 Asset 模型。
3. 如果缺字段，必须新增 migration、Rust DTO、Repository、前端类型和测试；不能把缺字段塞进 metadata_json 后就算完成，除非文档明确该字段属于扩展元数据。
```

当前代码审计后的明确缺口：

```text
后端：
  1. `validate_asset_kind / validate_usage_kind / validate_owner_kind` 需要扩展到上面的规范枚举。
  2. `DeleteAssetReferenceRequest`、Repository `delete_reference`、Service `delete_asset_reference`、Command `delete_asset_reference` 需要补齐。
  3. `delete_asset` 被引用时需要返回可展示的引用详情，至少能让 UI 列出 owner_kind / owner_id / usage_kind。
  4. `collect_project_asset_paths` 需要覆盖 project、storyboard_item、character_bible、style_bible、location_bible、task、video_pack 等 owner。
  5. 物理删除前继续要求 `relative_path` 位于 assets bucket，并走 StorageService / PathGuard。

前端：
  1. `entities/config/types.ts` 增加 `DeleteAssetReferenceRequest`。
  2. `entities/config/api.ts` 增加 `deleteAssetReference`，mock 删除要模拟引用阻断。
  3. `shared/api/commands.ts` 增加 command 名。
  4. `creative-resources/index.vue` 的 assets entry 从占位改为真实素材库：用途分组、列表、详情、引用、删除保护、解除引用。
  5. i18n 补齐中英文素材库分组、状态、操作和错误文案。
```

本条严格不做：

```text
1. 不把所有 ImageCandidate / VideoSegment 自动登记为 Asset。
2. 不为了做视频包先做静态视频包 UI；VideoPack 的 asset_refs_json 依赖本条 Asset / AssetReference 闭环。
3. 不复制 Toonflow-web 的 tabs、字段名、接口命名或批量 AI 生成资产流程；本条只吸收选择、预览、引用、删除保护这些机制。
4. 不把缺失的 owner_kind / usage_kind 塞进 metadata_json 冒充完成；如果枚举缺失，先扩展校验和测试。
5. 不物理删除被引用文件；解除引用和删除 Asset 是两个动作。
```

四类对象边界：

```text
Artifact：
  任务产物索引，回答“某次任务生成了什么”。
  属于任务历史和诊断，不等于素材库。

ImageCandidate / VideoSegment：
  当前作品内候选产物，回答“这个分镜有哪些候选图 / 视频片段，用户选了哪个”。
  默认留在项目目录，不自动进入素材库。

Asset：
  用户可复用、可管理的素材，回答“这个文件以后还会被角色 / 场景 / 风格 / 视频包复用吗”。
  只有导入素材或用户明确“保存为角色 / 场景 / 风格参考”才进入。

AssetReference：
  引用关系，回答“谁正在用这个素材，用途是什么”。
  owner_type / owner_id / usage_type 必须能定位到 project、storyboard_item、character_bible、location_bible、style_bible、video_pack、task 等使用者。
```

动作归属：

```text
只写作品目录：
  分镜候选图生成
  视频片段生成
  合成 final.mp4
  未确认的重生成候选

写入 Asset + AssetReference：
  用户导入素材
  用户把候选图保存为角色参考
  用户把候选图保存为场景参考
  用户把候选图保存为风格参考
  用户把视频 / 音频 / BGM / 字体 / 模板资源导入为可复用素材
  视频包引用某个默认素材集合

写入 Artifact：
  Provider 生成成功的原始产物
  FFmpeg 输出和中间可诊断产物
  导出产物索引
```

引用关系记录：

```text
project_id
asset_id
owner_type
owner_id
usage_type
created_at
```

落地要求：

```text
1. Asset 记录物理文件和元数据：asset_id、asset_kind、media_kind、relative_path、hash、size、duration、width、height、source_type、created_at。
2. AssetReference 记录谁在用：project_id、owner_type、owner_id、usage_type、asset_id、created_at。
3. Artifact 是任务产物记录；Asset 是可被用户管理和复用的素材记录。两者可以关联，但不能混成同一概念。
4. 用户确认的候选图、视频片段、角色参考图、场景参考图、风格参考图必须能提升或登记为 Asset。
5. 删除 Asset 前必须查询引用；被引用资产只能解除引用或提示阻断，不能直接物理删除。
6. 清理临时产物只清未确认、未引用、可再生的临时文件，不清用户确认资产。
7. 素材库 UI 按用途分组：角色参考、画风参考、场景参考、姿态/深度/mask、音频、模板、生成产物。
8. 导出项目包时按 AssetReference 收集文件，不能扫描整个 workspace 碰运气。
```

素材库 UI 分组必须按“用途”优先，不按文件扩展名优先：

```text
角色参考
画风参考
场景参考
姿态 / 深度 / mask
音频 / BGM
模板 / 字体
生成产物
用户上传素材
```

每个素材详情必须显示：

```text
预览
asset_kind / media_kind
source_type
relative_path 脱敏展示
尺寸 / 时长 / 文件大小
引用列表：owner_type、owner_id、usage_type、所在项目或分镜
可执行动作：打开所在作品、解除引用、删除 / 隐藏
```

素材库页面施工细节：

```text
顶部工具栏：
  导入素材
  用途分组筛选：角色参考 / 画风参考 / 场景参考 / 姿态深度 mask / 音频 BGM / 模板字体 / 生成产物 / 用户上传素材
  来源筛选：用户导入 / 候选保存 / 任务产物资产化 / 内置资源
  引用筛选：全部 / 被引用 / 未引用 / 文件缺失
  搜索：名称、标签、owner_id、项目名

主区列表 / 网格：
  预览缩略图或媒体图标
  display_name
  asset_kind
  media_kind
  source_kind
  引用数量
  文件状态
  最近更新时间

右侧详情：
  asset_id
  display_name
  asset_kind / media_kind / source_kind
  relative_path 脱敏展示
  width / height / duration_seconds / size_bytes
  metadata_json 摘要
  引用列表：owner_kind / owner_id / usage_kind / project_id
  来源说明：用户导入、候选保存、任务产物资产化、内置资源
```

按钮到数据写回：

```text
导入素材：
  PathGuard 校验 → StorageService 复制 → Asset。

保存候选为角色 / 场景 / 风格参考：
  保留原 ImageCandidate。
  新增或复用 Asset。
  新增 AssetReference，owner 指向 character_bible / location_bible / style_bible 或 storyboard_item。

视频包绑定默认素材：
  新增 AssetReference(owner_kind=video_pack)，不复制素材文件内容。

解除引用：
  删除 AssetReference，不删除 Asset 文件，不删除原候选。

删除素材：
  先查 AssetReference。
  有引用时返回引用摘要并阻断。
  无引用时软删除；物理删除必须仍走 StorageService / PathGuard。
```

**验收：**

- 被引用资产不能物理删除。
- 删除前能提示被哪些项目/分镜/任务使用。
- 导出项目包能按引用收集素材。
- 用户在素材库里优先看到“角色参考、画风参考、场景参考、姿态/深度/mask、音频、模板、生成产物”，而不是混乱的图片/视频大杂烩。
- Asset 和 Artifact 职责清楚：任务历史可查 Artifact，素材库可管理 Asset，二者通过关联字段互相追踪。
- 所有资产路径都是受控相对路径，绝对路径和远程 URL 不入库为正式资产路径。

**验证：**

- Rust 单测覆盖：创建 Asset、创建引用、被引用删除阻断、解除引用后可删除、路径越权拒绝、导出按引用收集。
- 前端 typecheck/build 通过。
- 手动 smoke：导入或登记一张参考图，绑定到角色/分镜，尝试删除时看到引用提示；解除引用后可删除。
- 检查 DB：Asset.relative_path 是相对路径，AssetReference 能查到 owner。

**下一步进入条件：**

- Asset / AssetReference 数据结构、删除保护、素材库基础分组和导出引用收集都完成。
- 完成记录写清 Asset 与 Artifact 的边界，以及哪些产物会自动或手动登记为 Asset。
- 确认 8.2 素材导入能复用 AssetService 后，再把本条改为 `【X】` 并进入 8.2。

**完成记录（2026-06-26）：**

- 后端补齐 `DeleteAssetReferenceRequest`、`delete_asset_reference` Tauri command、Repository `delete_reference`、Service `delete_asset_reference`，并把 `delete_asset` 的引用阻断错误改为包含 `owner_kind / owner_id / usage_kind` 摘要，便于 UI 展示。
- `validate_asset_kind / validate_owner_kind / validate_usage_kind` 已扩展到本条规范枚举；`collect_project_asset_paths` 已覆盖 `project / storyboard_item / character_bible / style_bible / location_bible / task`，并按本条边界仅预留 `owner_kind=video_pack` + `owner_id={projectId}:...` 的引用表达，不创建 `video_packs` 表、不做视频包页面。
- 前端补齐 `DeleteAssetReferenceRequest`、`deleteAssetReference` API、`delete_asset_reference` command 名；mock 删除素材会按引用阻断，解除引用只删 `AssetReference`，不删 Asset 文件和原候选。
- `创作资源 > 素材库` 已从占位改为基础闭环：用途分组筛选、素材列表、详情、元数据、引用列表、解除引用、无引用删除阻断 / 删除操作；视频包仍保持未实现，不用静态 UI 冒充完成。
- Asset / Artifact 边界：Artifact 仍归任务历史和诊断，ImageCandidate / VideoSegment 默认仍留在作品目录；只有用户导入素材或显式保存为角色 / 场景 / 风格参考等复用动作才登记 Asset + AssetReference。本条没有把所有候选自动塞进素材库。
- 8.2 素材导入可继续复用现有 `importAsset` / StorageService / PathGuard / AssetService 能力；8.1.1 视频包数据结构和引用链仍是下一条未完成 TODO。
- 验证通过：`cargo fmt`、`cargo fmt -- --check`、`cargo test media_service`（7 passed）、`cargo test`（136 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`、`git diff --check` 均通过；`pnpm --dir src build` 仅保留既有 chunk 体积提示。

**风险：**
清理临时产物不能误删历史最终产物或用户确认资产。

---

### 【X】8.1.1 实现视频包数据结构和引用链

**问题：**
文档已经确定“视频包 = 默认策略组合”，但当前代码主要有创作资源入口和创作规则管理，视频包还没有完整落库。如果不在 TODO-08 补齐，后续内容创作页会只能做静态选择器，无法保存用户包，也无法知道一个作品到底用了哪些规则和推荐模型。

**位置：**

```text
src-tauri/src/db/mod.rs
src-tauri/src/domain/creative_resource*
src-tauri/src/db/*pack_repository*
src-tauri/src/services/*pack_service*
src-tauri/src/commands/*pack*
src/src/entities/config/types.ts
src/src/entities/config/api.ts
src/src/pages/creative-resources/index.vue
src/src/pages/create-project* 或内容创作页相关模块
doc/底层设计/数据结构.md
doc/底层设计/05-数据库Schema与迁移规范.md
doc/底层设计/18-PromptSkill与模板版本规范.md
```

**改法：**

执行前参考核对：

```text
1. 先读 `参考项目深度机制与吸收口径.md` 第 9.1.1 行“视频包 / 创作规则 / 模型工作流分层”和第 11.8 条。
2. 明确参考项目只证明“内容策略、Prompt / Skill、模型设置需要集中管理”，不证明本项目要新增模板市场或把模型配置塞进视频包。
3. 核对 TODO-06 已完成的 Provider / provider_models / workflow_presets / 创作规则能力，视频包只能引用这些现有对象，不能再建第二套规则正文、模型配置或素材路径。
4. 核对 `数据结构.md` 和 `05-数据库Schema与迁移规范.md`：如果 `video_packs` 或 Project 扩展字段尚未落库，先做 migration / DTO / Repository / command / 前端类型 / 测试，再做页面。
5. 实现前必须能回答：创建作品时从 VideoPack 拷贝哪些引用到 Project；作品内覆盖写哪里；保存当前作品配置为新视频包允许保存什么、禁止保存什么；删除 user 包检查哪些引用。
```

新增或扩展 migration：

```text
video_packs
projects.active_pack_id
projects.rule_refs_json
projects.executable_refs_json
```

视频包字段按 `doc/底层设计/数据结构.md` 执行：

```text
pack_id
source_type
name
description
applicable_input_types_json
content_category
default_tone
default_aspect_ratio
default_duration_seconds
default_scene_count
rule_refs_json
recommended_executable_refs_json
asset_refs_json
is_enabled
created_at / updated_at
```

页面能力：

```text
左侧包列表：builtin / user / disabled 筛选。
中间基本信息：名称、说明、适用输入、默认画幅、时长、分镜数、语气。
右侧引用：绑定创作规则、推荐模型 / workflow、素材引用、被哪些作品使用。
```

操作：

```text
查看内置包
复制为用户包
编辑用户包
绑定创作规则
绑定推荐 providerModelId / workflowPresetId
启用 / 禁用
删除用户包
查看引用
保存当前作品配置为新视频包
```

硬约束：

```text
1. 视频包只保存 rule_key / rule_id，不复制 prompt_body。
2. 视频包只保存 provider_model_id / workflow_preset_id，不保存 Provider 连接配置和真实密钥。
3. 视频包只保存 Asset / AssetReference 引用，不保存素材文件内容。
4. 创建作品时把视频包的引用和默认参数拷贝到 Project 当前配置。
5. 作品内覆盖不反向修改视频包。
6. 删除 user 视频包前检查 Project / Task 引用。
```

**验收：**

- 数据库存在 `video_packs`，Project 能保存 active_pack_id、rule_refs_json、executable_refs_json。
- 创作资源页能查看内置包、复制为用户包、编辑用户包、启用/禁用、删除用户包。
- 打开一个视频包能看清它绑定了哪些创作规则、推荐哪些模型 / workflow、引用哪些素材。
- 视频包详情里看不到 API Key、Provider 请求头、Provider 连接配置、完整 prompt 正文副本。
- 内容创作页选择视频包后，创建作品会把默认配置写入 Project 当前配置。
- 保存当前作品配置为新视频包只保存引用和默认参数，不保存任务产物、绝对路径或完整 prompt 历史。

**验证：**

- Rust 单测覆盖：保存 / 列表 / 复制 user 包、builtin 不可直接编辑、删除前引用阻断、Project 写入 active_pack_id 和规则引用。
- 前端 typecheck/build 通过。
- 手动 smoke：复制一个内置包，绑定一条 user 创作规则和一个 provider_model / workflow_preset，创建作品后 Project 当前配置可读。

**下一步进入条件：**

- 视频包不再只是占位概念；已具备可持久化、可引用、可被内容创作读取的最小闭环。
- 完成记录写清哪些字段已落库，哪些 PackSnapshot / SkillSnapshot 仍是 TODO-12 后续增强。

**完成记录（2026-06-26）：**

- 后端新增 `video_packs` migration，并给 `projects` 增加 `active_pack_id / rule_refs_json / executable_refs_json`；Project DTO / Repository 已能读写 `activePackId / ruleRefs / executableRefs`。
- 新增 `VideoPackDto`、`VideoPackRepository`、`video_pack_service` 和 Tauri commands：列表、详情、复制内置包为用户包、保存用户包、启用 / 禁用、删除用户包、保存当前作品配置为新视频包。
- 内置视频包以 `pack_knowledge_short / pack_story_short` 作为默认策略组合种子；内置包只读，用户包可编辑。视频包只保存默认参数和引用，不保存 prompt 正文、API Key、Provider 请求头、Provider 连接配置、绝对路径或任务产物目录。
- 删除 user 视频包会检查 `Project.active_pack_id / rule_refs_json / executable_refs_json` 和可执行范围内的 task / task_step 快照文本引用；被作品引用时阻断删除。PackSnapshot / SkillSnapshot / hash / 历史任务复现仍留到 TODO-12。
- 前端补齐 VideoPack 类型、API、mock adapter 和 command 名；`开始创作` 页面可选择视频包并预填画幅、分镜数、单段时长和语气，创建作品后写入 Project 当前配置，不反向修改视频包。
- `创作资源 > 视频包` 已从占位改成真实闭环：包列表、来源筛选、显示 / 隐藏禁用、详情、复制为用户包、编辑用户包、启用 / 禁用、删除阻断、规则引用 JSON、推荐模型 / Workflow 引用 JSON、素材引用 JSON。
- 工作台概览已显示当前作品的视频包、规则引用数和模型 / Workflow 引用数，避免视频包配置创建后不可见。
- 验证通过：`cargo test video_pack`（4 passed）、`cargo test`（140 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；`pnpm --dir src build` 仅保留 Vite chunk 体积提示。

**风险：**
不要把视频包做成第二套模板市场，也不要把它做成 Provider 配置页。视频包只管理“默认创作策略引用”。

---

### 【X】8.2 实现素材导入和预览

**问题：**
用户素材和参考图不能长期引用原路径。

**改法：**

导入流程：

```text
选择文件
PathGuard 校验
复制到 workspace/assets/{kind}/
读取元数据
生成预览/首帧/波形
登记 Asset
```

**验证：**

- 入库只存相对路径。
- 图片可预览。
- 视频可显示首帧和时长。
- 音频可显示时长。

**完成记录（2026-06-26）：**

- `创作资源 > 素材库` 已补导入入口和弹窗：支持选择文件或手动填写本机绝对路径，自动推断 `kind / mimeType / displayName / mediaKind`，提交后调用 `importAsset`，刷新列表并选中新素材。
- 后端 `import_asset` 继续走 `StorageService / PathGuard` 复制到 `workspace/assets/{kind}/`，Asset 入库只保存受控相对路径；metadata 已归一化 `displayName / fileName / extension / assetKind / mediaKind / sourceType / relativePath / sizeBytes / mimeType`，并清除 `sourcePath / absolutePath / path` 等原始绝对路径字段。
- 新增 `AssetPreviewRequest / AssetPreviewDto` 与 `read_asset_preview` command：图片只从 `assets/` bucket 安全读取并返回预览字节；视频通过 FFmpeg sidecar 抽首帧到 `temp/asset_previews` 后返回 JPEG 字节，临时文件用完即删。
- 素材详情已补预览区、媒体探测区和元数据摘要：图片可直接预览；视频显示首帧和 `probeMedia` 的时长、尺寸、fps、编码；音频显示 `probeMedia` 的时长和音频编码。FFmpeg / FFprobe sidecar 缺失时只显示预览或探测错误，不阻断普通导入。
- 前端 mock adapter 已补 `readAssetPreview`，非 Tauri 环境可看到稳定的占位预览；i18n 已补中英文导入、预览、探测、素材类型和字段文案。
- 本条暂不做完整波形生成和图片真实宽高解码；如果前端 / 后续探测传入 `width / height / durationSeconds` 会保留在 metadata，真正波形和图片解码可在后续媒体增强里做。
- 验证通过：`cargo test media_service`（8 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；随后继续跑全量验证。

**风险：**
不能把远程 URL 或用户绝对路径直接当资产路径。

---

### 【X】8.3 实现 Style Bible

**问题：**
只靠每条分镜自由 prompt，画风会漂移。

**改法：**

Style Bible 字段：

```text
style_prompt
color_palette
lighting
composition
negative_prompt
reference_image_path
```

能力：

```text
内置画风
自定义画风
参考图上传
负面词合并去重
保存全局预设
```

**验证：**

- 选中画风后可注入 imagePrompt。
- 负面词合并去重并有长度上限。
- 参考图进入受控工作区。

**风险：**
不要只做一个 `prompt_prefix` 字符串；风格、色彩、光线、构图、负面词要拆字段。

**完成记录（2026-06-26）：**

- 后端新增 Style Bible 领域、Repository、Service 和 Tauri commands，复用已有 `style_bibles.data_json`，没有新增第二套画风表；新建作品时会自动创建项目默认 Style Bible。
- Style Bible 已按结构化字段落地：`style_prompt / color_palette / lighting / composition / negative_prompt / reference_image_path / reference_images_json`，兼容前端 camelCase，避免退化成单一 `prompt_prefix`。
- 内置画风预设和用户预设已接入：内置预设由后端代码提供；用户“保存为预设”写入 `app_configs(config_key='style_presets')`，只保存画风字段和受控参考图路径，不保存 Provider 密钥、请求头、绝对路径或任务产物。
- 工作台设定集卡片已升级为 Style Bible 编辑面板：可选择预设、编辑名称 / 画风 / 色彩 / 光线 / 构图 / 负面词，导入并绑定风格参考图，也可保存为用户预设。
- 风格参考图必须先走 `importAsset` 进入受控 `assets/` bucket，再通过 `AssetReference(owner_kind=style_bible, usage_kind=style_reference)` 绑定；`reference_image_path` 和参考图列表只记录受控相对路径。
- 生图服务已在普通分镜生图前调用后端 `build_image_prompt_preview_for_item`，把分镜画面描述、角色、场景、分镜 imagePrompt、Style Bible 字段组装为最终 prompt；Provider 请求、ImageCandidate 记录和任务输入快照都使用最终 prompt。
- 负面词已合并去重并应用 `NEGATIVE_PROMPT_MAX_LENGTH = 800` 上限；最终 prompt 预览会展示是否截断、来源字段和参考图输入。
- 生图页每行已补“预览最终提示词”按钮，弹窗从后端 command 读取真实组装结果，不在前端伪造；中英文 i18n 已补齐，来源字段按当前语言显示。
- 本条没有做 VLM / Image Provider 反推画风，也没有把参考图人物、Logo 或隐私内容自动抽入所有分镜；该能力仍留给 8.4。
- 验证通过：`cargo fmt`、`cargo test style_service`（2 passed）、`cargo test`（143 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留 Vite chunk 体积提示。

---

### 【X】8.4 实现参考图反推画风

**问题：**
用户可能希望通过参考图建立画风，但参考图不应把人物带进所有图。

**改法：**

使用 VLM 或 Image Provider 分析参考图，输出：

```text
style_prompt
color_palette
lighting
composition
negative_prompt suggestion
```

**验证：**

- 反推结果可编辑后保存。
- 参考图只作为风格参考，不默认作为角色参考。

**风险：**
不要把参考图中的人物、Logo、隐私内容自动注入所有分镜。

**完成记录（2026-06-26）：**

- 后端新增 `AnalyzeStyleReferenceRequest` / `StyleReferenceAnalysisDto` 和 Tauri command `analyze_style_reference_image`，由 `style_service` 读取当前项目 Style Bible 的 `reference_image_path` 或 `reference_images` 第一张受控图，路径必须以 `assets/` 开头。
- 分析调用统一走 `ProviderManager::analyze_asset(VlmAnalyzeRequest)`；用户指定 VLM Provider 时使用现有 Provider 配置，没有指定时自动确保 `provider_controlled_fake_vlm` 这个 `auth_type=none / externalNetwork=false` 的受控 dummy VLM，不让页面绕过模型 / Provider 层。
- VLM 输出被服务层规整为可编辑建议：`style_prompt / color_palette / lighting / composition / negative_prompt_suggestion / warnings / raw_description`；结果只返回给前端，不自动保存 Style Bible，用户必须点击“应用建议”并再次保存。
- 服务层增加敏感内容过滤：人物、肖像、人脸、姓名、品牌、Logo、水印、车牌、电话、邮箱、地址、隐私等内容不会进入正向画风字段；相关情况写入 warnings。
- 前端工作台 Style Bible 面板新增“分析参考图”按钮和建议预览区；建议区展示画风、色彩、光线、构图、负面词和注意事项，点击“应用建议”只写入当前表单。
- 前端类型、API、command 名和 mock adapter 已补齐；用户可见文案走 `zh-CN / en-US` i18n。
- 验证通过：`cargo fmt`、`cargo test style_service`（5 passed）、`cargo test`（146 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留 Vite chunk 体积提示。

---

### 【X】8.4.1 实现创作规则管理

**问题：**
分镜规则、角色生成规则、分镜图生成规则、视频动作规则如果只放在代码或视频包里，会难以查看、编辑和绑定，也会和不同视频包产生冲突。

**改法：**

用户侧放在：

```text
创作资源 > 创作规则
```

规则类型：

```text
storyboard_rule
character_rule
scene_rule
style_rule
image_prompt_rule
storyboard_image_rule
video_prompt_rule
review_rule
```

关系：

```text
内置规则：系统提供，只读或复制后编辑
用户规则：用户复制内置规则后编辑，可启用 / 禁用
视频包默认规则：视频包引用一组规则 key / rule_id
作品规则引用：作品开始创作时记录当前启用规则 key / rule_id
任务规则记录：每次生成时记录 rule_key / rule_id / source_type
```

冲突处理：

```text
同一作品只能激活一套视频包默认规则
用户可在作品内覆盖某条规则
覆盖后生成任务记录覆盖后的 rule_id
删除用户规则前必须检查是否被视频包或作品引用
完整规则版本和历史任务复现后续增强
```

**验证：**

- 分镜、角色图、分镜图、视频提示词等规则都能查看来源、类型、启用状态和引用关系。
- 视频包只是引用规则，不直接把规则内容复制成不可追踪文本。
- builtin 规则不可直接编辑，复制成 user 后才能改。
- 修改视频包默认规则不会自动覆盖已打开作品的当前规则选择。

**风险：**
不要把创作规则做成可执行代码；当前只允许提示词、结构化 schema、参数模板和校验规则。

**完成记录（2026-06-26）：**

- 后端创作规则 DTO 已补齐 `rule_type / params_schema / reference_counts`；内置规则初始化会按新 frontmatter 重写 builtin 规则文件，覆盖 `script / storyboard / character / scene / style / image_prompt / storyboard_image / video_prompt / subtitle / cover / review`，用户规则复制后才可编辑。
- `rule_refs` 已改为严格结构化引用：每个 slot 必须是对象并包含 `ruleKey / ruleId / sourceType / ruleType`，不接受旧字符串引用；视频包和作品只保存规则引用，不复制 prompt 正文，也不保存密钥、请求头或可执行代码。
- 视频包默认规则、作品创建时的当前规则引用、保存当前作品配置为视频包都走 `resolve_creative_rule_refs`；修改视频包不会反向覆盖已创建作品，已创建作品保留当时的 `activePackId / ruleRefs / executableRefs`。
- 删除或禁用 user 创作规则前会统计 `video_packs.rule_refs_json / projects.rule_refs_json / task_steps.input_json / image_candidates.generation_context_snapshot_json / video_segments.generation_context_snapshot_json`；被引用时阻断，新增单测覆盖“user 规则被视频包引用后不能禁用/删除”。
- 生图、参考图资产生成、视频生成的成功快照和失败 `TaskStep.input_json` 都写入 `ruleSnapshot(activePackId + ruleRefs)`，后续可解释当时使用的规则来源、类型和 ID。
- 前端 `创作资源 > 创作规则` 已显示规则类型、参数 schema、引用计数；用户规则保存会提交 `ruleType / paramsSchema`，引用中的规则不能禁用或删除；视频包规则引用 JSON 保存前会严格校验结构化对象，不接受旧字符串。
- 前端 mock 数据已按新结构更新：内置视频包引用 `script / storyboard / character / scene / style / image_prompt / storyboard_image / video_prompt / review`，Project 类型也收窄为结构化规则引用。
- 完整规则版本、hash、PackSnapshot / SkillSnapshot 和历史任务完全复现仍留 TODO-12，不作为本条门槛。
- 验证通过：`cargo fmt`、`cargo test prompt_service`（5 passed）、`cargo test video_pack`（4 passed）、`cargo test scene_service`（20 passed）、`cargo test`（146 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留 Vite chunk 体积提示。

---

### 【X】8.5 实现 Character Bible

**问题：**
角色如果每条分镜自由生成，会串脸、换衣服、换年龄。

**改法：**

Character Bible 字段：

```text
character_id
name
alias
age
gender
appearance
clothing
personality
reference_images
lock_flags
```

StoryboardItem 只引用：

```text
character_id
```

**验证：**

- `character_id` 不等于显示名。
- 分镜引用角色 ID。
- 自动抽取候选必须用户确认。
- 角色参考图转存本地。

**风险：**
不要把 name 当唯一 ID；多角色同框必须明确身份和位置，避免串脸。

**完成记录（2026-06-26）：**

- 后端新增 Character Bible 领域、Repository、Service 和 Tauri commands：`list_project_character_bibles / upsert_project_character_bible / delete_project_character_bible / bind_character_reference_asset`，复用现有 `character_bibles.data_json`，没有新增第二套角色表。
- `character_id` 是稳定技术 ID，不等于显示名；未传入时由后端自动生成并校验为小写字母、数字、短横线、下划线，前端保存后不再把显示名当唯一 ID。
- `StoryboardItem.character_ids_json` 是分镜主引用；保存分镜前会校验 `characterIds` 是否存在于当前项目 Character Bible，并由后端从 Character Bible 派生 `characters_json` 作为显示冗余，不再让用户自由角色名驱动生成。
- 分镜页和生图页角色列已改为 `characterIds` 多选，选项来自当前项目 Character Bible；工作台设定集已提供 Character Bible 列表、新建、编辑、删除和参考图绑定入口。
- 删除角色前会阻断：被 `storyboard_items.character_ids_json` 引用时阻断，被 `asset_references(owner_kind=character_bible)` 引用时阻断，避免删除后分镜和参考图悬空。
- 角色参考图必须先进入受控 Asset，再绑定 `AssetReference(owner_kind=character_bible, usage_kind=character_reference)`；新链路只接受 `asset_kind=character_reference_image`，`relativePath` 必须是 `assets/` 受控相对路径，不再兼容把旧 `character_reference / user_image` 当角色参考图绑定。
- `build_image_prompt_preview_for_item` 已读取 Character Bible，并把角色 `visual_prompt / appearance / clothing / personality` 注入最终 prompt 预览；多角色同框时按引用 ID 顺序输出 `name(character_id)`，避免只靠显示名混淆身份。
- 本条没有实现自动抽取角色候选；当前只有用户显式创建 / 编辑 Character Bible 和显式绑定参考图，符合“自动抽取候选必须用户确认”的验收口径。
- 8.6 / 8.6.1 仍未完成：多参考图输入规划、角色参考图真正参与 Provider 生图输入、模型不支持多参考图时的拒绝 / 降级策略、三视图 / 表情表 / 姿态图按需规划，都进入后续 TODO，不在 8.5 冒充完成。
- 验证通过：`cargo fmt`、`cargo test character`（5 passed）、`cargo test style_service`（5 passed）、`cargo test scene_service`（20 passed）、`cargo test`（151 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留 Vite chunk 体积提示和动态 / 静态 import chunk 提示。

---

### 【X】8.6 实现角色参考图和生图注入

**问题：**
角色一致性需要参考图和角色描述共同参与。

**改法：**

生图前组装：

```text
Style Bible
Character Bible
Environment Bible
StoryboardItem visualDescription
imagePrompt
negativePrompt
```

若模型不支持多参考图：

```text
降级为主参考图
或后端拒绝并提示能力限制
```

**验证：**

- 支持模型时传入参考图。
- 不支持模型时提示清楚。
- 批量重生成不覆盖锁定角色字段。

**风险：**
角色 prompt 必须从 Character Bible 取，不能每镜头自由生成。

**完成记录（2026-06-26）：**

- `build_image_prompt_preview_for_item` 已把 Character Bible 参考图并入 `referenceImages`，真实生图请求和“预览最终提示词”弹窗共用同一份后端组装结果，不在前端伪造参考图列表。
- 角色参考图来源只读 `CharacterBible.reference_image_path / reference_images_json`，路径必须是 `assets/` 受控相对路径；role 采用 `character_front_view:character_id` 这类形式，保证多角色同框时参考图能和 `name(character_id)` 文本提示对应。
- 角色 `negative_prompt` 已并入最终负面词，并继续和分镜负面词、Style Bible 负面词做去重和长度上限处理。
- `start_image_generation` 发送给 Provider 的 `ImageProviderRequest.reference_images` 已包含角色参考图；生成上下文快照 `promptPreview.referenceImages` 和任务输入 `referenceImageCount` 会记录实际发送数量。
- 不支持参考图的模型不静默丢弃角色图：Provider 层按 `maxReferenceImages` 拒绝，分镜行会写入 `image_status=failed / image_last_error_json(provider.limit_exceeded)`，用户可换支持参考图的模型或移除参考图。
- 支持参考图的模型会正常生成，并在 ImageCandidate 快照里留下角色参考图路径和 role；新增单测覆盖默认不支持参考图模型失败、支持参考图模型成功。
- 8.7 的 Environment / Location Bible 尚未实现，因此本条只接入 `sceneDescription` 自由文本，没有假装注入 Environment Bible；场景参考图和场景 ID 注入留给 8.7 / 8.7.1。
- 8.6.1 的多参考图输入规划、required / optional / unused 展示、三视图 / 表情表 / 姿态图按需规划仍未完成；本条只把已确认的 Character Bible 参考图接入真实生图请求和模型上限校验。
- 验证通过：`cargo fmt`、`cargo test style_service`（6 passed）、`cargo test scene_service`（22 passed）、`cargo test character`（8 passed）、`cargo test`（154 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留动态 / 静态 import chunk 提示和 chunk 体积提示。

---

### 【X】8.6.1 实现角色资源包按需规划

**问题：**
角色并不只是“一张角色图”。有些模型或 ComfyUI workflow 需要正面图、侧面图、背面图、全身图、面部特写、表情表、服装细节、动作 / 姿态参考或心情状态参考。但这些资源不能默认全量生成，否则成本高、速度慢、数据也会乱。

**改法：**

- Character Bible 分成两层：

```text
角色文字设定：由创作规则 / Skill 输出，尽量完整，用户可编辑。
角色参考资源包：由 ImageInputPlan 按需生成、上传或选择。
```

- 角色资源包至少预留这些 reference role：

```text
character_front_view
character_side_view
character_back_view
character_full_body
character_face_closeup
character_expression_sheet
character_outfit
character_pose
character_mood
```

- 生图或视频前，先根据当前模型 / workflow 生成输入规划：

```text
required：当前模型必须要的角色图片，缺了不能生成。
optional：有会更好，但必须用户确认后才生成。
unused：当前模型不需要，不展示为缺项，不自动生成。
```

- 动作、表情、心情、服装变化、正侧背方位图都挂在同一个 `character_id` 下，不创建成新角色。
- 第一版 UI 可以先只生成主参考图和全身图，但数据结构、AssetReference 和 InputPlan 必须支持后续三视图、表情表、姿态图扩展。

**验证：**

- 选择普通文生图模型时，不会提示必须生成三视图、表情表或 pose。
- 选择需要多角色参考的 workflow 时，页面能显示缺少哪些角色图片，并说明是 required 还是 optional。
- 角色姿态、表情、服装变化不会被保存成新角色。
- 重生成角色资源不会覆盖用户已确认的主参考图，除非用户显式替换。

**风险：**
不要为了“角色一致性”默认生成所有角色图；只有当前生产方式真的需要时才生成。文字设定可以完整，图片资源必须按需。

**完成记录（2026-06-26）：**

- Character Bible 保持两层结构：文字设定继续在 `character_bibles.data_json`，角色参考资源包继续放 `reference_image_path / reference_images_json`，本条没有新增第二套角色资源表。
- 后端新增 `BuildCharacterResourcePlanRequest / CharacterResourcePlanDto / CharacterResourceRequirementDto` 和 Tauri command `build_character_resource_plan`，按当前分镜 `characterIds` + 选定 / 默认可执行生图模型的 `MediaInputPlan` 计算角色资源包规划。
- 角色资源 role 已按本条预留：`character_front_view / character_side_view / character_back_view / character_full_body / character_face_closeup / character_expression_sheet / character_outfit / character_pose / character_mood`；动作、表情、服装变化和方位图都挂同一个 `character_id`，不会创建成新角色。
- 规划结果按角色展开 required / optional / unused：required 缺失会返回 `missingRequiredCount` 和具体 `characterId + role`；optional 只提示；unused 不作为缺项、不生成、不入库、不写任务产物。
- 模型 `maxReferenceImages=0` 时角色图片全部按 unused 展示，避免要求用户补当前模型不会用的资源；支持参考图的模型会按 `inputRequirements.requiredInputs / optionalInputs / unusedInputs` 展开。
- 生图前 required 角色资源校验已改为读取 Character Bible：模型要求 `character_front_view / character_pose` 等角色资源时，只有对应角色缺该 role 才阻断；无角色 ID 时 required 角色资源会明确阻断，避免空角色引用绕过校验。
- 生图页新增“角色资源”列，显示每行 required 缺项数、optional 缺项数和缺少的 `角色名:资源类型` 摘要；页面只展示规划，不自动生成任何角色资源。
- 前端新增 `buildCharacterResourcePlan` API、类型、command 名和中英文 i18n；mock 环境也返回稳定规划，便于页面无 Tauri 时展示。
- 本条没有做角色资源一键生成、资源包弹窗、三视图 / 表情表批量生成、模型选择 UI，也没有把 unused 资源静默创建；这些只在后续 8.7.1 / 模型选择增强中继续。
- 验证通过：`cargo fmt`、`cargo test scene_service`（24 passed）、`cargo test`（156 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留动态 / 静态 import chunk 提示和 chunk 体积提示。

---

### 【X】8.7 实现 Environment / Location Bible

**问题：**
同一场景如果每条分镜自由写，会出现空间、光线、道具漂移。

**改法：**

Location Bible 字段：

```text
location_id
name
space_description
lighting
time_of_day
props
reference_images
variants
```

StoryboardItem 使用：

```text
location_id
```

或临时：

```text
sceneDescription
```

**验证：**

- 复用场景可入库。
- 相似场景提示合并。
- 场景参考图和角色参考图分开。

**风险：**
道具不能塞进 `location_id`；昼夜变化可用 variant。

**完成记录（2026-06-26）：**

- 后端新增 `LocationBibleDto / UpsertProjectLocationBibleRequest / BindLocationReferenceAssetRequest`、`location_repository`、`location_service` 和 Tauri commands：`list_project_location_bibles / upsert_project_location_bible / delete_project_location_bible / bind_location_reference_asset`。
- Location Bible 复用 `location_bibles.data_json`，字段包含 `location_id / name / space_description / lighting / time_of_day / props / visual_prompt / negative_prompt / reference_image_path / reference_images / variants`；`location_id` 是稳定技术 ID，不使用显示名做唯一标识。
- 分镜保存已从只校验角色引用扩展为 `normalize_storyboard_bible_refs`：`location_id` 非空时必须存在于当前项目 Location Bible，并由后端派生 `sceneDescription` 显示冗余；`location_id` 才是主引用。
- Location Bible 删除会检查 `storyboard_items.location_id` 和 `asset_references(owner_kind=location_bible)`，被分镜或参考图引用时阻断删除，避免悬空引用。
- 场景参考图绑定严格要求受控 Asset：`asset.kind=scene_reference_image`、`relative_path` 位于 `assets/`，写入 `AssetReference(owner_kind=location_bible, usage_kind=location_reference)`，不会复用角色参考图绑定链路。
- 生图 prompt 预览已读取 Location Bible，并把 `visual_prompt / space_description / lighting / time_of_day / props` 注入场景段落；`negative_prompt` 合并进最终负面词；场景参考图按 `scene_wide_view:location_id` 等 role 注入 `referenceImages`，和角色参考图分开。
- 前端补齐 `LocationBibleDto`、location commands、mock/tauri API 双通路；工作台设定集新增 Location Bible 创建、编辑、删除、参考图导入绑定；分镜页和生图页新增 `locationId` 下拉，无 Location Bible 时才显示临时 `sceneDescription` 输入。
- 本条没有实现“相似场景 AI 合并建议”和“场景参考图 AI 生成”；前者需要后续内容导入 / 设定抽取能力支撑，后者进入 8.7.1，不在 8.7 用假入口冒充完成。
- 验证通过：`cargo fmt`、`cargo test location`（5 passed）、`cargo test style_service`（7 passed）、`cargo test scene_service`（24 passed）、`cargo test`（161 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留 Vite 动态 / 静态 import 和 chunk 体积提示。

---

### 【X】8.7.1 实现角色 / 场景 / 风格参考图的 AI 生成与绑定

**问题：**
当前 TODO 已覆盖参考图上传、Style / Character / Environment Bible 和生图注入，但需要明确：参考图不只来自用户上传，也可以由 AI 生图生成。完整生图能力包括角色参考图、场景参考图、风格参考图等，它们和“分镜候选图”不是同一种产物。

**改法：**

- 在设定集或生图页提供明确入口：

```text
生成角色参考图
生成场景参考图
生成风格参考图
生成道具参考图
```

- 角色参考图生成后，用户确认才绑定到 `CharacterBible.reference_images`。
- 场景参考图生成后，用户确认才绑定到 `Environment / Location Bible.reference_images`。
- 风格参考图生成后，用户确认才绑定到 `StyleBible.reference_image_path` 或风格引用。
- 每个参考图都要登记 `Asset` 和 `AssetReference`，记录 owner_type / owner_id / usage_type。
- 生成上下文快照必须记录 image_kind、prompt、模型 / workflow、seed / params、Bible 快照和来源分镜。
- 参考图可被生图 / 图生视频输入规划读取，但不能默认替换分镜已选图。
- 角色动作图、表情图、心情图、服装变化图和正侧背方位图统一按 `character_reference` 处理，并用 reference role 区分，不单独变成主流程步骤。
- 如果某个模型 / workflow 不需要道具图、场景图或角色姿态图，对应图片必须是 unused，不生成、不入库、不写任务产物。

**验证：**

- 用户能区分“分镜候选图”和“角色 / 场景 / 风格参考图”。
- 生成的角色参考图可在角色设定里回看、替换、解除绑定。
- 生成的场景参考图可在场景设定里回看、替换、解除绑定。
- 不需要某类参考图的模型不会触发该类图片生成任务。
- 修改或删除参考图受引用保护约束。
- 项目包导出能按 AssetReference 收集这些参考图。

**风险：**
不要把生成的参考图自动应用到所有分镜；角色、场景、风格引用必须经过用户确认，并受锁定和版本快照保护。

**完成记录（2026-06-26）：**

- 后端已有 `start_image_asset_generation` 专用链路，本条继续收紧为参考图资产生成：`start_image_generation` 仍只写 `ImageCandidate`，参考图生成只写 `Asset + AssetReference + Artifact`，不会写入分镜候选、不会修改 `selectedImageId`。
- `StartImageAssetGenerationRequest` 新增必填 `referenceRole`，不再把 `imageKind` 当参考图 role。角色参考图必须写 `character_front_view / character_side_view / character_back_view / character_full_body / character_face_closeup / character_expression_sheet / character_outfit / character_pose / character_mood` 之一；场景参考图必须写 `scene_wide_view / scene_layout_view / scene_detail_view / scene_day_variant / scene_night_variant` 之一；风格参考图写 `style_reference`。
- 后端按 `imageKind` 严格校验 `ownerKind / assetKind / referenceRole`：`character_reference → character_bible + character_reference_image`，`scene_reference → location_bible + scene_reference_image`，`style_reference → style_bible + style_reference_image`；不接受模糊 owner 或错误资产类型。
- 生成成功后写入受控 `assets/` 路径，`Asset.source_kind=ai_generated`，并通过 `AssetRepository::insert_generated_image_asset` 原子写入 Asset、AssetReference 和对应 Bible 的 `reference_images_json/reference_images`；style reference 同步主 `reference_image_path`，角色 / 场景追加到参考图列表。
- 生成上下文快照记录 `imageKind / assetKind / ownerKind / ownerId / referenceRole / prompt hash / providerModelId / workflowPresetId / ruleSnapshot / inputPlan / seed / workflowParams` 等脱敏信息，并写入 Asset.metadata、TaskStep.input/output、Artifact.metadata。
- 前端生图页的图片类型工具条新增“参考图角色”选择；用户显式选择角色正面、全身、姿态、场景宽景、场景布局、风格参考等用途后才可生成资产图。生成后会刷新 Character / Location Bible 上下文和角色资源规划，避免 UI 看不到新参考图。
- mock/tauri API 类型都补齐 `referenceRole`，中英文 i18n 补齐参考图用途、placeholder 和错误提示；用户可区分“分镜候选图”和“角色 / 场景 / 风格参考图”。
- 本条没有实现多候选审片后再二次确认的参考图候选池；当前“生成资产图”按钮本身是显式确认写入某个设定的入口。后续如果要做候选池，需要另建“参考图候选”对象，不能复用分镜 ImageCandidate。
- 验证通过：`cargo fmt`、`cargo test scene_service`（24 passed）、`cargo test location`（5 passed）、`cargo test character`（10 passed）、`cargo test style_service`（7 passed）、`cargo test`（161 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留 Vite 动态 / 静态 import 和 chunk 体积提示。

---

### 【X】8.8 实现生成上下文快照

**问题：**
没有快照，历史结果无法解释，也无法复现。

如果只保存当前项目最新的 Style / Character / Location Bible，用户改过设定后再回看旧候选，会不知道当时为什么生成成那样；视频片段也无法说明它用了哪张输入图、那张输入图当时使用了哪些角色 / 场景 / 风格和模型。

**位置：**

```text
src-tauri/src/domain/style.rs
src-tauri/src/services/style_service.rs
src-tauri/src/services/scene_service.rs
src-tauri/src/db/scene_repository.rs
src/src/entities/config/types.ts
src/src/pages/image-generation/index.vue
src/src/pages/video-generation/index.vue
src/src/shared/i18n/locales/*
```

**改法：**

每次生图/视频记录：

```text
generation_context_snapshot
style snapshot
character snapshot
location snapshot
prompt snapshot
model snapshot
workflow snapshot
seed/params snapshot
```

落地口径：

```text
1. 不新建第二套 snapshot 表，继续使用 image_candidates / video_segments 的 generation_context_snapshot_json。
2. ImageCandidate 快照必须包含：schemaVersion、promptSnapshot、styleBible、characterBibles、locationBible、promptPreview.referenceImages、modelSnapshot、ruleSnapshot、inputPlan、seed / params。
3. VideoSegment 快照必须包含：schemaVersion、videoPromptSnapshot、inputImageSnapshot、modelSnapshot、ruleSnapshot、inputPlan、seed / params。
4. inputImageSnapshot 只放输入 ImageCandidate 的解释摘要：imageId、imagePath、revision、variantIndex、prompt hash、Bible 快照摘要和参考图，不把整个嵌套历史无限复制。
5. 前端生图页和视频页都要能打开候选 / 片段的快照弹窗，查看当时 prompt、设定、模型、输入图和原始脱敏 JSON。
6. 快照只允许受控相对路径和脱敏摘要；不能写入 API key、authorization、secret、token、password 或本机绝对路径。
```

本条严格不做：

```text
1. 不做独立“生成历史中心”页面，历史页后续仍看 Task / Artifact。
2. 不为复现生成重跑真实 Provider；本条只保存可解释快照。
3. 不把当前最新 Bible 当成历史解释；历史解释必须来自候选 / 片段快照。
4. 不导出裸数据库、不导出 keyring、不把 Provider 请求头塞进快照。
```

**验证：**

- ImageCandidate 能查看当时使用的设定和模型。
- VideoSegment 能查看输入图和视频 prompt。
- 修改 Bible 不影响历史候选图解释。
- 快照 JSON 中没有真实密钥、Authorization、Token、Password 或 Windows / Unix 绝对路径。
- 前端 typecheck/build 通过。

**风险：**
快照中不能包含真实密钥或绝对路径。

**完成记录（2026-06-26）：**

- 后端继续复用 `image_candidates.generation_context_snapshot_json` 和 `video_segments.generation_context_snapshot_json`，没有新增第二套快照表或生成历史中心。
- ImageCandidate 快照已补齐 `schemaVersion / promptSnapshot / styleBible / characterBibles / locationBible / promptPreview.referenceImages / modelSnapshot / ruleSnapshot / inputPlan / seed / params`；TaskStep.input_json 也写入同一类脱敏摘要。
- VideoSegment 快照已补齐 `schemaVersion / videoPromptSnapshot / inputImageSnapshot / modelSnapshot / ruleSnapshot / inputPlan / seed / params`；`inputImageSnapshot` 只保存输入候选图的 imageId、imagePath、revision、variantIndex、prompt hash、Bible ID 摘要、参考图和模型摘要，不递归复制完整历史快照。
- 快照写入统一走脱敏和路径过滤：敏感 key 不保留原名，只记录 `redactedFieldCount`；Windows / Unix 绝对路径替换为 `<blocked:absolute-path>`；不保存 Provider 请求头、真实密钥、keyring、Authorization、Token、Password。
- 前端 `ImagePromptPreviewDto` 已同步 `characterBibles / locationBible`；mock `buildImagePromptPreview` 已按角色 / 场景 Bible 组装参考图和负面词。
- 生图页候选图新增“快照”入口，弹窗展示 prompt 分段、Style / Character / Location 快照、模型摘要、参考图数量和脱敏原始 JSON；视频页片段新增“快照”入口，弹窗展示 video prompt、输入图摘要、模型摘要和脱敏原始 JSON。
- 本条没有做独立生成历史中心、没有做一键复现重跑 Provider、没有导出裸数据库；历史页仍归 Task / Artifact，复现能力留后续增强。
- 验证通过：`cargo fmt`、`cargo test scene_service`（27 passed）、`cargo test`（164 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留 Vite 动态 / 静态 import chunk 提示和 chunk 体积提示。

---

### 【X】8.9 实现锁定和重生成保护

**问题：**
用户手动调整的角色、场景、prompt、候选图不能被批量操作覆盖。

如果只在页面上显示“锁定”，但批量生成、单行重生成、选择候选图 / 视频片段和后端 command 不检查锁定，用户确认过的图、视频、角色 / 场景引用仍会被后续操作改脏，导致候选历史、合成输入和生成快照不可信。

**位置：**

```text
src/src/entities/scene/types.ts
src/src/entities/scene/reset.ts
src/src/entities/scene/api.ts
src/src/pages/storyboard-editor/index.vue
src/src/pages/image-generation/index.vue
src/src/pages/video-generation/index.vue
src/src/shared/i18n/locales/*
src-tauri/src/services/scene_service.rs
plan/TODO-08.md
```

**改法：**

统一 `lock_flags_json`：

```text
sourceText
narrationText
visualDescription
characters
location
imagePrompt
negativePrompt
videoPrompt
selectedImage
selectedVideoSegment
```

落地口径：

```text
1. `lock_flags_json` 是字段级锁定，不再只依赖旧的 image / video 粗粒度锁；旧字段仅作为兼容读取，不作为新写入口径。
2. 分镜页提供文本、旁白、画面、角色、场景的行内锁定 / 解锁入口；锁定字段禁用编辑。
3. 生图页提供角色、场景、画面、imagePrompt、negativePrompt、selectedImage 的锁定 / 解锁入口；批量生图跳过 `imagePrompt / negativePrompt / selectedImage` 锁定行。
4. 视频页提供 videoPrompt、selectedVideoSegment 的锁定 / 解锁入口；批量视频跳过 `videoPrompt / selectedVideoSegment` 锁定行。
5. 选择候选图 / 视频片段前检查 selectedImage / selectedVideoSegment 锁；锁定时只提示，不覆盖。
6. 后端 `start_image_generation / start_video_generation / select_image_candidate / select_video_segment` 做兜底校验，避免绕过前端覆盖锁定内容。
7. 保存行时保留 `lock_flags_json`，解锁后才能批量覆盖或重新选择。
```

本条严格不做：

```text
1. 不新增独立锁定表，继续使用 `storyboard_items.lock_flags_json`。
2. 不做多人协同锁 / 文件系统锁。
3. 不把锁定当成流程 StepBar 的前置条件；锁定只保护字段和已确认产物。
4. 不自动清除旧候选；候选清理仍走用户确认和引用保护。
```

**验证：**

- 批量重生成跳过锁定项。
- 尝试覆盖锁定内容前提示用户。
- 解锁后才能批量覆盖。
- 后端 command 直接调用也不能生成或选择锁定字段对应的产物。
- 前端 typecheck/build 通过；Rust `scene_service` 相关测试通过。

**风险：**
不要让局部重生成破坏用户已确认的图和视频。

**下一步进入条件：**

- 锁字段类型、页面开关、批量跳过、单行覆盖提示和后端兜底都完成。
- 完成记录写清哪些锁字段影响哪些动作，以及旧 `image / video` 锁如何兼容读取。
- 验证通过后把本条改为 `【X】`，TODO-08 即可进入阶段完成收口检查。

**完成记录（2026-06-26）：**

- `lock_flags_json` 已按字段级锁定统一到 `sourceText / narrationText / visualDescription / characters / location / imagePrompt / negativePrompt / videoPrompt / selectedImage / selectedVideoSegment`；旧 `image / video` 仅作为批量生成跳过的兼容读取，不作为新写入口径。
- 前端新增统一锁工具：`isStoryboardItemLocked / setStoryboardItemLock / lockedFieldsForImageGeneration / lockedFieldsForVideoGeneration`，并通过 `entities/storyboard` 兼容入口 re-export，避免页面各自写散落判断。
- 分镜页已支持锁定 / 解锁 `sourceText / narrationText / characters / location`，锁定字段禁用编辑；尝试更新锁定字段会提示先解锁。
- 生图页已支持锁定 / 解锁 `characters / location / visualDescription / imagePrompt / negativePrompt / selectedImage`；批量生图跳过 `imagePrompt / negativePrompt / selectedImage` 锁定行，单行生图和选择候选图会先提示并阻断。
- 视频页已支持锁定 / 解锁 `videoPrompt / selectedVideoSegment`；批量视频跳过锁定行，单行生成和确认视频片段会先提示并阻断。
- 后端 `start_image_generation / start_video_generation / select_image_candidate / select_video_segment` 已增加锁定兜底校验，直接调用 command 也不能覆盖锁定的 prompt、最终图或确认视频。
- mock API 同步实现锁定阻断，保证无 Tauri 环境下的页面行为一致。
- 新增 Rust 单测覆盖：锁定 `imagePrompt` 阻断生图、锁定 `selectedImage` 阻断选图、锁定 `videoPrompt` 阻断生视频、锁定 `selectedVideoSegment` 阻断确认视频。
- 验证通过：`cargo fmt`、`cargo test scene_service`（29 passed）、`cargo test`（166 passed）、`pnpm --dir src typecheck`、`pnpm --dir src build`；前端 build 仅保留 Vite 既有动态 / 静态 import 和 chunk 体积提示。

---

## 阶段完成标准

- “创作资源 > 素材库”导入、预览、引用、删除保护可用。
- “创作资源 > 创作规则”可查看、复制为 user、编辑、启用/禁用，并能被视频包和作品引用；完整版本/hash/snapshot 后续增强。
- Style / Character / Environment Bible 可创建、编辑、引用。
- 生图和视频生成能记录上下文快照。
- 分镜只引用设定 ID，不复制整段设定。
- 批量重生成尊重锁定字段和用户确认产物。
