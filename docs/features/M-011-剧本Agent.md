# M-011 剧本 Agent

状态：已补源码事实和缺少逻辑

## 1. 参考源码

Toonflow-web：

```txt
D:\project\短视频\Toonflow-web-master\src\router\index.ts
D:\project\短视频\Toonflow-web-master\src\views\scriptAgent\index.vue
D:\project\短视频\Toonflow-web-master\src\stores\scriptAgent.ts
D:\project\短视频\Toonflow-web-master\src\utils\useChat.ts
```

Toonflow-app：

```txt
D:\project\短视频\Toonflow-app-master\src\socket\index.ts
D:\project\短视频\Toonflow-app-master\src\socket\routes\scriptAgent.ts
D:\project\短视频\Toonflow-app-master\src\socket\resTool.ts
D:\project\短视频\Toonflow-app-master\src\agents\scriptAgent\index.ts
D:\project\短视频\Toonflow-app-master\src\agents\scriptAgent\tools.ts
D:\project\短视频\Toonflow-app-master\src\routes\scriptAgent\getPlanData.ts
D:\project\短视频\Toonflow-app-master\src\routes\scriptAgent\setPlanData.ts
D:\project\短视频\Toonflow-app-master\src\routes\scriptAgent\updateData.ts
D:\project\短视频\Toonflow-app-master\src\routes\novel\getNovelData.ts
D:\project\短视频\Toonflow-app-master\src\routes\agents\getMemory.ts
D:\project\短视频\Toonflow-app-master\src\routes\agents\clearMemory.ts
D:\project\短视频\Toonflow-app-master\src\routes\script\delScript.ts
D:\project\短视频\Toonflow-app-master\src\routes\script\updateScript.ts
D:\project\短视频\Toonflow-app-master\src\lib\initDB.ts
```

Agent skill 文件：

```txt
D:\project\短视频\Toonflow-app-master\data\skills\script_agent_decision.md
D:\project\短视频\Toonflow-app-master\data\skills\script_agent_supervision.md
D:\project\短视频\Toonflow-app-master\data\skills\script_execution_skeleton.md
D:\project\短视频\Toonflow-app-master\data\skills\script_execution_adaptation.md
D:\project\短视频\Toonflow-app-master\data\skills\script_execution_script.md
```

## 2. 菜单入口和页面结构

路由入口：

```txt
/scriptAgent -> views/scriptAgent/index.vue
```

页面作用：

```txt
基于当前项目、小说原文、章节事件、历史记忆和工作区计划数据，让剧本 Agent 生成故事骨架、改编策略和剧本分集，并允许用户手动编辑。
```

页面结构：

```txt
Splitpanes 左右分栏
左侧 30%：Agent 对话区
右侧 70%：工作区数据 tabs
```

左侧区域：

```txt
聊天消息列表
输入框
发送/停止
设置弹出菜单
连接状态红/绿点
思考等级按钮
原文事件未完成提示蒙层
```

右侧 tabs：

```txt
故事骨架
改编策略
剧本
```

源码里注释掉的 tab：

```txt
章节事件 tab 被注释，没有作为当前页面正式 tab 展示。
```

## 3. 功能拆分

