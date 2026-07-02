# CORE-011 嵌入与记忆层

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`、`docs/features/M-002-设置.md`、`docs/features/M-011-剧本Agent.md`  
原则：先确认本文档，再改代码

---

## 0. 快速理解

```txt
一句话：把本地 ONNX 向量模型、Agent 记忆、摘要压缩和 Skill 检索这套底层能力补齐。
为什么现在做：剧本 Agent 和生产 Agent 后续都要读历史对话、摘要、RAG 和 Skill。
做完后有什么用：Agent 能保存记忆、按项目隔离记忆、检索相关历史、自动生成摘要、读取匹配 Skill。
这一步不碰什么：不做设置页面 UI，不做完整 Agent 业务决策，不做云端 embedding，不做模型下载器。
```

---

## 1. 本次做什么

```txt
目标：
  建立 VT Studio 的本地嵌入与记忆基础层，让后续 M-011/M-008 可以直接调用。

只做：
  1. 本地 ONNX embedding 初始化和文本向量化
  2. memories 表迁移
  3. 记忆写入、读取、清理
  4. 短期记忆、摘要记忆、RAG 召回
  5. 摘要生成能力，文本摘要复用 CORE-007 文本模型
  6. Skill embedding 补生成
  7. Skill 按 attribution / 向量相似度检索
  8. 给后续 Agent 提供 service API

不做：
  1. 设置页记忆配置 UI
  2. Skill 管理页面 UI
  3. 剧本 Agent 页面接入
  4. 生产 Agent 页面接入
  5. 自动下载 ONNX 模型文件
  6. 供应商 embedding adapter
  7. 云端向量数据库
```

---

## 2. 参考项目怎么做

源码已查完，以下是事实。

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-app/src/utils/agent/embedding.ts` | 使用 `@huggingface/transformers` + `onnxruntime-web`；读取 `o_setting.modelOnnxFile/modelDtype`；只允许本地模型；模型不存在直接抛错；`getEmbedding(text)` 返回 normalized mean pooling 向量 |
| `Toonflow-app/src/utils/agent/memory.ts` | `Memory(agentType,isolationKey)`；`add()` 写 message embedding；消息数达到阈值后生成 summary；`get()` 返回 shortTerm、summaries、rag；`deepRetrieve()` 先搜 summary，再用 AI 判断相关，再展开原 message |
| `Toonflow-app/src/utils/agent/skillsTools.ts` | 读取本地 skills Markdown；解析 frontmatter；提供 `activate_skill` 和 `read_skill_file` 工具；有路径越界保护 |
| `Toonflow-app/src/routes/agents/getMemory.ts` | 根据 `projectId/agentType/episodesId` 拼 isolationKey，只返回 `message` 记忆，转成聊天历史消息 |
| `Toonflow-app/src/routes/agents/clearMemory.ts` | 支持清 `message/summary/all`；清 message 时同步清 summary；清 summary 时把已总结 message 重置为未总结 |
| `Toonflow-app/src/routes/setting/memoryConfig/getMemory.ts` | 读取 messagesPerSummary、shortTermLimit、summaryMaxLength、summaryLimit、ragLimit、deepRetrieveSummaryLimit、modelOnnxFile、modelDtype |
| `Toonflow-app/src/routes/setting/memoryConfig/sureMemory.ts` | 保存上述 8 个记忆配置到 `o_setting` |
| `Toonflow-app/src/routes/setting/memoryConfig/delAllMemory.ts` | 直接清空全部 `memories` |

参考项目已有问题：

```txt
1. embedding 模型不存在时直接抛错，容易阻断启动或 Agent 运行。
2. memory.ts 里默认 messagesPerSummary=3，但 initDB 默认是 10，存在默认值不一致。
3. setting 的 delAllMemory 是全库清空，缺少当前项目/当前 Agent 范围保护。
4. Skill 文件读取依赖本地 skills 目录，路径安全必须保留。
5. embedding 走本地 ONNX，不走供应商模型；这是 VT Studio 已确认偏差 D-BASE-014。
```

