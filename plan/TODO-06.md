# TODO-06：Provider、模型能力与 Workflow 注册

> 目标：统一所有模型、工作流和 PromptSkill 的注册、选择、调用与校验。
> 本文件来自 `doc/底层设计/12`、`23`、`18`、`Provider与安全`、`doc/功能模块/18-设置与模型配置.md`。

---

## 阶段目标

建立：

```text
ProviderManager
ModelRegistry
WorkflowRegistry
provider_models
workflow_presets
list_executable_media_options
ImageInputPlan / VideoInputPlan
PromptSkill
结构化输出校验
```

业务层只能选择 `provider_model_id` 或 `workflow_preset_id`，不能按具体模型名、供应商、workflow key 写分支。

---

## 本阶段范围

包含：

- Provider 配置模型。
- API 模型能力矩阵。
- ComfyUI / RunningHub workflow preset 注册。
- 统一媒体能力选择器。
- Provider 连通性测试。
- 创作规则 / PromptSkill 文件化；MVP 只做 builtin/user、可编辑、schema 校验，完整版本/hash/snapshot 后续增强。
- LLM 结构化输出校验。
- “模型 / 工作流”页 Provider / 模型 / workflow preset 基础表单。
- 基于模型能力生成图片 / 视频输入规划，供前端动态渲染缺项和参数表单。

不包含：

- 所有真实供应商一次性接完。
- 执行用户上传 JS/TS/Python Provider。
- 未注册 workflow 执行。

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

### 【X】6.1 实现 ProviderManager

**问题：**
如果页面或业务服务直接调模型，后续能力矩阵、密钥安全、日志脱敏、重试都会失控。

**位置：**

```text
src-tauri/src/domain/provider*
src-tauri/src/service/provider*
src-tauri/src/provider* 或 adapters*
src-tauri/src/command/provider*
src-tauri/src/core/error*
src-tauri/src/security* / keyring*
src/src/entities/provider*
src/src/features/model-workflow*
```

如果当前项目已有 `Provider与安全`、`ProviderManager`、`ModelRegistry` 相关模块，优先扩展现有模块，不新建平行体系。

**改法：**

ProviderKind：

```text
llm
tts
image
video
vlm
workflow
```

ProviderManager 统一入口：

```text
call_llm
generate_image
generate_video
generate_tts
analyze_asset
run_workflow
```

落地要求：

```text
1. 建立 ProviderManager trait/service，业务层只能调用 ProviderManager，不直接调用供应商 SDK 或 HTTP 客户端。
2. 每类能力定义输入 DTO 和输出 DTO：LLM、image、video、tts、vlm、workflow。
3. Adapter 只负责和外部供应商通信，不直接写 SQLite，不直接改业务状态。
4. ProviderManager 负责密钥读取、超时、取消令牌、trace_id、错误归一、日志脱敏和能力校验入口。
5. 真实密钥只通过 key_alias 从 keyring 取；ProviderConfig、DTO、日志、导出都不能出现 secret。
6. ComfyUI / RunningHub 归入 WorkflowRegistry / workflow vendor，不作为普通 API ProviderKind 直接混用。
7. 当前阶段允许先实现 mock/dummy adapter 和 1 个最小真实 adapter 骨架，但架构必须能扩展多供应商。
8. 与 TODO-05 的取消机制对齐：Provider 调用必须能接收取消令牌，取消后不得写成功结果。
```

**验收：**

- 业务层不直接调用供应商 SDK。
- Provider adapter 不写数据库。
- 日志不出现 API Key。
- ProviderManager 暴露统一入口，至少能跑通 dummy LLM/image/video 调用的成功、失败、取消三种路径。
- 错误统一转换为 `TaskError / AppErrorDto`，并带 `provider.*` 错误码、trace_id、recover_action。
- ProviderKind 和 workflow vendor 边界清楚，ComfyUI / RunningHub 不会被塞进普通 API 模型调用分支。

**验证：**

- `rg` 检查页面和业务服务没有绕过 ProviderManager 直接调用供应商 SDK / HTTP 生成接口。
- Rust 单测覆盖：密钥不进日志、adapter 不写库、ProviderManager 错误归一、取消令牌生效、dummy adapter 成功返回。
- 前端 typecheck/build 通过。
- 手动 smoke：模型 / 工作流页或临时测试入口能触发 dry_run，并看到脱敏成功/失败结果。

**下一步进入条件：**

- ProviderManager 主入口、adapter 边界、密钥读取、错误归一、取消令牌全部落地。
- 完成记录写清当前实际接入的是 dummy adapter、真实 adapter 骨架还是某个真实供应商；没有接入的供应商不能写成已完成。
- 确认 6.2 的 Provider 设置页可以复用 6.1 的 DTO 和服务后，再把本条改为 `【X】` 并进入 6.2。

**风险：**
不要把 ComfyUI / RunningHub 当普通 ProviderKind，它们是 workflow vendor。

**完成记录：**

- 已新增 `domain::provider`，定义 ProviderManager 统一入口需要的 DTO：`ProviderRequestContext / LlmChatRequest / ImageProviderRequest / VideoProviderRequest / TtsProviderRequest / VlmAnalyzeRequest / WorkflowProviderRequest` 及对应响应 DTO。
- 已新增 `services::provider_service::ProviderManager`，统一暴露 `call_llm / generate_image / generate_video / generate_tts / analyze_asset / run_workflow / dry_run`。
- 已新增 dummy adapter，当前只做本地 dry_run 和协议验证，不发真实网络请求、不调用真实供应商 SDK、不产生费用。
- ProviderManager 已复用现有 `providers` 表、`ProviderRepository` 和 `KeyringService`；真实密钥只通过 `key_alias` 从 keyring 读取，ProviderConfig / DTO / 日志路径不返回 secret。
- Adapter 只返回统一 DTO，不写 SQLite、不改业务状态；数据库写入仍由 repository/service 层负责。
- ProviderManager 已接入 TODO-05 的 `CancellationToken`，取消会返回 `provider.cancelled`，不会落成功结果。
- 错误已统一为 `TaskError / AppErrorDto`，覆盖 `provider.auth_failed / provider.server_error / provider.capability_unsupported / provider.disabled / provider.not_found / provider.cancelled` 等路径，并带 trace/detail 且经过脱敏。
- workflow 边界已在 `run_workflow` 中保留：必须使用 `ProviderKind=workflow`；ComfyUI / RunningHub 后续作为 workflow vendor 接入，不塞进普通 API 模型分支。
- 已新增 Tauri command `provider_dry_run` 并注册到 `main.rs`，前端命令表和 config API 已接入。
- “模型 / 工作流”页已新增 ProviderManager dry_run 面板，可创建 dummy provider，并触发成功、失败、取消三种测试入口；这只是 6.1 的测试入口，不替代 6.2 的完整 Provider 设置页。
- 已执行 `rg` 检查，没有发现真实 HTTP / SDK 生成调用绕过 ProviderManager；当前命中项只包含 scene mock/stub 命名、KeyringService 自身和 ProviderManager 内部方法。
- 验证通过：`cargo fmt` 已执行；`cargo check` 通过；`cargo test` 74 个 Rust 单测全部通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。
- 当前实际接入范围：dummy adapter + ProviderManager 架构骨架 + dry_run 测试入口；未接入 OpenAI、DashScope、ComfyUI、RunningHub、Kling、Seedance 等真实供应商，后续 TODO 不得写成真实生成已完成。
- 复核结果：6.2 的 Provider 设置页可以复用本条新增的 Provider DTO、`provider_dry_run` 命令、现有 `ProviderConfigDto` 和 keyring 机制；可以进入 6.2。

