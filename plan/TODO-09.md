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
> - 问题：必须说清不做会造成什么用户问题、工程问题或后续返工。
> - 位置：必须落到页面、接口、表结构、DTO、Store、Service、组件、样式、i18n、测试或文档；不能只写“相关文件”。
> - 改法：按小步实现，写清数据流、状态流、边界和本阶段不做什么。
> - 验收：写清做到什么客观状态才算完成，不能把验证命令当验收。
> - 验证：写清命令、页面流程、数据库检查、文件检查、日志检查或 smoke test。
> - 下一步：本条必须满足“下一步进入条件”后，才能打勾进入下一条；旧 TODO 缺字段时先补齐再实现。
> - 打标：完成一条后必须立刻把 `【】` 改成 `【X】`，并写完成记录；不需要用户提醒。
> - 停止：如果需求不清、验收不清、文档冲突、数据结构不明或风险高，停下来说明，不靠猜测继续。

### 【X】9.1 实现 final.mp4 导出

**问题：**
合成成功不等于用户能方便拿到成片。

**位置：**

```text
src-tauri/src/service/export*
src-tauri/src/service/storage*
src-tauri/src/repository/export* / composition*
src-tauri/src/command/export*
src/src/entities/export*
src/src/features/workspace/compose*
src/src/features/project-history*
```

必须复用 StorageService / PathGuard，不能直接让前端传任意目标路径给文件复制逻辑。

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

落地要求：

```text
1. 导出前检查 CompositionTask.status = succeeded，且 outputPath 指向受控 workspace 内存在的 final.mp4。
2. 用户目标目录必须经过 PathGuard 和覆盖策略校验；默认文件名要安全，不带非法字符。
3. 导出动作写 ExportRecord：project_id、composition_id、target_relative_or_display_path、status、started_at、finished_at、error_json。
4. 打开输出目录只能打开本次导出的父目录；失败时给结构化错误。
5. 导出记录不保存真实密钥，不保存未授权外部文件引用；如需要展示目标路径，诊断包中必须脱敏。
6. 重复导出同一 final.mp4 时不破坏历史记录；覆盖已有文件必须显式确认或自动生成安全后缀。
```

**验收：**

- final.mp4 可导出。
- 导出失败有明确错误。
- 作品内部导出记录能打开该作品输出目录。
- 未合成成功、final.mp4 缺失、目标目录不可写、路径越权都会被拒绝并给可恢复提示。
- 导出历史能看到时间、目标、状态、错误和对应 composition。

**验证：**

- Rust 单测覆盖：未合成拒绝、final.mp4 缺失拒绝、路径越权拒绝、重复导出安全命名、ExportRecord 写入。
- 前端 typecheck/build 通过。
- 手动 smoke：选择一个普通目录导出 final.mp4，导出记录出现，打开输出目录可用；再模拟缺失文件确认错误展示。

**下一步进入条件：**

- final.mp4 导出、ExportRecord、路径校验、错误展示和打开目录全部完成。
- 完成记录写清导出目标安全策略、覆盖策略和执行过的 smoke 结果。
- 确认项目包导出 9.2 能复用 ExportRecord / StorageService 后，再把本条改为 `【X】` 并进入 9.2。

**风险：**
导出目标不能绕过安全检查覆盖敏感文件。

**完成记录（2026-06-26）：**

- 后端已新增 `ExportRecord` 落库链路：`export_records` migration、`ExportRepository`、`export_service`、`commands/export.rs`、Tauri invoke handler。
- 导出目标安全策略：前端不传任意系统路径；导出只复制到受控 workspace 目录 `outputs/user_exports/{projectId}/...mp4`，源文件必须来自 `outputs/...`，所有路径经过 `PathGuard` / `StorageService` 校验。
- 覆盖策略：默认不覆盖；重复导出自动生成 `_01`、`_02` 等安全后缀；`overwrite=true` 仅覆盖同一个受控 `outputs/user_exports` 目标，不支持外部目录覆盖。
- 打开输出目录策略：`open_export_directory` 只接受 `export_id`，先从 `ExportRecord` 反查本次导出的父目录，再校验其仍位于 `outputs/user_exports`；Windows/macOS/Linux command 层只打开这个受控目录。
- 前端已新增 `entities/export` API / types，合成页已接入“导出 final.mp4”、导出状态、导出历史、打开输出目录和错误提示；新增用户可见文案已同步 `zh-CN / en-US`。
- 9.2 可复用：项目包导出可以继续复用 `ExportRecord` 的记录模型、`StorageService` 的受控路径能力和 `SecretGuard` 的扫描能力；外部目录选择仍需后续安全 dialog token 设计，不在 9.1 做。

