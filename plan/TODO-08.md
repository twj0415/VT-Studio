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

技术侧仍可使用 `Asset / AssetReference / PromptSkill / SkillSnapshot / Project Bible` 等命名。

---

## 本阶段范围

包含：

- “创作资源 > 素材库”基础 UI 和数据流。
- “创作资源 > 创作规则”查看、编辑、版本化和引用关系。
- “创作资源 > 视频包”中的默认规则、默认模型偏好和默认素材引用。
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
> - 做什么：落地到具体文件、接口、页面、表结构、DTO、Store、Service、组件、样式、i18n、测试或文档。
> - 做到怎么样：UI、逻辑、样式、组件封装、多语言、状态、错误处理、安全、验证全部满足，才算完成。
> - 怎么做：按“改法”小步实现；不要引入本阶段明确排除的能力。
> - 打标：完成一条后必须立刻把 `【】` 改成 `【X】`，并写完成记录；不需要用户提醒。
> - 停止：如果需求不清、验收不清、文档冲突、数据结构不明或风险高，停下来说明，不靠猜测继续。

### 【】8.1 完善素材库 Asset / AssetReference

**问题：**  
生成图、视频片段、音频、封面、参考图、模板资源如果没有统一资产引用，导出和删除都会出问题。

**改法：**

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

引用关系记录：

```text
project_id
asset_id
owner_type
owner_id
usage_type
created_at
```

**验证：**

- 被引用资产不能物理删除。
- 删除前能提示被哪些项目/分镜/任务使用。
- 导出项目包能按引用收集素材。
- 用户在素材库里优先看到“角色参考、画风参考、场景参考、姿态/深度/mask、音频、模板、生成产物”，而不是混乱的图片/视频大杂烩。

**风险：**  
清理临时产物不能误删历史最终产物或用户确认资产。

---

### 【】8.2 实现素材导入和预览

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

**风险：**  
不能把远程 URL 或用户绝对路径直接当资产路径。

---

### 【】8.3 实现 Style Bible

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

---

### 【】8.4 实现参考图反推画风

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

---

### 【】8.4.1 实现创作规则管理

**问题：**  
分镜规则、角色生成规则、分镜图生成规则、视频动作规则如果只放在代码或视频包里，会难以查看、编辑、版本化，也会和不同视频包产生冲突。

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
视频包默认规则：视频包引用一组规则版本
作品规则快照：作品开始创作时固化规则版本
任务规则快照：每次生成时记录 skill_key / version / hash
```

冲突处理：

```text
同一作品只能激活一套视频包默认规则
用户可在作品内覆盖某条规则
覆盖后生成任务记录覆盖后的规则快照
删除或修改全局规则不影响历史作品
```

**验证：**

- 分镜、角色图、分镜图、视频提示词等规则都能查看来源和版本。
- 视频包只是引用规则，不直接把规则内容复制成不可追踪文本。
- 修改视频包默认规则不会影响已创建作品的历史快照。

**风险：**  
不要把创作规则做成可执行代码；当前只允许提示词、结构化 schema、参数模板和校验规则。

---

### 【】8.5 实现 Character Bible

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

---

### 【】8.6 实现角色参考图和生图注入

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

---

### 【】8.7 实现 Environment / Location Bible

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

---

### 【】8.8 实现生成上下文快照

**问题：**  
没有快照，历史结果无法解释，也无法复现。

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

**验证：**

- ImageCandidate 能查看当时使用的设定和模型。
- VideoSegment 能查看输入图和视频 prompt。
- 修改 Bible 不影响历史候选图解释。

**风险：**  
快照中不能包含真实密钥或绝对路径。

---

### 【】8.9 实现锁定和重生成保护

**问题：**  
用户手动调整的角色、场景、prompt、候选图不能被批量操作覆盖。

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

**验证：**

- 批量重生成跳过锁定项。
- 尝试覆盖锁定内容前提示用户。
- 解锁后才能批量覆盖。

**风险：**  
不要让局部重生成破坏用户已确认的图和视频。

---

## 阶段完成标准

- “创作资源 > 素材库”导入、预览、引用、删除保护可用。
- “创作资源 > 创作规则”可查看、编辑、版本化，并能被视频包和作品快照引用。
- Style / Character / Environment Bible 可创建、编辑、引用。
- 生图和视频生成能记录上下文快照。
- 分镜只引用设定 ID，不复制整段设定。
- 批量重生成尊重锁定字段和用户确认产物。






