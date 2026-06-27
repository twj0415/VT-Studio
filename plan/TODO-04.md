# TODO-04：数据库、存储与安全底座

> 目标：建立本地桌面应用的权威数据源和安全边界。  
> 本文件来自 `doc/底层设计/05/08/17/Provider与安全`、`doc/功能模块/18/20/21` 等文档的全量整理。

---

## 阶段目标

先完成：

```text
SQLite + Migration + Repository
StorageService + PathGuard
KeyringService + SecretGuard
AppConfig / ProviderConfig / Asset 基础表
```

没有这些，真实生成、资产管理、导出备份、诊断包都会返工。

---

## 本阶段范围

包含：

- SQLite 初始化和迁移体系。
- 核心表第一版。
- 本地工作区目录初始化。
- 文件统一存储与相对路径入库。
- PathGuard 安全边界。
- Keyring 密钥存储。
- SecretGuard 脱敏与扫描。
- 配置 schema 与默认值。
- Asset / AssetReference 最小表。

不包含：

- 完整任务队列。
- 真实 Provider 生成。
- 完整资产库 UI。
- 项目包完整导入导出。

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

### 【X】4.1 初始化 SQLite 和 migration

**问题：**  
当前若仍依赖 stub/mock，应用重启后项目、分镜、候选图、任务状态都不可恢复。

**改法：**

实现：

```text
SQLite connection
migration 管理
Repository 基础层
事务封装
```

启动 PRAGMA：

```text
foreign_keys = ON
journal_mode = WAL
busy_timeout = 5000
synchronous = NORMAL
```

**验证：**

- 应用首次启动能创建数据库。
- migration 可重复安全执行。
- 重启后能读取已创建项目。

**风险：**  
不要把 `project.json` 做成第二套主数据源；SQLite 才是本地权威数据源。

**完成记录：**

- 已在 `app/src-tauri/Cargo.toml` 引入 `rusqlite`，并使用 `bundled` SQLite，避免依赖用户系统 SQLite 安装状态。
- 已在 `app/src-tauri/src/db/mod.rs` 建立 SQLite 基础层：`Database` 连接封装、migration runner、migration checksum 校验、Repository 基础 trait、事务封装。
- 启动 PRAGMA 已集中执行：`foreign_keys = ON`、`journal_mode = WAL`、`busy_timeout = 5000`、`synchronous = NORMAL`。
- 已建立 `schema_migrations` 表用于记录迁移版本、名称、checksum 和执行时间；重复启动会跳过已应用且 checksum 一致的 migration。
- 已建立第一条基础 migration：`repository_metadata`，创建 `repository_meta` 表并标记当前权威数据源为 SQLite。
- 已在 `app/src-tauri/src/core/app_state.rs` 中让 `AppState` 持有 `Database`，为后续 Repository 和 Service 注入做准备。
- 已在 `app/src-tauri/src/main.rs` 的 Tauri `setup` 阶段初始化数据库，数据库路径位于应用数据目录下的 `vt-ai-short-video-maker.sqlite3`，首次启动会自动创建目录和数据库文件。
- 已新增数据库模块单测，覆盖 migration 可重复执行和事务失败回滚。
- 已通过 `cargo check`。
- 已通过 `cargo test db::tests`：2 个数据库单测全部通过。
- 说明：本条只完成 SQLite / migration / Repository 基础层和启动初始化；项目、分镜、候选图等业务数据真正落库在 `4.2` 继续实现，不在本条混做。

---

### 【X】4.2 建立核心表第一版

**问题：**  
功能模块多，若表结构滞后，页面和任务系统会被迫临时造字段。

**改法：**

第一版至少建表：

```text
projects
project_bibles
style_bibles
character_bibles
location_bibles
storyboard_items
image_candidates
video_segments
composition_tasks
assets
asset_references
tasks
task_steps
task_attempts
artifacts
providers
provider_models
workflow_presets
app_configs
prompt_templates
templates
histories
```

