# 授权合规清单

> 目标：发布前明确所有会进入 Windows 安装包的第三方代码、二进制、字体、模板和素材授权状态。当前清单是发布门禁，不是法律意见。

## 发布前硬门禁

- 安装包必须包含 `resources/licenses/THIRD_PARTY_NOTICES.md`。
- 任何进入 `resources/bin` 的二进制必须记录来源、版本、下载地址或构建方式、许可证和随包 notice 文件。
- FFmpeg / FFprobe 必须明确 LGPL/GPL 构建边界和启用编码器组合；不能只写“FFmpeg 免费”。
- Chromium / Playwright 必须包含上游 license、notice、third-party notices。
- 字体、图片、音频、模板素材必须记录来源；来源不明不得随包。
- 参考项目只允许借鉴流程和机制，不直接复制代码、模板、图片、Prompt 或二进制。

## 当前清单

| 类别 | 项 | 当前状态 | 发布前动作 |
|---|---|---|---|
| 桌面框架 | Tauri / tauri-build / tauri CLI | 使用中，依赖 license 文件在包目录存在 | 发布前汇总到 notice |
| 前端运行时 | Vue / Vue Router / Pinia / Vue I18n / Naive UI | 使用中，依赖 license 文件在 `src/node_modules` 存在 | 发布前汇总到 notice |
| 前端构建 | Vite / TypeScript / Tailwind CSS / Sass / vue-tsc | 构建期依赖 | 若随包分发其产物外的代码，汇总 notice |
| Rust 运行依赖 | serde / serde_json / rusqlite / keyring | 使用中 | 发布前生成 Rust dependency license 摘要 |
| SQLite | rusqlite bundled SQLite | 使用中 | 记录 SQLite public domain / rusqlite license |
| FFmpeg / FFprobe | `resources/bin/ffmpeg.exe`、`ffprobe.exe` | 当前缺失 | 补来源、版本、许可证、编码器配置和 notice 后才允许发布 |
| Node.js | `resources/bin/node.exe` | 当前缺失 | 补 Node.js license 和 notices |
| Chromium | `resources/bin/chromium/` | 当前缺失 | 补 Chromium license 和 third-party notices |
| Playwright | `resources/bin/node_modules/playwright-core/`、driver | package 依赖存在，打包源当前缺失 | 随包复制 license / NOTICE / ThirdPartyNotices |
| 内置模板 | `templates/builtin/**/*.html` | 项目内自有模板 | 新增模板必须保留来源记录；当前 3 个模板视为项目自有 |
| 字体 | 可导入素材类型，但当前无随包字体 | 当前未发现随包字体 | 后续新增字体必须记录授权和可商用范围 |
| 图标 | 当前以文本/组件样式为主，未发现独立图标包随包资产 | 当前无需额外素材授权 | 后续新增图标包或 SVG 集合必须记录 license |
| 参考项目 | `waoowaoo-main`、`Toonflow-web-master` | 只做产品/流程参考 | 禁止直接复制代码和素材，除非单独审查许可证 |

## FFmpeg 特别边界

发布前必须记录：

- 二进制来源：官方下载、第三方构建或自构建。
- 精确版本和构建参数。
- 是否启用 GPL 组件。
- 是否启用专利编码器。
- 是否需要随包提供 LGPL/GPL 文本、源代码获取方式或对象文件替换方式。

如果无法确认上述信息，发布版必须阻断。

## 模板和素材新增规则

新增以下文件前必须记录授权：

- 字体：`.ttf`、`.otf`、`.woff`、`.woff2`。
- 图片 / 视频 / 音频样例。
- 第三方 HTML / CSS 模板。
- 第三方 SVG / icon set。

记录格式：

```text
文件：
来源：
作者 / 权利方：
许可证：
是否允许商用：
是否允许修改：
是否要求署名：
随包 notice：
```
