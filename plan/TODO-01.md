# TODO-01：工程协议与产品口径统一

> 目标：先统一产品主线、文档口径、工程分层、枚举字典、主题与国际化规则。  
> 本文件来自 `doc/README.md`、`doc/开始之前/*`、`doc/底层设计/00/01/02/03/04/10/22` 等文档的全量整理。

---

## 阶段目标

当前主线固定为：

```text
输入文字 → 分镜 → 生图 → 图生视频 → 合成
workflowType = image_to_video
```

本阶段要先把“产品口径”和“工程协议”定死，避免后续页面、数据、Provider、任务系统各做各的。

---

## 本阶段范围

包含：

- 产品主线与扩展线口径统一。
- `inputType` 与 `workflowType` 拆分。
- 前端 / 后端分层边界。
- 枚举、字典、配置源头。
- Tauri Command / DTO 命名规范。
- UI 主题、状态色、国际化、StepBar 规则。
- 文档冲突优先级说明。

不包含：

- SQLite 全量实现。
- 真实 Provider 接入。
- FFmpeg 真实合成。
- TTS / 字幕 / 封面 / BGM 实现。

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
> - 样式：页面和组件常规布局、间距、尺寸、文本、响应式默认用 Tailwind；只有主题 token、Tailwind `@theme`、Naive UI 覆盖、伪元素、动画、复杂渐变、滚动条、浏览器全局规则、必须运行时计算的 `:style` 等必要场景才写 SCSS/CSS 或 inline style。
> - 下一步：本条必须满足“下一步进入条件”后，才能打勾进入下一条；旧 TODO 缺字段时先补齐再实现。
> - 打标：完成一条后必须立刻把 `【】` 改成 `【X】`，并写完成记录；不需要用户提醒。
> - 停止：如果需求不清、验收不清、文档冲突、数据结构不明或风险高，停下来说明，不靠猜测继续。

### 【X】1.1 统一当前产品主线口径

**问题：**  
部分文档仍把当前主线写成“脚本 → TTS → 字幕 → 封面 → 模板渲染 → 合成”，与当前目标冲突。

**位置：**

```text
plan/阶段路线图.md
doc/README.md
doc/开始之前/这个产品是什么.md
doc/开始之前/功能地图.md
doc/底层设计/技术栈与架构.md
doc/底层设计/22-UI详细设计与页面布局.md
doc/底层设计/19-测试策略与验收用例.md
```

**改法：**

统一描述为：

```text
输入文字 → 分镜 → 生图 → 图生视频 → 合成
```

并明确：

```text
脚本独立页、TTS、字幕、封面、BGM、模板动效是增强项，不是当前 image_to_video 主线前置。
```

**验证：**

- 文档中不再把 `脚本/TTS/字幕/封面` 写成当前必经主线。
- 文档明确 `image_to_video` 是当前主 workflow。
- 合成被纳入当前闭环，不再放到遥远后续。

**风险：**  
不要删除这些功能需求，只改优先级和 workflow 归属。

**完成记录：**

- `plan/阶段路线图.md` 和 `doc/` 中当前主线已统一为 `输入文字 → 分镜 → 生图 → 图生视频 → 合成`。
- TTS、字幕、封面、BGM、模板动效保留为后续增强，不再作为当前主线前置。
- 前端工作台和 StepBar 已同步为 `分镜 → 生图 → 视频 → 合成`。

---

### 【X】1.2 拆分 `inputType` 与 `workflowType`

**问题：**  
当前部分描述把内容来源、制作流程、成片形态混在一起。

**改法：**

```text
inputType = 用户输入什么
workflowType = 最后走什么生产流程
```

建议枚举：

```text
InputType:
- topic
- paste
- article
- novel
- material

WorkflowType:
- image_to_video
- digital_human
- material_edit
- image_slideshow
```

**验证：**

- 用户从“我的作品”点击“开始创作”后自动创建作品草稿，并进入作品工作台的“内容创作”步骤。
- “内容创作”步骤选择作品类型、内容来源、视频包和基础设置，不再做独立“新建项目”表单。
- 当前只开放 `workflowType=image_to_video`。
- `digital_human/material_edit/image_slideshow` 可展示但必须标记为后续能力，不能混入当前主线。

**风险：**  
不要继续让 `videoType` 或旧 `short_video/ai_video` 承担 workflowType 职责。

**完成记录：**

- 前端新增 `WorkflowType`，扩展 `InputType=material`。
- `Project` 类型、创建请求和 mock 数据已使用 `workflowType`，不再使用 `videoType`。
- 当前实现曾以“新建项目页”承载创建入口；后续产品口径统一调整为“我的作品 → 开始创作 → 内容创作”，技术侧仍可使用 `Project` 表达作品草稿。
- 当前只开放 `image_to_video`，其他 workflow 展示但禁用。
- Tauri stub DTO 已补 `workflow_type`。

