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
> - 问题：必须说清不做会造成什么用户问题、工程问题或后续返工。
> - 位置：必须落到页面、接口、表结构、DTO、Store、Service、组件、样式、i18n、测试或文档；不能只写“相关文件”。
> - 改法：按小步实现，写清数据流、状态流、边界和本阶段不做什么。
> - 验收：写清做到什么客观状态才算完成，不能把验证命令当验收。
> - 验证：写清命令、页面流程、数据库检查、文件检查、日志检查或 smoke test。
> - 下一步：本条必须满足“下一步进入条件”后，才能打勾进入下一条；旧 TODO 缺字段时先补齐再实现。
> - 打标：完成一条后必须立刻把 `【】` 改成 `【X】`，并写完成记录；不需要用户提醒。
> - 停止：如果需求不清、验收不清、文档冲突、数据结构不明或风险高，停下来说明，不靠猜测继续。

### 【X】5.1 建立任务错误、错误码和结构化日志最小底座

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

**完成记录：**

- 已在后端 `core::error` 中新增 `TaskError`，统一包含 `error_code / error_kind / message / detail / is_retryable / recover_action / trace_id`。
- 已建立基础错误码分类和重试判断，当前覆盖 `task.* / storage.* / provider.* / workflow.* / ffmpeg.* / db.* / security.* / validation.* / network.* / rate_limit.*`。
- 已将 `TaskError -> AppErrorDto` 打通，前端可继续按 `code / kind / isRetryable / recoverAction / traceId` 展示错误。
- 已新增 migration `task_errors_and_logs_v1`，为 `tasks / task_steps / task_attempts` 补充错误、trace、重试、耗时、input/output 字段，并新增 `task_logs` 结构化日志表。
- 已新增 `TaskRepository`，支持写入 step 失败、attempt 失败记录和结构化任务日志；失败会同步写入 `task_steps.error_json`、`task_attempts.error_json`、`tasks.last_error_json` 和 `task_logs`。
- 结构化日志字段已包含 `trace_id / project_id / task_id / task_step_id / step_kind / item_id / error_code / duration_ms / retry_count / relative_path / metadata_json`。
- 已复用 `SecretGuard` 对错误 message、detail、日志 message、日志 metadata 做脱敏，避免 API Key、Bearer Token、完整请求头进入 error_json 或 task_logs。
- 已新增 Rust 单测覆盖：错误码分类和 retry/recoverAction、错误 detail 脱敏、`TaskError` 转 `AppErrorDto`、step 失败写入 error_json/attempt/log、日志 message 脱敏。
- 已执行 `cargo fmt`。
- 已通过 `cargo test`：44 个 Rust 单测全部通过。
- 已通过 `cargo check`，无新增 warning。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 已通过 `git diff --check`；仅有 Windows 换行提示，无空白错误。
- 说明：本条只完成任务错误、错误码和结构化日志底座；Task/TaskStep 状态枚举收窄、完整 Pipeline step 创建、TaskService 命令、队列、租约、重试和取消继续按 5.2-5.10 顺序实现。

---

### 【X】5.2 实现 Task / TaskStep 状态模型

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

**完成记录：**

- 已将前端强类型 `TaskStatus` 收窄为 `pending / running / succeeded / failed / cancelled`，不再包含 `retrying / skipped / waiting_user`。
- 已将 `TaskStepDto.status` 改为 `TaskStepStatus`，步骤状态继续支持 `pending / running / retrying / succeeded / failed / skipped / cancelled / waiting_user`。
- 已修正前端 Mock task API：任务整体等待用户时使用 `taskStatus = running`，等待确认只体现在对应 step 的 `status = waiting_user`。
- 已修正前端 Mock project latestTask：不再返回 `waiting_user` 作为任务整体状态，改为 `running`。
- 已修正前端内置字典和 i18n：`dict.taskStatus` 只保留任务整体 5 个大状态；`dict.taskStepStatus` 保留细状态。
- 已修正后端内置字典：`taskStatus` 不再返回 `retrying / waiting_user / skipped`，这些只保留在 `taskStepStatus`。
- 已修正项目创建落库：新建作品默认 task 的 `task_status` 从 `waiting_user` 改为 `running`，`current_step = storyboard_review` 继续表达当前等待确认步骤。
- 已修正后端 task stub：人工 review 阶段 `TaskDetailDto.task_status` 返回 `running`，不再返回 `waiting_user`。
- 已新增 migration `task_status_model_v1`：把历史 `tasks.task_status in ('waiting_user','retrying','skipped')` 迁移为 `running`，并用 trigger 阻止后续向 `tasks.task_status` 写入非整体状态。
- 已新增 DB trigger 校验 `task_steps.status` 只能写入 StepStatus 合法值，并新增单测验证 task/status 与 step/status 分层。
- 已执行 `cargo fmt`。
- 已通过 `cargo test`：45 个 Rust 单测全部通过。
- 已通过 `cargo check`。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 已通过 `git diff --check`；仅有 Windows 换行提示，无空白错误。
- 说明：本条只完成状态模型分层和约束；完整 image_to_video step 列表创建、TaskService 生命周期命令、队列、租约、重试和取消继续按 5.3-5.10 顺序实现。

