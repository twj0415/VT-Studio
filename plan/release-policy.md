# 发布签名与更新预留策略

> 目标：第一版 Windows 发布不启用自动更新，但必须把签名、版本号、更新源和密钥边界固定下来，避免后续补更新时引入安全漏洞。

## 第一版口径

- 发布平台：Windows 10/11 x64。
- 安装包类型：NSIS。
- 自动更新：第一版禁用，只保留手动更新通道。
- 安装包签名：发布版要求代码签名；当前开发构建可显示“要求签名，当前未验证”，不能写成已签名。
- 更新源：后续只能使用 HTTPS。
- 更新包：后续必须校验签名，不允许从未签名来源自动更新。
- 更新配置：不得包含 API Key、token、password、private key 或任何密钥材料。

## 版本号规则

以下版本号必须保持一致：

```text
package.json
src/package.json
src-tauri/Cargo.toml
src-tauri/tauri.conf.json
```

应用内“系统设置 / 版本与更新”和诊断包 `summary.json` 必须显示同一版本号。

## 预检脚本

发布前必须执行：

```powershell
pnpm run verify:release-policy
```

检查内容：

- 四处版本号一致。
- 第一版未启用 Tauri updater。
- 更新 / 发布配置不包含 `http://` 更新源。
- 更新 / 发布配置不包含密钥或疑似密钥字段。

## 后续启用自动更新前置条件

启用自动更新前必须补齐：

- HTTPS 更新源。
- 更新 manifest 签名校验。
- 更新包签名校验。
- 更新前数据库备份。
- 更新后 migration 执行和失败回滚。
- sidecar 版本随应用版本管理。
