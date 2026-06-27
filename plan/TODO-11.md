# TODO-11：桌面打包、发布与合规

> 目标：确保开发环境能跑不等于发布版能跑，桌面打包、sidecar、初始化、授权合规必须单独验收。  
> 本文件来自 `doc/底层设计/20-桌面打包发布与更新规范.md`、`13-FFmpegSidecar与媒体处理规范.md`、`14-模板渲染与Chromium安全规范.md`。

---

## 阶段目标

完成第一版桌面发布基础：

```text
Windows 10/11 x64
Tauri 打包
FFmpeg / FFprobe sidecar
Chromium / Playwright 策略
首次启动初始化
打包后自检
备份恢复验证
授权合规清单
```

---

## 本阶段范围

包含：

- Windows 打包。
- 随包资源清单。
- 首次启动初始化。
- sidecar 自检。
- 发布版 smoke test。
- 安装包和更新预留。
- 许可证和素材授权检查。

不包含：

- 多平台完整发布。
- 自动更新服务端实现。
- 商店上架流程。

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

### 【X】11.1 明确第一版发布平台

**问题：**  
多平台同时处理会拖慢 MVP，Windows 路径和 sidecar 问题已经足够复杂。

**位置：**

```text
src-tauri/tauri.conf.*
src-tauri/Cargo.toml
app/package.json
doc/发布* 或 doc/底层设计/发布相关文档
plan/TODO-11.md
```

本条先锁定发布目标和验收环境，不做多平台打包矩阵。

**改法：**

第一版优先：

```text
Windows 10/11 x64
```

后续再扩展：

```text
macOS
Linux
```

发布基线：

```text
1. Windows 10/11 x64 是第一版唯一强制发布目标。
2. 开发路径、安装路径、workspace 路径必须覆盖中文、空格、长路径基础场景。
3. sidecar、migrations、内置规则、静态资源必须按 Windows 安装包检查。
4. macOS/Linux 只保留后续扩展说明，不作为当前阻塞项。
```

**验收：**

- Windows 10/11 x64 打包可安装、可启动。
- 工作区默认路径合理。
- 中文路径、空格路径可用。
- 发布文档明确第一版不承诺 macOS/Linux。
- 后续 TODO-11 条目都按 Windows 优先验收，不再混入多平台要求。

**验证：**

- 检查 Tauri 配置、package scripts 和发布说明中 Windows 目标一致。
- 在带中文或空格的工作区路径运行 dev/build smoke。
- 如执行安装包测试，记录 Windows 版本、安装路径和 workspace 路径。

**下一步进入条件：**

- 第一版发布平台、测试环境和不支持范围写清楚。
- 完成记录写清是否已经实际打包；如果未打包，只能标记为“发布目标锁定”，不能写成安装包已验证。
- 确认 11.2 随包资源清单只按 Windows 第一版整理后，再把本条改为 `【X】` 并进入 11.2。

**风险：**  
Windows junction、UNC、权限和杀软拦截要重点测试。

**完成记录（2026-06-27）：**

- 已锁定第一版发布目标为 `Windows 10/11 x64`，后续 macOS / Linux 只保留扩展说明，不作为 TODO-11 第一版阻塞项。
- `src-tauri/tauri.conf.json` 已启用 bundle，并把 bundle target 明确为 `nsis`；Windows NSIS 安装模式使用 `currentUser`，避免第一版默认要求管理员权限。
- `package.json` 已新增 `tauri:build:windows`，命令固定为 `tauri build --config src-tauri/tauri.conf.json --target x86_64-pc-windows-msvc --bundles nsis`。
- `doc/底层设计/20-桌面打包发布与更新规范.md` 已补第一版发布基线，明确 Windows 目标、路径场景和不承诺 macOS/Linux。
- 已确认 Tauri CLI schema 中 `nsis`、`installMode=currentUser`、`displayLanguageSelector` 字段存在且取值合法。
- 本条只完成“发布目标锁定”和工程入口约束；尚未执行真实 NSIS 安装包构建、安装、启动 smoke，不写成安装包已验证。
- 已确认 11.2 继续只按 Windows 第一版整理随包资源清单，不混入多平台矩阵。