| 功能 ID | 功能 | 要做到什么 |
|---|---|---|
| F-011-001 | 剧本 Agent 对话入口 | 初始化欢迎消息，连接 Socket，发送消息，接收流式消息，停止生成，显示连接状态和错误 |
| F-011-002 | 读取历史记忆 | 进入页面后读取 message 类型记忆，按聊天消息格式展示 |
| F-011-003 | 清理 Agent 记忆 | 支持清 message、summary、all，弹确认框，清理后刷新历史 |
| F-011-004 | 思考等级配置 | 如果模型支持 think，显示关闭/轻度/深度/极限，发 Socket 配置 |
| F-011-005 | 原文事件完成检查 | 读取小说原文列表，发现 eventState=0 时显示提示蒙层 |
| F-011-006 | 获取计划数据 | 读取 o_agentWorkData 的故事骨架和改编策略，同时读取 o_script 作为剧本列表 |
| F-011-007 | 编辑故事骨架 | 打开 Markdown 编辑弹窗，保存到 o_agentWorkData |
| F-011-008 | 编辑改编策略 | 打开 Markdown 编辑弹窗，保存到 o_agentWorkData |
| F-011-009 | 查看剧本卡片 | 展示分集剧本列表，支持单卡折叠和一键折叠/展开 |
| F-011-010 | 编辑剧本卡片 | 打开剧本内容编辑弹窗，保存后同步计划数据和 o_script |
| F-011-011 | 删除剧本卡片 | 二次确认，有 id 时调用删除剧本接口，再同步计划数据 |
| F-011-012 | Agent XML 写入工作区 | 识别 storySkeleton、adaptationStrategy、scriptItem XML，流式更新右侧数据 |
| F-011-013 | Agent 工具读取上下文 | 工具读取章节事件、小说原文、当前计划数据、已有剧本内容 |
| F-011-014 | 子 Agent 调度 | 决策层可调用故事骨架、改编策略、剧本生成、监督层子 Agent |

## 4. 页面操作细节

进入页面时执行：

```txt
1. 如果 messages 为空，插入欢迎消息和“开始”建议。
2. 调 /scriptAgent/getPlanData 读取工作区数据。
3. 调 /novel/getNovelData 读取小说原文，检查事件分析是否完成。
4. 连接 /socket/scriptAgent。
5. 如果只有欢迎消息，调 /agents/getMemory 读取历史消息。
6. 调 /project/getModelDetails key=scriptAgent，判断是否显示思考等级按钮。
```

聊天区按钮和操作：

| 操作 | 源码行为 | 成功反馈 | 失败反馈 |
|---|---|---|---|
| 发送消息 | `scriptAgentStore().chat(text)`，本地先插入 user 消息，再 emit `chat` | 流式 assistant 消息 | Socket 错误或 message error |
| 停止生成 | `scriptAgentStore().stopGenerate()`，本地把当前消息置为 stop，再 emit `stop` | 停止输出 | 无明确 toast |
| 点击建议 | 取 suggestion 的 prompt 直接发送 | 同发送消息 | 同发送消息 |
| 重连 | 弹确认框 | 调 reconnect | 源码实际调用了 productionAgentStore().reconnect() |
| 清 message 记忆 | 弹确认，POST `/agents/clearMemory` type=message | 成功 toast，刷新历史 | 接口错误 |
| 清 summary 记忆 | 弹确认，POST `/agents/clearMemory` type=summary | 成功 toast，刷新历史 | 接口错误 |
| 清 all 记忆 | 弹确认，POST `/agents/clearMemory` type=all | 成功 toast，刷新历史 | 接口错误 |
| 切换思考等级 | emit `updateThinkConfig` | 后端更新 thinkConfig | 无明确失败提示 |

右侧故事骨架 tab：

```txt
有内容：MdPreview 展示 planData.storySkeleton。
无内容：t-empty。
右上角“编辑”按钮打开 editMdPreivew 弹窗。
保存后 POST /scriptAgent/updateData。
```

右侧改编策略 tab：

```txt
有内容：MdPreview 展示 planData.adaptationStrategy。
无内容：t-empty。
右上角“编辑”按钮打开 editMdPreivew 弹窗。
保存后 POST /scriptAgent/updateData。
```

右侧剧本 tab：

```txt
无剧本：t-empty。
有剧本：两列 scriptCard 网格。
每张卡片显示 #序号、名称、折叠按钮、编辑按钮、删除按钮、正文。
正文用 pre 展示，空内容显示 noContent。
右下角悬浮按钮用于全部折叠/展开。
```

剧本卡片操作：

| 操作 | 源码行为 |
|---|---|
| 折叠/展开单卡 | 按 `id:{id}` 或 `index:{index}` 记录折叠状态 |
| 全部折叠/展开 | 遍历当前剧本列表设置 collapsedCards |
| 编辑 | 打开 80% 宽 t-dialog，MdEditor 编辑 content |
| 保存编辑 | 更新 `planData.script[index]`，调用 store.setPlanData，再刷新 getPlanData |
| 删除 | 弹确认；有 id 时调用 `/script/delScript`；无 id 时只从 planData 删除；然后 setPlanData + getPlanData |