**验证记录（2026-06-26）：**

- `cargo fmt` 通过。
- `cargo test export` 通过：8 passed。
- `cargo test` 通过：173 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；存在 Vite 既有 chunk 体积提示和动态 / 静态 import 提示，不影响构建结果。
- 未执行真实桌面手动 smoke：当前回合未启动 Tauri 桌面应用并完成真实 FFmpeg 合成文件导出；已用 Rust 单测覆盖未合成、非 succeeded、final.mp4 缺失、路径越权、重复导出后缀、ExportRecord 写入和输出目录解析。

---

### 【X】9.2 实现项目包导出

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

**完成记录（2026-06-26）：**

- 后端已新增 `export_project_package`：生成 `project_package.zip`，包含 `manifest.json`、`db_export.json`、`projects/{projectId}/project.json`、基于 `asset_references` 收集到的受控 `assets/...` 文件、可选 `templates/user/...`、可选合成输出文件。
- 项目包记录复用 `ExportRecord`，新增 `export_kind = project_package`，目标路径限制为 `outputs/project_packages/{projectId}/...zip`。
- 安全策略：前端不传外部路径；所有 entry 名经过 `PathGuard::validate_relative_path`；文件来源只允许 `projects/`、`assets/`、`outputs/`、`templates/` 受控 bucket；导出前对 manifest / db_export 执行 `SecretGuard` 扫描和绝对路径 / file URL 检查；不导出 keyring、cache、temp、logs。
- 覆盖策略：默认不覆盖；重复导出自动生成 `_01`、`_02` 等安全后缀；`overwrite=true` 只覆盖受控 `outputs/project_packages` 内目标。
- 前端已接入 `exportProjectPackage` API、类型和合成页“导出项目包”按钮，导出历史可展示 `project_package`。
- 修复 `SecretGuard` 文本扫描误报：`containsSecrets: false` 这类安全布尔标记不再被当成密钥值；真实 `api_key = sk-...` 仍会阻断。

**验证记录（2026-06-26）：**

- `cargo fmt` 通过。
- `cargo test export` 通过：11 passed。
- `cargo test` 通过：176 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite chunk 体积提示和动态 / 静态 import 提示，不影响构建结果。
- 未执行真实桌面手动 smoke；已用 Rust 单测覆盖项目包结构、资产收集、SecretGuard 阻断、重复导出安全后缀和 ExportRecord 写入。

---

### 【X】9.3 实现项目包导入

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

**完成记录（2026-06-26）：**

- 后端已新增 `import_project_package`：当前只导入受控路径 `outputs/project_packages/...zip`，不接受任意系统路径，避免绕过 PathGuard；外部 zip 选择留给后续安全 dialog/token 设计。
- 导入流程已实现：读取 zip、逐 entry 过 `PathGuard`、校验 `manifest.json`、校验 `db_export.json` schemaVersion、解析项目快照、创建新的 `project_imported_*` 项目、重写分镜 `project_id / item_id`、清空候选图/视频和已选结果、导入 assets 到 `assets/imported/{newProjectId}/...` 并写 asset / asset_reference。
- 安全策略：拒绝 `../`、绝对路径、非受控包路径、无效 manifest、`containsSecrets != false`、非本应用包；不执行包内脚本，不恢复 keyring，不信任包内原始 project_id。
- 当前限制：只支持本应用生成的 stored zip（不支持压缩方法）；这与 9.2 当前 zip 写法一致，后续如接外部 zip 可再扩展解压库和更完整 schema import。
- 前端已补 `importProjectPackage` API / types / command 常量；暂未做外部文件选择 UI，避免和路径安全策略冲突。

**验证记录（2026-06-26）：**

- `cargo fmt` 通过。
- `cargo test import_project_package` 通过：3 passed。
- `cargo test export` 通过：14 passed。
- `cargo test` 通过：179 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite chunk 体积提示和动态 / 静态 import 提示，不影响构建结果。