**已执行验证：**

```text
Node JSON/schema check：package.json 与 src-tauri/tauri.conf.json 可解析，Tauri schema 字段/枚举可查。
pnpm --dir src typecheck：通过。
pnpm --dir src build：通过；仍有既有动态/静态 import 和 chunk 体积警告。
cargo check：通过。
```

---

### 【X】11.2 整理随包资源清单

**问题：**  
开发环境文件齐全不代表安装包包含运行所需资源。

**位置：**

```text
src-tauri/tauri.conf.json
package.json
resources/bin/README.md
scripts/prepare-template-sidecar.ps1
scripts/verify-release-resources.ps1
doc/底层设计/20-桌面打包发布与更新规范.md
plan/TODO-11.md
```

**改法：**

安装包必须包含：

```text
前端静态资源
Rust 后端
FFmpeg
FFprobe
内置模板
内置 PromptSkill
内置字体/图标资源
默认配置 schema
SQLite migrations
Playwright driver / Chromium 策略
默认字典
```

本条只建立 Windows 第一版随包资源清单、打包源目录和发布前校验，不下载第三方二进制，不把缺失资源写成已随包。

资源归属：

```text
前端静态资源：src/dist，由 pnpm --dir src build 生成。
Rust 后端：src-tauri 编译产物，由 Tauri build 打包。
FFmpeg / FFprobe：resources/bin/ffmpeg.exe、resources/bin/ffprobe.exe。
Chromium / Playwright：resources/bin/node.exe、resources/bin/chromium.exe、resources/bin/chromium/chrome.exe、resources/bin/playwright-driver.js、resources/bin/node_modules/playwright-core/。
内置模板：当前由 Rust template_service 种子写入 workspace，不依赖源码 templates/ 目录随包。
内置 PromptSkill / 创作规则：当前由 Rust prompt_service 种子写入 workspace，不依赖外部 PromptSkill 目录随包。
默认字典：当前由 Rust dictionary_service 代码内置。
默认配置 schema：当前由配置服务 / Repository 代码内置。
SQLite migrations：当前由 src-tauri/src/db/mod.rs 的 MIGRATIONS 常量内置。
```

打包前新增：

```text
pnpm run verify:release-resources
```

**验证：**

- `pnpm run verify:release-resources` 能在缺少必需 sidecar 打包源时失败并指出缺什么。
- `pnpm run verify:release-resources` 在资源齐全后通过，才能执行 `pnpm run tauri:build:windows`。
- `-SkipHeavySidecars` 只用于验证脚本自身和配置清单，不得作为发布前通过依据。
- 全新机器安装后无需源码目录即可启动这一点放到 11.5 发布版 smoke test 验收。
- migrations 可执行放到 11.3 首次启动初始化和 11.5 smoke test 验收。

**验收：**

- 发布文档写清每类资源的打包来源和当前是“外部文件随包”还是“Rust 内置初始化”。
- Tauri `bundle.resources` 覆盖 Windows 第一版外部 sidecar 打包源。
- 发布前有固定脚本检查 FFmpeg、FFprobe、Node、Chromium、Playwright 打包源。
- 当前缺失的重型 sidecar 以失败项暴露，不能静默依赖全局安装。
- 11.3 可以基于这份清单实现首次启动初始化和资源复制策略。

**下一步进入条件：**

- 清单、脚本、文档和 package script 已落地。
- 完成记录必须写清当前 `resources/bin` 哪些文件仍缺失。
- 不把 `-SkipHeavySidecars` 的通过写成发布资源齐全。
- 明确 11.3 继续处理首次启动把随包资源初始化到应用数据 / workspace 的策略。

**风险：**  
不要依赖开发机上的全局 ffmpeg、node、pnpm、chrome。

**完成记录（2026-06-27）：**

