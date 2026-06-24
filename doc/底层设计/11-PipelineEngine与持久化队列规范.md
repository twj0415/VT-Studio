# PipelineEngine 与持久化队列规范

> 这篇定义任务如何创建、执行、暂停、恢复、取消、重试。它是后端生产线的核心施工规范。

## 一、核心原则

```text
1. 所有生产动作都必须进入 Task / TaskStep。
2. 任务状态以 SQLite 为准，内存只做执行缓存。
3. 每个 step 必须可记录输入、输出、错误和尝试次数。
4. Provider、FFmpeg、Chromium 这类耗时操作不得放在数据库长事务内。
5. review 节点必须显式进入 waiting_user。
6. 应用重启后能识别未完成任务并恢复或标记可恢复。
```

---

## 二、核心模型

```text
Task          一次用户触发的生产动作
TaskStep      任务中的一个阶段
TaskAttempt   某个 step 的一次执行尝试
Artifact      任务产物索引
ProgressEvent 实时通知，不作为最终状态
```

状态：

```text
TaskStatus: pending / running / succeeded / failed / cancelled
TaskStepStatus: pending / running / retrying / succeeded / failed / skipped / cancelled / waiting_user
```

---

## 三、TaskKind 到 Step 映射

### image_to_video_pipeline

当前主线由 `workflowType=image_to_video` 生成，不再使用旧的“脚本/TTS/字幕/封面全必做”链路。

```text
project_init
storyboard_generation
storyboard_review
image_prompt_generation
image_generation
image_review
video_prompt_generation
video_generation
video_review
final_composition
export
cleanup
```

### enhanced_media_pipeline（后续增强组合）

TTS、字幕、封面、模板动效不是不做，而是作为增强步骤按 workflow 配置插入。

```text
script_generation
script_review
tts_generation
subtitle_generation
cover_generation
cover_review
template_rendering
segment_composition
```

### script_only

```text
script_generation
script_review
```

### storyboard_only

```text
storyboard_generation
storyboard_review
```

### image_only

```text
image_prompt_generation
image_generation
image_review
```

### tts_only

```text
tts_generation
subtitle_generation
```

### render_only

```text
subtitle_generation
template_rendering
segment_composition
```

### compose_only

```text
segment_composition
final_composition
```

### export_only

```text
export
```

扩展 workflow 如 `digital_human / material_edit / image_slideshow / canvas_edit` 必须先在枚举规范中登记，再在本文件补 Step 映射。`image_to_video` 是当前主线，不归类为后续扩展。

---

## 四、Step 职责表

| Step | 输入 | 输出 | 可重试 | 人工节点 |
|---|---|---|---|---|
| project_init | Project | 初始化完整性校验 / 任务目录准备 | 是 | 否 |
| storyboard_generation | Project / source / Bible | StoryboardItem[] | 是 | 否 |
| storyboard_review | StoryboardItem[] | confirmed StoryboardItem[] | 否 | 是 |
| image_prompt_generation | StoryboardItem[] / Bible | imagePrompt / negativePrompt | 是 | 否 |
| image_generation | imagePrompt / refs | ImageCandidate[] | 是 | 否 |
| image_review | ImageCandidate[] | selectedImageId | 否 | 是 |
| video_prompt_generation | StoryboardItem[] | videoPrompt | 是 | 否 |
| video_generation | selectedImageId / videoPrompt | VideoSegment[] | 是 | 否 |
| video_review | VideoSegment[] | selectedVideoSegmentId | 否 | 是 |
| final_composition | selected video segments | exports/final.mp4 | 是 | 否 |
| export | final video | history / export result | 是 | 否 |
| cleanup | task temp | clean result | 否 | 否 |
| script_generation | Project / source | narrations / script draft | 是 | 否 |
| script_review | script draft | confirmed narrations | 否 | 是 |
| tts_generation | narration / voice config | audio_path / duration_seconds | 是 | 否 |
| subtitle_generation | narration / duration_seconds | subtitle_chunks / subtitles.json | 是 | 否 |
| cover_generation | image/video/title/template | cover_path | 是 | 否 |
| cover_review | cover_path | confirmed cover | 否 | 是 |
| template_rendering | item / template | rendered_frame_path | 是 | 否 |
| segment_composition | frame/video + audio | segment_path | 是 | 否 |