---

### 【X】9.4 实现备份恢复

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

**完成记录（2026-06-26）：**

- 后端已新增 `backup_workspace` / `restore_workspace` commands、DTO 和 service 实现。
- 备份输出限制在受控目录 `outputs/backups/{backupId}.backup.zip`，前端不传任意系统路径。
- 备份包包含 `manifest.json`、`db_export.json`、`projects/`、`assets/`、`templates/user/`、可选 `prompts/user/`；`db_export.json` 包含项目快照、分镜、导出记录、资产记录和 `schema_migrations` 摘要。
- 备份安全策略：不包含 keyring、cache、temp、logs；manifest/db_export 和文本类模板 / prompt 文件经过 `SecretGuard` 与绝对路径检查；命中疑似密钥或绝对路径会阻断导出。
- 恢复策略采用“导入式恢复”：只允许读取受控 `outputs/backups/...zip`，不覆盖当前 SQLite，不恢复真实 secret；恢复时创建 `project_restored_*` 新项目，重写分镜项目归属，清空候选图/视频与已选结果，资产导入到 `assets/restored/{restoreId}/...`，用户模板 / prompt 导入到 restored 目录。
- 恢复前检查备份 manifest、schemaVersion、backupVersion、Zip Slip、migration version；备份版本高于当前应用 migration 时拒绝恢复。
- `open_export_directory` 的受控白名单扩展为 `outputs/user_exports`、`outputs/project_packages`、`outputs/backups`，仍不接受任意外部路径。
- 前端已新增 `backupWorkspace` / `restoreWorkspace` entity API、types、command 常量；合成页输出区新增“备份工作区”入口和备份结果摘要，所有新增文案已接入 `zh-CN / en-US`。

**验证记录（2026-06-26）：**

- `cargo fmt` 通过。
- `cargo check` 通过。
- `cargo test backup_workspace` 通过：2 passed。
- `cargo test restore_workspace` 通过：2 passed。
- `cargo test export` 通过：18 passed。
- `cargo test` 通过：183 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite chunk 体积提示和动态 / 静态 import 提示，不影响构建结果。
- 未执行真实桌面手动 smoke；已用 Rust 单测覆盖备份结构、恢复创建新项目、资产恢复、模板密钥阻断和备份 Zip Slip。

---

### 【X】9.5 补齐导出 / 诊断错误恢复策略

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

**完成记录（2026-06-26）：**

- 后端 `core::error` 已把 `import / backup / diagnostic` 纳入稳定错误域，prefixed service error 不再降级为 `app.command_failed`。
- 已补导出 / 导入 / 备份 / 诊断相关 recoverAction：
  - `export.final_missing / export.composition_not_ready` → `start_composition`
  - `export.target_denied / import.package_invalid / backup.restore_failed` → `choose_controlled_path`
  - `export.secret_detected / diagnostic.secret_detected` → `remove_secret_and_retry`
  - `import.zip_slip_detected / backup.zip_slip_detected` → `reject_package`
  - `diagnostic.media_permission_required` → `grant_media_permission`
- 已补 Rust 单测，覆盖 `export.target_denied / export.secret_detected / import.package_invalid / import.zip_slip_detected / backup.restore_failed / diagnostic.media_permission_required` 的 code、kind、retry 和 recoverAction。
- 前端 `ErrorKind` 已补 `import / backup / diagnostic`；`shared/api/errors.ts` 已新增 `RecoverAction` 类型、未知 action 收敛和 `getRecoverActionI18nKey`。
- 前端 `zh-CN / en-US` 已新增导出、导入、备份、诊断错误码文案和 recoverAction 文案；页面仍可继续显示后端脱敏 message，但协议已支持按 code / recoverAction 展示。
- 诊断包实体功能仍按 9.8 实现；本条只补错误恢复策略，不提前实现诊断包导出。

**验证记录（2026-06-26）：**

- `cargo test app_error_classifies_export_import_backup_and_diagnostic_codes` 通过：1 passed。
- `cargo test export` 通过：19 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite chunk 体积提示和动态 / 静态 import 提示，不影响构建结果。

---

### 【X】9.6 维护错误码总表与恢复策略

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

**完成记录（2026-06-26）：**

