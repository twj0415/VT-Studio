# 创作规则与模板规范

> 这篇定义“创作规则”的存放、编辑、结构化输出、模型绑定，以及后续版本 / 快照能力的边界。
> 用户界面统一叫“创作规则”。技术实现可以继续使用 `SkillDefinition` 等命名，但不要把 `Skill` 作为普通用户可见菜单名。
> 当前 MVP 不强制 `SkillVersion`、`content_hash`、`schema_hash`、`PackSnapshot`、`SkillSnapshot`，这些属于后续增强。

---

## 一、核心原则

```text
1. 创作规则不写死在页面组件里。
2. 核心生成必须有输出 schema。
3. MVP 只区分 builtin 内置规则和 user 用户规则。
4. 内置规则不可直接修改，用户编辑时复制为 user 规则。
5. 创作规则只能生成建议、提示词或结构化输出，不能直接写数据库。
6. 视频包只引用创作规则，不直接编辑规则内容。
7. 作品工作台只显示当前规则名称、来源和跳转入口，不把完整版本系统作为第一版门槛。
```

---

## 二、用户侧分类

创作资源里的“创作规则”按用户可理解的类型分组：

```text
文案规则
分镜规则
设定集规则
生图规则
视频规则
审核 / 安全规则
```

说明：

```text
文案规则：一句话视频想法、已有文案整理、文章提炼、小说剧情改编。
分镜规则：分镜拆分、镜头节奏、画面描述、角色/场景引用、时长分配。
设定集规则：角色、场景、道具、画风抽取，生成 Bible 草稿。
生图规则：分镜图、角色图、场景图、风格图、尾帧图、控制图的提示词规则。
视频规则：普通图生视频、首尾帧视频、参考图视频、工作流视频的动作描述和视频提示词规则。
审核 / 安全规则：儿童内容安全、品牌安全、平台表达中性化、缺项检查、质量检查。
```

---

## 三、目录结构

```text
workspace/creative-rules/
  builtin/
    script/
    storyboard/
    bible/
    image_prompt/
    video_prompt/
    review/
  user/
    script/
    storyboard/
    bible/
    image_prompt/
    video_prompt/
    review/
```

技术文件可以继续使用 Skill 文件格式：

```text
skills/
  script/topic_narration.md
  script/paste_cleanup.md
  storyboard/children_education.md
  storyboard/knowledge.md
  bible/character_extract.md
  bible/environment_extract.md
  image_prompt/shot_frame.md
  image_prompt/character_reference.md
  video_prompt/image_to_video.md
  review/children_safety.md
```

---

## 四、frontmatter

```md
---
key: script.topic_narration
display_name: 一句话视频想法生成文案
category: script
version: 1.0.0
provider_kind: llm
output_schema: script_draft.schema.json
description: 根据一句话视频想法生成短视频脚本草稿
---

正文 prompt...
```

必填：

```text
key
display_name
category
provider_kind
output_schema
description
```

建议字段：

```text
version
applicable_pack_ids
recommended_model_ability
input_schema
test_cases
```

---

## 五、变量规范

变量格式：

```text
{{project.title}}
{{project.content_language}}
{{pack.tone}}
{{pipeline.target_scene_count}}
{{storyboard_item.narration_text}}
{{bible.style_prompt}}
{{character.reference_images}}
{{model.requirements}}
```

规则：

```text
1. 变量必须来自受控上下文。
2. 缺变量时报 prompt.variable_missing。
3. 用户文本进入 prompt 前可做长度限制和脱敏摘要。
4. 规则不能读取任意系统路径。
5. 规则不能自己决定调用哪个 Provider，只能声明推荐能力。
```

---

## 六、结构化输出

核心输出必须 schema 校验：

```text
ScriptDraft
CleanNarrationList
StoryboardItem[]
StyleBibleDraft
CharacterBibleDraft
EnvironmentBibleDraft
ImagePromptList
VideoPromptList
ReviewReport
```

流程：

```text
LLM 输出
→ JSON parse
→ schema validate
→ repair loop 最多 2 次
→ 仍失败进入人工修正或 step failed
```

### 6.1 设定集字段由创作规则约束