原文事件未完成提示：

```txt
getNovelData 返回列表中只要存在 eventState === 0，就显示 forceGenerateMask。
蒙层只有一个确认按钮，点击后隐藏。
源码没有强制继续生成事件的按钮。
```

## 5. 表单字段和校验

| 业务含义 | Toonflow 源码名 | 控件 | 必填 | 字段来源 | 说明 |
|---|---|---|---|---|---|
| 项目 ID | `projectId` | 隐式 | 是 | 当前项目 | Socket auth、记忆、计划数据和剧本读写都依赖 |
| 隔离 key | `isolationKey` | 隐式 | 是 | `${projectId}:scriptAgent` | 记忆隔离 |
| 用户消息 | `content` | t-chat-sender | 是 | 用户输入 | 空文本且无附件时 useChat 不发送 |
| 思考等级 | `thinkLevel` | popup 菜单 | 否 | 页面状态 | 0 关闭、1 轻度、2 深度、3 极限 |
| 故事骨架 | `storySkeleton` | MdPreview/MdEditor | 否 | o_agentWorkData.data | XML 标签 `<storySkeleton>` 会流式写入 |
| 改编策略 | `adaptationStrategy` | MdPreview/MdEditor | 否 | o_agentWorkData.data | XML 标签 `<adaptationStrategy>` 会流式写入 |
| 剧本条目名称 | `scriptItem attrs.name` / `script.name` | 卡片标题 | 是 | Agent XML 或 o_script.name | Agent 生成时按名称合并 |
| 剧本条目内容 | `scriptItem value` / `script.content` | MdEditor | 是 | Agent XML 或 o_script.content | 保存时同步到 o_script |
| 原文章节编号 | `chapterIndex` | Agent 工具参数 | 是 | o_novel.chapterIndex | 工具读取事件和原文 |
| 记忆类型 | `type` | 设置菜单 | 否 | message/summary/all | 默认 all |

## 6. 前端状态和 XML 写入

`stores/scriptAgent.ts` 维护的计划数据：

```ts
{
  storySkeleton: string;
  adaptationStrategy: string;
  script: { id?: number; name: string; content: string }[];
}
```

useChat XML 配置：

```txt
storySkeleton：keepInMessage=false
adaptationStrategy：keepInMessage=false
scriptItem：keepInMessage=false
```

XML 处理规则：

| XML 标签 | 行为 |
|---|---|
| `<storySkeleton>...</storySkeleton>` | 更新 `planData.storySkeleton` |
| `<adaptationStrategy>...</adaptationStrategy>` | 更新 `planData.adaptationStrategy` |
| `<scriptItem name="剧本名称">...</scriptItem>` | 按 name 查找已有剧本，存在则更新 content，不存在则 push |

保存时机：

```txt
每个 XML 标签 status=complete 时调用 setPlanData。
```

消息状态：

```txt
useChat 全局生成状态：idle / pending / streaming
TDesign 消息状态：pending / streaming / complete / error / stop
```

消息内容类型：

```txt
text
markdown
thinking
toolcall
search
reasoning
suggestion
attachment
```

## 7. 调用链路