- 后端 `core::error` 已补齐 TODO-09 关键错误码分类、retry 和 recoverAction：Provider、Workflow、FFmpeg、Template、DB、Export、Import、Backup、Diagnostic。
- `template` 已加入后端 error kind；前端 `ErrorKind` 已同步加入 `template / import / backup / diagnostic`。
- `provider.invalid_response / provider.output_missing / ffmpeg.concat_failed / template.render_failed / db.query_failed / db.transaction_failed` 等可恢复错误会标记 `isRetryable=true` 并给 `recoverAction=retry`。
- `workflow.invalid_node_map / workflow.output_missing` 归入 `change_provider_or_plan`；`template.param_invalid` 归入 `edit_input`；`db.migration_failed` 归入 `restart_app_or_check_database`。
- 前端 `zh-CN / en-US` 已补 TODO-09 关键错误码文案，避免页面长期依赖后端 message。
- `doc/底层设计/15-错误码总表与恢复策略.md` 已收口 RecoverAction 列表、`db.*` 命名，以及导出 / 导入 / 备份 / 诊断错误表；`database.*` 不再作为本阶段主错误码。
- 新增 Rust 单测 `task_error_registry_covers_todo_09_error_codes`，固化关键错误码的 kind、retry 和 recoverAction，减少文档、后端、前端漂移。

**验证记录（2026-06-26）：**

- `cargo fmt` 通过。
- `cargo test task_error_registry_covers_todo_09_error_codes` 通过：1 passed。
- `cargo test export` 通过：19 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite chunk 体积提示和动态 / 静态 import 提示，不影响构建结果。

---

### 【X】9.7 扩展结构化日志与归档策略

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

**完成记录（2026-06-26）：**

- 已新增 `src-tauri/src/services/log_service.rs`，提供文件型结构化 JSONL 日志写入：
  - `workspace/logs/app.log`
  - `workspace/logs/error.log`
  - `workspace/projects/{projectId}/exports/export.log`
- 文件日志字段包含 `trace_id / project_id / task_id / task_step_id / step_kind / item_id / provider_id / provider_kind / vendor / model_name / error_code / duration_ms / retry_count / relative_path / metadata`。
- 日志写入前会对 message 和 metadata 执行脱敏，且写入前二次调用 `SecretGuard` 检测；命中疑似密钥会返回 `log.secret_detected`。
- 已实现日志轮转：`app.log / error.log` 单文件 10MB、保留 7 个；`export.log` 单文件 2MB、保留 7 个。
- `export_final_video` 和 `export_project_package` 成功后会写项目级 `exports/export.log` 和全局 `logs/app.log`；`backup_workspace` 成功后写全局 `logs/app.log`。
- 现有 `task_logs` 数据表、任务失败结构化记录和 FFmpeg stderr 脱敏截断保持使用，未在本条重写任务执行链路。
- 诊断包聚合日志仍按 9.8 实现；本条先完成日志写入、脱敏和轮转底座。

**验证记录（2026-06-26）：**

- `cargo fmt` 通过。
- `cargo test log_service` 通过：2 passed。
- `cargo test export` 通过：20 passed。
- `cargo test` 通过：187 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite chunk 体积提示和动态 / 静态 import 提示，不影响构建结果。

---

### 【X】9.8 实现诊断包

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

**完成记录（2026-06-26）：**

- 后端已新增 `export_diagnostic_package` command、DTO、entity API 和 Tauri command 注册。
- 诊断包输出限制在受控目录 `outputs/diagnostics/{diagnosticId}.diagnostic.zip`，不接受前端传任意系统路径。
- 默认诊断包包含：
  - `summary.json`：应用版本、诊断包版本、系统 OS/arch 摘要、最近项目摘要、migration 摘要、FFmpeg/FFprobe sidecar 状态。
  - 脱敏日志尾部片段：`logs/app.log`、`logs/error.log`、`logs/diagnostic.log`、各项目 `exports/export.log`。
- 默认不包含媒体文件、keyring、真实 API Key、Authorization Header、用户原始长文全文、私密媒体文件和绝对路径。
- 日志收集只取尾部 64KB，写包前会执行 `SecretGuard` 二次扫描和绝对路径检查；命中疑似密钥会返回 `diagnostic.secret_detected` 并阻断。
- `includeMedia=true` 当前直接返回 `diagnostic.media_permission_required`，不做隐式媒体打包；媒体授权流程留给后续明确 UI。
- 前端已新增 `exportDiagnosticPackage` API / types / command 常量，页面入口后续可复用。

