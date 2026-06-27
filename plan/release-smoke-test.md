# 发布版 Smoke Test 清单

> 目标：验证 Windows 10/11 x64 安装包离开源码目录后仍能启动、初始化、自检、跑通受控主线，并清楚记录未执行的真实 Provider / FFmpeg 项。

## 前置门禁

发布版 smoke 只认安装包，不认浏览器 dev server。

必须先通过：

```powershell
pnpm run verify:release-resources
pnpm --dir src typecheck
pnpm --dir src build
cargo fmt --check
cargo check
pnpm run tauri:build:windows
```

如果 `verify:release-resources` 失败，停止安装包 smoke，并在记录里写清缺少的 `resources/bin/*` 文件。

## 安装环境记录

```text
日期：
执行人：
Windows 版本：
系统架构：x64
安装包路径：
安装路径：
AppData 路径：
workspace 路径：
是否中文路径：
是否空格路径：
是否离线环境：
```

## 主流程

| # | 步骤 | 通过标准 | 记录 |
|---|---|---|---|
| 1 | 安装应用 | NSIS 安装完成，无杀软拦截；可从开始菜单或安装目录启动 | 安装路径 |
| 2 | 首次启动 | 自动创建 AppData、SQLite、workspace buckets、默认配置、内置模板、内置创作规则 | AppData / workspace |
| 3 | 打开系统设置 | “运行环境自检”显示 workspace、SQLite、内置模板扫描状态 | 自检截图 / 状态 |
| 4 | 检查 sidecar | FFmpeg、FFprobe、Node、Chromium、Playwright 状态明确；缺失项不导致应用崩溃 | ready / missing |
| 5 | 从“我的作品”开始创作 | 可创建项目并进入工作台 | project_id |
| 6 | Mock 主线跑通 | 内容创作 → 分镜 → 生图 → 视频 → 合成页面流程可走；Mock / controlled fake 标识清楚 | 阶段结果 |
| 7 | 真实 FFmpeg 小样例合成 | 有 `sidecars/ffmpeg.exe` / `ffprobe.exe` 时生成可播放 `final.mp4`；无 sidecar 只记录未执行 | outputPath |
| 8 | 导出 final.mp4 | 写 ExportRecord，输出目录受控，可打开 | export_id |
| 9 | 重启恢复项目 | 重启后项目、任务、导出记录仍可读 | 恢复结果 |
| 10 | 导出诊断包 | 诊断包不包含密钥和本机绝对路径 | diagnostic_id |
| 11 | 卸载策略 | 卸载应用后按策略确认是否保留 AppData 数据 | 保留 / 删除 |

## 失败恢复

| 场景 | 期望 |
|---|---|
| 缺 FFmpeg / FFprobe | 设置页显示缺失；合成任务返回可恢复 sidecar 错误，不崩溃 |
| 缺 Chromium / Playwright | 设置页显示缺失；模板截图检查 skipped 或 failed，不影响打开应用 |
| 中文路径 / 空格路径 | workspace 读写、SQLite、自检、项目创建正常 |
| 离线环境 | 应用可启动，不运行时下载未知二进制 |
| 缺 final.mp4 导出 | 导出失败并写清恢复动作，不写成功记录 |

## 记录模板

```text
日期：
执行人：
分支 / 提交：
应用版本：
Windows 版本：
安装包路径：
安装路径：
workspace 路径：
路径场景：普通 / 中文 / 空格 / 长路径
网络状态：在线 / 离线
Provider：controlled fake / real provider（名称）
真实费用确认：是 / 否 / 不适用
FFmpeg sidecar：存在 / 缺失
Chromium sidecar：存在 / 缺失

前置门禁：
- verify:release-resources：
- typecheck：
- build：
- cargo fmt：
- cargo check：
- tauri:build:windows：

安装与启动：
- 安装：
- 首次启动：
- 初始化：
- 设置页自检：

主线结果：
- 创建作品：
- 内容导入：
- 分镜：
- 生图：
- 视频：
- 合成：
- 导出：
- 重启恢复：
- 诊断包：
- 卸载策略：

失败恢复：
- 缺 FFmpeg：
- 缺 Chromium：
- final.mp4 缺失：

结论：通过 / 未通过 / 部分通过
未执行项及原因：
后续修复项：
```

## 禁止写成通过

- 未生成并安装 NSIS 包，不能写成发布版 smoke 通过。
- `verify:release-resources` 未通过，不能继续写安装包通过。
- controlled fake 通过，不能写成真实 Provider 通过。
- 缺 `ffmpeg.exe / ffprobe.exe`，不能写成真实 `final.mp4` 通过。
- 只跑单测、typecheck 或浏览器页面，不能写成发布版 smoke 通过。
