# F-008-006 Production Agent 对话与调度

状态：等待用户确认  
所属菜单：M-008 生产工作台  
对应功能文档：`docs/features/M-008-生产工作台.md`  
原则：先确认本文档，再改代码

## 0. 快速理解

```txt
一句话：生产页右侧 Agent 能对话、调用工具、写导演计划、衍生资产、分镜表和分镜。
为什么现在做：生产页不是手工编辑器，参考项目核心能力是 Production Agent 调度。
做完后有什么用：用户可以让 Agent 协助完成生产链路中的计划、资产和分镜任务。
这一步不碰什么：不做剧本 Agent，不做剪映导出，不让前端直接执行工具写库。
```

## 1. 本次做什么

```txt
目标：
  对齐 Toonflow productionAgent store、Socket 和后端 Agent 工具。

只做：
  1. 右侧 Agent 面板
  2. 流式对话、停止、重连、清记忆
  3. 思考等级
  4. 获取 flowData 工具
  5. 添加/删除/生成衍生资产工具事件
  6. 生成分镜工具事件
  7. 新增分镜面板工具事件
  8. 兼容 CORE-008 的 Socket.IO 事件契约

不做：
  1. 剪映导出
  2. 剧本 Agent
  3. 模型供应商配置
```

## 2. 参考项目怎么做

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-web/src/views/production/components/rightChatBox/index.vue` | 右侧聊天面板、连接状态、清记忆、思考等级 |
| `Toonflow-web/src/stores/productionAgent.ts` | Socket 连接、事件监听、getFlowData、工具事件处理 |
| `Toonflow-app/src/socket/routes/productionAgent.ts` | chat/stop/updateContext/updateThinkConfig |
| `Toonflow-app/src/agents/productionAgent/index.ts` | 决策层运行 |
| `Toonflow-app/src/agents/productionAgent/tools.ts` | get_flowData、add_deriveAsset、generate_storyboard 等工具 |

参考项目关键事实：

```txt
1. isolationKey = projectId:productionAgent:episodesId。
2. 右侧面板最小 400px，最大窗口 80%，可拖拽。
3. store 监听 getFlowData、addDeriveAsset、delDeriveAsset、generateDeriveAsset、generateStoryboard、addStoryboard。
4. XML 标签 script/scriptPlan/storyboardTable 会流式写入 flowData。
5. get_flowData 返回给 Agent 时会删掉 prompt/flowId/src 等字段。
6. Agent 运行没有直接写任务中心，VT Studio 建议写。
7. VT Studio 已定 Agent 事件流使用 Socket.IO，不再改成 IPC event。
```

## 3. 用户操作

```txt
入口：
  生产页右侧 Agent 面板。

按钮/操作：
  1. 输入消息并发送。
  2. 停止生成。
  3. 清 message/summary/all 记忆。
  4. 切换思考等级。
  5. 打开/关闭右侧面板。

弹窗/表单：
  清记忆确认框。

成功反馈：
  Agent 流式回复，工具执行后流程节点更新。

失败反馈：
  连接失败、模型失败、工具失败、XML 非法。
```

## 4. 要做什么功能

### 1. 建立 Production Agent 会话

怎么做：
- 输入：projectId、scriptId。
- 输出：Agent 会话。
- 写什么数据：不直接写业务数据。
- 状态怎么变：disconnected -> connected。
- 异常怎么处理：缺 scriptId 或连接失败显示错误。
- 限制：隔离 key 必须包含 scriptId，不能和剧本 Agent 混用。

### 2. 流式对话和停止

怎么做：
- 输入：用户消息。
- 输出：assistant 流式消息。
- 写什么数据：写记忆，结构化结果由工具服务写。
- 状态怎么变：idle -> pending -> streaming -> complete/error/stop。
- 异常怎么处理：AbortError 之外的错误显示 error 消息。
- 限制：停止只停止当前请求。

### 3. Agent 工具事件落地

怎么做：
- 输入：工具调用事件。
- 输出：flowData 或业务数据更新。
- 写什么数据：通过 main services 写衍生资产、分镜等。
- 状态怎么变：工具调用中 -> 成功/失败。
- 异常怎么处理：工具参数非法拒绝写入。
- 限制：前端不能直接信任 Agent 输出写库。

必须覆盖的工具事件：

```txt
getFlowData：读取当前流程数据，返回给 Agent 前过滤大字段和本地路径
addDeriveAsset：新增衍生资产节点
delDeriveAsset：删除衍生资产节点
generateDeriveAsset：触发衍生资产生成
generateStoryboard：触发分镜生成
addStoryboard：新增分镜面板
updateContext：更新 productionAgent 上下文
updateThinkConfig：更新思考配置
```

### 4. 清理记忆和思考等级

怎么做：
- 输入：memoryType 或 thinkLevel。
- 输出：记忆清理结果或配置更新。
- 写什么数据：删除 memories 或更新会话配置。
- 状态怎么变：清理后刷新历史；思考等级更新。
- 异常怎么处理：失败提示并回滚 UI。
- 限制：只作用于 productionAgent 当前 scriptId。

## 5. 数据和状态

```txt
字段：
  memories
  agent_work_data
  assets
  storyboards
  tasks

接口/能力：
  agent.production.connect
  agent.production.chat
  agent.production.stop
  agent.production.clearMemory
  agent.production.updateThinkConfig

数据读写：
  读 flowData
  写记忆、衍生资产、分镜、工作区数据

任务状态：
  建议写 tasks，分类为 Production Agent

轮询/Socket：
  使用 CORE-008 Socket.IO，不改 IPC event

模型调用：
  Production Agent 模型配置

删除影响：
  无
```

## 6. VT Studio 怎么落

```txt
能力名：
  agent.production

调用链：
  renderer 右侧面板 -> window.vtStudio.agent.production -> main/services/agent/production

需要新增：
  右侧 Agent 面板
  production agent service
  工具调用 schema 校验
  task 记录接入

需要修改：
  flowData store 和工具事件同步
```

## 7. 偏差

```txt
和 Toonflow 不同的地方：
  工具写业务数据必须走 main service，并建议写任务中心。

原因：
  Agent 输出不能绕过服务层直接改前端状态或数据库。

是否写入 04：
  写入。
```

## 8. 验收

```txt
1. 右侧 Agent 面板可打开关闭。
2. 能发送消息并流式回复。
3. 能停止生成。
4. 记忆按 projectId:productionAgent:scriptId 隔离。
5. 思考等级只在支持模型时显示。
6. 工具调用参数非法不会写库。
7. 工具成功后流程数据更新。
8. 页面不直接访问 SQLite、skill 文件或模型 SDK。
```

## 9. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-F-008-006-001 | Production Agent 是否写任务中心 | 建议写 |
| C-F-008-006-002 | 是否用 IPC event 替代 Socket.IO | 不替代；已定使用 CORE-008 Socket.IO |

## 10. 执行后记录

```txt
改了哪些文件：（完成后填写）
验证结果：（完成后填写）
未完成事项：（完成后填写）
最终结论：（完成后填写）
```

## 11. 最后大白话

```txt
我这次准备怎么做：
1. 做生产页右侧 Agent。
2. Agent 能聊天、调用工具、更新生产流程。
3. 工具写数据必须走服务层。

我不会做什么：
1. 不做剪映导出。
2. 不做剧本 Agent。
3. 不让前端直接写数据库。

确认规则：
用户确认后才执行；未确认前只改文档。
```