- 已补齐本条位置、验收、验证和下一步进入条件，明确 11.2 只负责资源清单和打包前拦截，不下载第三方二进制、不冒充安装包 smoke。
- `doc/底层设计/20-桌面打包发布与更新规范.md` 已写清 Windows 第一版资源归属：
  - 前端静态资源由 `src/dist` 随 Tauri 打包。
  - Rust 后端由 `src-tauri` 编译产物随 Tauri 打包。
  - FFmpeg / FFprobe、Node、Chromium、Playwright 必须进入 `resources/bin`。
  - 内置模板、内置 PromptSkill / 创作规则、默认字典、默认配置 schema、SQLite migrations 当前由 Rust 代码内置初始化，不依赖外部源码目录随包。
- 新增 `scripts/verify-release-resources.ps1`，发布前检查 `tauri:build:windows`、Windows x64 target、NSIS bundle、`bundle.resources` 和必需 sidecar 打包源。
- `package.json` 已新增 `verify:release-resources` 脚本。
- `scripts/prepare-template-sidecar.ps1` 已改为同时复制 Chromium launcher 和完整 `chromium/` 目录，避免只打包 `chromium.exe` 导致运行时缺 DLL / pak / locales。
- `resources/bin/README.md` 已更新期望资源说明。
- 当前实际状态：`resources/bin` 仍只有 README，缺少 `ffmpeg.exe`、`ffprobe.exe`、`node.exe`、`chromium.exe`、`chromium/chrome.exe`、`playwright-driver.js` 和 `node_modules/playwright-core/`；完整 `pnpm run verify:release-resources` 会失败，这是发布前正确阻断。
- 已明确 11.3 继续处理首次启动初始化和运行时资源落点策略；11.5 再做安装包安装、启动和离线 smoke。

**已执行验证：**

```text
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-release-resources.ps1 -SkipHeavySidecars：通过，仅证明脚本和配置清单可执行。
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-release-resources.ps1：按预期失败，错误为缺少 resources/bin/ffmpeg.exe。
pnpm --dir src typecheck：通过。
cargo check：通过。
```

---

### 【X】11.3 实现首次启动初始化

**问题：**  
发布版首次启动要自动准备数据目录、数据库和内置资源。

**位置：**

```text
src-tauri/src/main.rs
src-tauri/src/services/startup_service.rs
src-tauri/src/services/mod.rs
src-tauri/src/services/storage_service.rs
src-tauri/src/db/mod.rs
src-tauri/src/db/config_repository.rs
src-tauri/src/services/template_service.rs
src-tauri/src/services/prompt_service.rs
plan/TODO-11.md
```

**改法：**

首次启动流程：

```text
创建应用数据目录
初始化 SQLite
执行 migrations
初始化默认配置
初始化内置字典
扫描内置模板
扫描 PromptSkill
检查 sidecar
进入欢迎/设置引导
```

本条后端先完成启动初始化，不做欢迎页 UI；欢迎 / 设置引导如需要独立页面，后续在系统设置或发布 smoke 阶段补。

**验证：**

- 删除数据目录后重新启动可完整初始化。
- 初始化失败有可恢复错误。
- 不会覆盖用户已有配置。

**验收：**

- Tauri `setup` 不再只手动创建 `workspace`，而是走统一启动初始化服务。
- 应用数据目录、workspace buckets、SQLite、migrations、默认配置、内置模板、内置创作规则会在首次启动自动准备。
- 随包 sidecar 如存在，会复制到受控 `workspace/sidecars`；已有运行时 sidecar 不覆盖。
- sidecar 缺失不阻止应用打开，只记录 ready 状态，相关任务启动前仍由 11.4 / 现有 require 检查阻断。
- 初始化不写入源码目录或安装目录。

**下一步进入条件：**

- 定向单测覆盖首次初始化、幂等初始化、随包 sidecar 复制且不覆盖已有运行时文件。
- `cargo fmt --check`、`cargo check`、前端 typecheck 通过。
- 完成记录写清未执行真实安装包首次启动 smoke；该 smoke 留到 11.5。

**风险：**  
初始化过程不能写入源码目录或安装目录。

**完成记录（2026-06-27）：**

- 新增 `startup_service::initialize_app_runtime`，Tauri `setup` 已改为统一调用该服务。
- 初始化流程已覆盖：
  - 创建应用数据目录。
  - 创建 `workspace` 和 `projects/assets/outputs/cache/temp/logs/templates/sidecars` buckets。
  - 打开 SQLite 并执行 `MIGRATIONS`。
  - 初始化默认 `app/pipeline/ui/export` 配置。
  - 写入内置模板。
  - 写入内置创作规则 / PromptSkill。
  - 扫描并标记可恢复任务。
  - 检查 FFmpeg 和模板 sidecar ready 状态。