---

## 3. 用户操作

本任务本身是底层能力，直接用户入口很少。

```txt
入口：
  1. Agent 页面进入时，后续会读取历史记忆。
  2. Agent 对话时，后续会写入 user/assistant 记忆。
  3. 设置页后续会读取和保存记忆配置。
  4. 设置页后续会提供清空记忆、重建 Skill embedding。

按钮/操作：
  本任务不新增页面按钮。

弹窗/表单：
  本任务不新增页面弹窗。

成功反馈：
  底层 service 返回成功；页面反馈由后续 M-002/M-011/M-008 处理。

失败反馈：
  1. ONNX 模型不存在：返回明确错误，不崩溃。
  2. embedding 失败：返回失败原因。
  3. 清理记忆失败：返回失败原因。
  4. Skill 文件不存在或越界：返回失败原因。
```

---

## 4. 要做什么功能

### 1. 本地 ONNX embedding 初始化

怎么做：
- 输入：`app_settings.modelOnnxFile`、`app_settings.modelDtype`。
- 输出：可复用的 embedding extractor。
- 写什么数据：不写数据库。
- 状态怎么变：未加载 -> 已加载。
- 异常怎么处理：
  - 模型路径配置不是数组：返回 `code=400`。
  - 模型文件不存在：返回 `code=400`，msg 写清实际缺哪个模型文件。
  - 初始化失败：释放半初始化状态，下次可重试。
- 限制：
  - 只允许读取 runtime models 目录下的本地模型。
  - `allowRemoteModels=false`，不联网下载。
  - 不接供应商 embedding 接口。

### 2. 文本向量化

怎么做：
- 输入：`text`。
- 输出：`number[]` embedding。
- 写什么数据：不直接写数据库。
- 状态怎么变：无。
- 异常怎么处理：
  - 空文本：返回空向量或明确失败，不能写脏数据。
  - 模型未加载：自动初始化。
- 限制：
  - 使用 mean pooling + normalize，对齐参考项目。
  - 向量 JSON 序列化后再入库。

### 3. memories 表迁移

怎么做：
- 输入：无。
- 输出：创建 memories 表和必要索引。
- 写什么数据：SQLite schema。
- 建议字段：
  - `id`
  - `isolation_key`
  - `type`：`message` / `summary`
  - `role`
  - `name`
  - `content`
  - `embedding`
  - `related_message_ids`
  - `summarized`
  - `created_at`
- 状态怎么变：数据库具备记忆读写能力。
- 异常怎么处理：迁移失败则启动失败。
- 限制：
  - 表名和字段用 VT Studio 命名，不照搬 `isolationKey/createTime` 旧命名。

### 4. 记忆隔离 key

怎么做：
- 输入：`projectId`、`agentType`、可选 `episodesId`。
- 输出：稳定 isolationKey。
- 写什么数据：不写。
- 状态怎么变：无。
- 规则：
  - 剧本 Agent：`${projectId}:scriptAgent`
  - 生产 Agent：`${projectId}:productionAgent:${episodesId}`
- 异常怎么处理：
  - productionAgent 缺 episodesId：返回失败。
  - projectId 缺失：返回失败。
- 限制：不同项目、不同 Agent、不同分集不能串记忆。

### 5. 添加记忆

怎么做：
- 输入：`agentType`、`isolationKey`、`role`、`content`、可选 `name/createTime`。
- 输出：新增 message 记忆。
- 写什么数据：memories 表。
- 状态怎么变：
  - 插入 `type=message`。
  - 生成 embedding。
  - 根据 messagesPerSummary 判断是否触发摘要。
- 异常怎么处理：
  - embedding 模型缺失：写入失败，返回明确错误；不插入无 embedding 的 message。
  - 摘要生成失败：message 保留，summary 不写，错误记录日志。
- 限制：
  - 空 content 不写。
  - role 保留 `user`、`assistant`、`assistant:*`，读取历史时再归一化。

