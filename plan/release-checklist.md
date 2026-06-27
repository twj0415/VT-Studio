# 发布前检查清单

> 目标：发布动作只能按固定清单执行。开发模式能跑、浏览器页面能打开、单测通过，都不能替代 Windows 安装包发布验收。

## 一键门禁

发布前执行：

```powershell
pnpm run verify:release-checklist
```

该脚本会调用 `scripts/verify-release-smoke.ps1`，串联资源、策略、授权、Rust、前端和安装包构建前置检查。任一步失败都不得发布。

## 必跑项

| 顺序 | 检查项 | 命令 / 记录 | 失败处理 |
|---|---|---|---|
| 1 | 发布资源清单 | `pnpm run verify:release-resources` | 缺 sidecar 时停止 |
| 2 | 发布策略 | `pnpm run verify:release-policy` | 版本不一致、HTTP 更新源、疑似密钥时停止 |
| 3 | 授权合规 | `pnpm run verify:license-compliance` | 缺 license / notice / sidecar 来源时停止 |
| 4 | Rust 格式 | `cargo fmt --check` | 修格式后重跑 |
| 5 | Rust check | `cargo check` | 修编译错误后重跑 |
| 6 | 前端 typecheck | `pnpm --dir src typecheck` | 修类型错误后重跑 |
| 7 | 前端 build | `pnpm --dir src build` | 修构建错误后重跑 |
| 8 | Windows NSIS build | `pnpm run tauri:build:windows` | 构建失败不得进入 smoke |
| 9 | 发布版 smoke | 按 `plan/release-smoke-test.md` 记录 | 未安装真实 NSIS 包不得写通过 |
| 10 | 诊断包脱敏 | 导出诊断包并检查无密钥 / 绝对路径 | 发现泄漏立即阻断 |
| 11 | 项目包导出 | 导出项目包并检查受控路径 | 路径或密钥失败立即阻断 |
| 12 | 安装卸载 | 安装、重启恢复、卸载保留数据策略 | 未记录不得发布 |

## 禁止替代项

- 不得用 `tauri:dev` 代替安装包启动。
- 不得用浏览器 dev server 代替桌面发布版。
- 不得用 controlled fake 通过写成真实 Provider 通过。
- 不得在 `verify:release-resources` 或 `verify:license-compliance` 失败时继续发布。
- 不得把缺 FFmpeg / Chromium 的状态写成真实合成或模板渲染通过。

## 当前已知阻断

`resources/bin` 当前缺少 FFmpeg、FFprobe、Node、Chromium、Playwright 打包源，发布前检查会在资源或授权门禁失败。这是正确阻断，不是脚本问题。