---

### 【X】6.2 实现模型 / 工作流页的 Provider 设置与密钥引用

**问题：**
Provider 连接配置、模型能力、workflow preset 不能混在一个表单。

如果没有可操作的 Provider 配置页，用户无法录入 provider_id、vendor、base_url、auth_type、key_alias，也无法确认密钥是否只进 keyring；后续 provider_models、workflow_presets 和 real_generate 会被迫在页面或业务服务里临时拼配置，造成密钥泄漏和返工。

**位置：**

```text
src-tauri/src/domain/config.rs
src-tauri/src/db/provider_repository.rs
src-tauri/src/services/config_service.rs
src-tauri/src/commands/config.rs
src/src/entities/config/types.ts
src/src/entities/config/api.ts
src/src/pages/model-workflow/index.vue
src/src/shared/i18n/locales/*
```

**改法：**

“模型 / 工作流”页拆三层：

```text
Provider 连接配置
API 模型能力配置
Workflow Preset 配置
```

ProviderConfig 只保存连接信息和 `key_alias`，不保存具体 model_name。

本条只把 Provider 连接配置做成可操作闭环：

```text
1. Provider 表单可新增 / 编辑 provider_id、provider_kind、vendor、display_name、base_url、auth_type、key_alias、enabled、config。
2. 保存 ProviderConfig 时只写 SQLite 中的连接元数据和 key_alias，不写 secret，也不允许 config.model_name。
3. 密钥单独保存到 keyring，页面只显示 key_alias 和 has_secret，不回显 secret。
4. 删除 ProviderConfig 只删除 SQLite 连接配置，不自动删除 keyring secret；页面明确提示可单独删除 secret。
5. Provider dry_run 复用 6.1 的 ProviderManager，不直接调供应商。
6. API 模型能力配置 / Workflow Preset 配置在本条只保留分层占位和边界说明，具体 CRUD 分别留到 6.3 / 6.4。
```

**验收：**

- 模型 / 工作流页能清楚分成 Provider、API 模型能力、Workflow Preset 三层。
- ProviderConfig 能新增、编辑、保存、列表刷新、删除；保存后返回 DTO 不包含 secret。
- key_alias 能单独保存真实 secret 到 keyring，页面只能看到 has_secret 状态，不显示 secret。
- 删除 ProviderConfig 不会自动删除 keyring secret，删除 secret 需要用户显式点单独按钮。
- 测试 Provider 调用 `provider_dry_run`，能展示成功或失败。
- 后端仍拒绝 `config.api_key / token / secret / model_name` 这类字段。

**验证：**

- 保存 Provider 不返回 secret。
- 删除 Provider 不自动删除 keyring secret，需明确策略。
- 测试 Provider 能返回成功/失败。
- Rust 单测覆盖：删除 ProviderConfig 不删除 keyring secret、保存返回不含 secret、model_name/secret 被拒绝。
- 前端 typecheck/build 通过。
- 手动 smoke：新增 dummy provider，保存 secret，刷新列表，dry_run 成功；删除 provider 后 secret 状态仍可通过 key_alias 查询到，需单独删除 secret。

**下一步进入条件：**

- Provider 设置页可操作闭环完成，并明确 6.3/6.4 未完成的能力矩阵 / workflow preset 仍是占位。
- 完成记录写清本条只完成 Provider 连接配置与密钥引用，不包含 provider_models 能力矩阵和 workflow_presets 注册。
- 验证通过后，把本条改为 `【X】` 并进入 6.3。

**风险：**
真实密钥不能进 SQLite、日志、DTO、导出配置。

**完成记录：**

- 已将“模型 / 工作流”页拆成 Provider 连接配置、API 模型能力、Workflow Preset 三层；本条只完成 Provider 连接配置与密钥引用，`provider_models` 能力矩阵和 `workflow_presets` 注册仍分别留给 6.3 / 6.4。
- Provider 连接配置已支持新增、编辑、保存、列表刷新、删除，字段覆盖 `provider_id / provider_kind / vendor / display_name / base_url / auth_type / key_alias / status / enabled / config`。
- ProviderConfig 只保存 SQLite 连接元数据和 `key_alias`；后端仍拒绝 `config.api_key / token / secret / model_name` 等敏感或越界字段，保存和列表返回 DTO 不包含 secret。
- 密钥已单独走 keyring：页面只提交 secret 到 `save_provider_secret`，只显示 `has_secret` 状态，不回显 secret；删除 ProviderConfig 不自动删除 keyring secret，仍需用户显式点击删除 secret。
- 已新增并注册 `delete_provider_config` Tauri command，前端 `deleteProviderConfig` 已接入；浏览器 Mock 也补齐 `providerSecretsByAlias`，保证保存、查询、删除 secret 状态一致。
- 测试 Provider 复用 6.1 的 `provider_dry_run`，当前仍是 dummy adapter，只做本地协议与状态验证，不发真实外部调用、不产生费用。
- 复核结果：模型能力和 workflow preset 只保留分层占位和边界说明，未把具体 model_name 或 workflow key 混入 ProviderConfig。
- 验证通过：`cargo check` 通过；`cargo fmt` 已执行；`cargo test` 75 个 Rust 单测全部通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。

---

### 【X】6.3 实现 provider_models 能力矩阵

**问题：**
不同模型支持的输入、比例、时长、分辨率不同，不能靠前端口头限制。

如果没有可维护的 API 模型能力矩阵，后续生图 / 视频 / TTS / VLM 会继续把具体 model_name、参数范围和输入要求写在页面或业务服务里，造成 ProviderManager 无法统一校验，也会让 disabled model 仍可能被执行。

**位置：**

```text
src-tauri/src/domain/config.rs
src-tauri/src/db/provider_repository.rs
src-tauri/src/services/config_service.rs
src-tauri/src/services/provider_service.rs
src-tauri/src/commands/config.rs
src-tauri/src/main.rs
src/src/entities/config/types.ts
src/src/entities/config/api.ts
src/src/pages/model-workflow/index.vue
src/src/shared/api/commands.ts
src/src/shared/i18n/locales/*
```

**改法：**

`provider_models` 记录：

```text
provider_id
provider_kind
vendor
model_name
capabilities
aspect_ratios
resolutions
duration_range
fps_range
input_requirements
status
```

落地要求：

```text
1. 本条只管理 API 模型能力矩阵，不登记 ComfyUI / RunningHub workflow preset。
2. 基于现有 provider_models 表做兼容扩展，详细能力存入 config_json，不能破坏已有迁移。
3. ProviderModel DTO 必须结构化表达 ability_types、input_modalities、output_modalities、feature_flags、limits、api_contract_verified、status / enabled。
4. 保存 ProviderModel 时必须校验 provider_id 存在、ProviderKind 匹配、workflow provider 不允许登记 API 模型、model_name 不能为空。
5. limits 至少能校验 aspect_ratios、resolutions、duration_range、fps_range、max_reference_images。
6. disabled model 不可执行；ProviderManager 在 provider_model_id 存在时必须再次校验模型存在、启用、provider_id / provider_kind 匹配和请求参数范围。
7. “模型 / 工作流”页 API 模型能力层必须支持新增 / 编辑 / 保存 / 删除 / 列表刷新，并显示 disabled / 不可执行原因。
8. 前端模型列表来自后端 provider_models，不允许页面写死某个供应商模型。
9. 具体统一媒体选择器 `list_executable_media_options` 的最终合并和 InputPlan 留到 6.5，本条只保证 provider_models 数据和基础后端校验可用。
```

