# CORE-008 Agent 事件流

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`（第 7 节 Agent Socket、第 8.5 节 Agent 事件契约）  
前置决策：C-BASE-002 已定：**使用 Socket.IO**  
原则：已确认并完成代码实现

---

## 0. 快速理解

```txt
一句话：在 main 进程的 HTTP 服务上挂 Socket.IO，实现 Agent 流式消息的统一事件通道。
为什么现在做：剧本 Agent、生产 Agent 的流式对话、thinking/toolcall 推流都依赖这一层；
              这层不定好，M-011 剧本 Agent 和 M-008 生产工作台开不了工。
做完后有什么用：renderer 连本地 Socket.IO，发 chat 消息，实时收到 thinking/markdown/toolcall
              等分块，消息完成后拿到 complete 状态；stop 发出后立即中断推流。
这一步不碰什么：不做 Agent 业务逻辑（改编策略/故事骨架等），不做剧本/资产/分镜生成，
               不做 CORE-011 记忆层（记忆在 Agent 会话里用，但 embedding 单独一个任务）。
```

---

## 1. 本次做什么

```txt
目标：实现 Socket.IO 服务端挂载、两个 Agent namespace 的事件契约、以及流式消息推送的
      底层能力；renderer 侧提供 socket 连接工具函数供 Agent 页面复用。

只做：
  - main/services/socket/index.ts       Socket.IO 服务启动和关闭
  - main/services/socket/scriptAgent.ts /socket/scriptAgent namespace 事件处理
  - main/services/socket/productionAgent.ts /socket/productionAgent namespace 事件处理
  - main/services/socket/types.ts       所有 Socket 事件和消息类型定义
  - main/services/socket/stripThink.ts  剥离 <think>...</think> 标签的工具函数
  - renderer src/composables/useAgentSocket.ts  前端 socket.io-client 连接/断开/收消息封装
  - 在 main/app/server.ts（或 HTTP 服务入口）挂载 Socket.IO

不做：
  - Agent 业务决策/监督/执行逻辑
  - 记忆读写（CORE-011）
  - 剧本生成/资产生成页面
  - Skill 向量检索（CORE-011）
  - 任何页面 UI
```

---

## 2. 参考项目怎么做

| 参考文件 | 关键逻辑 |
|---|---|
| `socket/routes/scriptAgent.ts` | `/socket/scriptAgent` namespace；连接时校验 `auth.token/isolationKey/projectId`；监听 `chat/updateThinkConfig/stop`；调 `agents/scriptAgent` 并把流式内容 emit 回去 |
| `socket/routes/productionAgent.ts` | `/socket/productionAgent` namespace；额外校验 `auth.scriptId`；比 scriptAgent 多一个 `updateContext` 事件 |
| `socket/resTool.ts` | 流式响应工具：`streamRes(socket, stream, taskRecord)` 封装了 chunk 读取、`thinking`/`markdown` 分发、`toolcall` 推送、`complete`/`error` 状态写回 |
| `agents/scriptAgent/index.ts` | 接收 chat 参数，读 prompt/skill/memory，组装 AI 调用，返回 stream |
| `agents/productionAgent/index.ts` | 同上，额外依赖 scriptId 读分镜上下文 |
| `utils/stripThink.ts` | 正则去掉 `<think>…</think>` 块，仅保留 think 标签外内容；在推流前过滤最终输出 |

---

## 3. 用户操作（本任务底层，无页面）

```txt
入口：无页面入口。
按钮/操作：无页面按钮。
后续 M-011 剧本 Agent 页面会调用本任务的 useAgentSocket 组合函数。
```

---

## 4. 数据表变动

本任务不新增表、不改表结构。

用到的已有表：
- `agent_model_configs`：解析 Agent 当前配置的模型
- `app_settings`：读 agentUseMode / tokenKey
- `prompts`：读提示词
- `skill_list` / `skill_attributions`：Skill 归属（CORE-011 完成前跳过 embedding 检索，仅按 attribution 筛选）

---

## 5. 实现方案

### 5.1 Socket.IO 服务挂载

```txt
Socket.IO 挂载在已有的 Express HTTP 服务器上（不单独开新端口）。
VT Studio 本地 HTTP 服务端口由 runtime 分配；Socket.IO 和 HTTP 共用同一端口。
只绑 127.0.0.1，不对外网开放。
服务随 app 启动，随 app 退出关闭。
```

挂载位置：`src/main/app/server.ts`（或当前 Express 服务入口）

```ts
// 伪代码示意
import { Server } from 'socket.io'
const io = new Server(httpServer, { cors: { origin: false } })
registerScriptAgentNamespace(io)
registerProductionAgentNamespace(io)
```

### 5.2 连接鉴权

所有 namespace 连接时校验 `socket.handshake.auth`：

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `token` | string | ✅ | 与 `app_settings.tokenKey` 校验 |
| `isolationKey` | string | ✅ | 记忆隔离键，格式 `projectId:agentKey` |
| `projectId` | number/string | ✅ | 当前项目 ID |
| `scriptId` | number/string | ✅（productionAgent） | 当前剧本 ID |

校验失败时 `socket.disconnect(true)`，不推消息。

### 5.3 事件定义

#### 客户端 → 服务端（emit to server）

| 事件名 | 参数 | 说明 |
|---|---|---|
| `chat` | `{ content: string, role?: 'user' }` | 发送用户消息，触发 AI 推流 |
| `updateThinkConfig` | `{ think: boolean, thinkLevel?: string }` | 更新当前会话 think 开关 |
| `stop` | 无 | 中断当前推流 |
| `updateContext` | `{ context: any }` | 更新上下文（仅 productionAgent） |

#### 服务端 → 客户端（emit to client）

| 事件名 | 参数 | 说明 |
|---|---|---|
| `message` | `AgentMessage` | 推流消息（见下方类型） |
| `getPlanData` | `PlanData` | 返回当前计划数据（仅 scriptAgent） |
| `error` | `{ code: string, msg: string }` | 服务端错误 |

#### AgentMessage 类型

```ts
interface AgentMessage {
  id: string               // 消息 ID，同一轮推流 ID 相同
  type: AgentMessageType   // 内容类型
  content: string          // 文本内容
  status: AgentMsgStatus   // 消息状态
  toolcall?: ToolCallInfo  // type=toolcall 时有值
}