---

### 【X】1.3 建立前端分层规则

**问题：**  
页面若直接 `invoke`、直接写 mock、直接写枚举 options，后续会难以维护。

**改法：**

前端目录按：

```text
app
shared
entities
features
widgets
pages
```

强规则：

```text
页面不得直接调用 Tauri invoke
页面不得直接拼本地文件路径
页面不得直接调用 Provider
页面不得散落 mock 数组
页面不得硬编码枚举 options
页面不得硬编码状态颜色
```

统一入口：

```text
shared/api
entities/*/api
entities/*/store
features/*/actions
```

**验证：**

- 全局搜索页面中无直接 `@tauri-apps/api/core.invoke`。
- 页面 options 来自字典或 store。
- 状态颜色来自 theme token 或字典 colorToken。

**风险：**  
临时页面 mock 必须放在统一数据层或 adapter，不能散在页面里。

**完成记录：**

- 前端已确认使用 Vue3 + TypeScript + Vite + Pinia + vue-i18n + Naive UI。
- 已接入 `tailwindcss`、`@tailwindcss/vite`、`sass`；入口为 `styles/tailwind.css` + `styles/global.scss`。
- 样式边界已定为硬规则：页面和组件常规布局、间距、尺寸、文本、响应式优先 Tailwind；SCSS/CSS 只承载 design tokens、Tailwind `@theme`、Naive UI 覆盖、伪元素、动画、复杂渐变、滚动条、浏览器全局规则和必须运行时计算的 `:style`。
- 页面层无直接 `@tauri-apps/api/core.invoke` / `invoke(` / `callCommand(`。
- 分镜状态文案已从页面本地 map 收口到 `sceneAssetStatus` 字典；页面 options 继续通过 `useDictOptions` 获取。
- 主题预览色已从设置页硬编码颜色迁到 `shared/theme` helper，来源统一使用 `themeMap` token。
- 分镜预览占位样式已从页面内硬编码渐变迁到 `entities/scene/ui.ts` + `global.scss`，颜色使用 theme/status token。
- `pnpm --dir app/frontend typecheck` 和 `pnpm --dir app/frontend build` 已通过。

---

### 【X】1.4 建立后端分层规则

**问题：**  
Command 如果直接写业务流程、Provider、文件读写，会破坏测试和任务恢复。

**改法：**

后端建议分层：

```text
commands
core
domain
services
providers
db
security
utils
```

强规则：

```text
Command 只接 DTO、调 Service、返回 DTO
Service 承载业务流程
Repository 只负责数据库
Provider 不写数据库
Repository 不调用 Provider
所有文件读写走 StorageService + PathGuard
所有 AI 调用走 ProviderManager
所有任务状态走 TaskService
```

**验证：**

- Command 内无 Provider/FFmpeg/文件系统直调。
- Provider adapter 不依赖 SQLite repository。
- Repository 不依赖 Provider。

**风险：**  
如果第一版为了快把逻辑写进 Command，后面断点续跑和重试会大面积返工。

**完成记录：**

- Rust 后端已建立模块边界：`commands / domain / services / core / db / providers / security / utils`。
- DTO 已从 command 文件拆到 `domain/*`，当前 Tauri command 只接请求 DTO、调用 `services::*_service`、返回响应 DTO。
- 当前 mock/stub 业务响应已迁到 `services/*_service.rs`，不再写在 command 内。
- `core/db/providers/security/utils` 已建立模块入口和边界说明；本阶段未提前实现 SQLite、Provider、FFmpeg 或文件系统能力。
- 搜索确认 command 层无 Provider、FFmpeg、文件系统直调。
- `cargo check` 已通过。

---

### 【X】1.5 建立枚举与字典统一源头

**问题：**  
枚举、状态、显示文案、颜色如果前后端各写一套，会出现漂移。

**改法：**

第一版强协议枚举至少包含：

```text
AppLocale
ContentLanguage
ThemePreset
LayoutDensity
InputType
InputProcessMode
WorkflowType
ProjectLifecycle
TaskKind
TaskStatus
TaskStepKind
TaskStepStatus
ProviderKind
ProviderVendor
ProviderStatus
ProviderAuthType
ModelCapability
FileBucket
FileAccessPolicy
ErrorKind
```

字典集中在后端内置字典服务，前端通过：

```text
get_dictionary
list_dictionaries
useDictOptions
```

**验证：**