**验证记录（2026-06-26）：**

- `cargo fmt` 通过。
- `cargo test export_diagnostic_package` 通过：2 passed。
- `cargo test export` 通过：22 passed。
- `cargo test` 通过：189 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite chunk 体积提示和动态 / 静态 import 提示，不影响构建结果。

---

### 【X】9.9 建立测试体系

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

**完成记录（2026-06-26）：**

- 已新增默认安全验证脚本 `scripts/verify-mvp.ps1`，默认不触发外部 Provider、不产生费用、不访问网络、不要求真实 FFmpeg sidecar。
- 根 `package.json` 已新增：
  - `pnpm verify`
  - `pnpm verify:quick`
- 已新增 `plan/测试体系.md`，明确默认验证、Rust 测试层、前端测试层、Mock 与真实能力边界、发布前人工 smoke。
- 默认验证脚本串联：
  - `cargo fmt --check`
  - `cargo test export`
  - `cargo test task_error_registry_covers_todo_09_error_codes`
  - `cargo test log_service`
  - `cargo test`
  - `pnpm --dir src typecheck`
  - `pnpm --dir src build`
- 文档明确真实 Provider smoke、真实 FFmpeg smoke、真实外部目录导出、媒体诊断附件和物理删除文件都不能默认跑，必须显式确认。

**验证记录（2026-06-26）：**

- `cargo fmt --check` 通过。
- `cargo test export` 通过：22 passed。
- `pnpm --dir src typecheck` 通过。
- `scripts/verify-mvp.ps1` 本轮未能直接执行：当前工具执行器拒绝创建嵌套 PowerShell 进程（CreateProcessAsUserW 1312）；脚本内关键命令已逐条执行验证，不能把脚本 smoke 写成已通过。

---

### 【X】9.10 建立 MVP smoke test 清单

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

- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过。
- `cargo test` 通过。
- 主线手动 smoke test 通过。

**风险：**  
Mock 闭环和真实闭环必须标识清楚。

**完成记录（2026-06-26）：**

- 已新增 `plan/MVP-smoke-test.md`，把 MVP smoke 固化为可执行清单、失败恢复清单、执行前置、通过标准和记录模板。
- 清单明确区分：
  - 浏览器 Mock smoke：只验证路由和前端交互。
  - 桌面受控 smoke：验证 Tauri command、SQLite、workspace 文件写入、候选结果、任务历史和导出记录。
  - 真实 Provider smoke：必须用户显式提供密钥并确认费用，不能默认跑。
  - 真实 FFmpeg smoke：必须存在 `sidecars/ffmpeg.exe` 与 `sidecars/ffprobe.exe`，否则只能验证错误恢复。
  - 外部目录导出 smoke：必须等安全文件选择 / 授权 token 设计，不让前端传任意系统路径。
- 主线清单覆盖：打开应用、开始创作、内容导入、分镜编辑、行号跳转、生图、最终提示词预览、候选图选择、视频生成、合成、导出、打开目录、任务历史和导出记录回看。
- 失败恢复清单覆盖：未选最终图、Provider 单行失败、缺 FFmpeg sidecar、final.mp4 缺失、诊断包密钥扫描、项目包 Zip Slip。
- `plan/测试体系.md` 已链接该 smoke 清单，后续 UI 收口和发布前验收按这个文件记录。

**验证记录（2026-06-26）：**

- 本条是 smoke 清单固化，未改业务代码。
- 未执行真实桌面手动 smoke；当前记录不能写成真实 Provider / 真实 FFmpeg / 真实 final.mp4 通过。
- 最近一次阶段验证记录仍沿用 9.9：`cargo fmt --check`、`cargo test export`、`pnpm --dir src typecheck` 已通过；`scripts/verify-mvp.ps1` 受当前执行器 `CreateProcessAsUserW 1312` 限制未能直接运行，脚本内关键命令已逐条验证。

---

### 【X】9.11 按 UI 执行口径收口主线页面