- 新增从 `app_data_dir/resources/bin` 到 `workspace/sidecars` 的随包 sidecar 初始化复制逻辑；只在目标不存在时复制，不覆盖用户已有运行时文件。
- sidecar 缺失只让 `ffmpeg_ready/template_sidecar_ready=false`，不阻止应用打开；真正媒体/模板任务仍在启动任务前检查 sidecar。
- 本条未执行真实 NSIS 安装包首次启动 smoke，也未验证卸载/保留数据；这些留到 11.5。

**已执行验证：**

```text
cargo fmt --check：通过。
cargo test startup_service -- --nocapture：2 passed。
cargo check：通过。
pnpm --dir src typecheck：通过。
```

---

### 【X】11.4 实现 sidecar 自检

**问题：**  
FFmpeg、FFprobe、Chromium 是打包后最容易失效的部分。

**位置：**

```text
src-tauri/src/domain/diagnostic.rs
src-tauri/src/services/diagnostic_service.rs
src-tauri/src/commands/diagnostic.rs
src-tauri/src/main.rs
src-tauri/src/services/ffmpeg_service.rs
src-tauri/src/services/template_service.rs
src/src/entities/config/types.ts
src/src/entities/config/api.ts
src/src/pages/settings/index.vue
src/src/shared/i18n/locales/zh-CN.ts
src/src/shared/i18n/locales/en-US.ts
plan/TODO-11.md
```

**改法：**

自检：

```text
ffmpeg -version
ffprobe -version
Chromium 可启动
默认模板可截图
工作区可读写
SQLite 可打开
```

本条只做运行时自检聚合和设置页展示；真实安装包中的完整 sidecar 可执行验证仍在 11.5 smoke test。

**验证：**

- 设置页或诊断页显示自检状态。
- sidecar 缺失不阻止打开应用。
- 相关任务启动前必须检查 sidecar。

**验收：**

- 后端提供统一 `run_runtime_self_check` command，返回 workspace、SQLite、FFmpeg/FFprobe、Chromium/Playwright、内置模板扫描、默认模板截图检查项。
- 设置页展示运行环境自检、视频合成工具和模板渲染工具状态。
- FFmpeg / FFprobe 仍通过 `require_ffmpeg_sidecars` 在合成、探测、字幕、BGM 等任务启动前阻断。
- 模板渲染仍通过 `require_template_sidecars` 在真实渲染前阻断。
- sidecar 缺失时自检返回 not ready / skipped，不阻止应用打开。

**下一步进入条件：**

- Rust 单测覆盖 sidecar 缺失时核心检查仍可通过、模板截图被跳过。
- 前端 typecheck/build 通过。
- 完成记录写清当前本机仍缺 `resources/bin` 发布资源，未做真实 sidecar smoke。

**风险：**  
sidecar 路径必须受控，不能让用户输入任意可执行路径直接执行。

**完成记录（2026-06-27）：**

- 新增 `RuntimeSelfCheckDto` 和 `RuntimeCheckItemDto`。
- 新增 `diagnostic_service::run_runtime_self_check`，聚合：
  - workspace 读写检查。
  - SQLite `SELECT 1` 检查。
  - FFmpeg / FFprobe sidecar 检查。
  - Node / Chromium / Playwright sidecar 检查。
  - 内置模板 manifest 扫描。
  - 默认模板截图检查；模板 sidecar 缺失时标记 skipped。
- 新增 Tauri command `run_runtime_self_check`，前端通过 `entities/config/api.ts` 调用。
- 设置页“本地媒体工具”卡片已改为展示运行环境自检、视频合成工具、模板渲染工具三组状态。
- 用户可见文案已补 `zh-CN/en-US`。
- 本机当前缺真实发布 sidecar，因此真实 FFmpeg/Chromium 可执行 smoke 未跑；完整发布资源检查仍会因 `resources/bin/ffmpeg.exe` 缺失而失败。

