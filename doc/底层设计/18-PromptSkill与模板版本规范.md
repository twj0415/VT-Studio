# 创作规则与模板版本规范

> 这篇定义“创作规则”的存放、编辑、版本、结构化输出、模型绑定和任务快照。
> 用户界面统一叫“创作规则”。技术实现可以继续使用 `SkillDefinition`、`SkillVersion`、`SkillSnapshot` 等命名，但不要把 `Skill` 作为普通用户可见菜单名。

---

## 一、核心原则

```text
1. 创作规则不写死在页面组件里。
2. 核心生成必须有输出 schema。
3. 创作规则必须有版本。
4. 任务创建时冻结规则快照。
5. 创作规则只能生成建议、提示词或结构化输出，不能直接写数据库。
6. 视频包只引用创作规则，不直接编辑规则内容。
7. 项目工作台只显示当前规则名称、版本和跳转入口。
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
version
provider_kind
output_schema
description
```

建议字段：

```text
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

约束：

```text
1. 视频包只保存规则引用和推荐配置，不复制规则正文。
2. 编辑视频包不等于编辑规则。
3. 编辑规则不直接影响历史任务。
4. 开始任务时必须生成 PackSnapshot + SkillSnapshot。
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
3. 已创建任务使用 snapshot，不受后续改动影响。
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
4. 删除用户规则不影响历史任务，因为历史任务有 snapshot。
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

## 十一、任务快照

任务快照至少记录：

```json
{
  "creative_rule_snapshot": [
    {
      "key": "script.topic_narration",
      "display_name": "一句话视频想法生成文案",
      "version": "1.0.0",
      "content_hash": "sha256_xxx",
      "output_schema_hash": "sha256_xxx"
    }
  ],
  "pack_snapshot": {
    "pack_id": "children_education",
    "version": "1.0.0"
  }
}
```

---

## 十二、禁止事项

```text
1. 禁止在 Vue 页面里写大段 prompt。
2. 禁止在 Rust service 里散落不可追踪 prompt。
3. 禁止核心生成只靠 Markdown/XML 解析。
4. 禁止修改历史任务使用的规则 snapshot。
5. 禁止把创作规则做成可执行插件。
6. 禁止让视频包、模型配置、素材库各自藏一份重复规则。
```