| 用户动作 | Toonflow 前端调用 | Toonflow 后端文件 | 读写对象 | 成功反馈 | 失败反馈 |
|---|---|---|---|---|---|
| 打开页面 | `/scriptAgent/getPlanData` | `routes/scriptAgent/getPlanData.ts` | `o_agentWorkData/o_script` | 渲染右侧 tabs | 页面无统一兜底 |
| 检查原文事件 | `/novel/getNovelData` | `routes/novel/getNovelData.ts` | `o_novel` | 需要时显示蒙层 | 页面无统一兜底 |
| 读取历史消息 | `/agents/getMemory` | `routes/agents/getMemory.ts` | `memories` | 渲染历史聊天 | 页面无统一兜底 |
| 清理记忆 | `/agents/clearMemory` | `routes/agents/clearMemory.ts` | `memories` | 成功 toast | 接口错误 |
| 查询是否支持思考 | `/project/getModelDetails` | `routes/project/getModelDetails.ts` | `o_agentDeploy` | 显示思考按钮 | 不显示 |
| 建立 Socket | `/socket/scriptAgent` | `socket/routes/scriptAgent.ts` | Socket 连接 | connected 变绿 | token 或 isolationKey 失败断开 |
| 发送消息 | emit `chat` | `socket/routes/scriptAgent.ts` | Agent/Memory | 流式返回 | assistant 消息 error |
| 停止生成 | emit `stop` | `socket/routes/scriptAgent.ts` | AbortController | 本地 stop | 无明确后端确认 |
| 切换思考等级 | emit `updateThinkConfig` | `socket/routes/scriptAgent.ts` | thinkConfig | 后端更新配置 | 无明确反馈 |
| Agent 获取计划数据 | emit `getPlanData` callback | 前端 store 监听 | `planData` | 工具拿到数据 | 无明确失败处理 |
| 保存计划数据 | `/scriptAgent/setPlanData` | `routes/scriptAgent/setPlanData.ts` | `o_agentWorkData/o_script` | 无 toast或调用方 toast | 接口错误 |
| 编辑骨架/策略 | `/scriptAgent/updateData` | `routes/scriptAgent/updateData.ts` | `o_agentWorkData` | 成功 toast | error toast |
| 删除剧本 | `/script/delScript` | `routes/script/delScript.ts` | `o_script/o_agentWorkData/o_storyboard/o_scriptAssets/o_video` | 成功 toast | 接口错误 |

## 8. 后端 Agent 链路

Socket 命名空间：

```txt
socket/index.ts 注册 /api/socket/scriptAgent
前端 store 使用 ${baseUrl}/socket/scriptAgent
baseUrl 通常包含 /api，因此实际连接到 /api/socket/scriptAgent
```

连接鉴权：

```txt
1. 从 socket.handshake.auth.token 取 token。
2. 读取 o_setting key=tokenKey。
3. jwt.verify(token, tokenKey)。
4. 缺 token 或校验失败，断开连接。
5. 缺 isolationKey，断开连接。
```

chat 事件：

```txt
1. 收到 { content }。
2. abort 上一个 AbortController。
3. 创建新的 AbortController。
4. resTool.newMessage("assistant", "统筹")。
5. 组装 AgentContext。
6. 调 runDecisionAI(ctx)。
7. 非 AbortError 时 msg.error(errorMessage)。
8. finally 清理 abortController。
```

runDecisionAI：

```txt
1. Memory("scriptAgent", isolationKey)。
2. memory.add("user", text)。
3. 读取 skills/script_agent_decision.md。
4. memory.get(text)，拼接 RAG、历史摘要、近期对话。
5. 读取 o_project 项目信息。
6. 读取 o_novel 章节数量。
7. 用 u.Ai.Text("scriptAgent:decisionAgent", think, thinlLevel).stream。
8. system = 决策层 skill。
9. assistant = 项目信息 + Memory。
10. user = 用户输入。
11. tools = memory tools + scriptAgent tools + 子 Agent tools。
12. onFinish 写 memory.add("assistant:decision", 去 XML 后文本)。
13. consumeFullStream 把 reasoning/text/tool 等流式写回前端。
```

项目信息包含：

```txt
小说名称：o_project.name
小说类型：o_project.type
小说简介：o_project.intro
目标改编影视视觉手册/画风：o_project.artStyle
目标改编视频画幅：o_project.videoRatio，默认 16:9
章节数量：o_novel.length
```

## 9. Agent 工具

决策层可用工具：

