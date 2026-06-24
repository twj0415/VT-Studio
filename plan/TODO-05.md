# TODO-05：任务队列与 PipelineEngine

> 目标：把所有生成动作纳入可恢复、可重试、可取消、可追踪的任务系统。  
> 本文件来自 `doc/底层设计/06-任务状态机.md`、`11-PipelineEngine与持久化队列规范.md`、`09-错误日志与事件规范.md`、`doc/功能模块/17-任务与历史.md`。

---

## 阶段目标

实现：

```text
TaskService
TaskStateMachine
PipelineEngine
TaskStep
TaskAttempt
Artifact
ProgressEvent
TaskError
基础错误码
结构化任务日志
断点续跑
失败重试
取消任务
人工 review 节点
```

所有生产动作必须以 SQLite 为最终状态源头，不能靠页面本地状态或内存队列。

---

## 本阶段范围

包含：

- Task / TaskStep 状态机。
- image_to_video pipeline 步骤定义。
- 持久化任务队列。
- 租约恢复。
- 幂等执行。
- 重试与退避。
- 取消安全点。
- `waiting_user` 人工确认节点。
- ProgressEvent 推送。
- 任务错误分类、基础错误码和任务日志。
- 任务历史基础。

不包含：

- 真实 Provider 实现细节。
- 完整 UI 工作台。
- 打包发布。

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

### 【】5.1 建立任务错误、错误码和结构化日志最小底座

**问题：**  
Task、Provider、Workflow、FFmpeg 都会依赖错误码、重试判断和日志关联；如果等到 TODO-09 才建立，会导致 TODO-05/06/07 先写一套临时错误和日志，再返工。

**改法：**

基于 TODO-02 的 `AppErrorDto`，先建立任务执行需要的最小底座：

```text
TaskError
TaskStep.error_json
TaskAttempt.error_json
error_code
error_kind
is_retryable
recover_action
trace_id
```

基础错误域先覆盖：

```text
task.*
storage.*
provider.*
workflow.*
ffmpeg.*
db.*
```

结构化任务日志至少包含：

```text
trace_id
project_id
task_id
task_step_id
step_kind
item_id
error_code
duration_ms
retry_count
relative_path
```

**验证：**

- step 失败能写入 `error_json`，并可被前端按 code 展示。
- retry 判断来自错误 kind / is_retryable，不靠字符串包含。
- 任务日志可关联一次完整任务链路。
- 日志经过 SecretGuard 脱敏。

**风险：**  
不要把 Provider 原始错误、完整请求头、API Key、绝对路径或长篇用户原文直接写入 error_json / 日志。

---

### 【】5.2 实现 Task / TaskStep 状态模型

**问题：**  
生成过程不能只给一个总状态，否则无法重试单步、断点续跑和定位失败。

**改法：**

TaskStatus 只保留：

```text
pending
running
succeeded
failed
cancelled
```

TaskStepStatus 包含：

```text
pending
running
retrying
succeeded
failed
skipped
cancelled
waiting_user
```

**验证：**

- `retrying/skipped/waiting_user` 是 StepStatus，不是 TaskStatus。
- 每个 TaskStep 有独立 input/output/error。
- 页面展示状态来自数据库。

**风险：**  
不要用页面本地状态代替任务状态机。

---

### 【】5.3 定义 image_to_video pipeline

**问题：**  
当前主线步骤必须固定，否则页面、任务、日志、恢复策略无法对齐。

**改法：**

当前 pipeline：

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

后续可插入增强步骤：

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

**验证：**

- 创建 image_to_video 任务时生成完整 task_steps。
- review 节点进入 `waiting_user`。
- TTS/字幕/封面不是当前主线强制步骤。

**风险：**  
不要把增强步骤写死为当前必经步骤。

---

### 【】5.4 实现 TaskService 命令

**问题：**  
任务生命周期需要统一入口。

**改法：**

实现：

```text
create_task
start_task
cancel_task
resume_task
retry_task_step
approve_task_step
get_task_detail
list_tasks
```

**验证：**