角色、场景、风格、道具等文字设定不能写死在 Vue 页面或 Rust 代码里。它们由当前启用的设定集规则输出，并由 `output_schema` 校验。

例如角色抽取规则可以要求输出：

```text
name
role
gender
age
height
body
face
hair
clothing
signature_items
personality
emotion_baseline
relationship
story_function
visual_prompt
negative_prompt
```

如果用户后期希望角色信息更细，可以编辑或复制内置规则，扩展为：

```text
front_view_description
side_view_description
back_view_description
face_closeup_description
expression_set
action_pose_set
outfit_variants
```

规则：

```text
1. 字段要求来自当前 Skill 的 input_schema / output_schema。
2. 页面只按 schema 渲染和校验，不私自发明必填字段。
3. 修改用户规则只影响新任务。
4. MVP 阶段不要求历史任务完整复现旧规则正文；历史主要保留当时的生成产物、任务输入输出和错误信息。
5. 完整 SkillSnapshot、hash、规则版本复现属于后续增强。
6. 规则可以生成结构化 JSON，但不能直接写数据库。
```

---

## 七、视频包绑定规则

视频包是内容策略组合，创作规则是具体生成方法。

```text
视频包可以绑定：
  默认文案规则
  默认分镜规则
  默认设定集规则
  默认生图规则
  默认视频规则
  默认审核 / 安全规则
  推荐模型能力
```

### 7.1 分层职责

后续实现必须按这五层施工，不能把它们合并成一个大配置：

| 层 | 用户侧名称 | 技术落点 | 管什么 | 不管什么 |
|---|---|---|---|---|
| 视频包 | 视频包 | packs / project pack config | 内容类型、默认语气、结构、画幅、时长、分镜数、规则引用、推荐模型 / workflow、默认素材引用 | 不保存 prompt 正文、不保存 API Key、不保存 Provider 连接 |
| 创作规则 | 创作规则 | creative_rules / SkillDefinition | prompt_body、output_schema、params_schema、校验规则、测试样例 | 不直接写业务表、不调用 Provider、不执行代码 |
| Provider 连接 | Provider | providers | vendor、base_url、auth_type、key_alias、enabled | 不保存 modelName、不保存 workflow 参数 |
| API 模型能力 | API 模型能力 | provider_models | modelName、abilityTypes、input/output modalities、limits、featureFlags | 不登记 workflow、不保存密钥 |
| Workflow Preset | Workflow Preset | workflow_presets | workflowKey / workflowId、paramSchema、nodeMap、outputMap、limits | 不混进 provider_models、不保存 key |

### 7.2 视频包建议字段

视频包保存的是“引用和默认参数”，不是一份可执行脚本。

```json
{
  "pack_id": "pack_knowledge_short",
  "source_type": "builtin",
  "name": "知识科普短视频",
  "description": "适合 30-90 秒竖屏知识内容",
  "applicable_input_types": ["topic", "paste", "article"],
  "default_aspect_ratio": "vertical_9_16",
  "default_duration_seconds": 60,
  "default_scene_count": 8,
  "default_tone": "清楚、克制、有信息量",
  "rule_refs": {
    "script": "script.topic_narration",
    "storyboard": "storyboard.knowledge_default",
    "image_prompt": "image_prompt.shot_frame",
    "video_prompt": "video_prompt.image_to_video",
    "review": "review.default_safety"
  },
  "recommended_executable_refs": {
    "llm": { "provider_model_id": "pm_llm_default" },
    "image": { "provider_model_id": "pm_image_default" },
    "video": { "workflow_preset_id": "wf_i2v_default" }
  },
  "asset_refs": []
}
```

规则：

```text
1. rule_refs 只存 rule_key / rule_id。
2. recommended_executable_refs 只存 providerModelId 或 workflowPresetId。
3. asset_refs 只存素材库 Asset / AssetReference。
4. 创建作品时复制“引用和默认参数”到项目当前配置，不复制规则正文。
5. 用户在作品内覆盖规则或模型时，只影响当前作品，不反向修改视频包。
6. “保存当前作品配置为新视频包”只能保存引用、默认参数和素材引用，不能保存真实密钥、任务产物目录或完整 prompt 历史。
```