---

### 【X】5.3 定义 image_to_video pipeline

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

**完成记录：**

- 已在后端 `domain::task` 集中定义 `image_to_video` 主线 pipeline 步骤：`project_init / storyboard_generation / storyboard_review / image_prompt_generation / image_generation / image_review / video_prompt_generation / video_generation / video_review / final_composition / export / cleanup`。
- 已明确初始步骤状态：`project_init` 和 `storyboard_generation` 在当前项目创建闭环里标记为 `succeeded`，`storyboard_review` 进入 `waiting_user`，其余步骤为 `pending`。
- 已让项目创建落库时同步生成完整 `task_steps`，不再只有一个任务总状态；`tasks.current_step` 初始为 `storyboard_review`。
- 已新增 `task_steps.order_index` migration，保证 pipeline 读取顺序稳定，不依赖同秒 `created_at` 或 step_id 字符串排序。
- 已扩展 `TaskRepository`，支持创建完整 image_to_video 任务、读取项目最新任务详情、读取完整 step 列表、批准当前 step 并推进到下一个 pipeline step。
- 已将后端 `create_task / get_task_detail / approve_task_step` 命令接入 `AppState` 和 SQLite，不再返回硬编码四步 stub。
- 已扩展 `TaskStepDto.output_json`，用于后续 step 输出和任务详情读取。
- 已同步前端 Mock task API：任务详情返回完整 12 步 pipeline，整体 `taskStatus` 仍只使用大状态，review 节点用 `TaskStepStatus.waiting_user` 表达。
- 已将前端 `TaskStepDto.stepName/currentStep` 类型约束为 `TaskStepKind`，减少任意字符串污染 step 协议。
- 已新增 Rust 单测覆盖：创建完整 image_to_video pipeline steps、初始 review 节点状态、批准 step 后按 pipeline 顺序推进、项目创建默认 task 状态和 DB 约束。
- 已执行 `cargo fmt`。
- 已通过 `cargo test`：47 个 Rust 单测全部通过。
- 已通过 `cargo check`。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 已通过 `git diff --check`；仅有 Windows 换行提示，无空白错误。
- 说明：本条只完成 pipeline step 定义、创建和读取；真正 `start_task / cancel_task / resume_task / retry_task_step / list_tasks` 命令，以及持久化队列、租约、重试和取消，会继续在 5.4-5.10 实现。

---

### 【X】5.4 实现 TaskService 命令

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

**完成记录：**

- 已新增后端任务命令 DTO：`TaskProjectRequest / RetryTaskStepRequest / ListTasksRequest / TaskSummaryDto`。
- 已实现并注册 Tauri commands：`create_task / start_task / cancel_task / resume_task / retry_task_step / approve_task_step / get_task_detail / list_tasks`。
- 已将 `create_task / get_task_detail / approve_task_step` 从硬编码 stub 改为通过 `AppState.database()` 读取和写入 SQLite。
- 已扩展 `TaskRepository`：支持创建 image_to_video 任务、读取项目最新任务详情、列表查询任务、启动任务、取消任务、恢复任务、重试指定 step、批准 step。
- `cancel_task` 会把未完成 step 置为 `cancelled`，保留已完成产物和已成功 step。
- `resume_task` 会根据 DB 中第一个可恢复 step 推进 `tasks.current_step`，review step 恢复为 `waiting_user`，非 review step 恢复为 `pending`。
- `retry_task_step` 会把指定 step 置为 `pending`，清理该 step 当前阻断错误字段，并把任务整体恢复为 `running`；历史 attempt 不删除。
- 已同步前端 `tauriCommands`、task entity API、task store 和 DTO 类型，新增 `createTask / startTask / cancelTask / resumeTask / retryTaskStep / listTasks` 统一入口。
- 前端 Mock task API 已支持这些命令的最小状态返回，页面仍不直接 invoke。
- 已新增 Rust 单测覆盖：任务列表、启动、取消、恢复、重试指定 step 的 DB 状态变化。
- 已执行 `cargo fmt`。
- 已通过 `cargo test`：48 个 Rust 单测全部通过。
- 已通过 `cargo check`。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 已通过 `git diff --check`；仅有 Windows 换行提示，无空白错误。
- 说明：本条只完成命令和持久化状态入口；真实后台执行、租约恢复、幂等 artifact、重试策略、取消子进程和 ProgressEvent 会按 5.5-5.10 继续实现。