- 数据库存枚举 code，不存中文、不存数字下标。
- 页面无固定 options。
- 状态显示文案走 i18n 和 dictionary。

**风险：**  
第一版可以手写 TS 类型，但必须集中维护，不能页面私自复制。

**完成记录：**

- 前端核心枚举已集中在 `shared/enums/generated.ts`，覆盖 `AppLocale / ContentLanguage / ThemePreset / LayoutDensity / InputType / InputProcessMode / WorkflowType / ProjectLifecycle / TaskKind / TaskStatus / TaskStepKind / TaskStepStatus / ProviderKind / ProviderVendor / ProviderStatus / ProviderAuthType / ModelCapability / FileBucket / FileAccessPolicy / ErrorKind`，并额外包含当前页面需要的 `AspectRatio / SceneAssetStatus`。
- 前端字典已集中到 `shared/dict/dict.registry.ts` 的 `dictRegistry`；`useDictOptions`、`entities/dictionary/api.ts`、`shared/dict/appOptions.ts` 都从同一源头读取，不再重复定义 options。
- 后端内置字典服务 `services/dictionary_service.rs` 已补齐同一批核心字典 code，`list_dictionaries` 返回统一列表，`get_dictionary` 对未知 code 返回错误，不再静默回落到 `inputType`。
- 首页 `workflowType` 和 `projectLifecycle`、分镜页 `sceneAssetStatus` 已通过 dictionary 显示；页面内未检出本地固定 options 或本地状态文案表。
- 用户可见文案的全量 i18n 不提前塞入本条，继续由 `1.6 建立 UI 主题、状态色与国际化规则` 承接；本条已完成枚举、字典、状态文案源头收口。
- 样式执行边界已同步写入 `plan/阶段路线图.md`、`plan/README.md`、当前 TODO 执行规则和 `styles/global.scss` 文件头。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

### 【X】1.6 建立 UI 主题、状态色与国际化规则

**问题：**  
UI 如果只改 primary 色、文案硬编码中文，会影响后续主题和语言切换。

**改法：**

实现：

```text
primitive tokens
semantic tokens
component tokens
ThemePreset: graphite / aurora / ember / porcelain / sandstone
LayoutDensity: compact / comfortable
AppLocale: zh-CN / en-US
ContentLanguage 独立于 AppLocale
```

当前 StepBar：

```text
分镜 → 生图 → 视频 → 合成
```

**验证：**

- 用户可见文案走 i18n。
- 状态色走 theme token / dictionary colorToken。
- `appLocale` 和 `contentLanguage` 不混用。
- Naive UI 作为基础组件库，不手搓基础控件。

**风险：**  
不要把内容语言当界面语言使用。

**完成记录：**

- 前端主题已通过 `App.vue` 在启动时加载配置，并同步 `themePreset / layoutDensity / appLocale`；主题变量写入 `documentElement`，`body[data-density]` 控制密度，`document.documentElement.lang` 跟随 `appLocale`。
- Tailwind v4 已通过 `styles/tailwind.css` 的 `@theme inline` 消费语义 token；SCSS 保留为主题变量来源、浏览器全局规则、Naive UI 覆盖、伪元素、动画、复杂渐变、滚动条等必要场景。
- 五套 `ThemePreset` 已补齐状态色 CSS 变量 `--st-*`；Tailwind 暴露 `--color-status-*`；状态显示通过 dictionary `colorToken` + `shared/theme/status.ts` 映射到状态 class，不再让页面直接硬编码状态色。
- `useDictOptions` 已接入 `vue-i18n`，字典 label 会随 `appLocale` 响应式切换；字典 code/value/colorToken 保持稳定。
- 当前路由页面和 `AppShell` 的主要用户可见 UI 文案已迁到 `shared/i18n/locales/zh-CN.ts` 与 `en-US.ts`；页面层搜索只剩业务示例内容，不再作为界面文案硬编码。
- `appLocale` 仅用于界面语言；`contentLanguage` 仍作为项目内容语言字段，并通过 `contentLanguage` 字典展示，没有混用。
- StepBar 继续固定为 `分镜 → 生图 → 视频 → 合成`，对应英文为 `Storyboard → Images → Video → Composition`。
- 验证通过：`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend typecheck`、`pnpm --dir D:\project\vt-ai-short-video-maker\app\frontend build`、`cargo check`。

---

## 阶段完成标准

- 文档主线口径统一为 `image_to_video`。
- 前端、后端分层规则落文档并开始在代码中执行。
- 核心枚举、字典、主题、i18n 有统一入口。
- 页面不直接 invoke 的规则明确。
- 当前主线 StepBar 固定为 `分镜 → 生图 → 视频 → 合成`。