### 7.3 页面布局

```text
视频包页：
  左侧：包列表，支持内置 / 用户 / 已禁用筛选
  中间：基本信息，名称、说明、适用输入、默认画幅、时长、分镜数、语气
  右侧：绑定规则、推荐模型 / workflow、素材引用、引用关系

创作规则页：
  左侧：规则分类
  中间：规则列表
  右侧：规则正文、schema、参数、测试输出、引用关系

模型 / 工作流页：
  Provider、API 模型能力、Workflow Preset、测试记录四个 Tab
```

页面必须能让开发者和用户看清：

```text
这个视频包用了哪些创作规则
推荐哪些可执行模型 / workflow
引用哪些素材
被哪些作品使用
哪些配置只在当前作品覆盖
```

约束：

```text
1. 视频包只保存规则引用和推荐配置，不复制规则正文。
2. 编辑视频包不等于编辑规则。
3. 编辑规则不直接改动已生成产物。
4. MVP 开始任务时记录当前 pack_id、rule_key、provider/model/workflow 选择和关键输入输出。
5. PackSnapshot + SkillSnapshot 后续增强，不作为第一版强制门槛。
```

### 7.4 实现时的引用链检查表

如果后续任务写“实现视频包 / 创作规则 / 模型工作流分层”，必须逐项落到下面这些表、字段和页面区域，不能自由发挥成一个大 JSON。

```text
VideoPack 只保存：
  pack_id
  source_type
  name / description
  applicable_input_types_json
  content_category
  default_tone
  default_aspect_ratio
  default_duration_seconds
  default_scene_count
  rule_refs_json
  recommended_executable_refs_json
  asset_refs_json
  enabled

CreativeRule 只保存：
  rule_id
  rule_key
  source_type
  module
  name / description
  prompt_body
  output_schema
  params_schema
  enabled

Provider 只保存：
  provider_id
  provider_kind
  vendor
  display_name
  base_url
  auth_type
  key_alias
  enabled

ProviderModel 只保存：
  provider_model_id
  provider_id
  model_name
  ability_types
  input_modalities
  output_modalities
  limits_json
  feature_flags_json
  enabled

WorkflowPreset 只保存：
  workflow_preset_id
  provider_id
  workflow_key
  workflow_id
  workflow_version
  ability_types
  param_schema
  node_map
  output_map
  limits_json
  enabled

ProjectRuntimeConfig / Project 当前配置只保存：
  active_pack_id
  rule_refs_json
  executable_refs_json
  用户在作品内覆盖后的参数
```

页面怎么显示：

```text
视频包页：
  左侧包列表：builtin / user / disabled、适用 inputType、引用计数。
  中间基本信息：名称、说明、适用输入、默认画幅、时长、分镜数、语气。
  右侧引用：规则引用、推荐 providerModelId / workflowPresetId、素材引用、被哪些作品使用。

创作规则页：
  左侧分类：文案、分镜、设定集、生图、视频、审核。
  中间列表：rule_key、名称、来源、启用状态、最近测试结果、引用数量。
  右侧编辑：prompt_body、output_schema、params_schema、测试样例、引用关系。

模型 / 工作流页：
  Provider 连接 Tab 只管连接和 key_alias。
  API 模型能力 Tab 只管 provider_models。
  Workflow Preset Tab 只管 workflow_presets。
  测试记录 Tab 只管 dry_run / real_generate 脱敏结果。
```

创建作品时的数据流：

```text
1. 用户在内容创作页选择 VideoPack。
2. 后端读取 VideoPack 的默认参数和三类引用。
3. 写入 Project.active_pack_id、Project.rule_refs_json、Project.executable_refs_json 和基础参数。
4. 用户在作品内改规则或模型，只改 Project 当前覆盖。
5. 任务创建时，把 pack_id、rule_key / rule_id、providerModelId / workflowPresetId、关键参数摘要写入 TaskStep.input_json 或 generation_context_snapshot。
```

保存当前作品配置为新视频包：