### 6. 自动摘要

怎么做：
- 输入：同 isolationKey 下未总结 message。
- 输出：一条 summary 记忆。
- 写什么数据：
  - 新增 `type=summary`。
  - summary 写 embedding。
  - 原 message 标记 `summarized=1`。
- 状态怎么变：多条 message -> 一条 summary。
- 异常怎么处理：
  - CORE-007 文本模型未配置：跳过摘要，保留 message 未总结状态。
  - summary embedding 失败：不标记 message 已总结。
- 限制：
  - 摘要模型调用复用 CORE-007 文本模型，不给模型层新增 embedding 类型。
  - summaryMaxLength 控制摘要长度。

### 7. 读取记忆上下文

怎么做：
- 输入：`isolationKey`、查询文本。
- 输出：
  - `shortTerm`：最近未总结 message
  - `summaries`：最近 summary
  - `rag`：相似 message
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：
  - embedding 失败：返回 shortTerm/summaries，rag 为空并给出失败原因。
  - 配置缺失：使用 CORE-009 默认值。
- 限制：
  - shortTermLimit、summaryLimit、ragLimit 必须从 app_settings 读取。

### 8. 深度检索

怎么做：
- 输入：`keyword`。
- 输出：和关键词相关的原始 message 内容。
- 写什么数据：不写。
- 状态怎么变：无。
- 流程：
  1. keyword 生成 embedding。
  2. 向量检索 summary。
  3. 用文本模型判断哪些 summary 相关。
  4. 展开 relatedMessageIds，返回原始 message。
- 异常怎么处理：
  - 没有 summary：返回空数组。
  - AI 判断失败：返回空数组，不影响主对话。
- 限制：这是 Agent tool 能力，页面不直接调用。

### 9. 获取历史消息

怎么做：
- 输入：`projectId`、`agentType`、可选 `episodesId`。
- 输出：聊天页面可渲染的历史消息。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：无记忆时返回空数组。
- 限制：
  - 只返回 `type=message`。
  - role 归一化为 `user` / `assistant`。
  - 不返回 embedding。

### 10. 清理记忆

怎么做：
- 输入：`projectId`、`agentType`、可选 `episodesId`、`type=message|summary|all`。
- 输出：清理成功。
- 写什么数据：删除或更新 memories。
- 状态怎么变：
  - all：删除该 isolationKey 全部记忆。
  - message：删除 message，同时删除 summary。
  - summary：删除 summary，并把已总结 message 重置为 `summarized=0`。
- 异常怎么处理：失败则事务回滚。
- 限制：
  - 不做全库清空默认入口。
  - 设置页如果做“清空全部记忆”，必须二次确认，并单独记录范围。

### 11. Skill embedding 补生成

怎么做：
- 输入：无，或指定 skillId。
- 输出：skill_list references 类型的 embedding/state 更新。
- 写什么数据：skill_list.embedding、skill_list.state、updated_at。
- 状态怎么变：
  - state=-1 -> state=1。
  - 失败时 state 保持 -1 或写失败状态，不能写半截 embedding。
- 异常怎么处理：
  - ONNX 模型不存在：整体返回失败原因，不改原数据。
  - 单条失败：记录失败数量和 skillId。
- 限制：
  - main skill description 为空，不生成 embedding。
  - 只给 references skill 生成 embedding。

### 12. Skill 检索和读取工具

怎么做：
- 输入：agent attribution、用户问题、可选 workspace/attachedSkills。
- 输出：匹配到的 Skill 摘要和安全读取工具。
- 写什么数据：不写。
- 状态怎么变：无。
- 逻辑：
  - 先按 skill_attributions 找当前 Agent 可用 Skill。
  - references skill 可按 embedding 相似度排序。
  - main skill 按指定 attribution 读取。
  - 读取 Markdown 时必须限制在 skills 根目录内。