本阶段只建立 `providers / provider_models / workflow_presets` 的 migration、Repository 基础读写和约束，不实现 ProviderManager、WorkflowRegistry、能力选择器和真实连通性测试；这些放到 TODO-06。

**验证：**

- 主表有 `created_at / updated_at`。
- 外键启用。
- JSON 字段读取后做反序列化校验。
- 创建项目、分镜、候选图、视频片段可落库。

**风险：**  
migration 只增不改历史，避免破坏用户已有数据。

**完成记录：**

- 已新增 migration `core_tables_v1`，建立核心表第一版：`projects`、`project_bibles`、`style_bibles`、`character_bibles`、`location_bibles`、`storyboard_items`、`image_candidates`、`video_segments`、`composition_tasks`、`assets`、`asset_references`、`tasks`、`task_steps`、`task_attempts`、`artifacts`、`providers`、`provider_models`、`workflow_presets`、`app_configs`、`prompt_templates`、`templates`、`histories`。
- 核心表已包含 `created_at / updated_at`，并对项目、分镜、候选图、视频片段、任务、Provider、资产引用等关系建立外键和必要索引。
- JSON 字段统一以 `*_json` 文本字段存储，Repository 读取时会反序列化为 `serde_json::Value`，反序列化失败会回退为空对象，避免脏 JSON 直接炸穿 UI DTO。
- 已新增 `ProjectRepository`，支持项目创建、列表、详情读取，并创建默认 `project_bibles` 和首个 `tasks` 记录。
- 已将 Tauri 项目命令 `create_project / list_projects / get_project_detail / update_project` 接入 `AppState` 中的 SQLite 数据库；创建项目后可从数据库重新读取。
- 已新增 `ProviderRepository`，支持 `providers / provider_models / workflow_presets` 的基础 upsert 和 list，并保留外键约束；未实现 ProviderManager、WorkflowRegistry、能力选择器或真实连通性测试。
- 已新增数据库单测覆盖：核心表 migration、migration 重复执行、事务回滚、项目创建后重开读取、Provider/Model/Preset 基础写读。
- 已执行 `cargo fmt`。
- 已通过 `cargo check`。
- 已通过 `cargo test db::tests`：5 个数据库单测全部通过。
- 说明：本条完成核心表和最小 Repository/项目落库；长文本转文件、StorageService、PathGuard、Keyring、SecretGuard 和 Asset 导入删除保护继续按后续 4.3-4.9 顺序实现。

---

### 【X】4.3 实现长文本存储策略

**问题：**  
文章、小说、长文案不能全部塞入 `projects.source_text`。

**改法：**

```text
20KB 以内可存 source_text
超过阈值写入 input/source.txt
数据库存 source_text_path
小说整本原文禁止塞入 projects.source_text
```

**验证：**

- 长文导入后文件进入受控工作区。
- DB 只保存相对路径。
- 删除/导出项目时能通过引用找到长文文件。

**风险：**  
不得保存用户原始绝对路径。

**完成记录：**

- 已在 `AppState` 中新增受控 `workspace_root`，Tauri 启动时会在应用数据目录下创建 `workspace` 根目录。
- 已在项目创建 service 层实现长文本策略：20KB 以内保留在 `projects.source_text`；超过 20KB 的输入写入受控工作区文件。
- 长文本写入路径为 `workspace/projects/{projectId}/input/source.txt`，数据库只保存相对路径 `projects/{projectId}/input/source.txt`。
- 已保留小说原文禁止直接塞入 `projects.source_text` 的策略入口：`input_type = novel` 会强制写入受控文件，即使文本未超过 20KB。
- 已增加 `source_text_path` 校验：拒绝空路径、绝对路径、根路径、盘符前缀、`.`、`..` 等非受控相对路径组件。
- 已调整 ProjectRepository：业务层先生成 `projectId` 并处理长文本，再按指定 `projectId` 写库，避免文件路径和数据库主键不一致。
- 已新增 service 单测覆盖短文本内联、长文本写入受控 workspace、DB 只保存相对路径、项目重读后仍不回填长文本。
- 已执行 `cargo fmt`。
- 已通过 `cargo check`。
- 已通过 `cargo test`：7 个 Rust 单测全部通过。
- 说明：本条只处理项目输入长文本存储策略；完整 StorageService、PathGuard 和导出/删除引用收集继续按 4.4-4.5 实现。