**验收：**

- 后端提供 provider_models 的列表、新增 / 编辑、删除接口，返回 DTO 不包含 secret。
- ProviderModel 能表达 `provider_id / provider_kind / vendor / model_name / ability_types / input_modalities / output_modalities / feature_flags / limits / api_contract_verified / status`。
- 保存时拒绝不存在的 provider、workflow provider、ProviderKind 不匹配、空 model_name、空 ability_types、secret-like config、非法 duration / fps / reference image 限制。
- ProviderManager 在传入 `provider_model_id` 时会校验模型属于当前 provider、能力类型匹配、模型启用状态和 image/video 请求参数范围；disabled model 不可执行。
- “模型 / 工作流”页的 API 模型能力层不再是占位，能基于已配置 Provider 新增、编辑、保存、删除 API 模型能力矩阵。
- Workflow Preset 层仍保持 6.4 占位，不把 workflow preset 混进 provider_models。

**验证：**

- 参数超过能力矩阵时后端拒绝。
- disabled model 不可执行。
- 前端模型选择来自后端能力列表。
- Rust 单测覆盖：保存 / 列表 provider_models、拒绝 workflow provider、拒绝非法 limits、disabled model 拒绝执行、duration / aspect_ratio / fps 超限拒绝。
- 前端 `typecheck` / `build` 通过。
- 手动 smoke：新增 dummy Provider 后，在 API 模型能力层新增模型，刷新列表可见；禁用模型后 dry_run 或 ProviderManager 校验拒绝。

**风险：**
业务层不得按 `model_name` 写 if/else。

不能把 workflow preset 塞进 provider_models；不能把密钥、token、api_key 或 secret-like 字段写入 model config；不能只靠前端禁用做安全校验。

**下一步进入条件：**

- provider_models CRUD、DTO、后端校验、ProviderManager 基础能力校验和前端 API 模型能力层全部完成。
- 完成记录写清本条只完成 API 模型能力矩阵，不包含 workflow_presets 注册，也不包含 6.5 的统一媒体选择器和 InputPlan。
- 验证命令和关键单测通过后，把本条改为 `【X】` 并进入 6.4。

**完成记录：**

- 已完成 API 模型能力矩阵的结构化 DTO：`ProviderModelDto / ListProviderModelsRequest / DeleteProviderModelRequest`，表达 `provider_id / provider_kind / vendor / provider_model_id / model_name / ability_types / input_modalities / output_modalities / feature_flags / limits / input_requirements / api_contract_verified / status / enabled / config`。
- 已基于现有 `provider_models` 表兼容扩展：表结构不重建，详细能力矩阵写入 `config_json`；`model_id` 作为本项目稳定主键，`provider_model_id / model_name` 作为供应商侧模型标识。
- 已新增 provider_models 后端 CRUD：`list_provider_models / upsert_provider_model / delete_provider_model`，并注册 Tauri command 和前端命令表。
- 已新增后端校验：provider 必须存在；workflow provider 不能登记 API 模型；ProviderKind / vendor 必须匹配 ProviderConfig；`model_name / provider_model_id / ability_types` 不能为空；`limits / input_requirements / config` 不允许 secret-like 字段；非法 duration / fps / maxReferenceImages 会被拒绝。
- ProviderManager 已在传入 `provider_model_id` 时按矩阵再次校验模型存在、属于当前 provider、kind 匹配、模型启用状态、ability_type、aspect_ratio、resolution、duration、fps、参考图数量；disabled model 和超限参数会被后端拒绝。
- “模型 / 工作流”页的 API 模型能力层已从占位改为可操作表单和列表，支持新增、编辑、保存、删除、刷新和模型 dry_run；前端模型列表来自后端 `provider_models`，不在页面硬编码供应商模型。
- Provider 连接 dry_run 已保持为 Provider 连接测试，不再默认伪造 `provider_model_id`；模型 dry_run 由模型能力层传入已登记的 `model_id`。
- 浏览器 Mock 已补齐 provider_models 状态，保存、列表、删除、disabled model 检查与 Tauri 路径保持同一语义。
- 本条只完成 API 模型能力矩阵，不包含 `workflow_presets` 注册，也不包含 6.5 的 `list_executable_media_options` 最终合并、`ImageInputPlan / VideoInputPlan` 动态输入规划。
- 验证通过：`cargo fmt` 已执行；`cargo check` 通过；`cargo test` 78 个 Rust 单测全部通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。

---

### 【X】6.4 实现 workflow_presets 注册

**问题：**
ComfyUI / RunningHub workflow 不能裸 key 执行，否则参数、节点、输出不可控。

如果没有可维护的 workflow preset 注册层，后续生图 / 视频工作流会把 workflow_key、workflow_id、node_map、output_map 和参数 schema 直接散落在页面或业务服务里；这会导致未注册 workflow 被执行、节点缺失时无法定位、远程输出 URL 可能绕过任务目录入库。

**位置：**

```text
src-tauri/src/domain/config.rs
src-tauri/src/db/provider_repository.rs
src-tauri/src/services/config_service.rs
src-tauri/src/services/provider_service.rs
src-tauri/src/commands/config.rs
src-tauri/src/main.rs
src/src/entities/config/types.ts
src/src/entities/config/api.ts
src/src/pages/model-workflow/index.vue
src/src/shared/api/commands.ts
src/src/shared/i18n/locales/*
```

**改法：**

workflow preset 包含：

```text
workflow_preset_id
vendor
name
capabilities
param_schema
node_map
output_map
default_params
status
```

落地要求：

```text
1. 本条只管理 ComfyUI / RunningHub workflow preset，不登记普通 API 模型。
2. 基于现有 workflow_presets 表做兼容扩展，详细 preset 元数据存入 config_json，不能破坏已有迁移。
3. WorkflowPreset DTO 必须结构化表达 workflow_preset_id、provider_id、vendor、workflow_key、workflow_id、workflow_version、ability_types、input_modalities、output_modalities、limits、param_schema、node_map、output_map、default_params、status / enabled。
4. 保存 WorkflowPreset 时必须校验 provider_id 存在、ProviderKind=workflow、vendor 只能是 comfyui / runninghub，且与 ProviderConfig vendor 匹配。
5. RunningHub preset 必须有 workflow_id；ComfyUI preset 必须有 workflow_key。
6. param_schema / node_map / output_map 必须是 JSON object；node_map / output_map 不能为空；node_map 路径必须是 ComfyUI 风格 node.inputs.field 或 RunningHub 参数键。
7. ProviderManager.run_workflow 在 workflow_preset_id 存在时必须校验 preset 已注册、启用、provider_id / vendor 匹配、param_schema / node_map / output_map 完整；未注册 workflow 不可执行。
8. “模型 / 工作流”页 Workflow Preset 层必须支持新增 / 编辑 / 保存 / 删除 / 列表刷新，并显示 disabled / 不可执行状态。
9. 真实 workflow 调用、上传素材、远程 URL 下载和 InputPlan 留到后续真实生成 / 6.5；本条只做注册、校验和 dry_run 级别验证。
```