- failed step 可单步重试。
- waiting_user 节点可批准后继续。
- cancel 请求后不会继续跑下游。

**风险：**  
鉴权失败、密钥错误等不可重试错误不要反复重试。

---

### 【】5.5 实现持久化队列与租约恢复

**问题：**  
如果只用内存队列，应用关闭后 running 任务会丢失。

**改法：**

字段建议：

```text
worker_id
lease_expires_at
started_at
finished_at
cancel_requested
```

启动时扫描：

```text
running 且 lease 过期的任务
running 且无 worker 的任务
```

**验证：**

- 应用重启后 running 任务可标记可恢复。
- waiting_user 重启后仍保持 waiting_user。
- 已完成 step 不重复执行。

**风险：**  
恢复不能只看 DB，也要检查文件是否存在和 artifact 记录。

---

### 【】5.6 实现幂等与 artifact 记录

**问题：**  
重试时如果不幂等，会重复生成文件、重复扣费、覆盖用户选择。

**改法：**

幂等 key：

```text
task_id + step_kind + input_hash
```

每步成功写：

```text
artifacts
output_json
business resource status
```

**验证：**

- 相同输入重复 retry 不覆盖已确认结果。
- step 成功后能从 artifact 找到产物。
- 历史错误记录不被新错误覆盖。

**风险：**  
重生成应产生新候选或 revision，不覆盖用户已选产物。

---

### 【】5.7 实现 TaskAttempt 和重试策略

**问题：**  
失败原因和每次尝试都要可追踪。

**改法：**

记录：

```text
attempt_id
task_step_id
started_at
finished_at
input_json
output_json
error_json
retry_count
duration_ms
```

重试策略来自任务快照：

```text
retry_policy_snapshot
```

**验证：**

- 每次失败都有 attempt。
- retry 不删除旧 attempt。
- 不同错误类型使用不同恢复动作。

**风险：**  
内容审核失败可中性化重试；密钥错误不应自动重试。

---

### 【】5.8 实现取消任务

**问题：**  
取消不仅要改状态，还要停止 Provider/FFmpeg/Chromium 子进程。

**改法：**

取消流程：

```text
cancel_requested = true
到安全点退出
停止 Provider 请求或忽略结果
kill FFmpeg/Chromium 子进程树
写 cancelled
保留已完成产物
```

**验证：**

- 取消后不进入下游 step。
- 已完成产物不删除。
- 子进程不残留。

**风险：**  
Windows 子进程树清理要单独验证。

---

### 【】5.9 实现 ProgressEvent

**问题：**  
前端需要进度，但不能把事件当最终状态。

**改法：**

ProgressEvent 字段：

```text
trace_id
project_id
task_id
task_step_id
step_kind
status
progress
message
error_code
```

**验证：**

- 事件可驱动 UI 刷新。
- 页面刷新后仍能从 DB 读到真实状态。
- ProgressEvent 丢失不影响任务最终状态。

**风险：**  
不要让前端通过事件自行推导最终业务状态。

---

### 【】5.10 实现下游 reset 规则和锁定保护

**问题：**  
用户修改上游字段后，下游产物状态需要局部失效，但不能全项目重跑。

**改法：**

规则：

```text
修改 imagePrompt → 影响 image_candidates / selectedImageId / video_segments / composition
修改 selectedImageId → 影响 video_segments / composition
修改 videoPrompt → 影响 video_segments / composition
修改 selectedVideoSegmentId → 影响 composition
```

`lock_flags_json` 保护用户手动字段。

**验证：**

- 批量重生成跳过锁定项。
- 修改一个分镜不重置全项目。
- 下游失效有明确提示。

**风险：**  
不要一改字段就全项目重新生成。

---

## 阶段完成标准

- Task / TaskStep / TaskAttempt / Artifact 可落库。
- 任务错误、基础错误码和结构化任务日志可用。
- image_to_video pipeline 能创建、启动、暂停 review、继续、失败、重试、取消。
- 重启后任务状态可恢复。
- ProgressEvent 仅做通知，DB 是最终状态。
- 下游 reset 和 lock_flags 规则可用。