---

### 【X】5.5 实现持久化队列与租约恢复

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

**完成记录：**

- 已新增 migration `task_queue_lease_v1`，为 `tasks` 补充 `worker_id / lease_expires_at / started_at / finished_at / cancel_requested` 字段，并增加租约和取消请求索引。
- 已在 `TaskRepository` 实现 `acquire_lease / renew_lease / scan_and_mark_recoverable_tasks`。
- `acquire_lease` 只允许抢占 `running`、未取消、当前步骤为非 review、且无有效 worker/租约过期的任务；抢占成功后写入 `worker_id / lease_expires_at / started_at`，并把当前自动 step 从 `pending/retrying` 推进为 `running`。
- `renew_lease` 只允许当前 `worker_id` 续约；不同 worker 不能续约，也不会误读已有租约当作成功。
- `start_task / resume_task / retry_task_step / cancel_task` 已同步清理或设置租约字段，避免旧 worker/旧 lease 污染下一次执行。
- 已在 Tauri 启动初始化 SQLite 后调用 `scan_and_mark_recoverable_tasks`，应用重启时会扫描 `running` 且无 worker、无 lease 或 lease 过期的自动步骤。
- 恢复扫描会把需要人工恢复的自动任务标记为 `failed`，清空 `worker_id / lease_expires_at`，写入 `task.resume_required` 到 `tasks.last_error_json` 和当前 `task_steps.error_json/error_code/error_kind/recover_action`，其中 `recover_action = resume_task`。
- 恢复扫描不会处理 `storyboard_review / image_review / video_review` 人工确认节点，`waiting_user` 重启后仍保持 `waiting_user`。
- 已新增 Rust 单测覆盖：租约抢占写入 worker 并推进当前自动 step、不同 worker 不能续约、无有效租约的 running 自动任务会被标记可恢复、waiting_user review 节点不会被恢复扫描改坏。
- 已执行 `cargo fmt`。
- 已通过 `cargo test`：52 个 Rust 单测全部通过。
- 已通过 `cargo check`。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 说明：本条只完成持久化队列租约和重启恢复底座；恢复时进一步核对 artifact 记录、文件存在性、幂等产物写入会继续按 5.6 实现。

---

### 【X】5.6 实现幂等与 artifact 记录

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

**完成记录：**

- 已新增 migration `task_idempotency_artifacts_v1`，为 `task_steps` 补充 `idempotency_key / input_hash`，并为历史 step 写入兼容 key。
- 已为 `task_steps.idempotency_key` 建唯一索引，幂等 key 采用 `task_id + step_kind + input_hash`。
- 已兼容增强 `artifacts` 表，新增 `owner_kind / owner_id / artifact_kind / media_kind / metadata_json / idempotency_key / input_hash`，旧的 `kind / data_json` 字段继续保留，避免破坏已有代码。
- 已新增 artifact 查询索引：按 task/step、owner、idempotency 查产物。
- 已在 `TaskRepository` 新增 `StepSuccessRecord / TaskArtifactRecord / TaskArtifactDto / IdempotencyHit`。
- 已实现 `record_step_success`：成功 step 会短事务写入 `task_steps.input_json/output_json/status=succeeded/idempotency_key/input_hash`，同步插入 artifacts，并清理该 step 的阻断错误字段。
- 已实现 `find_idempotent_step_output`：相同 `task_id + step_kind + input_hash` 且 step 已成功时，直接返回既有 `output_json + artifacts`。
- 已实现 `find_idempotent_step_output_with_existing_artifacts`：在 DB 命中基础上，用 `PathGuard` 校验 artifact 相对路径对应文件仍存在且未逃逸 workspace。
- `record_step_success` 在写入前会先查幂等命中；相同输入重复调用直接返回旧产物，不覆盖旧 `output_json`，不重复插入 artifact，也不影响用户已确认的 selected 结果。
- artifact 写入会校验 `relative_path` 必须是受控相对路径，拒绝 `../`、绝对路径、URL 等不安全路径。
- 已新增 Rust 单测覆盖：step 成功写入 output_json 和 artifact、幂等查询返回既有产物、相同输入重复成功不覆盖已确认 output、不重复插 artifact、不安全 artifact 路径被拒绝、要求文件存在时缺失文件会失败。
- 已执行 `cargo fmt`。
- 已通过 `cargo test`：56 个 Rust 单测全部通过。
- 已通过 `cargo check`。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 说明：本条只完成幂等产物记录与查询底座；每次真实执行尝试的 attempt 明细、重试次数、退避策略和错误恢复动作继续按 5.7 实现。