**验收：**

- 后端提供 workflow_presets 的列表、新增 / 编辑、删除接口，返回 DTO 不包含 secret。
- WorkflowPreset 能表达 `provider_id / vendor / workflow_key / workflow_id / ability_types / input_modalities / output_modalities / limits / param_schema / node_map / output_map / default_params / status`。
- 保存时拒绝不存在的 provider、非 workflow provider、vendor 不匹配、RunningHub 缺 workflow_id、空 node_map、空 output_map、secret-like config。
- ProviderManager.run_workflow 使用 `workflow_preset_id` 时会校验 preset 存在、启用、vendor / provider 匹配和 map 完整；未注册 workflow、disabled preset、缺 node_map / output_map 都会被后端拒绝。
- “模型 / 工作流”页 Workflow Preset 层不再是占位，能基于 workflow Provider 新增、编辑、保存、删除 preset。
- API 模型能力层仍只管理 provider_models，不把 workflow preset 混进 provider_models。

执行前必须校验：

```text
param_schema
node_map
output_map
```

**验证：**

- 未注册 workflow 执行失败。
- 缺节点返回 `workflow.invalid_node_map`。
- 缺输出返回 `workflow.output_missing`。
- RunningHub 远程 URL 下载到任务目录后再入库。
- Rust 单测覆盖：保存 / 列表 workflow_presets、拒绝非 workflow provider、拒绝 RunningHub 缺 workflow_id、拒绝空 node_map / output_map、未注册 workflow 拒绝执行、disabled preset 拒绝执行、缺 node_map / output_map 返回对应错误。
- 前端 `typecheck` / `build` 通过。
- 手动 smoke：新增 dummy workflow Provider 后，在 Workflow Preset 层新增 ComfyUI preset，刷新列表可见；禁用 preset 后 dry_run 或 ProviderManager 校验拒绝。

**风险：**
不允许执行用户上传的任意 JS/TS/Python Provider 代码。

不能执行未注册 workflow；不能把 workflow preset 塞进 provider_models；不能把 API Key、token、secret 写入 preset config；本条不做真实 ComfyUI / RunningHub 外部调用，不下载远程 URL。

**下一步进入条件：**

- workflow_presets CRUD、DTO、后端校验、ProviderManager.run_workflow 基础注册校验和前端 Workflow Preset 层全部完成。
- 完成记录写清本条只完成 workflow preset 注册和校验，不包含真实 workflow 执行、远程 URL 下载，也不包含 6.5 的统一媒体选择器和 InputPlan。
- 验证命令和关键单测通过后，把本条改为 `【X】` 并进入 6.5。

**完成记录：**

- 已完成 workflow preset 注册 DTO：`WorkflowPresetDto / ListWorkflowPresetsRequest / DeleteWorkflowPresetRequest`，表达 `workflow_preset_id / provider_id / vendor / workflow_key / workflow_id / workflow_version / ability_types / input_modalities / output_modalities / limits / param_schema / node_map / output_map / default_params / status / enabled / config`。
- 已基于现有 `workflow_presets` 表兼容扩展：表结构不重建，详细 preset 元数据写入 `config_json`；workflow preset 仍独立于 `provider_models`，没有混入普通 API 模型能力矩阵。
- 已新增 workflow_presets 后端 CRUD：`list_workflow_presets / upsert_workflow_preset / delete_workflow_preset`，并注册 Tauri command 和前端命令表。
- 已新增后端校验：provider 必须存在且为 `ProviderKind=workflow`；vendor 只允许 `comfyui / runninghub` 并必须匹配 ProviderConfig；RunningHub 必须有 `workflow_id`；`param_schema / node_map / output_map / default_params / config` 必须是 JSON object；`node_map / output_map` 不能为空；secret-like 字段会被拒绝。
- ProviderManager.run_workflow 已在传入 `workflow_preset_id` 时按注册表再次校验 preset 存在、启用状态、provider_id、vendor、param_schema、node_map、output_map；未注册 workflow、disabled preset、空 node_map、空 output_map 会被后端拒绝。
- “模型 / 工作流”页的 Workflow Preset 层已从占位改为可操作表单和列表，支持基于 workflow Provider 新增、编辑、保存、删除、刷新和 dry_run 校验。
- 当前 dry_run 仍走 dummy adapter，只做本地协议与注册校验，不发真实 ComfyUI / RunningHub 外部调用、不上传素材、不下载远程 URL、不产生费用。
- 本条只完成 workflow preset 注册和校验，不包含真实 workflow 执行，不包含远程 URL 下载，也不包含 6.5 的统一媒体选择器、`ImageInputPlan / VideoInputPlan`。
- 验证通过：`cargo fmt` 已执行；`cargo check` 通过；`cargo test` 81 个 Rust 单测全部通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。

---

### 【X】6.5 实现 `list_executable_media_options`

**问题：**
前端不应该自己合并 API 模型和 workflow preset，也不应该硬编码某个模型需要哪些图片、参数或 ComfyUI 节点。

如果没有统一选择器，图片 / 视频生成页会继续各自拼接 provider_models 和 workflow_presets，导致 disabled 项仍可能被选择、模型能力和 workflow 参数表单不一致、角色资源包图片被误判为必须全量生成，后续接入真实生成时会大量返工。

**位置：**

```text
src-tauri/src/domain/media.rs
src-tauri/src/services/media_service.rs
src-tauri/src/commands/media.rs
src-tauri/src/main.rs
src/src/entities/config/types.ts
src/src/entities/config/api.ts
src/src/pages/model-workflow/index.vue
src/src/shared/api/commands.ts
src/src/shared/i18n/locales/*
```

**改法：**

后端统一返回：

```text
sourceType: provider_model | workflow_preset
sourceId
label
providerKind
capabilities
constraints
inputPlan
status
```

业务调用时只能二选一：

```text
provider_model_id
workflow_preset_id
```

默认规则：不能同时为空，也不能同时有值。

`inputPlan` 至少分为：

```text
ImageInputPlan:
- image_kind / asset_kind:
  storyboard_frame
  character_reference
  scene_reference
  style_reference
  prop_reference
  end_frame
  control_image
  cover_image
- prompt
- negativePrompt
- aspectRatio
- resolution
- seed
- characterReference[]
- styleReference[]
- pose / depth / mask / referenceAsset[]
- workflowParams

VideoInputPlan:
- startFrame
- endFrame
- characterReference[]
- styleReference[]
- videoPrompt
- durationSeconds
- fps
- resolution
- motionStrength / cameraMotion
- pose / depth / mask / referenceAsset[]
- workflowParams
```

每个输入项必须能说明：

```text
input_key：输入字段名，例如 prompt、character_front_view、pose、endFrame
input_group：text / image / video / audio / workflow_param
owner_type：storyboard_item / character_bible / environment_bible / style_bible / prop_bible / project
owner_id：所属对象 ID，可为空但不能含糊
requirement：required / optional / unused
source_options：generate / upload / select_existing / derive_from_selected_image
missing_reason：缺什么，为什么缺
ui_schema：前端怎么渲染，不由页面猜
constraints：数量、比例、尺寸、格式、时长、参考图上限
normalized_params：后端最终可执行参数
```

典型能力映射：