---

## 五、执行流程

```text
create_task
→ 冻结 snapshot_json
→ 创建 task_steps
→ 创建任务目录
→ start_task
→ PipelineEngine 获取租约
→ 按 order_index 执行 step
→ 每步写 task_attempt
→ 成功写 output_json / artifacts / StoryboardItem / ImageCandidate / VideoSegment 状态
→ 失败按 retry_policy 重试
→ review 节点根据 review_required 进入 waiting_user 或 skipped
→ 全部完成后 Task=succeeded
```

---

## 六、租约与恢复

任务执行时写：

```text
worker_id
lease_expires_at
```

规则：

```text
1. Worker 每隔 N 秒续租。
2. 应用启动时扫描 task_status=running 且 lease_expires_at 已过期的任务。
3. 如果当前 step 是幂等可恢复 step，置为 pending/retrying 后恢复。
4. 如果无法判断外部 Provider 是否仍在执行，置为 failed，并提示用户重试该 step。
5. waiting_user 不需要租约，应用重启后仍保持 waiting_user。
```

---

## 七、幂等规则

每个 TaskStep 必须有：

```text
idempotency_key = task_id + step_kind + input_hash
```

执行前检查：

```text
1. output_json 是否已有可用产物。
2. Artifact 文件是否存在且通过 PathGuard。
3. StoryboardItem / ImageCandidate / VideoSegment 对应状态是否已经 succeeded。
```

如果产物完整，可以跳过重复执行并补写状态。

---

## 八、重试规则

默认：

```text
max_attempts = 3
backoff = 2s, 5s, 10s
```

可重试：

```text
provider.timeout
provider.rate_limited
network.timeout
ffmpeg.process_interrupted
template.browser_crashed
```

不可重试：

```text
provider.auth_failed
validation.invalid_input
storage.path_denied
security.path_escape
ffmpeg.not_found
```

每次尝试必须写入 `task_attempts`。

---

## 九、取消规则

取消不是立即杀状态，而是：

```text
cancel_task
→ tasks.cancel_requested = true
→ 当前 step 到安全点检查
→ 终止 Provider/FFmpeg/Chromium 子进程
→ 当前 step=cancelled
→ task=cancelled
```

规则：

```text
1. 已完成产物不删除。
2. 未完成临时文件可进入 cleanup。
3. cancelled 任务可 resume，是否可恢复由 step 决定。
```

---

## 十、Review 节点

人工节点：

```text
script_review
storyboard_review
image_review
cover_review
```

执行到人工节点时：

```text
step_status=waiting_user
task_status=running
current_step_kind=xxx_review
```

用户确认：

```text
approve_task_step(approved=true)
→ step_status=succeeded
→ PipelineEngine 继续下一个 step

如果 `review_required.xxx=false`：

```text
创建对应 review step
→ step_status=skipped
→ PipelineEngine 自动继续
```
```

用户拒绝：

```text
approved=false
→ 根据业务进入 failed / cancelled / 回退上一步
```

---

## 十一、分镜级资源并发

这些步骤可按 StoryboardItem 或其候选资源并发：

```text
tts_generation
image_generation
subtitle_generation
template_rendering
segment_composition
```

规则：

```text
1. 并发数来自 PipelineConfig.max_concurrent_provider_calls。
2. Provider 调用和本地 FFmpeg 并发要分开限流。
3. 单个 StoryboardItem / ImageCandidate / VideoSegment 失败不应静默跳过，必须记录 item_id / image_id / segment_id 和 error_json。
4. step 总进度 = succeeded_item_count / total_item_count。
```