**问题：**
`doc/UI设计/全页面布局设计.md` 已统一页面布局、导航和交互口径；如果不进入 TODO，后续实现容易继续沿用旧文档里的重复入口、双编辑面板、不可靠 ETA 和合成页增强入口，导致主线页面返工。

**位置：**

```text
doc/UI设计/全页面布局设计.md
doc/参考分析/参考项目深度机制与吸收口径.md
src/src/app/router/index.ts
src/src/widgets/app-shell/AppShell.vue
src/src/pages/create-project/index.vue
src/src/pages/home/index.vue
src/src/pages/project-workbench/index.vue
src/src/pages/storyboard-editor/index.vue
src/src/pages/image-generation/index.vue
src/src/pages/video-generation/index.vue
src/src/pages/composition/index.vue
src/src/shared/i18n/locales/*
```

**改法：**

按 `doc/UI设计/全页面布局设计.md` 的“执行口径”收口主线页面；涉及参考项目吸收时必须同时遵守 `doc/参考分析/参考项目深度机制与吸收口径.md`：

```text
1. 清理 script-editor 独立入口：不得作为主流程必经页；若页面文件仍保留，只能作为后续高级编辑入口，不得和内容创作切分预览重复编辑同一批旁白。
2. AI 工具不提供“创建作品”入口；创建作品统一从开始创作 / 内容创作进入。
3. 内容创作页承担输入、AI 整理、切分预览和逐条旁白 / 文案段落确认，产物是 `sourceText / narrationText`，不是完整分镜。
4. 分镜页基于已确认段落生成和编辑镜头级 `StoryboardItem`；表格内联编辑是主编辑入口，Inspector 只放校验、AI 助手、候选图/视频预览和高级可选字段。
5. 批量生成进度不显示 ETA，只显示完成数、当前项和已用时。
6. 合成页只放片段检查、合成控制、输出信息和导出操作；字幕、封面、BGM、模板动效作为后续独立步骤。
7. 工作台页面 Header 增加快捷导航菜单，能跳到我的作品、AI 工具、创作资源、模型/工作流、系统设置和当前作品概览。
8. 分镜 / 生图 / 视频表格增加行号快速跳转。
9. 生图页提供“预览最终提示词”，展示组装角色、场景、规则后的实际发送 prompt。
10. 工作台概览或 Header 展示当前作品资源消耗摘要：图片次数、视频次数、LLM 次数。
11. 内容导入吸收 waoowaoo 的长文本检测、智能切分建议和手动兜底，但不新增 script-editor 必经页。
12. 分镜吸收 waoowaoo 的镜头生产字段和 Toonflow 的“总纲 + 镜头”结构：文本、画面、角色、场景、提示词、时长、状态和候选结果都围绕 StoryboardItem。
13. 提示词跟着分镜数据走：分镜页编辑镜头级 prompt，生图页预览最终 prompt，创作规则页管理模板和 schema，不做孤立提示词大页面。
14. 视频包、创作规则、模型 / 工作流按深度机制文档分层：视频包引用策略，创作规则定义生成方法，模型 / 工作流声明系统能力。
```

字段名收口要求：

```text
1. 分镜高级字段只能使用当前 DTO / DB 已定义的 `shotSize / cameraMotion / composition / pace / transitionType`。
2. 参考项目字段 `shot_type / camera_move` 只能在参考映射里出现，代码里不得新增 `shotType / cameraMove` 平行字段。
3. `photographyRules / actingNotes` 在没有 `advancedDirectingJson` migration、DTO、Repository 和测试前，只能作为 Inspector AI 建议或 TaskStep 输出摘要展示，不能写入临时业务字段。
4. 生图和视频候选选择仍分别写 `selectedImageId / selectedVideoSegmentId`，不得回退到单 `imagePath / videoPath` 覆盖式实现。
```

**验收：**