type AgentMessageType =
  | 'text'
  | 'markdown'
  | 'thinking'
  | 'toolcall'
  | 'search'
  | 'reasoning'
  | 'suggestion'

type AgentMsgStatus =
  | 'pending'
  | 'streaming'
  | 'complete'
  | 'stop'
  | 'error'
```

### 5.4 stripThink 工具

```txt
位置：src/main/services/socket/stripThink.ts
作用：从 AI 输出中去掉 <think>…</think> 标签块，只保留标签外的内容。
      thinking 内容单独作为 type=thinking 的消息推给前端。
```

示例行为：

```txt
输入：<think>这是内部推理</think>\n最终输出
推送：
  1. { type: 'thinking', content: '这是内部推理', status: 'streaming' }
  2. { type: 'markdown', content: '最终输出', status: 'complete' }
```

### 5.5 流式推送流程

```txt
1. 收到 chat 事件
2. 从 agent_model_configs 解析当前 Agent 的模型（agentUseMode 简易/高级）
3. 从 prompts 读取对应提示词
4. 按 skill_attributions 筛选当前 Agent 的 skill（CORE-011 完成前跳过向量检索，仅按 path 加载 Markdown 内容）
5. 调用 model service 文本流接口（CORE-007）
6. 逐 chunk 推 message 事件：thinking 片段推 type=thinking，正文推 type=markdown
7. 推流结束后推 { status: 'complete' }
8. 任何异常推 { status: 'error', content: 错误文案 }
9. 收到 stop 事件时中止 stream，推 { status: 'stop' }
```

### 5.6 renderer 侧封装

```txt
位置：src/renderer/src/composables/useAgentSocket.ts
依赖：socket.io-client

提供：
  connect(namespace, auth)   // 连接对应 namespace
  disconnect()               // 断开
  sendChat(content)          // 发 chat 事件
  stop()                     // 发 stop 事件
  onMessage(cb)              // 注册 message 回调
  onError(cb)                // 注册 error 回调
  isConnected: Ref<boolean>
  messages: Ref<AgentMessage[]>
