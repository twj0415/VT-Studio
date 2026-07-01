# CURRENT

## 当前唯一焦点

底座真实链路补齐：优先保证用户能简单配置 API，并让分镜、生图等主流程走真实 Provider；未实现的能力必须明确失败，不能伪装成功。

## 当前代码事实

1. 前端主线页面、Tauri command、SQLite、Task、Provider、Scene、Export 等底座已有。
2. OpenAI-compatible LLM adapter 已打通，可用于连接测试和 LLM 分镜生成。
3. OpenAI-compatible 文生图 adapter 已有最小真实链路，支持 `/images/generations` 的 `b64_json` 和 HTTPS `url` 下载，生成文件会写入受控 workspace。
4. FFmpeg 合成底座已有，但真实 sidecar smoke 仍需实际环境验证。
5. 任务底座已有，任务中心一级页面、路由、Rail 入口、工作台快捷菜单入口已补。
6. `script-editor` 页面文件仍存在，但未注册主流程路由。
7. 视频、Workflow 已补通用 HTTP / 异步轮询 / ComfyUI 最小执行 adapter：支持提交任务、轮询状态、解析 URL/base64 输出、下载真实 bytes，并由 Service 写入受控 workspace；仍需真实供应商配置和 smoke 后才能标记 `real_provider`。
8. OpenAI-compatible TTS `/audio/speech` 最小真实 adapter 已补，Provider 返回音频 bytes 后由 Service 写入受控 workspace；仍需真实 key smoke 后才能标记 `real_provider`。
9. OpenAI-compatible VLM `/chat/completions` + image data URL 最小真实 adapter 已补，Style service 会从受控 `assets/` 安全读取图片 bytes；仍需真实视觉模型 smoke 后才能标记 `real_provider`。
10. 分镜页“重新生成”已走真实 Tauri LLM 命令，输出会写入 Storyboard 并记录 Task/TaskStep/Attempt/Artifact。
11. 快速配置页已支持 LLM / 生图 / 视频 / 配音 / 图片理解模型保存 Provider、Keyring 密钥和 ProviderModel；假 key 只会测试失败。
12. 分镜页新增镜头已改为 Tauri command 持久化创建，不再使用前端 `Promise.resolve` 临时假数据。
13. Provider 普通“测试连接”已改为非生成测试：OpenAI-compatible 走 `/models` 检查连通和鉴权，不触发生图/配音/视频生成；“真实生成测试”仍需用户显式确认。
14. 任务详情 DTO 和任务中心页面已展示步骤错误码、trace、可重试状态、恢复建议、尝试历史、产物列表和基础资源摘要（LLM 调用、图片/视频/音频产物、token 汇总）；仍不做金额估算。
15. Provider 配置已拆出 `protocolKind`：`vendor` 表示服务商，`providerKind` 表示能力大类，`protocolKind` 表示实际调用协议。当前已实现 `openai_compatible`、`generic_http / generic_async / generic_async_video / http_async`、`comfyui`；其他协议名未映射时会明确失败，不能伪装生成成功。

## 本轮完成标准

1. 快速配置入口可保存 Provider、Keyring 密钥、ProviderModel。
2. 假 key `sk-123456` 只能作为示例，测试连接失败也要给通俗提示，不能伪造成功。
3. 真实 key 配好后，分镜页可调用 LLM 生成结构化镜头表。
4. LLM 生成失败进入任务中心，并给出错误码和恢复建议。
5. 不把未 smoke 的 video / workflow / TTS / VLM 写成真实完成；只有真实 key 和真实产物验证通过后才能升级为 `real_provider`。

## 下一步

继续执行 `BACKLOG.md`：优先补内容创作闭环、分镜单行 AI 助手、任务中心重试/取消/继续，以及真实供应商配置样例和 smoke 记录。
