# TODO-09：导出备份、诊断与测试补齐

> 目标：让应用从“能跑”变成“可恢复、可诊断、可验证”。  
> 本文件来自 `doc/功能模块/21-导出与项目备份.md`、`doc/底层设计/09/15/16/19`。

---

## 阶段目标

建立：

```text
final.mp4 导出
项目包导出/导入
备份恢复
导出 / 诊断错误恢复策略
错误码总表补齐
日志归档与脱敏验收
诊断包
测试体系
E2E smoke test
```

---

## 本阶段范围

包含：

- 导出 final.mp4。
- 打开输出目录。
- 导出封面、字幕、项目包。
- 导入项目包。
- 导出安全扫描。
- 基于 TODO-02 / TODO-05 补齐导出、导入、诊断、模板、备份相关错误码和恢复动作。
- 日志归档、截断、脱敏验收。
- 诊断包。
- Rust / 前端 / Tauri / Provider / FFmpeg / 模板 / E2E 测试。

不包含：

- 桌面安装包签名。
- 自动更新正式实现。
- 云端同步。

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

### 【】9.1 实现 final.mp4 导出

**问题：**  
合成成功不等于用户能方便拿到成片。

**改法：**

导出流程：

```text
检查 CompositionTask 成功
检查 exports/final.mp4 存在
目标路径 PathGuard 校验
复制到用户选择目录
写 history
提供打开输出目录
```

**验证：**

- final.mp4 可导出。
- 导出失败有明确错误。
- 作品内部导出记录能打开该作品输出目录。

**风险：**  
导出目标不能绕过安全检查覆盖敏感文件。

---

### 【】9.2 实现项目包导出

**问题：**  
用户需要备份和迁移项目，但项目包不能泄露密钥和绝对路径。

**改法：**

项目包包含：

```text
manifest.json
db_export.json
project snapshot
assets/
templates/
cover/
subtitles/
exports/final.mp4
```

导出前扫描：

```text
API Key
Authorization header
keyring 内容
绝对路径
工作区外引用
zip 路径穿越
```

**验证：**

- 项目包能依据 asset_references 收集素材。
- 不包含真实密钥。
- 不包含系统绝对路径。
- 命中风险时阻断导出。

**风险：**  
导出包不得包含 cache、临时文件、keyring secret、Provider 完整请求头。

---

### 【】9.3 实现项目包导入

**问题：**  
导入包是不可信输入，必须防 Zip Slip 和恶意资源。

**改法：**

导入流程：

```text
验证 manifest
扫描路径
拒绝绝对路径
拒绝 ../
解压到隔离临时目录
校验 db_export schema
复制到新项目工作区
写入 SQLite
提示重新录入密钥
```

**验证：**

- Zip Slip 被拒绝。
- 导入后文件引用可恢复。
- key_alias 存在但真实密钥需重新配置。

**风险：**  
不信任包内任何脚本，不自动执行包内模板 JS。

---

### 【】9.4 实现备份恢复

**问题：**  
用户数据需要可恢复，但备份不能带密钥。

**改法：**

备份包含：

```text
SQLite dump 或受控 db export
workspace 项目文件
配置快照，不含 secret
migrations version
```

恢复后检查：

```text
migration
文件引用
key_alias
sidecar
模板
PromptSkill
```

**验证：**

- 备份可恢复到新工作区。
- 恢复后项目能打开。
- 真实密钥需重新录入。

**风险：**  
不要把 keyring 内容打进备份。

---

### 【】9.5 补齐导出 / 诊断错误恢复策略

**问题：**  
TODO-02 已定义 `AppErrorDto`，TODO-05 已建立任务错误底座；本阶段要补齐导出、导入、备份和诊断场景，否则用户遇到数据迁移或导出失败时无法恢复。

**改法：**

补齐场景：

```text
export.target_denied
export.final_missing
export.secret_detected
import.package_invalid
import.zip_slip_detected
backup.restore_failed
diagnostic.secret_detected
diagnostic.media_permission_required
```

每个错误必须配置：

```text
kind
is_retryable
recoverAction
user_message_i18n_key
```

**验证：**

- 导出、导入、备份、诊断失败都返回 AppErrorDto。
- 前端可根据 recoverAction 显示重试、重新选择目录、重新录入密钥或打开诊断入口。
- 错误 detail 已脱敏。

**风险：**  
不要把项目包路径、用户绝对路径、key_alias 对应真实密钥或诊断扫描命中内容原样展示给用户。

---

### 【】9.6 维护错误码总表与恢复策略