| 工具 | 作用 | 读取对象 | 返回 |
|---|---|---|---|
| `get_novel_events` | 获取指定章节事件 | `o_novel` | 章节编号、标题、事件文本 |
| `get_planData` | 获取当前工作区数据 | 前端 `getPlanData` callback | 指定 key 对应内容 |
| `get_novel_text` | 获取指定章节原文 | `o_novel.chapterData` | 原文文本 |
| `get_script_content` | 获取已有剧本内容 | `o_script` | `<scriptItem name="...">...</scriptItem>` |

`get_novel_events` 参数：

```txt
chapterIndexs: number[]
```

查询字段：

```txt
id
chapterIndex as index
reel
chapter
chapterData
event
eventState
```

`get_planData` 参数：

```txt
key: storySkeleton | adaptationStrategy | script
```

源码里的 planData schema：

```txt
storySkeleton: string
adaptationStrategy: string
script: string
```

前端真实 planData：

```txt
script 是数组：{ id?: number; name: string; content: string }[]
```

`get_novel_text` 参数：

```txt
chapterIndex: string
```

`get_script_content` 参数：

```txt
ids: string[]
```

## 10. 子 Agent

| 子 Agent 工具 | 模型 key | skill 文件 | assistant 名称 | 记忆 key | 输出要求 |
|---|---|---|---|---|---|
| `run_sub_agent_storySkeleton` | `scriptAgent:storySkeletonAgent` | `script_execution_skeleton.md` | 编剧 | `assistant:execution:storySkeleton` | 必须输出 `<storySkeleton>` |
| `run_sub_agent_adaptationStrategy` | `scriptAgent:adaptationStrategyAgent` | `script_execution_adaptation.md` | 编剧 | `assistant:execution:adaptationStrategy` | 必须输出 `<adaptationStrategy>` |
| `run_sub_agent_script` | `scriptAgent:scriptAgent` | `script_execution_script.md` | 编剧 | `assistant:execution:script` | 必须输出一个或多个 `<scriptItem name="...">` |
| `run_supervision_agent` | `scriptAgent:supervisionAgent` | `script_agent_supervision.md` | 编辑 | `assistant:supervision` | 返回监督结果 |

子 Agent 执行共同逻辑：

```txt
1. 完成父消息。
2. 创建子消息。
3. 用指定模型 key 和 skill stream。
4. tools 仍包含 scriptAgent tools。
5. consumeFullStream 写回前端。
6. 完整输出去 XML 后写入 Memory。
7. 创建新的 assistant 消息，名称是“视频策划”。
```

剧本子 Agent 额外上下文：

```txt
读取 o_script 的 id 和 name。
构建 “可用剧本(ID:名称)”。
读取 o_novel 章节数量。
要求输出 scriptItem XML，不允许添加额外标签。
```

## 11. 数据和状态

数据库参考：

`o_agentWorkData`：

```txt
id
projectId
episodesId
key
data
createTime
updateTime
```

剧本 Agent 用法：

```txt
key = scriptAgent
projectId = 当前项目
episodesId 不参与剧本 Agent 查询
data 保存故事骨架、改编策略和源码尝试保存的 script 快照
```

`o_script`：

```txt
id
name
content
projectId
extractState
createTime
errorReason
```

`o_novel`：

```txt
id
chapterIndex
reel
chapter
chapterData
projectId
eventState
event
errorReason
createTime
```

`memories`：

```txt
id
isolationKey
type
role
name
content
embedding
relatedMessageIds
summarized
createTime
```

`o_scriptAssets`：

```txt
scriptId
assetId
```

本地文件：

```txt
剧本 Agent 读取 data/skills 下的 prompt 文件。
页面 Markdown 编辑器阻止图片拖拽上传。
本模块不直接读写素材文件。
```

任务记录：

```txt
Toonflow 剧本 Agent 对话没有写任务中心。
VT Studio 如果要把长时间 Agent 生成纳入任务中心，属于增强能力。
```

状态流转：

```txt
聊天生成：idle -> pending -> streaming -> complete/error/stop
记忆：message、summary 两类，message 可 summarized=0/1
原文事件：eventState=0 代表存在未完成事件分析
```

轮询/Socket：