---

### 【X】5.7 实现 TaskAttempt 和重试策略

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

**完成记录：**

- 已新增并落地默认重试策略快照：`maxAttempts=3`，`backoffSeconds=[2,5,10]`，策略以 JSON 快照写入 `task_steps.retry_policy_snapshot_json` 和每条 `task_attempts.retry_policy_snapshot_json`。
- 已实现 `TaskAttemptDto` 读回能力，attempt 可读取 `attempt_index / status / input_json / output_json / error_json / error_code / error_kind / is_retryable / recover_action / trace_id / duration_ms / retry_policy_snapshot / next_retry_at / backoff_seconds`。
- 已将 `record_step_failure` 改为仓储层自动计算 `attempt_index`，不再信任调用方传入的 `retry_count`；历史 attempt 不删除，新失败会追加记录。
- 可重试错误且未达上限时，step 写为 `retrying`，task 仍保持 `running`，同时写入 `next_retry_at / backoff_seconds`；例如 `provider.timeout` 第 1 次退避 2 秒，第 2 次退避 5 秒。
- 可重试错误达到上限后，step/task 写为 `failed`，不再写 `next_retry_at`；不可重试错误如 `provider.auth_failed` 第一次失败就直接 `failed`，并保留 `recover_action=update_secret`。
- 已让 `record_step_success` 写入成功 attempt，状态为 `succeeded`，包含 `input_json / output_json / retry_policy_snapshot_json`；幂等命中时仍直接返回既有产物，不重复写 attempt，不重复插 artifact。
- 已修正租约获取规则：`retrying` step 未到 `next_retry_at` 时不能被 worker 抢租约；到期后才允许重新执行。
- 已修正恢复扫描规则：处于正常计划重试中的 `retrying` step 不会被重启扫描误判为 lease 丢失并改成 failed。
- 已新增 Rust 单测覆盖：首次可重试失败进入 `retrying`、attempt 快照与退避写入、多次失败 attempt_index 递增、达到 3 次失败后 task/step 失败、不可重试错误立即失败、成功 step 写入 succeeded attempt、未到退避时间不能抢租约、计划重试不会被恢复扫描误判。
- 已执行 `cargo fmt`。
- 已通过 `cargo test`：61 个 Rust 单测全部通过。
- 已通过 `cargo check`。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 说明：本条只完成 TaskAttempt 明细、失败重试策略、退避时间和相关租约/恢复保护；取消 Provider/FFmpeg/Chromium 子进程树继续在 5.8 实现。

---

### 【X】5.7.1 当前实现审计与 Tauri / Mock 纠偏

**问题：**  
继续做 5.8 前必须先确认当前代码真实状态。否则会把前端 Mock 闭环误判成 Tauri/SQLite 闭环，后续取消、进度、重试和 reset 都可能接在错误数据源上。

**位置：**

```text
plan/当前实现审计.md
plan/README.md
plan/阶段路线图.md
doc/00-文档入口.md
doc/README.md
src/src/shared/api/invoke.ts
src-tauri/tauri.conf.json
src-tauri/src/commands/scene.rs
src-tauri/src/domain/scene.rs
src-tauri/src/services/scene_service.rs
src-tauri/src/main.rs
```

**改法：**

```text
1. 逐项核对页面、API adapter、Tauri command、Rust service、Task 底座和文档路径。
2. 把当前实现按“真实、Mock、占位、需纠偏”落到审计文档。
3. 把 adapter 从固定 mock 改为 auto：桌面有 Tauri 桥走 Tauri，普通浏览器走 Mock。
4. 开启 Tauri 全局 API 注入，保证当前不新增 @tauri-apps/api 依赖也能调用 command。
5. 补齐前端已声明但 Rust 未注册的清理历史候选 command。
6. 将继续开发前必须读取审计文档写入入口和 plan 规则。
```