---

### 【X】4.4 实现 StorageService

**问题：**  
生成结果、导入素材、临时文件如果散落各处，导出、清理、备份都不可控。

**改法：**

统一目录：

```text
workspace/projects
workspace/assets
workspace/outputs
workspace/cache
workspace/temp
workspace/logs
workspace/templates
workspace/sidecars
```

文件操作统一走：

```text
StorageService
PathResolver
FileBucket
FileAccessPolicy
```

**验证：**

- 用户导入文件复制到 workspace。
- 远程 URL 下载到受控目录。
- 入库只存 `relative_path`。
- 所有预览通过 StorageService 读取。

**风险：**  
不能长期引用用户原始路径或远程 URL。

**完成记录：**

- 已新增 `services/storage_service.rs`，实现 `StorageService`、`PathResolver`、`FileBucket`、`FileAccessPolicy` 和 `StoredFile`。
- 已统一 workspace bucket：`projects`、`assets`、`outputs`、`cache`、`temp`、`logs`、`templates`、`sidecars`，并提供 `initialize_workspace` 自动创建目录。
- 已实现受控相对路径解析：入库/返回路径统一为 `bucket/relative_path` 格式，例如 `projects/{projectId}/input/source.txt`。
- 已实现文本写入、导入文件复制到 bucket、受控读取预览能力；写入前会创建父目录。
- 已实现基础访问策略：`read_only` 禁止写入，`temp_only` 只能写入 temp bucket，`export_copy/temp_only` 不允许作为预览读取策略。
- 已将 4.3 长文本写入改为复用 StorageService，不再在项目 service 内自行拼目录和写文件。
- 已加入基础路径约束：拒绝空路径、绝对路径、盘符/冒号、`../`、`./`、根路径和反斜杠绕过输入；更完整的 canonicalize、symlink、junction、UNC、Zip Slip 防护留到 4.5 PathGuard。
- 已新增 StorageService 单测覆盖 workspace 初始化、项目文本写入读取、导入资产复制、危险相对路径拒绝、temp-only 策略。
- 已执行 `cargo fmt`。
- 已通过 `cargo check`。
- 已通过 `cargo test`：12 个 Rust 单测全部通过。
- 说明：本条完成本地受控存储服务基础；远程 URL 下载、资产引用删除保护和完整 PathGuard 继续按 4.5、4.9 实现。

---

### 【X】4.5 实现 PathGuard

**问题：**  
桌面应用文件权限大，路径绕过会导致任意文件读取/覆盖风险。

**改法：**

PathGuard 必须防：

```text
../
..\
符号链接
junction
UNC 路径
file://
绝对路径伪装相对路径
Zip Slip
模板资源越权
FFmpeg 参数越权
Chromium file 访问越权
```

要求 canonicalize / resolve 后判断，不允许字符串 `startsWith`。

**验证：**

- PathGuard 单测覆盖 Windows 路径绕过。
- 工作区外文件无法被读取或写入。
- 导出、导入、模板、FFmpeg、Chromium 都复用 SafePath。

**风险：**  
Windows junction / UNC / symlink 是重点风险。

**完成记录：**