**已执行验证：**

```text
cargo fmt --check：通过。
cargo test diagnostic_service -- --nocapture：1 passed。
cargo check：通过。
pnpm --dir src typecheck：通过。
pnpm --dir src build：通过；仍有既有动态/静态 import 和 chunk 体积警告。
```

---

### 【X】11.5 建立发布版 smoke test

**问题：**  
源码环境测试通过不代表安装包可用。

**位置：**

```text
package.json
scripts/verify-release-smoke.ps1
plan/release-smoke-test.md
plan/测试体系.md
plan/TODO-11.md
```

**改法：**

发布版 smoke test：

```text
安装应用
首次启动
初始化数据库
打开系统设置
检查 sidecar
从“我的作品”点击“开始创作”
Mock 主线跑通
真实 FFmpeg 小样例合成
导出 final.mp4
重启恢复项目
导出诊断包
卸载/保留数据策略检查
```

本条只建立发布版 smoke 的固定清单、前置门禁和预检脚本，不把当前缺少 sidecar 的状态写成安装包 smoke 通过。真实安装、启动、离线和卸载验证必须等 `verify:release-resources` 通过并成功生成 NSIS 安装包后执行。

**验证：**

- 全新 Windows 环境通过。
- 中文路径和空格路径通过。
- 离线环境可打开应用。
- `pnpm run verify:release-smoke` 会先执行资源门禁、Rust / 前端检查和 Windows NSIS build。
- `verify:release-resources` 失败时，smoke 预检必须整体失败并停止，不继续后续构建或安装验证。
- 真实 Provider、真实费用和真实 FFmpeg 小样例合成必须单独记录，不能用 controlled fake 替代。

**验收：**

- 有固定发布版 smoke 清单，明确只认 Windows NSIS 安装包，不认浏览器 dev server 或 Tauri dev。
- 清单覆盖首次启动初始化、设置页自检、项目创建、Mock 主线、真实 FFmpeg 小样例、导出、重启恢复、诊断包、卸载策略。
- 有固定预检脚本串联 `verify:release-resources`、`cargo fmt --check`、`cargo check`、前端 typecheck/build 和 `tauri:build:windows`。
- 预检脚本对原生命令退出码敏感，任一步失败都会退出非 0。
- 缺少 `resources/bin/*` 重型 sidecar 时必须阻断安装包 smoke，并在完成记录里写清未执行真实安装包验证。

**下一步进入条件：**

- 发布版 smoke 文档和预检脚本已落地。
- 已验证当前缺 `resources/bin/ffmpeg.exe` 时，`verify-release-smoke.ps1 -SkipTauriBuild` 会在第一步失败并返回非 0。
- 完成记录明确：当前没有生成 / 安装 NSIS 包，没有把发布版 smoke 写成通过。
- 11.6 可以继续做签名、版本和更新预留；真实安装包 smoke 等资源补齐后再按 `plan/release-smoke-test.md` 执行。

**风险：**  
真实 Provider smoke test 需要显式密钥和用户确认，不能默认跑。

**完成记录（2026-06-27）：**

- 新增 `plan/release-smoke-test.md`，明确发布版 smoke 只认 Windows NSIS 安装包，不认浏览器 dev server、单测、typecheck 或 Tauri dev。
- 新增 `scripts/verify-release-smoke.ps1`，按顺序执行发布资源门禁、Rust 格式检查、Rust check、前端 typecheck、前端 build，并在未传 `-SkipTauriBuild` 时执行 `pnpm run tauri:build:windows`。
- `package.json` 已新增 `verify:release-smoke` 脚本。
- 发布版 smoke 清单已覆盖安装、首次启动、SQLite / workspace 初始化、设置页自检、sidecar 状态、创建作品、controlled fake 主线、真实 FFmpeg 小样例、导出、重启恢复、诊断包和卸载策略。
- 已修复 `verify-release-smoke.ps1` 对原生命令退出码不敏感的问题：`pnpm run verify:release-resources` 失败后脚本会立即 throw 并返回非 0，不再继续后续步骤。
- 当前真实安装包 smoke 未执行：`resources/bin` 仍缺 `ffmpeg.exe` 等发布 sidecar，`verify:release-resources` 正确阻断；没有生成或安装 NSIS 包，不能写成发布版 smoke 通过。

