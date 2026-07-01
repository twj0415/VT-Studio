# 项目文档入口

`docs/` 是当前唯一权威文档目录。旧 `doc/` 和 `plan/` 已删除，后续不要再引用它们。

## 阅读顺序

```text
README.md
01-产品与功能地图.md
02-主流程与数据模型.md
03-Provider任务安全.md
04-UI与参考项目.md
05-模块规格.md
CURRENT.md
BACKLOG.md
ACCEPTANCE.md
```

## 第一版主线

```text
workflowType = image_to_video
inputType = topic | paste | article

我的作品
→ 内容创作
→ 分镜
→ 生图
→ 视频
→ 合成
→ 导出
```

任务中心是一级核心入口，贯穿所有生成、失败、重试、取消、诊断和资源消耗统计。

## 一级菜单

```text
我的作品
任务中心
AI 工具
创作资源
模型 / 工作流
系统设置
```

## 执行红线

1. 不能把 UI 当成功能完成。
2. 不能把 mock、dummy、controlled fake 当真实 Provider。
3. 不能绕过 Tauri command、Service、Repository。
4. API Key 只能进 keyring，不能进 SQLite、日志、导出包、诊断包。
5. 候选图和候选视频必须是集合，不能用单字段覆盖历史。
6. `script-editor` 不作为主流程必经页。
7. AI 工具不能创建作品，创建作品统一从“我的作品 → 内容创作”进入。
8. 不新增“模板创建”替代视频包。
9. 生成动作必须有 Task / TaskStep / Attempt / Artifact。
10. 未真实 smoke 不得宣称上线可用。

## 当前最重要的断点

```text
真实生图 adapter
真实图生视频 adapter 或 workflow adapter
真实 FFmpeg sidecar smoke
设置页全量可用性审计
资源消耗统计真实来源
```

## 当前已打通的底座

```text
任务中心：已有 /tasks 页面、一级 Rail 入口、工作台快捷菜单入口。
Provider/API 快速配置：已有普通用户入口，可保存 Provider、Keyring 密钥、ProviderModel，并测试连接。
真实 LLM adapter：OpenAI-compatible /chat/completions 已接入。
LLM 分镜生成：分镜页“重新生成”走 Vue → Tauri command → scene_service → ProviderManager → Task 记录 → Storyboard 写回。
LLM 分镜输出：sourceText / narrationText / visualGoal / visualDescription / characters / sceneDescription / imagePrompt / negativePrompt / videoPrompt / durationSeconds 等字段跟镜头绑定。
```

## 当前未打通的真实外部能力

```text
真实生图 adapter
真实图生视频 adapter 或 workflow adapter
真实 TTS adapter
真实 VLM adapter
真实 workflow adapter
真实 FFmpeg sidecar smoke
```

## 当前代码现实

```text
任务底座：已有 Tauri command、前端 task store、作品内任务摘要。
任务中心：已有一级路由、页面、Rail 入口、工作台快捷菜单入口；仍需补更细的日志、行级定位和资源消耗。
Provider：OpenAI-compatible LLM / image / tts / vlm 已有最小真实 adapter；video / workflow 已有通用 HTTP 异步轮询 adapter 和 ComfyUI 最小 adapter。未做真实 key smoke 的能力不能标 real_provider，也不能伪装成功。
script-editor：页面文件仍存在，但未注册到主流程路由；应删除或降级为后续高级编辑入口。
AI 工具：当前受 showAiToolsEntry 产品模式开关控制，可能隐藏。
```