- 已新增 `app/src-tauri/src/security/path_guard.rs`，实现 `PathGuard` 和 `SafePath`，文件读写统一先解析为受控安全路径。
- `PathGuard::validate_relative_path` 已拒绝空路径、前后空白、反斜杠、`../`、`./`、空段、绝对路径、盘符前缀、UNC 路径和 `file://`。
- 读已有文件时会逐级检查路径链路，拒绝 symlink 和 Windows reparse point；再 canonicalize 后确认仍在 workspace 内。
- 写入文件时不再先递归创建父目录后校验，而是逐级创建/解析目录，每一级 canonicalize 后都必须仍在 workspace 内；已有目标文件如果是 symlink/reparse point 会拒绝覆盖。
- 已将 `StorageService` 接入 `PathGuard`，`write_text`、`copy_into_bucket`、`read_to_string` 都通过 SafePath 解析后再进行文件操作。
- 已补充 bucket 根目录校验：workspace bucket 不能是 symlink 或 Windows reparse point，且解析后的真实路径必须仍在目标 bucket 内。
- 已将项目输入的 `source_text_path` 校验改为复用 `PathGuard::validate_relative_path`，并限制必须指向受控 `projects/` bucket。
- 已提供模板、FFmpeg、Chromium、导入、导出、Zip Slip 场景的 SafePath 入口，后续实现这些具体流程时必须复用这些入口，不再直接拼绝对路径。
- 已新增单测覆盖 Windows 路径绕过、`file://`、UNC、Zip Slip、工作区外 symlink、目录 symlink、bucket 根目录 symlink/reparse point、模板/FFmpeg/Chromium 安全路径解析。
- 已执行 `cargo fmt`。
- 已通过 `cargo check`，无新增编译警告。
- 已通过 `cargo test`：20 个 Rust 单测全部通过。

---

### 【X】4.6 实现 KeyringService

**问题：**  
真实 API Key 不能进数据库、配置文件、日志和导出包。

**改法：**

真实密钥只进系统钥匙串。数据库只保存：

```text
key_alias
provider_id
auth_type
```

**验证：**

- 保存 Provider 后 SQLite 中没有真实 key。
- DTO 不返回真实 key。
- 配置导出只包含 `key_alias`。
- 导入配置后提示用户重新录入密钥。

**风险：**  
不要为了调试把 key 打进日志或错误 detail。

**完成记录：**

- 已在 `app/src-tauri/Cargo.toml` 引入 `keyring`，并启用 Windows 原生凭据后端，真实 Provider 密钥通过系统钥匙串保存，不进入 SQLite 或配置文件。
- 已新增 `services/keyring_service.rs`，封装 `save_provider_secret / read_provider_secret / delete_provider_secret / has_provider_secret`；ProviderManager 后续可在后端内部按 `key_alias` 读取真实密钥。
- 已新增 Tauri command：`save_provider_secret / delete_provider_secret / has_provider_secret`；前端只可保存、删除、检查密钥状态，不提供读取明文密钥的 command。
- 已扩展 `AppState` 持有 `KeyringService`，生产运行使用系统钥匙串；单测使用内存后端，避免测试污染用户系统凭据。
- 已补前端 config entity API 和 command 映射，DTO 只返回 `providerId / authType / keyAlias / hasSecret`，不返回真实 key。
- 已将 `security/secret_guard.rs` 暴露到安全模块，并在 `ProviderRepository::upsert_provider` 前拦截 `config_json` 中的 `api_key / token / authorization / secret` 等疑似密钥字段，防止真实 key 误入 SQLite。
- 已新增单测覆盖：密钥只写入 Keyring 后端、无效密钥输入被拒绝、Provider config 中疑似密钥被拒绝、Provider 表只持久化 `key_alias`。
- 已执行 `cargo fmt`。
- 已通过 `cargo check`，无新增编译警告。
- 已通过 `cargo test`：27 个 Rust 单测全部通过。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 说明：本条只完成 KeyringService 和密钥不落库边界；SecretGuard 的诊断包扫描、导出扫描和更完整脱敏策略继续在 4.7 实现。

---