**已执行验证：**

```text
cmd /c powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\verify-release-smoke.ps1 -SkipTauriBuild：按预期失败，第一步缺少 resources/bin/ffmpeg.exe，脚本整体 exit 1。
```

---

### 【X】11.6 建立安装包签名与更新预留

**问题：**  
未签名安装包容易被拦截；更新机制后续要预留安全边界。

**位置：**

```text
src-tauri/src/domain/diagnostic.rs
src-tauri/src/services/diagnostic_service.rs
src-tauri/src/commands/diagnostic.rs
src-tauri/src/services/export_service.rs
src-tauri/src/main.rs
src/src/entities/config/types.ts
src/src/entities/config/api.ts
src/src/pages/settings/index.vue
src/src/shared/api/commands.ts
src/src/shared/i18n/locales/zh-CN.ts
src/src/shared/i18n/locales/en-US.ts
scripts/verify-release-policy.ps1
scripts/verify-release-smoke.ps1
plan/release-policy.md
plan/TODO-11.md
```

**改法：**

第一版记录：

```text
代码签名策略
安装包生成流程
版本号规则
更新源 HTTPS 要求
更新包签名校验预留
回滚策略预留
```

本条不伪造代码签名，不启用自动更新；只建立版本 / 发布信息通道、诊断包记录、设置页展示和静态策略门禁。

**验证：**

- 版本号能在应用和诊断包中显示。
- 更新相关配置不含密钥。
- `pnpm run verify:release-policy` 检查版本一致、自动更新未启用、无 HTTP 更新源、无密钥字段。
- 诊断包 `summary.json` 包含 release 信息，且不包含真实密钥或本机绝对路径。

**验收：**

- 后端提供 `get_app_release_info` command，返回版本号、Windows x64 平台、手动更新通道、签名要求和更新安全要求。
- 设置页显示“版本与更新”，用户能看到应用版本、发布平台、自动更新状态、安装包签名状态和更新安全策略。
- 诊断包 `summary.json` 写入同一份 release 信息。
- 发布策略文档明确第一版不启用自动更新，后续更新必须 HTTPS + 签名校验。
- 发布前脚本能阻断版本不一致、HTTP 更新源和疑似密钥配置。

**下一步进入条件：**

- `cargo fmt --check`、`cargo test diagnostic_service -- --nocapture`、`cargo test export_service::tests::export_diagnostic_package_writes_summary_and_redacted_logs -- --nocapture`、`cargo check`、`pnpm --dir src typecheck` 通过。
- 完成记录写清当前开发构建未做真实代码签名，也未生成已签名安装包。
- 11.7 可以继续整理许可证和素材授权清单。

**风险：**  
不要从非 HTTPS 或未签名来源自动更新。

**完成记录（2026-06-27）：**

- 新增 `AppReleaseInfoDto` 和 `get_app_release_info` command，发布信息固定返回应用名、版本号、identifier、`windows-x64`、手动更新通道、自动更新关闭、HTTPS 要求、更新包签名要求、安装包签名要求和当前签名验证状态。
- 设置页新增“版本与更新”卡片，显示应用版本、发布平台、更新通道、自动更新状态、安装包签名状态和更新安全策略；文案已补 `zh-CN/en-US`。
- 诊断包 `summary.json` 已加入 `release` 字段，并继续经过密钥扫描和绝对路径扫描。
- 新增 `scripts/verify-release-policy.ps1` 和 `package.json` 脚本 `verify:release-policy`，检查四处版本号一致、第一版未启用 updater、配置无 `http://` 更新源、配置无疑似密钥字段。
- `scripts/verify-release-smoke.ps1` 已串入 `verify:release-policy`，资源门禁通过后会继续检查发布策略。
- 新增 `plan/release-policy.md`，明确第一版不启用自动更新，后续自动更新必须 HTTPS、manifest / 更新包签名校验、更新前备份、migration 和回滚策略。
- 当前未生成真实已签名 NSIS 安装包，也未执行签名验证；设置页会如实显示“要求签名，当前未验证”。