- 异常怎么处理：
  - 文件不存在：返回错误。
  - 路径越界：直接拒绝。
  - embedding 不可用：降级为按 attribution 返回，不做相似排序。
- 限制：
  - 不在本任务做 Skill 管理页面。
  - 不直接把文件系统暴露给 renderer。

---

## 5. 数据和状态

```txt
字段：
  app_settings.modelOnnxFile
  app_settings.modelDtype
  app_settings.messagesPerSummary
  app_settings.shortTermLimit
  app_settings.summaryMaxLength
  app_settings.summaryLimit
  app_settings.ragLimit
  app_settings.deepRetrieveSummaryLimit
  memories.*
  skill_list.embedding
  skill_list.state
  skill_attributions.*

接口/能力：
  embedding.init
  embedding.embedText
  memory.add
  memory.getContext
  memory.getHistory
  memory.clear
  memory.deepRetrieve
  skill.rebuildEmbeddings
  skill.resolveForAgent

数据读写：
  读 app_settings
  读写 memories
  读写 skill_list embedding/state
  读 skill_attributions

任务状态：
  不写 tasks 表；如果后续 Skill embedding 重建要做进度，再单独接任务队列。

轮询/Socket：
  不轮询，不新增 Socket 事件；后续 Agent Socket 调用本 service。

模型调用：
  embedding：本地 ONNX
  summary/deepRetrieve 相关判断：复用 CORE-007 文本模型

删除影响：
  只删除 memories，不删除项目、脚本、资产、Skill 文件。
```

---

## 6. VT Studio 怎么落

```txt
能力名：
  embedding
  memory
  skillRetrieval

调用链：
  Agent service -> memory service -> embedding service/database/model service
  设置 service -> memory config service -> app_settings/memories
  Skill service -> skillRetrieval -> skill_list/skill_attributions/file-system

需要新增：
  src/main/services/embedding/index.ts
  src/main/services/memory/index.ts
  src/main/services/memory/migrations.ts
  src/main/services/skill-retrieval/index.ts
  src/shared/types/memory.ts

需要修改：
  src/main/services/database/migrations.ts 注册 memories 迁移
  src/main/services/database/seed.ts 在 CORE-011 完成后可补 Skill embedding 生成入口
```

注意：

```txt
1. renderer 页面不能直接调用 embedding。
2. renderer 页面不能直接读写 memories。
3. 所有对外 IPC 仍返回 { code, data, msg }。
4. main 内部 service 可以抛 VtError，由 IPC handler 统一转换。
5. ONNX 模型路径必须走 CORE-004 文件层/运行目录规则，不能写死项目目录。
```

### 6.1 参考项目复核结论

本轮重新看了参考项目真实源码：

```txt
embedding.ts
  做法：@huggingface/transformers + onnxruntime-web，mean pooling + normalize。
  问题：模型不存在直接抛错；没有初始化锁；初始化半失败后状态不清晰；错误里包含绝对路径。

memory.ts
  做法：message 入库时同步生成 embedding；达到阈值后生成 summary；get 返回 shortTerm/summaries/rag。
  问题：add 写 message、写 summary、标记 summarized 没有事务包住；
        summary 失败时没有稳定补偿策略；
        embedding JSON 解析没有防御；
        默认 messagesPerSummary=3，但 initDB 默认 10。

skillsTools.ts
  做法：解析 frontmatter，读取主 Skill 和资源文件，限制在 skills 根目录。
  优点：路径越界保护方向正确。
  问题：scanSkills 可扫传入路径，必须在 VT Studio 中收紧为受控 skills 根目录；
        日志直接 console；
        Skill embedding 和 Skill 文件读取耦合不够清楚。

routes/agents
  做法：getMemory 只读 message；clearMemory 支持 message/summary/all。
  问题：clearMemory 没有事务；all 只按 isolationKey 还可以接受，
        但 setting/memoryConfig/delAllMemory 是全库清空，风险过高。

initDB
  做法：默认初始化 skill_list 时直接调用 getEmbedding 生成 references skill embedding。
  问题：空库初始化会依赖 ONNX 模型存在，模型缺失会让数据库初始化链路变脆。
```

