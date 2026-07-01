# BACKLOG

## P0 上线底座

1. 当前代码真实状态审计
   - 核对所有主流程页面是否调用真实 Tauri command。
   - 核对所有 Provider 相关能力属于占位、controlled fake 还是真实 adapter。
   - 核对所有设置项是否可点、可保存、可测试。
   - 已补：`/tasks` 任务中心路由、Rail 入口、工作台快捷菜单入口。
   - 核对 `script-editor`：当前页面文件存在但未注册主流程路由，决定删除或降级。
   - 核对 AI 工具：当前代码受 `showAiToolsEntry` 控制，决定保留开关还是固定显示。
   - 核对任务状态枚举：当前代码是 `pending / running / succeeded / failed / cancelled`，不要按文档自由新增状态。

2. Provider/API 配置向导
   - 普通模式：服务商、Base URL、API Key、模型、测试连接。
   - 高级模式：ProviderModel、WorkflowPreset、能力矩阵、schema。
   - 已补：测试区分 dry_run 和 real_generate；dry_run 是非生成连接测试，real_generate 才允许触发实际生成。
   - 已补：快速配置会同时保存 Provider、Keyring 密钥和 ProviderModel；LLM、图片、视频、TTS、VLM 都会落模型能力。

3. 真实 LLM adapter
   - 已补：OpenAI-compatible `/chat/completions` 最小真实 adapter。
   - 已补：分镜页“重新生成”可调用 LLM，输出结构化校验后写入 Storyboard。
   - 待补：内容创作页主题生成、Prompt 生成、AI 工具统一接入同一套 LLM。
   - 错误必须脱敏并进入任务记录。

4. 任务中心
   - 已补：全局任务列表和任务详情页入口。
   - 已补：步骤错误码、trace、可重试状态、恢复建议、尝试历史、Artifact 展示。
   - 已补：基础资源摘要（LLM 调用、图片/视频/音频产物、token 汇总）。
   - 待补：失败行号定位。
   - 重试、取消、继续。
   - 待补：供应商价格表、真实计费回执和费用金额展示。

## P1 主线闭环

5. 内容创作闭环。
6. 分镜闭环。
   - 已补：LLM 结构化分镜生成和写回。
   - 待补：AI 助手细粒度优化单行字段。
7. 最终 prompt 预览。
8. 真实生图闭环。
   - 已补：OpenAI-compatible `/images/generations` 最小真实 adapter、候选图写 workspace、产物记录。
   - 待补：更多供应商差异、模型参数映射、用户可选模型和更细任务反馈。
9. 真实视频闭环。
   - 已补：通用 HTTP / 异步轮询视频 adapter，支持 submit、poll、success/failed 状态、URL/base64 输出、下载 bytes、写入 workspace。
   - 已补：Workflow 生成可走通用 HTTP adapter 或 ComfyUI 最小 adapter，产物同样写入 workspace。
   - 待补：真实供应商配置样例、参数映射细化、真实 key smoke；未 smoke 前不能标记真实可用。
10. TTS / VLM 最小真实 adapter。
   - 已补：OpenAI-compatible TTS `/audio/speech`，Provider 返回 bytes 后写入受控 workspace。
   - 已补：OpenAI-compatible VLM `/chat/completions` + image data URL，输入图片只从受控 `assets/` 读取。
   - 待补：真实 key smoke、供应商差异、模型参数映射、任务中心 Artifact 细节。
11. FFmpeg 合成和导出。

## P2 资源和增强

12. 创作资源。
13. AI 工具。
14. 字幕、封面、BGM、模板、小说长内容、数字人口播、素材成片、画布精修、本地记忆/RAG。