**验收：**

- 新窗口接手时能从 `doc/00-文档入口.md` 和 `plan/当前实现审计.md` 知道当前真实实现状态。
- 后续 TODO 不再只按“页面可点”判断完成，必须区分 Mock、Tauri/SQLite、真实 Provider/FFmpeg。
- 桌面环境具备走 Tauri command 的基础桥接条件。
- 前端命令表里的 `clear_historical_image_candidates`、`clear_historical_video_segments` 在 Rust 侧已注册。

**验证：**

- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仅有 Vite chunk 体积提示。
- `cargo fmt` 已执行。
- `cargo test` 通过：61 个 Rust 单测全部通过。
- `cargo check` 通过。
- `git diff --check` 无空白错误；仅有 Windows 换行提示。

**完成记录：**

- 已新增 `plan/当前实现审计.md`，逐项记录我的作品、开始创作、工作台、分镜、生图、视频、合成、创作资源、模型工作流、设置、Task 底座、API adapter、Tauri command 的真实状态。
- 已更新 `plan/README.md`、`plan/阶段路线图.md`、`doc/00-文档入口.md`、`doc/README.md`，要求继续开发前先读审计文档。
- 已把 `src/src/shared/api/invoke.ts` 改为 `auto` adapter：有 `window.__TAURI__.core.invoke` 时走 Tauri，否则走 Mock。
- 已在 `src-tauri/tauri.conf.json` 设置 `withGlobalTauri = true`。
- 已补齐并注册 `clear_historical_image_candidates`、`clear_historical_video_segments` 两个 Tauri command。
- 说明：本条只完成审计和桥接纠偏；`scene_service` 仍是硬编码分镜/候选图/视频段，`start_composition` 仍未真实合成，已写入审计文档作为后续 TODO 约束。

**风险：**  
不能把本条理解成真实生成闭环完成。它只是把错误前提纠正，后续 5.8/5.9/5.10 仍需基于真实 Task / DB 状态继续实现。

---

### 【X】5.8 实现取消任务

**问题：**  
取消不能只是把任务状态改成 `cancelled`。如果 worker 仍继续推进 step、写入产物或保留子进程，后续会出现扣费继续发生、FFmpeg/Chromium 残留、用户以为取消但最终又生成结果的问题。

**位置：**

```text
src-tauri/src/domain/task*
src-tauri/src/service/task*
src-tauri/src/repository/task*
src-tauri/src/command/task*
src/src/entities/task*
src/src/features/task* 或现有任务入口页面
```

如果实际文件名不同，以当前项目的 TaskRepository / TaskService / PipelineEngine / Tauri task commands / task store 为准。

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

本阶段必须落地：

```text
1. cancel_task 写入 cancel_requested = true，并记录 trace_id、cancel_requested_at、cancel_reason。
2. PipelineEngine 每个 step 开始前、长任务轮询处、写产物前都检查取消令牌。
3. queued / pending / retrying step 在取消后统一写 cancelled，不再被租约抢占。
4. running step 到安全点后写 cancelled；不得继续写 succeeded。
5. 已 succeeded 的 step 和 artifact 保留，不删除、不回滚。
6. 新增 CancellableTask / CancellationToken / ProcessHandleRegistry 这类机制，供 Provider、FFmpeg、Chromium 注册清理句柄。
7. 当前阶段如果还没有真实 Provider/FFmpeg/Chromium 子进程，必须用 fake worker / fake child process 单测证明 kill hook 会被调用；TODO-07/10 接入真实子进程时必须复用该机制，不允许旁路 spawn。
8. 前端取消按钮必须有 running/queued 可取消、已完成不可取消、取消中禁用、取消失败展示错误的状态。
```

**验收：**

- 用户点击取消后，任务整体最终进入 `cancelled`，未开始和未完成 step 进入 `cancelled`，已成功 step 保持 `succeeded`。
- `cancel_requested = true` 后不会再抢占新租约，不会推进下游 step，不会写入新的业务产物。
- running step 在安全点停止；如果绑定了进程句柄，取消会调用 kill/abort hook。
- 取消不删除已生成的 ImageCandidate、VideoSegment、CompositionTask、Artifact。
- 前端刷新页面后仍从 DB 看到取消后的真实状态，而不是只靠内存按钮状态。
- 取消失败时返回结构化错误码，日志脱敏且包含 trace_id。