结论：

```txt
参考项目方案可以作为业务语义参考，不能直接照搬工程实现。
VT Studio 必须把 embedding、memory、skillRetrieval 拆成三个 main service，
把失败降级、事务、路径边界、日志、验证入口补完整。
```

### 6.2 VT Studio 专业化设计

复杂度判断：

```txt
不是简单 CRUD，也不是纯 UI 功能。
属于中等复杂度底层能力，关键风险在启动稳定性、模型缺失、事务一致性、后续 Agent 调用契约。
需要轻量架构设计后再实现，不需要另起大型架构文档。
```

分层：

```txt
embedding service
  只负责：读取配置、解析 runtime models 路径、初始化 extractor、embedText、dispose。
  不负责：写 memories、写 skill_list、调用文本模型。

memory service
  只负责：isolationKey、memories 表读写、摘要触发、RAG、deepRetrieve、清理。
  依赖：embedding service、database、CORE-007 text model。
  不负责：Socket UI 消息拼装、页面状态、设置页表单。

skillRetrieval service
  只负责：skill_attributions 查询、references skill 相似度排序、main skill 读取、路径保护。
  依赖：embedding service、skill_list、受控 skills 目录。
  不负责：Skill 管理页面、扫描任意外部目录、文件编辑 UI。
```

运行目录补齐：

```txt
当前 VT Studio file-system 运行目录已有 vendors/cache/logs 等目录，但没有 models/skills。
CORE-011 需要补：
  userData/models
  userData/skills

模型默认路径：
  userData/models/all-MiniLM-L6-v2/onnx/model_fp16.onnx

Skill 文件默认路径：
  userData/skills/*.md
  userData/skills/references/*.md

要求：
  所有读取必须 safeJoin/assertInsideRoot。
  不从项目源码目录读取模型和用户数据。
```

依赖设计：

```txt
当前 package.json 没有 @huggingface/transformers、onnxruntime-web 或 onnxruntime-node。
如果确认做本地 ONNX embedding，需要新增依赖。

建议：
  第一版优先沿用参考项目组合：@huggingface/transformers + onnxruntime-web。
  transformersEnv.allowRemoteModels=false。
  transformersEnv.allowLocalModels=true。
  transformersEnv.localModelPath 指向 userData/models。

不建议：
  不自己手写 tokenizer/ONNX 推理。
  不接供应商 embedding。
  不自动联网下载模型。
```

embedding 初始化策略：

```txt
1. 使用单例缓存 extractor。
2. 使用 loadingPromise 防止并发重复初始化。
3. 初始化前校验 modelOnnxFile 必须是 string[]，不能包含绝对路径或 ..
4. 校验最终模型文件必须在 userData/models 内。
5. 模型缺失返回明确错误，但不影响应用启动和数据库初始化。
6. 初始化失败清空 loadingPromise，下次允许重试。
7. disposeEmbedding 用于应用退出或后续模型切换。
```

memory 写入策略：

```txt
message 写入：
  先生成 embedding，成功后再入库。
  空 content 拒绝写入。
  embedding 失败则不插入无向量 message，避免后续 RAG 脏数据。

summary 写入：
  message 插入成功后检查未总结数量。
  summary 生成失败：保留 message，记录日志，不标记 summarized。
  summary embedding 失败：不写 summary，不标记 message。
  summary 插入 + message summarized=1 必须事务提交。

读取上下文：
  shortTerm/summaries 不依赖 embedding，可优先返回。
  RAG embedding 失败时 rag=[]，并记录 warning，不让 Agent 主链路崩。
```

向量检索策略：

```txt
第一版 SQLite 存 JSON embedding，应用层计算 cosine similarity。
原因：当前数据量小，避免引入向量数据库或 sqlite-vss 扩大复杂度。

必须做防御：
  embedding JSON 解析失败跳过该行并记录日志。
  向量长度不一致跳过。
  similarity 排序只返回 limit 内结果。

后续如果 memories 量大，再单独做向量索引增强，不放进 CORE-011。
```

