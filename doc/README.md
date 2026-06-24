# vt-ai-short-video-maker 文档导航

> 这套文档是给 AI 照着开发的施工说明书。
> `doc/` 是需求和规范库，不等于当前开发顺序。
> 真正开发顺序以项目 `plan/阶段路线图.md` 的 TODO 为准。

---

## 一、先读这几篇

| 文档 | 作用 |
|---|---|
| `开始之前/这个产品是什么.md` | 理解产品定位、当前主线、成功标准 |
| `开始之前/功能地图.md` | 看所有功能模块、优先级、模块边界、阅读顺序 |
| `功能模块/README.md` | 按当前优先级整理全部功能模块，开发前先看 |
| `../plan/阶段路线图.md` | 当前真实开发 TODO 和执行顺序 |

当前主线：

```text
输入文字 → 分镜 → 生图 → 图生视频 → 合成
```

当前 workflow：

```text
workflowType = image_to_video
```

TTS、字幕、封面、BGM 不是不做，而是后续增强 TODO；数字人口播、素材成片、纯图文成片是独立 workflow。

---

## 二、底层设计

这些是全项目共用规则，功能文档不重复定义。

| 文档 | 负责什么 |
|---|---|
| `底层设计/00-工程规范总纲.md` | 全项目最高优先级工程纪律、单一事实来源、开发阅读顺序 |
| `底层设计/01-枚举字典与配置规范.md` | 业务枚举、系统字典、配置分层、错误码、任务快照、Provider 能力矩阵 |
| `底层设计/02-前端工程规范.md` | Vue3 前端目录、Tauri invoke 封装、Pinia Store 边界、字典 options 使用 |
| `底层设计/03-后端工程规范.md` | Rust 后端分层、Command、Service、Repository、Pipeline、Provider、Storage 封装 |
| `底层设计/04-UI主题与国际化规范.md` | UI 风格、主题 token、Naive UI themeOverrides、多语言、状态色 |
| `底层设计/05-数据库Schema与迁移规范.md` | SQLite 表结构、索引、迁移、事务边界、Repository 规则 |
| `底层设计/06-任务状态机.md` | Task / TaskStep 状态流转、人工确认节点、断点续跑、任务快照 |
| `底层设计/08-文件存储与路径规范.md` | FileBucket、工作区目录、相对路径、导入文件、PathGuard 使用规则 |
| `底层设计/09-错误日志与事件规范.md` | AppError、错误码、日志脱敏、ProgressEvent |
| `底层设计/10-Tauri命令与DTO协议.md` | Tauri command、前后端 DTO、分页、Patch、API Client 规则 |
| `底层设计/11-PipelineEngine与持久化队列规范.md` | PipelineEngine、TaskStep 映射、租约、幂等、重试、恢复 |
| `底层设计/12-Provider接口契约与能力矩阵Schema.md` | LLM/TTS/Image/Video/VLM/Workflow 请求响应 DTO 与能力矩阵 schema |
| `底层设计/13-FFmpegSidecar与媒体处理规范.md` | FFmpeg/FFprobe sidecar、自检、命令封装、音画对齐、BGM、拼接 |
| `底层设计/14-模板渲染与Chromium安全规范.md` | HTML 模板、Chromium 渲染、安全限制、模板参数 DSL、截图输出 |
| `底层设计/15-错误码总表与恢复策略.md` | 完整错误码、可重试判断、用户恢复动作、任务失败策略 |
| `底层设计/16-日志追踪与诊断包规范.md` | tracing 字段、日志滚动、Provider/FFmpeg 脱敏、诊断包导出 |
| `底层设计/17-配置Schema与默认值规范.md` | App/System/Provider/Pipeline/UI/Export 配置 schema、默认值、快照 |
| `底层设计/18-PromptSkill与模板版本规范.md` | Prompt/Skill frontmatter、结构化输出、版本、模型绑定、快照 |
| `底层设计/19-测试策略与验收用例.md` | 后端、前端、Provider、FFmpeg、模板、端到端验收测试 |
| `底层设计/20-桌面打包发布与更新规范.md` | Tauri 打包、sidecar、数据目录、首次启动、更新、备份恢复 |
| `底层设计/21-资产一致性与设定集规范.md` | Style/Character/Environment/Prop Bible、资产版本、生成快照 |
| `底层设计/22-UI详细设计与页面布局.md` | 页面布局、信息架构、核心页面交互和 UI 施工细节 |
| `底层设计/23-模型适配与工作流注册规范.md` | 多模型适配、ComfyUI/RunningHub workflow preset、能力注册和执行规则 |
| `底层设计/数据结构.md` | Project / StoryboardItem / ImageCandidate / VideoSegment / Task 的唯一字段定义 |
| `底层设计/技术栈与架构.md` | Vue3 + Tauri 2 + Rust + SQLite + FFmpeg + Chromium 的分层架构总览 |
| `底层设计/Provider与安全.md` | Provider Manager、密钥存储、PathGuard、重试、限流、安全红线 |

硬规则：