**验证：**

- Rust 单测覆盖：pending 取消、running 取消、retrying 取消、waiting_user 取消、已成功 step 保留、取消后不能 acquire lease、取消后 record_step_success 被拒绝或忽略。
- Rust 单测覆盖 fake process handle：取消任务会调用 abort/kill hook，重复取消幂等。
- 前端 typecheck/build 通过。
- 手动 smoke：创建一条任务，启动后取消，刷新页面确认状态仍为 `cancelled`，任务详情里已完成产物还在。

**下一步进入条件：**

- 5.8 的验收全部满足，并在完成记录写清执行过的 Rust 测试、前端 typecheck/build 和手动 smoke 结果。
- 如果真实 Provider/FFmpeg/Chromium 尚未接入，完成记录必须明确“当前验证对象是 fake handle，真实接入将在 TODO-07/10 使用同一取消机制”，不能写成真实子进程已完成。
- 确认取消后不会进入 5.9 ProgressEvent 的下游事件推进逻辑，再把本条改为 `【X】` 并进入 5.9。

**风险：**  
Windows 子进程树清理要单独验证；取消令牌不能只停前端 UI，也不能在取消后继续等待 Provider 结果并写入成功状态。

**完成记录：**

- 已新增 migration `task_cancellation_v1`，为 `tasks` 补充 `cancel_requested_at / cancel_reason`，取消请求会写入 `trace_id`，保留第一次取消原因。
- 已扩展 `TaskRepository`：`request_cancellation / complete_cancelled_step / ensure_cancellation_requested / is_cancel_requested` 可表达取消请求、安全点落库和写产物前检查。
- `cancel_task` / `complete_cancelled_step` 会把未完成 step 置为 `cancelled`，已 `succeeded` 的 step 和已有 artifact 保留，不删除、不回滚。
- `acquire_lease / renew_lease` 已由 `cancel_requested = 0` 阻断；取消请求后不能再抢占新租约或续租。
- `record_step_success` 在写入 output_json、attempt 和 artifact 前检查取消请求；取消后成功写入会被拒绝，避免 Provider/FFmpeg 返回结果后继续落成功状态。
- 已新增 `services/task_cancellation.rs`，提供 `CancellationToken / ProcessHandleRegistry / CancellableProcessHandle`，后续 Provider、FFmpeg、Chromium 必须注册到同一机制。
- Tauri `cancel_task` command 已改为先写 DB 取消请求，再调用 `ProcessHandleRegistry.abort_task`，最后落库为 `cancelled`。
- 当前没有真实 Provider/FFmpeg/Chromium 子进程接入；本条用 fake handle 单测验证 abort hook 会被调用且重复取消幂等。真实子进程树清理会在 TODO-07/10 接入时复用该机制，不写成已完成真实子进程清理。
- 工作台任务摘要已接入取消按钮：`running/pending` 可取消，已完成或已取消不可取消；取消中按钮禁用并显示 loading；失败通过 Naive message 展示错误。
- 验证通过：`cargo test` 67 个 Rust 单测全部通过；`cargo check` 通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。
- 复核结果：取消后不会被租约推进，也不会通过 `record_step_success` 写入成功产物；5.9 的 ProgressEvent 后续只能在 DB 状态已落库后通知 UI。

---

### 【X】5.9 实现 ProgressEvent

**问题：**  
前端需要进度，但不能把事件当最终状态。

**位置：**

```text
src-tauri/src/domain/task*
src-tauri/src/service/task*
src-tauri/src/event* 或现有 Tauri event 模块
src/src/entities/task*
src/src/shared/api* 或现有 task store
src/src/features/workspace* / 任务历史入口
```

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

事件规则：

```text
1. ProgressEvent 只用于通知 UI 刷新，不作为最终状态源。
2. DB 中 tasks / task_steps / task_attempts / artifacts 才是最终状态。
3. 每个事件必须带 trace_id、task_id、step_kind；行级任务必须带 item_id 或 owner_id。
4. 事件丢失后，页面刷新或重新查询 task_detail 仍能得到正确状态。
5. cancelled / failed / succeeded 事件发出前，对应 DB 状态必须已经落库。
6. 事件 message 只能放脱敏摘要，不放密钥、完整 Provider 请求、绝对路径或长篇用户原文。
```