---

## 十二、状态重置规则

修改上游字段必须重置下游状态：

| 改动 | 需要 reset |
|---|---|
| narration / source_text | storyboard_generation / image_prompt_generation / audio_status / subtitle_status / video_prompt_generation / final_composition |
| subtitle_chunks | final_composition（启用字幕烧录时） |
| image_prompt | image_status / video_status / final_composition |
| selected_image_id / image_path | video_status / final_composition |
| video_prompt | video_status / final_composition |
| selected_video_segment_id / video_path | final_composition |
| template_params | template_rendering / final_composition（启用模板增强且字段已定义时） |
| BGM 配置 | final_composition / export |

规则：

```text
1. reset 只清当前引用，不删除历史 artifact。
2. 用户锁定字段不允许批量 reset，除非用户显式确认。
```

---

## 十三、锁定规则

StoryboardItem 只使用 `lock_flags_json` 作为锁定来源，不再使用 `is_user_locked`：

```json
{
  "narration": true,
  "storyboard": false,
  "image_prompt": true,
  "image": false,
  "audio": false,
  "subtitle": false,
  "template": false
}
```

批量重生成必须跳过对应锁定项。

---

## 十四、PipelineStep 接口

```rust
#[async_trait]
pub trait PipelineStep {
    fn kind(&self) -> TaskStepKind;
    fn input_dependencies(&self) -> Vec<TaskStepKind>;
    fn can_retry(&self) -> bool;
    fn is_idempotent(&self) -> bool;
    async fn execute(&self, ctx: &mut PipelineContext) -> AppResult<StepOutput>;
}
```

`PipelineContext` 至少包含：

```text
task_id
project_id
snapshot
storage_service
provider_manager
repositories
event_emitter
cancel_token
```

---

## 十五、禁止事项

```text
1. 不允许页面直接启动 Provider 调用。
2. 不允许 command 内部 spawn 后台任务后不入库。
3. 不允许只用内存 Map 保存任务。
4. 不允许用中文字符串作为任务状态。
5. 不允许失败后只写日志不写 task_step.error_json。
6. 不允许重试时覆盖历史错误记录。
```


---

## 十六、脚本产物存储规则

```text
启用脚本增强时：
- script_generation.output_json 保存 draft_narrations。
- script_review.output_json 保存 confirmed_narrations。
- storyboard_generation 优先读取 confirmed_narrations。

未启用脚本增强时：
- storyboard_generation 直接读取 Project 的 topic/source_text/source_text_path/input_options。
- fixed 模式不创建 script_generation step，只把切分后的 textSegments 作为 StoryboardItem.source_text。
```

---

## 十七、产物、重试和错误记录边界

```text
artifacts：记录每次生成或替换的历史产物。
ImageCandidate.image_path / VideoSegment.video_path / CompositionTask.output_path：记录当前使用的产物路径。
tasks.result_json：记录 TaskResult，供任务详情读取。
histories：记录完成后的历史列表和回看。

task_attempts：记录每次真实执行尝试。
TaskStep.retry_count：步骤级重试次数。
StoryboardItem.retry_count / ImageCandidate.retry_count / VideoSegment.retry_count：单项资源生成重试次数。

StoryboardItem.last_error_json / ImageCandidate.last_error_json / VideoSegment.last_error_json：单项最后错误。
TaskStep.error_json：步骤级汇总或阻断错误。
Task.last_error_json：任务最后阻断错误。
```

---

## 十八、project_init 和 cleanup 边界

```text
create_project 创建 Project、默认 Bible、空 Storyboard。
project_init 不重复创建基础业务数据，只校验初始化完整性，并准备 task 目录。
cleanup 只清 task/temp、临时 filelist、未完成残片；不删除 audio/images/rendered/segments/exports 等可恢复产物。
```