清理策略：

```txt
memory.clear 必须按 isolationKey 范围执行。
type=message：删除该 isolationKey 的 message 和 summary。
type=summary：删除该 isolationKey 的 summary，并把该 isolationKey 下 summarized=1 的 message 重置为 0。
type=all：删除该 isolationKey 全部记忆。

不提供默认全库清空 service。
如果后续设置页要全库清空，必须独立 task + 二次确认 + 明确影响范围。
```

Skill 检索策略：

```txt
main skill：
  按 attribution 精确读取主 skill，不做 embedding 排序。

references skill：
  先按 skill_attributions 过滤可用范围。
  有 embedding 时按用户问题向量相似度排序。
  无 embedding 或 embedding 不可用时降级为 attribution 顺序返回。

读取文件：
  只允许 userData/skills 内相对路径。
  文件不存在、路径越界、frontmatter 缺字段都返回明确错误。
```

### 6.3 图片和文件附件边界

用户疑问：Agent 后续能不能发图片、文件、素材？

结论：

```txt
可以支持，但不能把图片/文件二进制直接塞进 CORE-011 记忆层。
CORE-011 只做文本 embedding 和记忆索引。
图片、视频、音频、文档本体归 CORE-004/CORE-012 文件和媒体服务管理。
Agent 消息附件能力归后续 F-011/F-008 页面和 Agent 任务。
```

专业边界：

```txt
1. memories.content 只存可检索文本：
   用户文本、assistant 文本、图片说明、OCR 文本、文件摘要、工具结果摘要。

2. 图片/文件只存引用，不存二进制：
   例如 fileId、mediaId、relativePath、mimeType、size、thumbnailUrl、source。

3. 图片本身不在 CORE-011 做 embedding：
   当前 all-MiniLM-L6-v2 是文本 embedding 模型，不是图文多模态模型。
   如果后续要图片语义检索，应另开图像 embedding / OCR / caption 任务。

4. Agent 能不能“看图”取决于后续模型层：
   如果供应商文本模型支持 image input，则由 CORE-007/Agent 服务传图片引用转 base64 或 URL。
   CORE-011 只负责把“这张图的描述/摘要/引用”放进记忆。

5. 记忆表建议预留 metadata：
   用 TEXT JSON 保存附件引用、来源、token 估算、摘要来源等扩展信息。
   这样后面支持图片/文件消息时不用立刻重做 memories 表。
```

建议落地：

```txt
memories 增加 metadata TEXT 可空字段。
第一版只写文本消息 metadata=null。
后续发图片/文件时：
  文件进入受控目录或媒体服务
  memory.add 写 content=图片说明/文件摘要
  metadata 写 attachment refs

不做：
  不在 CORE-011 做上传 UI。
  不在 CORE-011 做图片 OCR。
  不在 CORE-011 做图片 caption。
  不在 CORE-011 做多模态模型调用。
```

### 6.4 性能和体验设计

这个任务如果只追求“能跑”，后面体验会很差。VT Studio 必须按下面规则做。

启动性能：

```txt
1. 应用启动不初始化 ONNX extractor。
2. 数据库 seed 不调用 embedding，不因为模型缺失拖垮启动。
3. 第一次真正需要 embedText 时才 lazy init。
4. 模型缺失只让 embedding/RAG/Skill 语义排序失败，不影响登录、设置、项目打开。
```

交互体验：

```txt
1. Agent 主对话不能因为 RAG 失败整体不可用。
2. memory.getContext 即使 embedding 缺失，也要返回 shortTerm/summaries。
3. RAG 失败时记录日志，返回 rag=[] 和可读 warning，页面由后续 Agent 任务决定是否提示。
4. 清理记忆必须快速完成；大范围清理必须走事务。
```

计算性能：