```text
允许保存：
  当前规则引用
  推荐 providerModelId / workflowPresetId
  默认画幅 / 时长 / 分镜数 / 语气
  用户显式选择的可复用素材引用

禁止保存：
  API Key
  Provider 完整连接细节
  Task / Artifact / ImageCandidate / VideoSegment 产物目录
  完整 prompt 历史
  Provider 请求头
  本地绝对路径
```

删除保护：

```text
删除 user 视频包：
  检查 Project.active_pack_id。
  检查 Project.rule_refs_json / executable_refs_json 的历史引用。
  检查未完成 TaskStep 快照。

删除 user 创作规则：
  检查 video_packs.rule_refs_json。
  检查 Project.rule_refs_json。
  检查 TaskStep.input_json / generation_context_snapshot。

删除 Provider / ProviderModel / WorkflowPreset：
  检查 executable_refs_json。
  检查未完成任务。
  历史任务保留脱敏快照，不因连接删除而无法查看。
```

---

## 八、模型绑定

创作规则可声明推荐能力：

```json
{
  "required_capabilities": ["text_generation", "json_mode"],
  "preferred_model_abilities": ["llm_json"]
}
```

规则：

```text
1. 绑定只做默认选择，不跳过 Provider 能力校验。
2. 用户可在模型 / 工作流里配置实际 Provider 和模型能力。
3. 运行中的任务使用创建时的 Provider / 模型 / workflow 配置，避免执行中被设置变更打断。
4. 创作规则完整 snapshot 后续增强。
```

---

## 九、内置与用户自定义

```text
builtin：应用内置，不直接修改。
user：用户自定义，可启用/禁用。
```

覆盖规则：

```text
1. 默认使用 builtin 最新稳定版。
2. 用户编辑内置规则时，复制为用户规则。
3. 用户启用自定义规则时，只影响新任务。
4. 删除用户规则前必须检查视频包和作品引用。
5. 删除用户规则不删除已生成产物；历史任务完整复现依赖后续 SkillSnapshot 增强。
```

---

## 十、权限边界

创作规则可做：

```text
读取受控上下文
生成结构化 JSON
生成 prompt 文本
生成 review report
声明推荐模型能力
```

创作规则不可做：

```text
直接写数据库
直接读系统文件
直接调用 Provider
直接删除资产
直接改任务状态
执行任意 JS / TS / Python 代码
```

---

## 十一、任务记录边界

MVP 任务至少记录：

```json
{
  "pack_id": "children_education",
  "creative_rule_keys": ["script.topic_narration", "storyboard.default"],
  "provider_model_id": "model_xxx",
  "workflow_preset_id": null,
  "input_summary": {},
  "output_summary": {}
}
```

更具体地说，每次内容生成、分镜、生图、视频、审核任务都要记录脱敏上下文：

```json
{
  "pack_id": "pack_knowledge_short",
  "rule_refs": {
    "storyboard": "storyboard.knowledge_default",
    "image_prompt": "image_prompt.shot_frame"
  },
  "provider_model_id": "pm_image_default",
  "workflow_preset_id": null,
  "model_constraints_summary": {
    "ability_type": "text_to_image",
    "aspect_ratio": "vertical_9_16",
    "max_reference_images": 2
  },
  "user_overrides": {
    "image_prompt_locked_count": 3,
    "custom_duration_seconds": 4
  },
  "input_summary": {},
  "output_summary": {}
}
```

不得记录：

```text
API Key
完整 Provider 请求头
本地绝对路径
大段未脱敏 Provider 原始响应
完整未脱敏用户隐私文本
```

后续增强再补：

```text
SkillVersion
content_hash
schema_hash
PackSnapshot
SkillSnapshot
历史任务完全复现
模板市场级版本管理
```

---

## 十二、禁止事项

```text
1. 禁止在 Vue 页面里写大段 prompt。
2. 禁止在 Rust service 里散落不可追踪 prompt。
3. 禁止核心生成只靠 Markdown/XML 解析。
4. 禁止把后续版本 / 快照能力做成 MVP 硬门槛。
5. 禁止把创作规则做成可执行插件。
6. 禁止让视频包、模型配置、素材库各自藏一份重复规则。
```