**问题：**  
前面阶段会逐步增加错误码，本阶段必须把错误码总表补齐并固化，避免文档、前端 i18n、Rust enum 和任务 error_json 漂移。

**改法：**

覆盖领域：

```text
provider.auth_failed
provider.rate_limited
provider.timeout
provider.content_policy
provider.invalid_response
storage.path_denied
storage.file_missing
workflow.invalid_node_map
workflow.output_missing
ffmpeg.sidecar_missing
ffmpeg.probe_failed
ffmpeg.concat_failed
template.param_invalid
template.render_failed
db.migration_failed
export.target_denied
import.zip_slip_detected
diagnostic.secret_detected
```

**验证：**

- 每个错误有 kind、是否可重试、用户建议动作。
- 任务失败写入 task_steps.error_json。
- 前端错误文案和后端错误码一一对应。
- 未登记错误码不得直接透出原始错误。

**风险：**  
不要把 Provider 原始错误完整透出给用户。

---

### 【】9.7 扩展结构化日志与归档策略

**问题：**  
TODO-05 已有任务日志最小底座；本阶段要补齐应用日志、导出日志、诊断日志和归档策略，保证问题可定位且不会泄密。

**改法：**

日志字段：

```text
trace_id
project_id
task_id
task_step_id
step_kind
item_id
image_id
segment_id
provider_id
provider_kind
vendor
model_name
error_code
duration_ms
retry_count
relative_path
```

日志文件：

```text
workspace/logs/app.log
workspace/logs/error.log
workspace/projects/proj_xxx/tasks/task_xxx/logs/task.log
workspace/projects/proj_xxx/tasks/task_xxx/logs/ffmpeg.log
workspace/projects/proj_xxx/exports/export.log
workspace/logs/diagnostic.log
```

**验证：**

- 日志可关联一次完整任务。
- FFmpeg stderr 截断保存。
- SecretGuard 二次扫描。
- 日志大小有上限和轮转策略。
- 诊断包只收集脱敏日志片段。

**风险：**  
日志不得打印 API Key、Bearer Token、完整请求头、绝对用户路径、长篇原文全文。

---

### 【】9.8 实现诊断包

**问题：**  
用户反馈问题需要诊断，但诊断包是泄密高风险入口。

**改法：**

默认包含：

```text
应用版本
系统信息摘要
配置摘要，不含 secret
错误日志脱敏片段
任务步骤状态
错误码
sidecar 状态
```

默认不包含：

```text
API Key
Authorization Header
keyring 内容
用户原始小说全文
私密媒体文件
绝对路径
```

**验证：**

- 导出前 SecretGuard 扫描。
- 命中疑似密钥阻断导出。
- 用户可选择是否附带媒体文件。

**风险：**  
诊断包不能成为绕过导出安全的通道。

---

### 【】9.9 建立测试体系

**问题：**  
桌面 AI 生产链路长，只靠手动点页面无法保证质量。

**改法：**

测试层：

```text
Rust 单元测试
Repository / Migration 测试
Service 集成测试
Provider mock 测试
ModelRegistry / WorkflowRegistry 测试
Workflow preset contract test
FFmpeg smoke test
Template render test
Tauri command contract test
前端组件测试
端到端生成 smoke test
打包后自检测试
```

**验证：**

- typecheck/build/cargo test 可运行。
- Mock E2E 跑通主线。
- 真实能力有可控 smoke test。

**风险：**  
真实 Provider 测试可能产生费用，必须默认 mock，真实测试需显式确认。

---

### 【】9.10 建立 MVP smoke test 清单

**问题：**  
页面能打开不等于主线闭环完成。

**改法：**

必须跑通：

```text
打开桌面应用
在“我的作品”点击“开始创作”，自动创建作品草稿
输入一段文字
生成 / 编辑分镜
进入生图表格
每行生成候选图
每行选择最终图
进入视频阶段
生成 / 确认视频片段
进入合成阶段
生成最终视频
打开输出目录
作品工作台内部任务历史和导出记录可回看
```

**验证：**

- `pnpm --dir app/frontend typecheck` 通过。
- `pnpm --dir app/frontend build` 通过。
- `cargo test` 通过。
- 主线手动 smoke test 通过。

**风险：**  
Mock 闭环和真实闭环必须标识清楚。

---

## 阶段完成标准

- final.mp4 可导出和打开目录。
- 项目包导出/导入有安全扫描。
- 导出、导入、备份、诊断错误码和恢复策略补齐。
- 日志归档、截断、轮转和诊断包脱敏可用。
- 关键测试和 MVP smoke test 有固定清单并可执行。