```txt
1. embedding 初始化使用 loadingPromise，避免并发重复加载模型。
2. embedText 输入要限制最大长度，超长文本先截断或摘要，避免 ONNX 推理卡死。
3. RAG 默认只查当前 isolationKey，不跨项目扫描。
4. 相似度计算只对有 embedding 的行执行，坏 embedding 直接跳过。
5. 第一版保持 ragLimit 小值，避免一次拉太多向量进内存。
6. Skill embedding 重建 service 要支持单条失败继续统计，但不做页面进度 UI。
```

数据膨胀控制：

```txt
1. message 达到阈值后摘要压缩，减少长期上下文体积。
2. getHistory 不返回 embedding 和 metadata 中的本地绝对路径。
3. 后续如果 memories 增长明显，再单独做归档/压缩/向量索引增强。
4. 不在本任务引入向量数据库，避免过早复杂化。
```

体验验收要加：

```txt
1. 无 ONNX 模型时，应用能启动，登录能用，数据库能初始化。
2. 无 ONNX 模型时，Agent 仍可拿 shortTerm/summaries，RAG 降级为空。
3. memory.add 遇到 embedding 缺失时返回明确错误，不写半截脏数据。
4. Skill 检索无 embedding 时按 attribution 降级，不让 Agent 报硬错误。
5. 日志写清楚是哪一层降级，终端不刷大对象和绝对路径。
```

验证策略：

```txt
除了 typecheck/build，还要做 service 级验证：
1. migrations 后 memories 表和索引存在。
2. userData/models 不存在模型时，embedding.embedText 返回清晰错误，应用不崩。
3. memory.getContext 在 embedding 缺失时仍能返回 shortTerm/summaries，rag 为空。
4. memory.clear message/summary/all 范围正确，并验证事务结果。
5. skillRetrieval 路径越界被拒绝。
6. 如果本机已放 ONNX 模型，再验证 embedText 返回 number[]。
```

---

## 7. 偏差

| 偏差 | 原因 | 是否写入 04 |
|---|---|---|
| embedding 只走本地 ONNX，不走供应商接口 | 本地优先、离线、零成本、不泄露创作内容 | 已有 `D-BASE-014` |
| ONNX 模型不存在时不让应用启动崩溃 | 模型文件可能后置下载；底层能力可返回明确错误 | 不新增偏差，属于桌面端容错 |
| 不提供全库清空记忆的默认底层入口 | 防止误删所有项目记忆 | 不新增偏差，属于安全修正 |
| memories 字段改为 snake_case | VT Studio 统一数据库命名 | 不算业务偏差 |

---

## 8. 验收

```txt
1. typecheck 通过。
2. build 通过。
3. memories 表创建成功，索引存在。
4. modelOnnxFile/modelDtype 能从 app_settings 正确读取。
5. ONNX 模型存在时，embedText 能返回 number[]。
6. ONNX 模型不存在时，返回清晰错误，不崩溃。
7. memory.add 能写入 message 记忆和 embedding。
8. 达到 messagesPerSummary 后能生成 summary，并标记原 message summarized=1。
9. memory.getContext 能返回 shortTerm、summaries、rag。
10. deepRetrieve 无 summary 时返回空数组，不报错。
11. getHistory 只返回 message，不返回 embedding。
12. clear message 会同步清 summary。
13. clear summary 会把 summarized=1 的 message 重置为 0。
14. 不同 projectId/agentType/episodesId 的 isolationKey 不串数据。
15. Skill references embedding 可补生成，失败时不写半截数据。
16. Skill 文件读取有路径越界保护。
17. 不新增设置页面 UI，不接入 Agent 完整业务。
```

---