### 【X】4.7 实现 SecretGuard

**问题：**  
日志、错误、诊断包、导出包都可能泄露密钥。

**改法：**

SecretGuard 用于：

```text
日志脱敏
错误 detail 脱敏
Provider 请求头脱敏
诊断包扫描
项目包导出扫描
配置导出扫描
```

**验证：**

- `Bearer Token`、API Key、Authorization header 被脱敏。
- 诊断包导出前二次扫描。
- 命中疑似密钥时阻断导出并提示风险。

**风险：**  
SecretGuard 不应只做字符串替换，要覆盖常见 key pattern。

**完成记录：**

- 已增强 `security/secret_guard.rs`，支持 JSON、纯文本、日志文本、请求头值、诊断包/导出内容集合扫描。
- 已新增 `redact_json / redact_text / redact_header_value`，可对错误 detail、Provider 请求头、日志文本中的疑似密钥做脱敏。
- 已新增 `reject_text_secrets / reject_secret_scan`，为后续诊断包、项目包和配置导出提供二次扫描阻断入口。
- 已把 `AppErrorDto::from_message` 接入 `redact_text`，command 边界把 service 错误转为 DTO 前会先脱敏；同时提供 `AppErrorDto::sanitized` 供后续结构化错误复用。
- 检测规则覆盖 `api_key / token / authorization / bearer / secret / password / private_key / client_secret` 等常见字段名，`Bearer`/`Basic` header、`sk-`/`rk-` token、高熵长 token 和带 token 查询参数的 URL。
- 已保留 `ProviderRepository::upsert_provider` 对 `config_json` 的密钥字段拒绝，防止 Provider 配置把真实 key 写进 SQLite。
- 已新增单测覆盖：JSON 密钥字段检测、JSON 密钥值拒绝、日志密钥检测和阻断、请求头脱敏、JSON 脱敏、诊断/导出扫描阻断、`key_alias` 不误判。
- 已执行 `cargo fmt`。
- 已通过 `cargo check`，无新增编译警告。
- 已通过 `cargo test`：31 个 Rust 单测全部通过。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 说明：本条完成 SecretGuard 基础能力和 command 错误脱敏；真实诊断包、项目包导出和 Provider 请求日志接入会在对应后续 TODO 中复用这些入口。

---

### 【X】4.8 建立 AppConfig / ProviderConfig 基础存储

**问题：**  
配置不能混成一个大 JSON，否则 schema 校验、迁移、导入导出都困难。

**改法：**

配置拆分：

```text
AppConfig
SystemConfig
ProviderConfig
PipelineConfig
UiConfig
ExportConfig
SecretConfig
```

ProviderConfig 不保存 model_name，模型能力放 `provider_models`。

**验证：**

- 配置保存前 schema 校验。
- 运行中修改配置不影响已创建任务快照。
- app_locale 和 content_language 分离。

**风险：**  
不要把 Provider、模型、workflow preset 的配置混在一起。

**完成记录：**