```txt
对话使用 Socket.IO。
无轮询。
```

模型调用：

```txt
统一走 u.Ai.Text(agentKey, think, thinlLevel).stream。
agentKey 来自 Agent 模型配置。
```

## 12. 源码冲突和风险

1. `setPlanData.ts` 的 zod schema 只声明 `storySkeleton` 和 `adaptationStrategy`，但后面又读取 `data.script` 并 `script.map`；按 schema 通过的数据没有 script，会有运行风险。
2. 前端 `setPlanData` 发送了 `script` 数组，但后端 schema 没声明 script；VT Studio 必须统一 schema。
3. `tools.ts` 里的 `planData.script` 定义为 string，但前端真实是数组；工具读取 `script` 时语义不一致。
4. `updateData.ts` 要求 `script[].id` 是必填 number，但前端编辑故事骨架/改编策略时传的是 `planData.script`，里面新生成的剧本可能没有 id。
5. 页面“重连”按钮调用的是 `productionAgentStore().reconnect()`，不是 `scriptAgentStore().connect()` 或 reconnect，属于明显引用错误。
6. `handleClearMemory` 类型声明包含 `"reconnect"`，但 memoryTypeLabel 没有 reconnect，实际调用只传 message/summary/all。
7. `getPlanData.ts` 首次创建 o_agentWorkData 时 data 只有 storySkeleton 和 adaptationStrategy，没有 script；script 是后续从 o_script 单独查出来拼上。
8. Agent XML 用 scriptItem name 匹配剧本，重名会覆盖；VT Studio 应使用稳定 id 或生成唯一分集 key。
9. 删除剧本会删除分镜、视频和部分文件，但没有看到删除 o_assets 资产和所有视频文件的完整清理，删除级联范围要和 M-005/M-008 对齐。
10. Socket stop 前端发送 `{ messageId }`，后端 `socket.on("stop", () => {})` 不使用参数；语义能工作，但契约不严谨。
11. 页面显示“强制生成”蒙层，但按钮只关闭蒙层，没有触发事件分析生成。

## 13. 源码未找到

```txt
源码未找到剧本 Agent 生成过程写入任务中心的逻辑。
源码未找到剧本 Agent 对话附件上传的实际 UI。
源码未找到章节事件 tab 的正式启用实现；当前 tab 被注释。
源码未找到强制生成蒙层触发事件分析的实现。
源码未找到 scriptAgent 页面离开时主动 disconnect 的代码；store useChat 设置 manageLifecycle=false。
```

## 14. 需要你确认

| 编号 | 问题 | 建议 |
|---|---|---|
| C-011-001 | VT Studio 是否用 IPC event 替代 Socket.IO | 已定：继续用 CORE-008 Socket.IO，保留参考项目消息事件语义 |
| C-011-002 | 剧本 Agent 生成是否进入任务中心 | 建议进入任务中心，至少记录开始、完成、失败、停止 |
| C-011-003 | Agent 生成的 scriptItem 是否允许按 name 覆盖 | 建议不要只按 name，VT Studio 应使用稳定分集 id |
| C-011-004 | 原文事件未完成时是否允许继续生成剧本 | 建议允许但明确提示风险；不要用没有动作的“强制生成”蒙层 |

## 15. VT Studio 落地要求

1. 页面只调用 `window.vtStudio.agent.script.*`，不直接连数据库、不直接读 skill 文件。
2. 计划数据结构必须统一：故事骨架、改编策略、剧本列表不能前后端各写一套 schema。
3. 剧本 Agent 写入真实剧本表时必须走事务：工作区数据和 scripts 表要保持一致。
4. 记忆按 `projectId:scriptAgent` 隔离，生产 Agent 使用 `projectId:productionAgent:episodesId`，不能混用。
5. 对话消息事件要保留 message/content 分层，支持 thinking、toolcall、markdown、error、stop。
6. Agent 子任务不能写进 Vue 页面，必须在 main/services/agent 或等价服务层。
7. skill 文件只能由主进程服务读取，renderer 不接触本地 skill 路径。
8. 源码里的 schema 冲突、错误 store 引用、按名称覆盖剧本都要修正，不能照搬。