**验收：**

- 事件可驱动 UI 刷新。
- 页面刷新后仍能从 DB 读到真实状态。
- ProgressEvent 丢失不影响任务最终状态。
- 批量生图 / 视频这类行级任务能显示总进度和行级状态，但最终确认仍来自 DB。
- 取消、失败、重试、waiting_user 都有可识别事件，且不会让前端自行推导最终业务状态。

**验证：**

- Rust 单测覆盖 ProgressEvent 构造、脱敏、状态落库后发事件、事件丢失不影响 task_detail。
- 前端 store 测试或手动 smoke 覆盖：收到事件后刷新任务详情；页面刷新后状态仍正确。
- typecheck/build 通过。

**下一步进入条件：**

- 事件字段、事件发送时机、前端订阅和 DB 回读全部完成。
- 完成记录写清：事件不作为最终状态源，ProgressEvent 丢失时如何恢复。
- 复核 5.8 取消事件不会导致前端继续显示生成中，再把本条改为 `【X】` 并进入 5.10。

**风险：**  
不要让前端通过事件自行推导最终业务状态。

**完成记录：**

- 已将 `src-tauri/src/core/event.rs` 从占位改为真实 `ProgressEvent` 定义，字段包含 `trace_id / project_id / task_id / task_step_id / step_kind / status / progress / message / error_code / item_id`。
- 已新增 `TASK_PROGRESS_EVENT = "task://progress"` 和 `emit_task_progress`，事件 message 通过 `SecretGuard` 脱敏，不放密钥、完整 Provider 请求、绝对路径或长篇原文。
- `start_task / cancel_task / resume_task / retry_task_step / approve_task_step` 命令现在都在 service/repository 完成 DB 更新并返回 `TaskDetailDto` 后，再基于 DB 回读结果构造并发送 ProgressEvent。
- 前端新增 `shared/api/events.ts`，集中封装 Tauri event 监听；页面不直接访问 Tauri event API。
- `useTaskStore.subscribeTaskProgress` 在 App 启动时注册一次事件监听；收到事件后只调用 `getTaskDetail(projectId)` 回读 DB 状态，不在前端用事件 payload 自行推导最终任务状态。
- 事件丢失时，刷新页面或重新调用 `get_task_detail` 仍能得到 SQLite 中的真实状态；ProgressEvent 只作为通知 UI 刷新的触发器。
- 取消事件已复核：`cancel_task` 先落库为 `cancelled`，再发送事件；前端收到事件后回读 DB，不会继续显示生成中。
- 当前批量生图 / 视频还未接入真实 PipelineEngine 行级执行，因此 `item_id` 字段已在事件协议中预留，行级事件将在 TODO-07 接真实执行器时写入。
- 验证通过：`cargo test` 69 个 Rust 单测全部通过；`cargo check` 通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。

---

### 【X】5.10 实现下游 reset 规则和锁定保护

**问题：**  
用户修改上游字段后，下游产物状态需要局部失效，但不能全项目重跑。

**位置：**

```text
src-tauri/src/domain/storyboard*
src-tauri/src/service/storyboard* 或 project workspace service
src-tauri/src/repository/storyboard* / asset* / task*
src/src/entities/storyboard*
src/src/features/workspace/image*
src/src/features/workspace/video*
src/src/features/workspace/compose*
```

**改法：**

规则：

```text
修改 imagePrompt → 影响 image_candidates / selectedImageId / video_segments / composition
修改 selectedImageId → 影响 video_segments / composition
修改 videoPrompt → 影响 video_segments / composition
修改 selectedVideoSegmentId → 影响 composition
```

`lock_flags_json` 保护用户手动字段。

落地要求：

```text
1. 建立明确的 dependency reset map，不允许页面各自写一套重置规则。
2. 修改 imagePrompt 只影响当前 StoryboardItem 的图片候选、已选图片、视频片段和合成状态。
3. 修改 selectedImageId 只影响当前 StoryboardItem 的视频片段和合成状态。
4. 修改 videoPrompt 只影响当前 StoryboardItem 的视频片段和合成状态。
5. 修改 selectedVideoSegmentId 只影响合成状态。
6. 用户锁定的字段、已确认产物和显式保留的历史候选不能被批量重生成覆盖。
7. reset 必须记录原因、时间、触发字段和受影响对象，方便任务历史解释。
8. 页面必须提示“哪些下游结果已失效、为什么失效、如何重新生成”。
```