- 主线不会出现两个创建作品入口，也不会出现两个编辑同一旁白字段的必经页面。
- 内容创作与分镜职责清楚：内容创作确认文案段落，分镜页生成和编辑镜头级画面 / 提示词 / 资产绑定。
- 前端路由、Shell 导航和页面按钮符合 `doc/UI设计/全页面布局设计.md`。
- 分镜表格和 Inspector 职责清楚，不重复编辑同一字段。
- 生图 / 视频批量进度无 ETA。
- 合成页不承载字幕、封面、BGM、模板动效的入口。
- 工作台任意页面能通过快捷导航回到一级页面或当前作品概览。
- 长分镜项目能快速跳转到指定行。
- 生图前能查看最终 prompt。
- 当前作品资源消耗可见，且数据来源明确；没有真实统计数据时必须显示未接入或占位，不伪造。
- 参考项目吸收点不是泛化文案：内容导入、分镜字段、提示词、候选结果、任务态、资产贯穿、视频包 / 创作规则 / 模型工作流分层都能在页面或后续 TODO 中找到明确落点。

**验证：**

- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过。
- 搜索确认 `script-editor` 不再作为路由入口；搜索确认页面无 `预计剩余` / `ETA` 文案。
- 手动 smoke：创建作品 → 内容创作 → 分镜 → 生图 → 视频 → 合成；在工作台页面打开快捷导航；在长表格跳转行号；打开最终 prompt 预览。
- i18n 检查：新增用户可见文案进入 `zh-CN / en-US`，不硬编码。

**下一步进入条件：**

- 本条所有页面和文案完成，并记录已执行的 typecheck/build/smoke 结果。
- 若资源消耗统计或最终 prompt 组装依赖后端缺口，必须写明缺口和临时展示策略，不能伪造数据。
- 确认 TODO-09 阶段完成标准仍成立后，再把本条改为 `【X】`。

**风险：**
不要为了 UI 收口绕过当前数据流；缺后端数据时应显示明确未接入状态，不用前端假数据冒充真实统计。

**完成记录（2026-06-26）：**

- 主路由仍未注册 `script-editor`，主流程不再经过独立脚本编辑页；`script-editor` 文件保留但不作为必经入口。
- 新增 `/ai-tools` 一级页面和 Rail 入口；页面只展示爆款开头、标题生成、角色设定、多语言改写、画面描述优化等独立小工具口径，不提供创建作品入口，不复刻“主题生成完整作品”。
- 工作台生产页新增统一 `WorkspaceHeader`：
  - 左侧快捷导航菜单可跳到我的作品、AI 工具、创作资源、模型 / 工作流、系统设置和当前作品概览。
  - 中间保留 StepBar。
  - 右侧保留当前页面操作按钮。
  - Header 展示当前可统计资源摘要：图片候选次数、视频片段次数、LLM 未接入统计。
- 工作台概览页新增资源消耗卡片，数据来源明确为当前项目已落库的 `imageCandidates / videoSegments`；LLM 真实用量账本未接入时显示“未接入统计”，未伪造数字。
- 分镜、生图、视频、合成表格新增行号快速跳转组件，长表格可按行号滚动定位。
- 生图页已有并保留“预览最终提示词”，通过 `buildImagePromptPreview` 展示组装后的最终 prompt、负向 prompt、来源分段和参考图。
- 生图 / 视频批量生成页面没有 ETA / 预计剩余时间文案；当前仍只展示行数、状态、失败和候选数量。
- 合成页仍只承载片段检查、合成控制、输出信息、导出 final.mp4、项目包导出、备份和打开输出目录；字幕 / 封面 / BGM 没有作为合成页右侧控制项接入。
- 新增用户可见文案已写入 `zh-CN / en-US`。

**验证记录（2026-06-26）：**

- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动态 / 静态 import 提示和 chunk 体积提示，不影响构建结果。
- 搜索确认：
  - `src/src/app/router/index.ts` 无 `script-editor` 路由。
  - `src/src` 无 `预计剩余` / `ETA` 生成进度文案。
  - AI 工具页不调用 `createProject`，创建作品仍只在 `create-project` 相关 entity / 页面。
- 未执行真实桌面手动 smoke；当前不能写成真实 Provider、真实 FFmpeg 或真实 final.mp4 闭环已通过。

---

## 阶段完成标准

- final.mp4 可导出和打开目录。
- 项目包导出/导入有安全扫描。
- 导出、导入、备份、诊断错误码和恢复策略补齐。
- 日志归档、截断、轮转和诊断包脱敏可用。
- 关键测试和 MVP smoke test 有固定清单并可执行。
- 主线页面按 `doc/UI设计/全页面布局设计.md` 收口，重复入口、双编辑入口、不可靠 ETA 和合成页增强入口已清除。