```text
分镜图：StoryboardItem.imagePrompt + aspectRatio + optional referenceAsset[]
角色参考图：Character Bible 草稿 / 角色描述 + character_reference image_kind
场景参考图：Environment / Location Bible 草稿 + scene_reference image_kind
风格参考图：Style Bible 草稿 / 风格描述 + style_reference image_kind
道具图：prop 描述 + prop_reference image_kind
尾帧图：当前分镜 / 已选图 / 下一分镜意图 + end_frame image_kind
控制图：pose / depth / mask / referenceAsset[] + control_image image_kind
封面图：cover 标题 / 已选图 / 作品主题 + cover_image image_kind
普通图生视频：startFrame + videoPrompt + durationSeconds
首尾帧模型：startFrame + endFrame + videoPrompt + durationSeconds
参考图模型：startFrame + characterReference/styleReference + videoPrompt
ComfyUI / RunningHub：根据 workflow_preset.param_schema 返回 pose/depth/mask/workflowParams
无分镜图模型：资产图或参考图 + prompt + 模型参数，不强制 selectedImageId
```

前端只能根据 `inputPlan.required / optional / constraints` 渲染字段、缺项提示和禁用状态，不能在页面里按模型名写分支。

落地要求：

```text
1. 本条复用现有 list_executable_media_options 命令名，但返回值改为从 provider_models + workflow_presets 动态合并，不再返回 mock 静态数据。
2. 后端 DTO 必须支持 sourceType、sourceId、providerId、providerKind、vendor、label、capabilities、constraints、inputPlan、status、enabled、disabledReason、normalizedParams。
3. provider_model 选项只来自 provider_models；workflow_preset 选项只来自 workflow_presets；不能互相混用。
4. API 模型选项必须携带 provider_model_id，workflow 选项必须携带 workflow_preset_id，且同一 option 不能同时有两者。
5. disabled provider / model / preset 必须返回为不可选，并给 disabledReason；不能在后端结果里直接过滤到用户无法诊断。
6. inputPlan 至少覆盖 text_to_image、image_to_image、text_to_video、image_to_video、first_frame_i2v、start_end_frame_i2v、reference_to_video，以及 workflow preset param_schema 派生的 workflowParams。
7. 角色资源包输入必须按 inputPlan 表达；默认只要求 character_front_view，其余角色图默认 optional 或 unused，除非模型 / workflow 配置显式要求。
8. 前端只在“模型 / 工作流”页展示统一选择器和 inputPlan smoke，不接入生成页真实执行；生成页最终替换留给后续真实生成 TODO。
9. 本条不做真实 Provider 调用、不上传素材、不下载远程 URL、不执行 workflow。
```

角色资源包里的图片也必须通过 ImageInputPlan 表达，不能默认全量生成：

```text
character_front_view       角色正面图
character_side_view        角色侧面图
character_back_view        角色背面图
character_full_body        角色全身图
character_face_closeup     面部特写
character_expression_sheet 表情表
character_outfit           服装细节 / 服装变化
character_pose             姿态 / 动作参考
character_mood             心情 / 情绪状态参考
```

如果当前模型只需要 `character_front_view`，就只提示补这张图；如果 ComfyUI workflow 需要 `front_view + face_closeup + pose`，才提示这些 required 输入；其余角色图片必须标记为 unused 或 optional，不能自动生成。

**验收：**

- 后端 `list_executable_media_options` 从 SQLite 合并 API 模型和 workflow preset，返回结构化 `ExecutableMediaOptionDto`，不再返回 mock 固定候选。
- 同一候选只能是 `provider_model` 或 `workflow_preset`，不能同时有 `provider_model_id` 和 `workflow_preset_id`，也不能两者都为空。
- disabled provider、disabled model、disabled workflow preset 会返回 disabled 状态和不可执行原因。
- 不同能力会返回不同 inputPlan：图片模型、视频模型、首尾帧、参考图、workflowParams 不能共用一套固定表单。
- workflow preset 的 `param_schema` 能映射为 workflow_param 输入项，并合并 `node_map / output_map / default_params` 到 constraints / normalizedParams。
- 前端“模型 / 工作流”页能刷新统一可执行选项，展示 source、capability、status、必填 / 可选 / unused 输入项、缺失原因和约束摘要。
- 前端不按 model_name / workflow_key 写分支，只按后端 `sourceType / inputPlan / constraints` 渲染。

**验证：**

- API 图片模型和 ComfyUI 生图 workflow 能在同一选择器展示。
- 不同图片 / 视频模型返回不同 `ImageInputPlan / VideoInputPlan`。
- 不同图片类型返回不同输入规划；分镜图、角色参考图、场景参考图、风格参考图、道具图、尾帧图、控制图和封面图不能共用一套固定表单。
- 前端能根据输入规划动态展示必填项、可选项、缺失原因和参数范围。
- disabled 项不可选择。
- 后端再次校验能力。
- Rust 单测覆盖：provider_models + workflow_presets 合并、disabled 项、二选一 source id、workflow param_schema 派生 workflowParams、角色资源包只默认要求 front_view。
- 前端 `typecheck` / `build` 通过。
- 手动 smoke：新增 dummy image/video ProviderModel 和 ComfyUI preset 后，统一选择器能展示并展开 inputPlan；禁用后同一选项显示不可选原因。

**风险：**
不能只靠前端禁用来保证安全；后端必须按能力矩阵和输入规划再次校验。

不能把 workflow preset 塞进 provider_models；不能把密钥、token、api_key 或 secret-like 字段写入选择器结果；本条不做真实生成、不下载远程 URL。

**下一步进入条件：**

- `list_executable_media_options` 已从真实注册数据动态返回 API 模型和 workflow preset，前端已能按 inputPlan 展示。
- 完成记录写清本条只完成统一选择器和输入规划 smoke，不包含生成页全面改造、真实 Provider 调用、真实 workflow 执行和远程 URL 下载。
- 验证命令和关键单测通过后，把本条改为 `【X】` 并进入 6.6。

**完成记录：**

- 已复用现有 `list_executable_media_options` 命令名，将原 mock 静态返回改为从 SQLite 的 `provider_models` 和 `workflow_presets` 动态合并。
- 已升级后端 DTO：`ExecutableMediaOptionDto / MediaInputPlanDto / MediaInputRequirementDto`，返回 `sourceType / sourceId / providerId / providerKind / vendor / capabilities / constraints / inputPlan / status / enabled / disabledReason / normalizedParams`。
- 已保证候选来源二选一：API 模型只带 `provider_model_id`，workflow preset 只带 `workflow_preset_id`；ProviderManager dry_run 已新增后端校验，拒绝同时传入两种 source id。
- disabled provider、disabled model、disabled workflow preset、provider 状态异常、空 `node_map / output_map` 等都会返回不可执行原因，前端不再只能看到候选消失。
- 已实现基础 `ImageInputPlan / VideoInputPlan` 语义：覆盖 `text_to_image / image_to_image / text_to_video / image_to_video / first_frame_i2v / start_end_frame_i2v / reference_to_video`，并输出 required / optional / unused、source_options、ui_schema、constraints 和 normalized_params。
- 已支持 workflow preset 的 `param_schema` 派生 `workflowParams.*` 输入项，并把 `node_map / output_map / default_params / workflowKey / workflowId` 放入 constraints 或 normalizedParams，供后续真实执行复用。
- 已按角色资源包规则处理角色图片：默认只让 `character_front_view` 可配置，其他角色图默认 `unused`；只有 `input_requirements` 显式声明时才会变成 required / optional，避免自动全量生成。
- “模型 / 工作流”页已新增“统一可执行选择器”层，能刷新候选、展示 API 模型和 workflow preset、显示不可执行原因、展开 inputPlan，并对可执行候选触发 dry_run smoke。
- 前端 Mock adapter 已同步动态合并 provider_models / workflow_presets，不再返回旧 mock 固定候选；页面只按 `sourceType / inputPlan / constraints` 渲染，没有按 model_name / workflow_key 写分支。
- 本条只完成统一选择器和输入规划 smoke，不包含生成页全面改造，不包含真实 Provider 调用，不包含真实 ComfyUI / RunningHub workflow 执行，不上传素材，不下载远程 URL。
- 验证通过：`cargo fmt` 已执行；`cargo check` 通过；`cargo test` 83 个 Rust 单测全部通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。