**已执行验证：**

```text
pnpm run verify:release-policy：通过，版本 0.1.0，一版自动更新保持关闭。
cargo fmt --check：通过。
cargo test diagnostic_service -- --nocapture：2 passed。
cargo test export_service::tests::export_diagnostic_package_writes_summary_and_redacted_logs -- --nocapture：1 passed。
cargo check：通过。
pnpm --dir src typecheck：通过。
```

---

### 【X】11.7 建立授权合规清单

**问题：**  
FFmpeg、Chromium、Playwright、字体、模板素材都有授权风险。

**位置：**

```text
resources/licenses/THIRD_PARTY_NOTICES.md
src-tauri/tauri.conf.json
scripts/verify-license-compliance.ps1
scripts/verify-release-smoke.ps1
package.json
plan/license-compliance.md
plan/TODO-11.md
```

**改法：**

检查并记录：

```text
FFmpeg license
FFprobe license
Chromium license
Playwright license
Naive UI / Vue / 依赖 license
字体授权
图标授权
模板素材授权
参考项目许可证边界
```

本条建立授权清单、随包 notice 和发布前门禁；不下载第三方二进制，不把当前缺失的 FFmpeg / Chromium / Node / Playwright 打包源写成已合规。

**验证：**

- 安装包包含 license 说明。
- 不使用来源不明字体/模板/素材。
- 参考项目代码不直接复制污染许可证。
- `pnpm run verify:license-compliance` 在缺少必需 sidecar 授权对象时失败。
- `scripts/verify-license-compliance.ps1 -AllowMissingSidecars` 只用于验证清单、notice 和脚本逻辑，不得作为发布合规通过依据。

**验收：**

- `resources/licenses/THIRD_PARTY_NOTICES.md` 存在并随 Tauri 打包。
- `plan/license-compliance.md` 写清依赖、sidecar、模板、字体、图标、参考项目的授权状态和发布前动作。
- 发布前有固定脚本检查 license notice、Tauri resources、内置模板外链和 sidecar 授权对象。
- 参考项目边界明确：只借鉴机制，不直接复制代码 / 资产 / 模板 / Prompt / 二进制。
- 当前缺失的 FFmpeg / FFprobe / Node / Chromium / Playwright 打包源以失败项暴露，不冒充已授权。

**下一步进入条件：**

- `pnpm run verify:license-compliance` 的失败原因符合当前 sidecar 缺失事实。
- `scripts/verify-license-compliance.ps1 -AllowMissingSidecars` 通过，用于证明清单、notice 和打包配置可检查。
- `pnpm --dir src typecheck`、`cargo check` 通过。
- 11.8 可以继续把发布前 checklist 固化，并把 license 检查纳入发布前门禁。

**风险：**  
特别注意 FFmpeg 编码器组合的 LGPL/GPL 边界。

**完成记录（2026-06-27）：**

- 新增 `resources/licenses/THIRD_PARTY_NOTICES.md`，作为 Windows 安装包随包 notice 入口，记录应用依赖、运行时 sidecar、项目自有模板和参考项目边界。
- `src-tauri/tauri.conf.json` 的 `bundle.resources` 已加入 `../resources/licenses/**/*`。
- 新增 `plan/license-compliance.md`，按桌面框架、前端依赖、Rust 依赖、SQLite、FFmpeg / FFprobe、Node、Chromium、Playwright、内置模板、字体、图标和参考项目写清当前状态和发布前动作。
- 新增 `scripts/verify-license-compliance.ps1` 和 `package.json` 脚本 `verify:license-compliance`，检查 notice、Tauri 打包资源、内置模板外链和必需 sidecar 授权对象。
- `scripts/verify-release-smoke.ps1` 已串入 `verify:license-compliance`，资源和发布策略通过后会继续执行授权合规门禁。
- 当前真实发布合规未通过：`resources/bin` 仍缺 FFmpeg / FFprobe / Node / Chromium / Playwright 打包源，完整 `verify:license-compliance` 会失败，这是正确阻断；没有写成安装包授权已通过。