```text
字段只在 数据结构.md 定义。
枚举、字典、配置只在 01-枚举字典与配置规范.md 定义。
任务状态流转只在 06-任务状态机.md 定义。
模型调用和文件读写必须遵守 Provider与安全.md、08-文件存储与路径规范.md。
API 模型能力看 provider_models，ComfyUI/RunningHub 工作流看 workflow_presets。
前端不承载生产线逻辑，生产线在 Tauri Rust 后端。
功能模块不得重复维护底层枚举、字典和配置取值。
```

---

## 三、参考分析

| 文档 | 作用 |
|---|---|
| `参考分析/功能对比总表.md` | Toonflow / Pixelle / 我们 的功能级对照和推荐结论 |
| `参考分析/两个项目怎么做的.md` | 源码级链路拆解，包含关键文件和函数 |

总原则：

```text
链路工程借鉴 Pixelle
AI 内容组织借鉴 Toonflow
字幕、封面、任务可靠性自研做亮点
安全问题自己补强
```

---

## 四、功能模块

全部功能模块已在 `功能模块/README.md` 按优先级整理。

| 优先级 | 模块 |
|---|---|
| P0 前置基础 | `18-设置与模型配置.md`、`20-资产库与媒体管理.md` |
| P1 当前主线 | `01-内容导入.md`、`03-分镜.md`、`07-生图提示词.md`、`08-AI生图.md`、`14-AI视频.md`、`13-视频合成.md`、`21-导出与项目备份.md` |
| P2 一致性与可靠性 | `17-任务与历史.md`、`04-画风设定.md`、`05-角色设定.md`、`06-场景设定.md` |
| P3 后续增强 | `02-脚本生成.md`、`09-配音TTS.md`、`10-字幕.md`、`11-封面.md`、`12-模板与动效.md` |
| P4 独立 workflow | `15-数字人口播.md`、`19-素材导入与素材成片.md`、`16-画布精修.md` |

---

## 五、推荐阅读顺序

### 产品/架构理解

```text
开始之前/这个产品是什么.md
→ 开始之前/功能地图.md
→ 功能模块/README.md
→ ../plan/阶段路线图.md
→ 底层设计/00-工程规范总纲.md
→ 底层设计/技术栈与架构.md
→ 底层设计/数据结构.md
→ 参考分析/功能对比总表.md
```

### 开发当前主线

```text
../plan/阶段路线图.md
→ 功能模块/README.md
→ 功能模块/01-内容导入.md
→ 功能模块/03-分镜.md
→ 功能模块/07-生图提示词.md
→ 功能模块/08-AI生图.md
→ 功能模块/14-AI视频.md
→ 功能模块/13-视频合成.md
→ 功能模块/21-导出与项目备份.md
→ 功能模块/17-任务与历史.md
→ 功能模块/18-设置与模型配置.md
→ 功能模块/20-资产库与媒体管理.md
```

### 开发某个功能

```text
../plan/阶段路线图.md
→ 功能模块/README.md
→ 功能模块/对应文档
→ 底层设计/00-工程规范总纲.md
→ 底层设计/01-枚举字典与配置规范.md
→ 底层设计/数据结构.md
→ 底层设计/05-数据库Schema与迁移规范.md（涉及入库/查询时）
→ 底层设计/10-Tauri命令与DTO协议.md（涉及前后端通信时）
→ 底层设计/11-PipelineEngine与持久化队列规范.md（涉及任务执行时）
→ 底层设计/12-Provider接口契约与能力矩阵Schema.md（涉及模型调用时）
→ 底层设计/23-模型适配与工作流注册规范.md（涉及多模型、ComfyUI、RunningHub、workflow preset 时）
→ 底层设计/Provider与安全.md（涉及模型/文件/密钥安全时）
→ 底层设计/02-前端工程规范.md（写前端时）
→ 底层设计/03-后端工程规范.md（写后端时）
→ 底层设计/04-UI主题与国际化规范.md（涉及 UI/主题/语言时）
→ 参考分析/两个项目怎么做的.md
→ 参考项目源码
```

---

## 六、目录维护规则

当前文档只按四类目录组织：

```text
开始之前/
底层设计/
参考分析/
功能模块/
```

后续新增文档优先放入这四类目录，保持目录直白、按功能找文档。

---

## 七、写文档的规则

1. 不用黑话，目录和文件名一看就懂。
2. 一个模块一个文档，不把多个功能揉在一起。
3. 每个“怎么做”都要有出处：先看推荐结论，再读参考项目源码。
4. 不凭空编文件名、函数名、字段名。
5. 字段命名必须语义明确，路径用 `_path`，URL 用 `_url`，ID 用 `_id`，时长用 `_duration_seconds` 且单位秒，布尔用 `is_ / has_ / should_ / can_ / requires_`。
6. 前端文档按 Vue3 + TypeScript + Pinia + Naive UI 写，不硬编码文案，不过度设计。
7. 枚举 code 使用 snake_case 稳定值，不存中文、不存数字下标。
8. 明确指出参考项目的坑，不照搬坏设计。
9. 功能文档要写清怎么做，不用排期话术替代设计细节。