---

### 【X】6.6 实现创作规则 / PromptSkill 文件化

**问题：**
创作规则如果散落在 Vue 页面或 Rust 字符串里，后续会很难查看、修改、绑定视频包和诊断问题。用户侧统一叫“创作规则”，技术侧可以使用 `PromptSkill / SkillDefinition` 等命名，但本阶段不强制做完整 `SkillVersion / content_hash / schema_hash / SkillSnapshot`。

如果不先把创作规则文件化，后续脚本、分镜、生图 prompt、视频 prompt、字幕、封面和审核规则会继续分散在页面、service 字符串或测试 mock 中；用户无法复制内置规则、编辑自定义规则，也无法确认 LLM 输出 schema，后续再做版本、hash、任务快照会大范围返工。

**位置：**

```text
workspace/prompts/builtin/*
workspace/prompts/user/*
src-tauri/src/domain/config.rs 或 domain/prompt*
src-tauri/src/services/config_service.rs 或 services/prompt*
src-tauri/src/commands/config.rs 或 commands/prompt*
src-tauri/src/main.rs
src/src/entities/config/types.ts 或 entities/prompt*
src/src/entities/config/api.ts 或 entities/prompt*
src/src/pages/creative-resources/index.vue
src/src/shared/api/commands.ts
src/src/shared/i18n/locales/*
```

**改法：**

内置目录：

```text
workspace/prompts/builtin/script
workspace/prompts/builtin/storyboard
workspace/prompts/builtin/image_prompt
workspace/prompts/builtin/video_prompt
workspace/prompts/builtin/subtitle
workspace/prompts/builtin/cover
workspace/prompts/builtin/review
```

frontmatter 必填：

```text
key
name
module
provider_kind
output_schema
description
source_type = builtin | user
enabled
```

MVP 落地规则：

```text
1. builtin 内置规则只读，不能直接编辑。
2. 用户点击编辑 builtin 时，复制为 user 规则后再编辑。
3. user 规则可保存、启用、禁用、删除。
4. 视频包和项目只引用规则 key / rule_id，不复制规则正文。
5. LLM 输出必须按 output_schema 校验。
6. 当前任务可记录 rule_id / rule_key / source_type，完整版本/hash/snapshot 后续放到 TODO-12。
```

落地要求：

```text
1. 建立创作规则文件目录初始化：workspace/prompts/builtin/* 与 workspace/prompts/user/*。
2. builtin 规则从应用内默认模板初始化到 workspace；已存在文件不覆盖用户工作区内容。
3. 规则文件必须使用 frontmatter + markdown body；frontmatter 至少包含 key、name、module、provider_kind、output_schema、description、source_type、enabled。
4. 后端提供 list / get / save user rule / clone builtin to user / delete user rule / enable-disable user rule 的基础接口。
5. 保存 user 规则时必须校验 frontmatter、module、provider_kind、source_type、enabled、output_schema 是 JSON object，且正文不能为空。
6. builtin 规则只读；编辑 builtin 必须先 clone 到 user 规则。
7. 页面用户文案统一叫“创作规则”；技术 DTO 可以叫 PromptSkill / CreativeRule。
8. 视频包 / 项目真实绑定、任务历史 snapshot、content_hash、schema_hash、版本迁移留到 TODO-12；本条只提供 rule key / rule id 级别引用基础。
9. 不允许把创作规则做成可执行插件；只允许提示词、结构化 schema、参数模板和校验规则。
```

**验收：**

- 首次调用规则接口会在 workspace 下创建 builtin/user 目录和一组内置规则文件。
- 后端能列出 builtin 与 user 规则，返回 frontmatter、body、enabled、source_type、module、provider_kind、output_schema，不返回任何 secret。
- builtin 规则不可直接保存或删除；clone builtin 后生成 user 规则，user 规则可编辑、启用、禁用、删除。
- 保存非法 frontmatter、空 body、非法 module/provider_kind/source_type、非 JSON object 的 output_schema 会被拒绝。
- 创作资源页能查看规则列表、选择规则、复制 builtin 为 user、编辑 user 规则、保存、启用/禁用、删除。
- 页面内不新增大段硬编码 prompt；内置 prompt 模板由后端初始化文件承载。

**验证：**

- Vue 页面无大段 prompt。
- Rust service 无不可追踪 prompt。
- builtin 规则不可直接编辑，复制成 user 后可编辑。
- 保存规则时校验 frontmatter 和 output_schema。
- 视频包或项目能引用当前启用规则。
- 不要求本阶段生成 SkillVersion、content_hash、schema_hash 或 SkillSnapshot。
- Rust 单测覆盖：初始化 builtin、解析 frontmatter、拒绝编辑 builtin、clone builtin、保存 user、拒绝非法 schema、删除 user。
- 前端 `typecheck` / `build` 通过。
- 手动 smoke：进入创作资源页，能看到内置规则，复制一条为 user 后编辑保存，再启用 / 禁用 / 删除。

**风险：**
不要把创作规则做成可执行插件；当前只允许提示词、结构化 schema、参数模板和校验规则。历史任务完整复现是后续增强，不作为 TODO-06 门槛。

不能把 API Key、token、secret 写入规则 frontmatter 或正文；不能让用户上传 JS/TS/Python 作为规则执行。

**下一步进入条件：**

- 创作规则文件化、builtin/user 读写边界、frontmatter/schema 校验和前端基础管理闭环完成。
- 完成记录写清本条只做规则文件化 MVP，不包含 SkillVersion、content_hash、schema_hash、SkillSnapshot、视频包真实绑定和可执行插件。
- 验证命令和关键单测通过后，把本条改为 `【X】` 并进入 6.7。

**完成记录：**