## 16. 缺少的逻辑和实现细节

### F-011-001 剧本 Agent 对话入口

参考项目 `useChat.ts` 的真实流式事件不是只有 `message`。CORE-008 必须兼容底层事件名，业务页再按需要组装 UI。

必须明确：

```txt
服务端 -> 客户端：message、message:update、content:add、content:update、error
客户端 -> 服务端：chat、stop、regenerate、updateThinkConfig
连接事件：connect、disconnect、connect_error/reconnect
message:update：更新整条消息状态
content:add：新增 thinking/markdown/toolcall/suggestion 等内容块
content:update：更新内容块状态或追加内容
regenerate：参考项目前端有事件，后端业务重生成需要历史消息支持，不能伪实现
```

VT Studio 已在 CORE-008 确认使用 Socket.IO，不再建议把 M-011 改成 IPC event。

参考项目进入页面会初始化欢迎消息、连接 Socket、展示右侧计划数据。VT Studio 可以用 IPC event 替代 Socket，但消息语义要保留。

必须明确：

```txt
连接：进入页面建立会话
发送：用户消息 + 当前项目上下文 + think 配置
接收：thinking、markdown、toolcall、error、stop
停止：停止当前 messageId，不只是关闭按钮 loading
离开页面：源码未找到主动 disconnect，VT Studio 必须释放会话监听
```

### F-011-002 获取和保存计划数据

参考源码 schema 冲突明显，VT Studio 必须统一。

必须明确：

```txt
计划数据：storySkeleton、adaptationStrategy、scripts
首次创建：没有计划时创建空故事骨架和改编策略
剧本列表：从真实 scripts 表读取，不只存在 agentWorkData
保存：agentWorkData 和 scripts 表事务同步
schema：前端、服务、Agent 工具必须共用同一类型
```

### F-011-003 读取原文上下文

剧本 Agent 依赖原文和事件摘要。

必须明确：

```txt
读取范围：当前项目全部章节
事件未完成：发现 eventState=running 时提示
强制继续：源码按钮只关蒙层，不触发分析；VT Studio 要明确是继续生成还是去分析
上下文内容：章节名、正文、事件摘要、失败原因
```

### F-011-004 写入/更新剧本数据

Agent 写剧本不能只改右侧卡片。

必须明确：

```txt
新剧本：写 scripts 表
已有剧本：按稳定 id 更新，不按 name 覆盖
重名剧本：必须允许存在或明确阻止，不能靠名称匹配
删除剧本：走 M-005 删除链路
写入后：同步右侧计划数据和剧本列表
```

### F-011-005 清理和读取 Agent 记忆

记忆必须按项目和 Agent 隔离。

必须明确：

```txt
读取类型：message、summary
清理范围：message、summary、all
二次确认：清理记忆必须确认
隔离 key：projectId:scriptAgent
清理后：刷新历史消息和摘要状态
```

### F-011-006 剧本 Agent 决策/执行/监督层

参考项目有决策层、执行层、监督层和工具调用，不能做成单轮聊天。

必须明确：

```txt
决策层：判断下一步调用哪个子 Agent 或工具
执行层：故事骨架、改编策略、剧本生成
监督层：检查输出是否符合格式和目标
工具：读取章节事件、小说原文、当前计划、已有剧本
XML 写入：storySkeleton、adaptationStrategy、scriptItem
输出校验：XML 不合法时不能写入业务数据
任务中心：参考源码未写任务；VT Studio 如果记录 Agent 任务，写入 04
```

### Agent 和模型协议

剧本 Agent 只应该依赖文本模型能力，不知道底层是 OpenAI SDK、兼容 HTTP 还是其他供应商。

必须明确：

```txt
页面只传 agentKey/thinkLevel/message
模型解析由 Model Adapter 完成
供应商错误返回给 Agent 服务，再映射到 UI
renderer 不保存 apiKey，不执行供应商代码
```