## 9. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-CORE-011-001 | embedding 是否固定使用本地 ONNX，不做供应商 embedding | 建议确认；已符合 `D-BASE-014`，也更适合本地创作工具 |
| C-CORE-011-002 | ONNX 模型不存在时是否返回错误但不阻断应用启动 | 建议确认；避免第一次启动因为模型文件缺失直接崩 |
| C-CORE-011-003 | 是否不提供默认“清空全部项目记忆”的底层入口 | 建议确认；全库清空风险高，后续设置页如果要做必须二次确认 |
| C-CORE-011-004 | Skill embedding 重建是否先做 service，不做进度 UI | 建议确认；进度 UI 属于设置页任务，当前只做底层 |
| C-CORE-011-005 | 是否允许新增本地 ONNX embedding 依赖和 runtime models/skills 目录 | 建议确认；当前项目没有相关依赖和目录，不补就只能做空壳 |
| C-CORE-011-006 | memories 是否预留 metadata 字段保存后续图片/文件引用，但本任务不做附件 UI/多模态 | 建议确认；这是低成本预留，能避免后续支持图片文件时重做表结构 |

---

## 10. 执行后记录

```txt
改了哪些文件：
  package.json
  pnpm-lock.yaml
  scripts/verify-core-011.mjs
  src/main/app/runtime.ts
  src/main/services/file-system/paths.ts
  src/main/services/file-system/directories.ts
  src/main/services/database/migrations.ts
  src/main/services/database/seed.ts
  src/main/services/memory/migrations.ts
  src/main/services/memory/index.ts
  src/main/services/embedding/index.ts
  src/main/services/skill-retrieval/index.ts
  src/shared/constants/status.ts
  src/shared/types/memory.ts

验证结果：
  1. node_modules\.bin\tsc.CMD --noEmit -p tsconfig.node.json 通过
  2. node_modules\.bin\vue-tsc.CMD --noEmit -p tsconfig.web.json 通过
  3. node scripts\verify-core-011.mjs 通过
     - 使用 Electron 环境验证 better-sqlite3 ABI
     - memories 表和 metadata 字段创建成功
     - runtime models/skills 目录创建成功
     - 无 ONNX 模型时 embedText 返回明确错误
     - memory.getContext 在无 ONNX 时 shortTerm/summaries 可返回，rag 降级为空
     - memory.clear summary 返回结构正确
     - skill.resolveForAgent 无 embedding 时按 attribution 降级
     - readSkillFile 路径越界被拦截
  4. node_modules\.bin\electron-vite.CMD build 通过

未完成事项：
  1. 未提供设置页记忆配置 UI，归 F-002-008。
  2. 未提供 Skill 管理页面 UI，归 F-002-007。
  3. 未接入剧本 Agent/生产 Agent 页面，归 F-011/F-008。
  4. 未自动下载 ONNX 模型；需要用户或后续安装任务把模型放到 userData/models/all-MiniLM-L6-v2/onnx/model_fp16.onnx。
  5. 本机当前未放 ONNX 模型，因此已验证缺模型降级路径；真实 embedText 返回 number[] 需模型文件存在后再验。

最终结论：
  CORE-011 底层已完成。本地 ONNX embedding、memories 迁移、记忆 service、Skill 检索 service、runtime models/skills 目录、缺模型降级、路径越界保护和 service 级验证均已落地。
```

---

## 11. 最后大白话

```txt
我这次准备怎么做：
1. 先让本地 ONNX 模型能把文本变成向量。
2. 建一个 memories 表，专门存 Agent 对话记忆、摘要和向量。
3. Agent 每说一轮话，就可以把内容写进记忆。
4. 记忆多了以后，自动压缩成摘要。
5. Agent 下次提问时，可以拿近期记忆、摘要和相似历史一起当上下文。
6. Skill 也可以补 embedding，后续 Agent 能按语义找更合适的 Skill。

我不会做什么：
1. 不做记忆设置页面。
2. 不做 Skill 管理页面。
3. 不做剧本 Agent 完整业务。
4. 不下载 ONNX 模型。
5. 不用云端 embedding。
6. 不清空项目、资产、脚本等业务数据。

确认规则：
用户确认本文档后才执行；未确认前只整理 task 文档，不写业务代码。
```