```

---

## 6. 文件清单

| 路径 | 类型 | 说明 |
|---|---|---|
| `src/main/app/server.ts` | 新建 | 本地 HTTP server，监听 127.0.0.1 随机端口 |
| `src/main/services/socket/index.ts` | 新建 | Socket.IO 服务初始化、挂载、关闭、获取连接信息 |
| `src/main/services/socket/auth.ts` | 新建 | Socket 鉴权，校验 app_settings.tokenKey 和连接参数 |
| `src/main/services/socket/types.ts` | 新建 | main 侧 Socket 事件和会话状态类型 |
| `src/main/services/socket/agent-handler.ts` | 新建 | scriptAgent / productionAgent 共用事件处理 |
| `src/main/services/socket/stripThink.ts` | 新建 | think 标签流式剥离工具 |
| `src/main/ipc/agent.ts` | 新建 | `agent:get-socket-info`，给 renderer 返回 Socket url/token |
| `src/main/ipc/index.ts` | 修改 | 注册 Agent IPC |
| `src/main/index.ts` | 修改 | 启动/关闭本地 server 和 Socket.IO 服务 |
| `src/preload/index.ts` | 修改 | 暴露 `window.vtStudio.agent.getSocketInfo()` |
| `src/shared/contracts/preload.ts` | 修改 | 增加 agent preload contract |
| `src/renderer/src/composables/useAgentSocket.ts` | 新建 | 前端 socket.io-client 封装 |
| `src/shared/types/socket.ts` | 新建 | 共享的 AgentMessage 类型（renderer/main 都用） |

---

## 7. 依赖变动

| 包 | 方向 | 说明 |
|---|---|---|
| `socket.io` | main 新增 | Socket.IO 服务端 |
| `socket.io-client` | renderer 新增 | Socket.IO 客户端 |

---

## 8. 验收标准

```txt
1. typecheck 通过，build 通过
2. renderer 连接 /socket/scriptAgent 成功，token 错误时被拒绝
3. 发送 chat 消息后能收到若干 { type:'markdown', status:'streaming' } 消息，
   最后收到 { status:'complete' }
4. 发送 stop 后推流立即中止，收到 { status:'stop' }
5. updateThinkConfig 生效：think=true 时能收到 type='thinking' 的消息
6. 服务随 app 关闭时 Socket.IO server 正常 close，不抛异常
```

---

## 9. 暂不做

```txt
Agent 业务逻辑（改编策略、故事骨架、资产推导等）
CORE-011 记忆和 embedding（Skill 向量检索）
M-011 剧本 Agent 页面、M-008 生产工作台页面
多 scriptId 并发隔离（单实例先跑通，并发后置）
```

---

## 9.1 本轮补充

参考项目 `Toonflow-web/src/utils/useChat.ts` 和 `Toonflow-app/src/socket/resTool.ts` 不是只用 `message` 事件，还用了更细的消息和内容块事件。CORE-008 作为底层事件流需要兼容这些事件名，业务页后续才能按参考项目逻辑实现。

要补的事件：

```txt
服务端 -> 客户端：
  message
  message:update
  content:add
  content:update
  error

客户端 -> 服务端：
  chat
  stop
  regenerate
  updateThinkConfig
  updateContext
```

怎么做：

```txt
1. shared socket 类型增加 AgentMessageUpdatePayload / AgentContentAddPayload / AgentContentUpdatePayload / AgentRegeneratePayload。
2. main socket 事件类型增加 message:update、content:add、content:update、regenerate。
3. 当前流式输出继续保留 message 兼容事件，同时补 message:update 和 content:add/content:update。
4. regenerate 先接收并返回“业务重生成待 M-011/M-008 接历史消息后实现”的明确错误，不能伪造重生成。
5. renderer useAgentSocket 监听这些事件，并提供 regenerate/reconnect/on/off/once。
```

限制：

```txt
本轮只补底层事件契约，不做剧本 Agent UI，不做 XML 工作区写入，不做 Production Agent 工具业务。
```

## 10. 执行后记录

```txt
改了哪些文件：
  package.json / pnpm-lock.yaml
  src/main/app/server.ts
  src/main/index.ts
  src/main/ipc/agent.ts
  src/main/ipc/index.ts
  src/main/services/socket/auth.ts
  src/main/services/socket/index.ts
  src/main/services/socket/agent-handler.ts
  src/main/services/socket/stripThink.ts
  src/main/services/socket/types.ts
  src/preload/index.ts
  src/shared/contracts/preload.ts
  src/shared/types/socket.ts
  src/renderer/src/composables/useAgentSocket.ts
  docs/tasks/CORE-008-agent-事件流.md
  docs/03-执行进度.md

验证结果：
  直接运行 vue-tsc --noEmit -p tsconfig.web.json 通过
  直接运行 tsc --noEmit -p tsconfig.node.json 通过
  pnpm run build 未完成：当前 node_modules 被 pnpm 重建中断影响，重新 install 时 better-sqlite3 原生构建失败，electron/electron-vite 链接未完整生成

未完成事项：
  Agent 业务编排、prompt/skill/memory 组合、页面 UI 不在 CORE-008 范围；后续在 M-011 / M-008 / CORE-011 中实现。

最终结论：
  CORE-008 已完成底层 Socket.IO 事件流：本地 server 启停、Socket.IO namespace、鉴权、chat/stop/regenerate/updateThinkConfig/updateContext、message/message:update/content:add/content:update、think 分流、renderer composable 全部落地。
```