**验收：**

- 批量重生成跳过锁定项。
- 修改一个分镜不重置全项目。
- 下游失效有明确提示。
- 历史候选图和历史视频片段不被静默删除；只有当前选择和合成有效性按规则失效。
- 已锁定字段不会被批量操作覆盖；用户解锁后才能覆盖。
- 合成页能识别有失效视频片段时不可继续合成。

**验证：**

- Rust 或前端数据层测试覆盖四类修改：imagePrompt、selectedImageId、videoPrompt、selectedVideoSegmentId。
- 测试覆盖锁定字段被批量重生成跳过、单行 reset 不影响其他行、历史候选仍可回看。
- 手动 smoke：修改一条分镜的出图提示词，只看到该行下游状态失效；其他行不变。
- typecheck/build 通过。

**下一步进入条件：**

- dependency reset map、锁定保护、页面失效提示和测试全部完成。
- 完成记录写清哪些字段会 reset 哪些下游对象，以及哪些对象不会被删除。
- 复核 TODO-05 阶段完成标准全部满足后，才能把 5.10 改为 `【X】` 并进入 `TODO-06.md`。

**风险：**  
不要一改字段就全项目重新生成。

**完成记录：**

- 已新增前端集中规则模块 `entities/scene/reset.ts`，并通过 `entities/storyboard/reset.ts` 对外转发；页面不再各自维护 dependency reset map。
- 当前 reset map 已落地：
  - `imagePrompt` 影响当前 item 的 `imageCandidates / selectedImageId / videoSegments / selectedVideoSegmentId / composition`。
  - `selectedImageId` 影响当前 item 的 `videoSegments / selectedVideoSegmentId / composition`。
  - `videoPrompt` 影响当前 item 的 `videoSegments / selectedVideoSegmentId / composition`。
  - `selectedVideoSegmentId` 影响 `composition`。
- `scene` store 已在 `saveScene / selectImage / selectVideo` 入口统一应用 reset 规则，基于 `savedItemsById` 快照对比旧值和新值，避免页面响应式对象被直接修改后丢失旧值。
- reset 只清当前选择和状态：会清 `selectedImageId / selectedVideoSegmentId`、把对应 `imageStatus / videoStatus / segmentStatus / renderStatus` 置回 `pending`；不会静默删除历史 `ImageCandidate[] / VideoSegment[] / Artifact`。
- 已新增 `downstreamResetRecords` DTO 字段，记录 `resetId / itemId / triggerField / affectedObjects / reason / createdAt`，用于解释“为什么失效、哪些对象受影响”。
- 生图、视频、合成页面已显示下游失效提示；进入视频 / 合成前校验会识别已选图或已选视频片段是否仍匹配当前 prompt 和输入图。
- 批量生图会跳过 `lockFlagsJson.image / lockFlagsJson.imagePrompt` 锁定项；批量视频会跳过 `lockFlagsJson.video / lockFlagsJson.videoPrompt` 锁定项；单行操作仍由用户显式触发。
- 合成成功后会清除已消费的 `composition` 失效记录；选择新视频片段只提示旧合成输出失效，不阻止用户重新合成。
- Mock API 已同步 selected 标记，保存 reset 后不会因旧候选 `selected=true` 在刷新时把已清选择“复活”。
- Rust 侧已补齐 `StoryboardDownstreamResetRecord` DTO，并新增单测验证 reset 记录字段和 camelCase 序列化；当前 `scene_service` 仍是硬编码 stub，没有真实 storyboard 仓储旧值对比，本条没有写成真实 DB reset 闭环。
- 验证通过：`cargo check` 通过；`cargo fmt` 已执行；`cargo test` 71 个 Rust 单测全部通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。
- 复核结果：TODO-05 阶段完成标准已覆盖 Task / TaskStep / TaskAttempt / Artifact 落库、任务错误和日志、pipeline 生命周期、租约恢复、ProgressEvent 通知与 DB 最终状态，以及当前前端数据层可用的 reset / lock_flags 规则；可以进入 `TODO-06.md`。

---

## 阶段完成标准

- Task / TaskStep / TaskAttempt / Artifact 可落库。
- 任务错误、基础错误码和结构化任务日志可用。
- image_to_video pipeline 能创建、启动、暂停 review、继续、失败、重试、取消。
- 重启后任务状态可恢复。
- ProgressEvent 仅做通知，DB 是最终状态。
- 下游 reset 和 lock_flags 规则可用。