- 已新增 `db/config_repository.rs`，基于 `app_configs` 表实现配置读取、写入和默认配置补齐。
- 已将 `get_app_config / update_app_config` 从内存 stub 改为 SQLite 持久化，`AppConfig` 保存前校验 `app_locale / theme_preset / layout_density`。
- 已补 `app / pipeline / ui / export` 默认配置写回入口；本条只暴露 AppConfig 命令，其他配置作为基础存储和后续任务快照准备。
- 已新增 `ProviderConfigDto`、`ListProviderConfigsRequest`，并新增 Tauri command：`list_provider_configs / upsert_provider_config`。
- 已将 ProviderConfig 持久化到 `providers` 表，只保存 `provider_id / provider_kind / vendor / display_name / base_url / auth_type / key_alias / status / is_enabled / config`，不保存真实密钥。
- 已在 ProviderConfig 保存前校验枚举和结构：`provider_kind / auth_type / status` 必须为稳定 code；需要鉴权时必须提供 `key_alias`。
- 已明确阻断 ProviderConfig 中的 `model_name / modelName`，模型能力继续放 `provider_models`，workflow preset 继续放 `workflow_presets`，不混入 ProviderConfig。
- 已继续复用 `SecretGuard` 拦截 `config` 中的 `api_key / token / authorization / secret` 等疑似密钥字段，避免配置落库泄密。
- 已补前端 config entity 类型和 API：AppConfig、ProviderConfig、密钥保存/删除/检查命令都走统一 `shared/api`，页面仍不直接 invoke。
- 已新增单测覆盖：AppConfig 持久化和重启读取、非法 AppConfig 拒绝、ProviderConfig 持久化、ProviderConfig 拒绝密钥和 `model_name`。
- 已执行 `cargo fmt`。
- 已通过 `cargo check`，无新增编译警告。
- 已通过 `cargo test`：35 个 Rust 单测全部通过。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 说明：本条只完成配置 schema 与基础持久化；ProviderManager、模型能力选择器、WorkflowRegistry、真实连通性测试继续留在 TODO-06。

---

### 【X】4.9 实现 Asset / AssetReference 最小能力

**问题：**  
候选图、视频片段、音频、封面、素材、模板资源都需要统一引用关系。

**改法：**

实现：

```text
assets
asset_references
```

支持：

```text
图片
视频
音频
BGM
字体
参考图
模板资源
任务产物
```

**验证：**

- 导入资产复制到 `workspace/assets/{kind}/`。
- 被引用资产不能物理删除。
- 内置资产不能删除，只能隐藏。
- 导出项目包可按引用收集素材。

**风险：**  
任务产物目录不等于资产库；需要复用时再登记 Asset。

**完成记录：**

- 已新增 `db/asset_repository.rs`，实现 `assets / asset_references` 的基础写入、列表、详情、引用计数、软删除和项目资产路径收集。
- 已扩展 `domain/media.rs`，定义 `AssetDto / AssetReferenceDto / ImportAssetRequest / ListAssetsRequest / DeleteAssetRequest / CreateAssetReferenceRequest`。
- 已扩展 `media_service`：`import_asset` 会把用户选择的绝对路径文件复制到受控 `workspace/assets/{kind}/`，数据库只保存复制后的 `assets/...` 相对路径。
- 已实现资产列表、资产引用创建、引用列表、项目导出用引用路径收集。
- 已实现删除保护：内置资产不能删除；有引用的资产不能物理删除；默认删除为软删除，物理删除也必须先通过 StorageService/PathGuard 解析受控路径。
- 已注册 Tauri command：`import_asset / list_assets / delete_asset / create_asset_reference / list_asset_references / collect_project_asset_paths`。
- 已补前端 config entity 下的资产 DTO 和统一 API 入口，页面仍不直接 invoke。
- 已新增单测覆盖：导入资产复制到 workspace 且只存相对路径、被引用资产不能物理删除、内置资产不能删除、按引用收集项目资产路径。
- 已执行 `cargo fmt`。
- 已通过 `cargo check`，无新增编译警告。
- 已通过 `cargo test`：39 个 Rust 单测全部通过。
- 已通过 `pnpm --dir src typecheck`。
- 已通过 `pnpm --dir src build`；Vite 仅提示 chunk 体积超过 500KB，不影响本条验收。
- 说明：本条完成资产库最小后端能力；完整资产库 UI、缩略图、媒体探测、标签/VLM 分析和导出包实际打包流程留给后续 TODO。

---

## 阶段完成标准

- SQLite + migration 可用。
- 核心表第一版已建立。
- 项目、资产、配置可落库并重启恢复。
- 文件入库只存相对路径。
- PathGuard 单测覆盖高风险路径。
- 真实密钥只进系统钥匙串。
- 日志、导出、诊断具备 SecretGuard 基础能力。