- 已新增创作规则后端 DTO 与命令：`CreativeRuleDto / ListCreativeRulesRequest / CreativeRuleIdRequest / SaveCreativeRuleRequest / SetCreativeRuleEnabledRequest`，并注册 `list_creative_rules / get_creative_rule / clone_creative_rule_to_user / save_user_creative_rule / set_user_creative_rule_enabled / delete_user_creative_rule`。
- 已新增 `prompt_service`，首次调用会初始化 `workspace/prompts/builtin/*` 与 `workspace/prompts/user/*` 目录，并写入 script、storyboard、image_prompt、video_prompt、subtitle、cover、review 的内置规则文件；已存在文件不会被覆盖。
- 规则文件使用 frontmatter + markdown body；frontmatter 校验 `key / name / module / provider_kind / output_schema / description / source_type / enabled`，其中 `output_schema` 必须是 JSON object，正文不能为空。
- 已实现 builtin/user 边界：builtin 规则只读，不允许直接保存、启停或删除；编辑 builtin 必须先 clone 为 user 规则；user 规则支持保存、启用 / 禁用、删除。
- 已接入密钥防护：`output_schema / description / body` 会拒绝 secret-like 内容，规则不会保存 API Key、token、secret。
- “创作资源”页已从占位升级为创作规则管理入口，支持查看内置规则、筛选模块和来源、复制为用户规则、编辑用户规则、保存、启用 / 禁用和删除。
- 页面用户可见文案统一叫“创作规则”；技术侧使用 `CreativeRule` DTO，不把“Skill”作为用户菜单名。
- 页面内没有新增大段 prompt；内置规则由后端初始化到 workspace 文件。前端浏览器 Mock 仅保留短规则样例用于非 Tauri 开发态。
- 本条只完成规则文件化 MVP，不包含 `SkillVersion / content_hash / schema_hash / SkillSnapshot`，不包含视频包真实绑定，不包含历史任务完整复现，不包含可执行插件，也不执行用户上传 JS/TS/Python。
- 验证通过：`cargo fmt` 已执行；`cargo check` 通过；`cargo test` 87 个 Rust 单测全部通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。

---

### 【X】6.7 实现 LLM 结构化输出校验

**问题：**
LLM 输出不稳定，核心数据不能直接写表。

如果没有统一结构化输出校验，脚本、分镜、生图 prompt、视频 prompt、字幕、资产分析和审核结果会继续靠页面或 service 临时解析，Markdown 包裹 JSON、字段缺失、数量不一致都可能直接写入核心表，后续失败很难恢复。

**位置：**

```text
src-tauri/src/domain/provider.rs 或 domain/structured_output*
src-tauri/src/services/provider_service.rs
src-tauri/src/services/prompt_service.rs 或 services/structured_output*
src-tauri/src/commands/prompt.rs 或 commands/provider.rs
src-tauri/src/main.rs
src/src/entities/config/types.ts 或 entities/prompt*
src/src/entities/config/api.ts 或 entities/prompt*
src/src/pages/creative-resources/index.vue
src/src/shared/api/commands.ts
src/src/shared/i18n/locales/*
```

**改法：**

核心输出必须 schema 校验：

```text
ScriptNarrations
StoryboardItems
ImagePromptList
VideoPromptList
SubtitleChunks
AssetAnalysis
ReviewReport
```

流程：

```text
JSON parse
schema validate
repair loop 最多 2 次
仍失败则 step failed 或 waiting_user
```

落地要求：

```text
1. 建立统一 StructuredOutputValidator，不在页面或业务 service 里各写一套 JSON parse。
2. 支持清理 Markdown fenced JSON，但不能把普通散文强行猜成 JSON。
3. 支持最小 JSON Schema 子集：type、required、properties、items、enum、minItems、maxItems。
4. 支持 expected_count 校验数组数量；数量不一致返回结构化错误，不写核心表。
5. 支持 repair_attempts 计数和最多 2 次限制；本条只返回 repairNeeded，不调用真实 LLM 修复。
6. 校验结果必须包含 parsed_json、valid、errors、repair_needed、attempt_count、max_attempts。
7. 创作资源页可对当前规则的 output_schema 粘贴样例输出做校验 smoke。
8. 不合格输出不能写核心表；本条先提供统一校验入口，后续生成流程逐步接入。
```

**验收：**