**已执行验证：**

```text
pnpm run verify:license-compliance：按预期失败，错误为缺少 resources/bin/ffmpeg.exe，需要先记录 FFmpeg 来源和授权。
powershell -ExecutionPolicy Bypass -File scripts\verify-license-compliance.ps1 -AllowMissingSidecars：通过，仅证明清单、notice、Tauri resources 和脚本逻辑可检查；不得作为发布合规通过依据。
pnpm --dir src typecheck：通过。
cargo check：通过。
```

---

### 【X】11.8 建立发布前检查清单

**问题：**  
发布动作不可逆，必须有固定 checklist。

**位置：**

```text
plan/release-checklist.md
scripts/verify-release-checklist.ps1
scripts/verify-release-smoke.ps1
package.json
plan/TODO-11.md
```

**改法：**

发布前检查：

```text
前端 typecheck
前端 build
cargo test
Tauri build
发布版 smoke test
sidecar 自检
诊断包脱敏检查
项目包导出检查
许可证检查
版本号检查
安装卸载检查
```

本条只固化发布前清单和总门禁脚本；真实安装包 smoke、安装卸载和离线环境记录仍必须在资源补齐并构建 NSIS 后按 `plan/release-smoke-test.md` 手动执行。

**验证：**

- 每次发布前按 checklist 执行。
- 失败项不得跳过发布。
- `pnpm run verify:release-checklist` 会串联资源、发布策略、授权、Rust、前端和 Windows NSIS build 前置门禁。
- 当前缺 sidecar 时，发布前检查必须失败，不能绕过。

**验收：**

- 有固定 `plan/release-checklist.md`，明确发布前必跑项、失败处理和禁止替代项。
- 有固定 `verify:release-checklist` 脚本作为发布前入口。
- 总门禁复用 `verify-release-smoke.ps1`，避免维护两套发布检查顺序。
- 当前资源缺失时总门禁失败，失败原因可追溯到 `resources/bin/*`。
- 阶段完成标准中未满足的真实安装包构建 / 安装 / smoke 仍保留为阻断，不写成已发布。

**下一步进入条件：**

- `pnpm run verify:release-checklist` 已验证会因当前缺 `resources/bin/ffmpeg.exe` 失败。
- `pnpm --dir src typecheck`、`cargo check` 通过。
- TODO-11 全部条目打勾后，阶段总结必须写清哪些是真完成，哪些仍因真实 sidecar / NSIS smoke 未执行而阻断发布。

**风险：**  
不要只以开发模式 `tauri:dev` 能跑作为发布依据。

**完成记录（2026-06-27）：**

- 新增 `plan/release-checklist.md`，固定发布前必跑项：资源清单、发布策略、授权合规、Rust 格式、Rust check、前端 typecheck/build、Windows NSIS build、发布版 smoke、诊断包脱敏、项目包导出、安装卸载。
- 新增 `scripts/verify-release-checklist.ps1` 和 `package.json` 脚本 `verify:release-checklist`，作为发布前总入口。
- 总入口复用 `scripts/verify-release-smoke.ps1`，确保发布资源、发布策略、授权合规、Rust、前端和 Tauri build 前置检查顺序一致。
- 文档明确禁止用 `tauri:dev`、浏览器 dev server、controlled fake 或缺 sidecar 的状态替代发布版 smoke。
- 当前真实发布仍被阻断：`resources/bin` 缺 FFmpeg / FFprobe / Node / Chromium / Playwright 打包源，因此发布前总门禁会失败，不允许写成可发布。

**已执行验证：**

```text
powershell -ExecutionPolicy Bypass -File scripts\verify-release-checklist.ps1 -SkipTauriBuild：按预期失败，第一步缺少 resources/bin/ffmpeg.exe，发布前总门禁正确阻断。
pnpm --dir src typecheck：通过。
cargo check：通过。
```

---

## 阶段完成标准

- Windows 安装包可构建、安装、启动。
- 首次启动可初始化数据库和内置资源。
- FFmpeg / FFprobe / Chromium 自检可用。
- 发布版 smoke test 通过。
- 授权合规清单已记录。
- 发布前 checklist 固化。