- 后端提供统一结构化输出校验函数和 Tauri smoke 命令。
- Markdown ```json 包裹输出能被清洗并校验；非 JSON 文本会被拒绝。
- schema required / properties / array items / enum / 数量不一致能给出明确错误。
- repair_attempts 小于 2 时失败结果标记 `repair_needed=true`；达到上限后标记不可修复。
- 创作资源页可用当前创作规则的 `output_schema` 校验用户粘贴 JSON 样例，并展示通过/失败和错误列表。
- 本条不执行真实 LLM 修复，不把校验失败输出写核心表。

**验证：**

- Markdown 包裹 JSON 可被清洗或拒绝。
- 数量不一致会触发修复或失败。
- 不合格输出不能写核心表。
- Rust 单测覆盖：fenced JSON 清洗、非 JSON 拒绝、required 缺失、数组 item 校验、enum 拒绝、expected_count 不一致、repair_attempts 上限。
- 前端 `typecheck` / `build` 通过。
- 手动 smoke：创作资源页选择一条规则，粘贴符合 / 不符合 schema 的 JSON，分别显示通过和失败。

**风险：**
不要用普通文本解析代替 schema 校验。

不能把失败输出写入核心表；不能在本条里假装已经完成真实 LLM repair loop 或全流程接入。

**下一步进入条件：**

- 统一结构化输出校验、关键 schema 子集、repair_needed 标记、前端 smoke 全部完成。
- 完成记录写清本条只做校验与 smoke，不包含真实 LLM 修复调用，也不代表所有生成流程已接入。
- 验证命令和关键单测通过后，把本条改为 `【X】` 并进入 6.8。

**完成记录：**

- 已新增统一结构化输出 DTO：`ValidateStructuredOutputRequest / StructuredOutputValidationResult`，返回 `parsed_json / valid / errors / repair_needed / attempt_count / max_attempts`。
- 已新增 `structured_output_service::validate_structured_output`，统一处理 LLM 输出 JSON parse、Markdown fenced JSON 清理、schema 子集校验和 expected_count 校验；页面和业务层不需要各写一套 JSON 解析。
- 支持最小 JSON Schema 子集：`type / required / properties / items / enum / minItems / maxItems`；非 JSON 普通文本会被拒绝，不会被猜测成 JSON。
- `expected_count` 已支持根数组、`items`、`prompts`、`narrations` 的数量校验；数量不一致返回结构化错误，不写核心表。
- repair 次数已限制最多 2 次；失败且未到上限时返回 `repair_needed=true`，达到上限后不再标记可修复。本条只返回修复需求，不调用真实 LLM 修复。
- 已新增并注册 Tauri command `validate_structured_output`，前端命令表、类型和 config API 均已接入；浏览器 Mock 也按同一语义提供最小 schema 校验。
- “创作资源”页已新增“样例输出校验”区域，可对当前规则的 `output_schema` 粘贴 LLM 输出样例，填写 `expected_count / repair_attempt_count`，展示通过 / 失败、repair 状态、错误列表和解析后的 JSON。
- 本条只完成统一结构化输出校验和创作资源页 smoke，不包含真实 LLM repair 调用，不代表脚本、分镜、生图 prompt、视频 prompt、字幕、资产分析和审核等所有生成流程都已接入。
- 不合格输出不会通过本条接口写入核心表；后续生成流程接入时必须先调用统一校验入口，再决定 step failed 或 waiting_user。
- 验证通过：`cargo fmt` 已执行；`cargo test structured_output` 5 个结构化输出单测全部通过；`cargo check` 通过；`cargo test` 92 个 Rust 单测全部通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。

---

### 【X】6.8 实现 dry_run / real_generate 测试

**问题：**
Provider 保存成功不代表模型真实可用。

如果只有“保存配置成功”，用户会误以为 API 模型或 workflow 一定能执行；后续生图 / 视频 / TTS / VLM 真实任务失败时，很难区分是 Provider 连接、密钥、能力矩阵、workflow preset、参数边界还是真实供应商错误。真实生成还可能产生费用，不能在保存配置或刷新列表时自动触发。

**位置：**

```text
src-tauri/src/domain/provider.rs
src-tauri/src/services/provider_service.rs
src-tauri/src/commands/provider.rs
src-tauri/src/main.rs
src/src/entities/config/types.ts
src/src/entities/config/api.ts
src/src/pages/model-workflow/index.vue
src/src/shared/api/commands.ts
src/src/shared/i18n/locales/*
```

**改法：**

支持：

```text
dry_run：轻量连通性/参数校验
real_generate：真实调用，必须用户明确点击并二次确认
```

落地要求：

```text
1. 在 ProviderManager 上建立统一测试入口，测试模式显式区分 dry_run / real_generate。
2. 保留既有 provider_dry_run 兼容入口，但新增 provider_generation_test 作为页面主入口。
3. dry_run 只能做本地协议、密钥存在性、ProviderKind、provider_model_id / workflow_preset_id、能力矩阵、workflow preset 和参数边界校验；不得触发真实生成。
4. real_generate 必须由用户显式点击，后端必须校验 real_generate_confirmed=true；video real_generate 还必须带二次确认 token。
5. 当前阶段仍只走 dummy adapter，不发真实外部网络请求、不产生费用；真实供应商 adapter 后续接入时复用同一确认门禁。
6. 测试结果必须返回 trace_id、test_mode、provider_id、provider_kind、status、message、output_summary、billable、real_generate_confirmed。
7. output_summary 和错误 detail 不能包含 API Key、Bearer Token、secret、Authorization 等敏感信息。
8. “模型 / 工作流”页 Provider、API 模型、Workflow Preset、统一可执行选择器都能触发 dry_run；real_generate 只能通过明确确认按钮触发。
9. 测试失败只展示结构化错误，不写核心生成结果表，不创建真实作品产物。
```

**验收：**

- 后端提供 `provider_generation_test` 命令，并保留 `provider_dry_run` 兼容命令。
- LLM / Image / Video / TTS / VLM 的 dry_run 能通过 ProviderManager 对应能力入口完成 dummy 测试。
- API 模型测试会携带 `provider_model_id`，workflow 测试会携带 `workflow_preset_id`，二者不能混用。
- disabled provider / model / preset、缺失密钥、能力不匹配、workflow preset 缺 map 等仍会被后端拒绝。
- real_generate 没有确认时后端拒绝；video real_generate 没有二次确认 token 时后端拒绝。
- 当前 real_generate 只返回 dummy adapter 结果，明确 `billable=false`，不发真实外部调用、不产生费用。
- 前端页面能显示 test_mode、trace_id、message、output_summary 和失败错误，并提供 real_generate 的二次确认入口。
- 测试结果、错误信息、前端 DTO 和日志路径不包含真实 secret。

**验证：**

- LLM/Image/Video/TTS/VLM 能按能力测试。
- Video real_generate 有二次确认。
- 测试结果不泄露密钥。
- Rust 单测覆盖：五类 Provider dry_run、real_generate 未确认拒绝、video real_generate 缺 token 拒绝、确认后 dummy real_generate 成功且不计费、source id 混用拒绝、secret 不进结果。
- 前端 `typecheck` / `build` 通过。
- 手动 smoke：模型 / 工作流页选 Provider 或模型候选，dry_run 成功；点 real_generate 时必须确认，取消不会调用后端。

**风险：**
真实生成可能产生费用，不能保存配置时自动触发。

不能为了演示把 real_generate 做成自动调用；不能把测试结果当作真实生成产物入库；不能把 Provider 密钥写入返回结果、日志或诊断信息。

**下一步进入条件：**

- dry_run / real_generate 测试入口、后端确认门禁、前端二次确认、脱敏结果和验证命令全部完成。
- 完成记录写清当前 real_generate 仍是 dummy adapter 模拟，不包含真实供应商网络调用和费用产生。
- 验证命令和关键单测通过后，把本条改为 `【X】`，复核 TODO-06 阶段完成标准，再进入 TODO-07。

**完成记录：**

- 已新增统一测试 DTO：`ProviderGenerationTestRequest / ProviderGenerationTestResponse`，返回 `trace_id / test_mode / provider_id / provider_kind / status / message / output_summary / billable / real_generate_confirmed`。
- 已新增 `ProviderManager::generation_test` 和 Tauri command `provider_generation_test`；旧 `provider_dry_run` 兼容入口仍保留，并内部转到 `dry_run` 模式。
- `dry_run` 继续只做本地协议、密钥存在性、ProviderKind、`provider_model_id / workflow_preset_id`、能力矩阵、workflow preset 和参数边界校验，不触发真实生成。
- `real_generate` 已强制后端确认门禁：必须传 `real_generate_confirmed=true`；视频 Provider 和视频类 workflow preset 还必须传 `confirm_token=REAL_GENERATE_VIDEO`，否则后端拒绝。
- 当前 `real_generate` 仍只走 dummy adapter，返回 `billable=false / externalNetwork=false`，不发真实外部网络请求、不调用真实供应商、不产生费用，也不写真实生成产物。
- API 模型测试会携带 `provider_model_id`；workflow 测试会携带 `workflow_preset_id`；二者混用仍被 `provider.source_conflict` 拒绝。
- 测试结果只返回脱敏 summary；单测覆盖 keyring 里存在真实 secret 时，响应序列化后不包含 secret、Authorization 或 Bearer。
- “模型 / 工作流”页已改用 `provider_generation_test` 作为主测试入口；Provider、API 模型、Workflow Preset、统一可执行选择器都能触发 `dry_run`，并提供需用户确认的 `real_generate` 按钮。
- 页面会展示 `test_mode`、是否计费、trace_id、message、output_summary 和失败错误；取消确认不会调用后端。
- 本条只完成 dry_run / real_generate 测试门禁和 dummy smoke，不包含真实 OpenAI、DashScope、ComfyUI、RunningHub、Kling、Seedance 等供应商调用，不包含真实素材上传、远程 URL 下载或费用产生。
- 复核 TODO-06 阶段完成标准：ProviderManager、provider_models、workflow_presets、统一可执行选择器、ImageInputPlan / VideoInputPlan smoke、创作规则文件化、LLM 结构化校验、disabled / 未注册 / 越权参数后端拒绝均已覆盖；可以进入 TODO-07。
- 验证通过：`cargo fmt` 已执行；`cargo test generation_test` 4 个测试全部通过；`cargo check` 通过；`cargo test` 96 个 Rust 单测全部通过；`pnpm --dir src typecheck` 通过；`pnpm --dir src build` 通过，仍仅有 Vite chunk 体积超过 500KB 的既有提示。

---

## 阶段完成标准

- ProviderManager、ModelRegistry、WorkflowRegistry 可用。
- API 模型与 workflow preset 分离存储。
- 前端统一通过 `list_executable_media_options` 选择，并通过 `ImageInputPlan / VideoInputPlan` 动态渲染模型输入。
- 用户侧创作规则可查看、可复制为 user、可编辑、可启用，并有 schema 校验；完整版本/hash/snapshot 后续增强。
- LLM 核心输出必须 schema 校验。
- 未注册 workflow、越权参数、disabled 模型都会被后端拒绝。
