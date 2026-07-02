# F-011-012 Agent XML 写入工作区

状态：等待用户确认  
所属菜单：M-011 剧本 Agent  
对应功能文档：`docs/features/M-011-剧本Agent.md`  
原则：先确认本文档，再改代码

## 0. 快速理解

```txt
一句话：Agent 输出 XML 标签时，把故事骨架、改编策略、剧本条目写到右侧工作区。
为什么现在做：Agent 不能只在聊天里说结果，必须把结构化产物落到可编辑数据里。
做完后有什么用：流式生成过程中右侧内容会同步更新，完成后保存。
这一步不碰什么：不定义全部子 Agent，不直接信任非法 XML，不按 name 覆盖剧本。
```

## 1. 本次做什么

```txt
目标：
  对齐 Toonflow useChat XML 配置：storySkeleton、adaptationStrategy、scriptItem。

只做：
  1. 识别 storySkeleton XML
  2. 识别 adaptationStrategy XML
  3. 识别 scriptItem XML
  4. 流式更新右侧工作区
  5. XML 完成后持久化
  6. 非法 XML 拦截

不做：
  1. 子 Agent 调度
  2. 图片/视频生成
  3. 生产工作台 flowData
```

## 2. 参考项目怎么做

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-web/src/utils/useChat.ts` | XML 标签 `storySkeleton/adaptationStrategy/scriptItem` 不保留在消息内，更新 planData |
| `Toonflow-web/src/stores/scriptAgent.ts` | XML complete 时调用 setPlanData |
| `Toonflow-app/src/routes/scriptAgent/setPlanData.ts` | 保存 agentWorkData，并尝试同步 o_script |

参考项目关键事实：

```txt
1. XML 标签 keepInMessage=false。
2. `<storySkeleton>` 更新 planData.storySkeleton。
3. `<adaptationStrategy>` 更新 planData.adaptationStrategy。
4. `<scriptItem name="...">` 按 name 查找已有剧本，存在则更新 content，不存在则 push。
5. VT Studio 不能只按 name 覆盖，必须用稳定 id 或生成唯一分集 key。
```

## 3. 用户操作

```txt
入口：
  用户发送消息后 Agent 流式输出。

按钮/操作：
  无直接按钮。

弹窗/表单：
  无。

成功反馈：
  右侧故事骨架、改编策略、剧本卡片实时更新，完成后保存。

失败反馈：
  XML 不合法时显示 Agent 输出格式错误，不写入业务数据。
```

## 4. 要做什么功能

### 1. 解析 storySkeleton XML

怎么做：
- 输入：Agent 流式内容。
- 输出：storySkeleton 文本。
- 写什么数据：XML complete 后保存 agent_work_data。
- 状态怎么变：右侧预览流式更新。
- 异常怎么处理：标签不闭合或内容为空时不保存。
- 限制：不把 XML 原文展示在聊天正文里。

### 2. 解析 adaptationStrategy XML

怎么做：
- 输入：Agent 流式内容。
- 输出：adaptationStrategy 文本。
- 写什么数据：XML complete 后保存 agent_work_data。
- 状态怎么变：右侧预览流式更新。
- 异常怎么处理：格式错误不保存。
- 限制：只更新改编策略字段。

### 3. 解析 scriptItem XML

怎么做：
- 输入：`scriptItem` 的 name 和 content。
- 输出：剧本条目。
- 写什么数据：写 scripts 表，并同步 agent_work_data。
- 状态怎么变：右侧剧本卡片新增或更新。
- 异常怎么处理：name 为空、content 为空、XML 非法时拒绝写入。
- 限制：不能只按 name 覆盖已有剧本。

### 4. 完成后持久化

怎么做：
- 输入：XML complete 事件。
- 输出：保存结果。
- 写什么数据：agent_work_data 和 scripts 表。
- 状态怎么变：saving -> saved。
- 异常怎么处理：保存失败时右侧标记未保存，提示用户。
- 限制：写入必须走服务层事务。

## 5. 数据和状态

```txt
字段：
  agent_work_data.data.storySkeleton
  agent_work_data.data.adaptationStrategy
  scripts.id
  scripts.name
  scripts.content

接口/能力：
  agent.script.applyXmlPatch
  agent.script.setPlanData

数据读写：
  写 agent_work_data
  写 scripts

任务状态：
  如果 Agent 生成写 tasks，本任务只同步结构化结果

轮询/Socket：
  流式事件来自 agent.script 会话

模型调用：
  无直接调用，消费 Agent 输出

删除影响：
  无
```

## 6. VT Studio 怎么落

```txt
能力名：
  agent.script.applyXmlPatch

调用链：
  agent stream parser -> main/services/agent/scriptPlan -> database

需要新增：
  XML 流式解析器
  scriptItem 写入策略
  planData 保存事务

需要修改：
  useChat 等价逻辑，不能把 XML 写进普通消息正文
```

## 7. 偏差

```txt
和 Toonflow 不同的地方：
  scriptItem 不按 name 覆盖，使用稳定 id 或生成唯一 key。

原因：
  同名剧本会误覆盖。

是否写入 04：
  写入，属于修正源码风险。
```

## 8. 验收

```txt
1. storySkeleton XML 能写入故事骨架。
2. adaptationStrategy XML 能写入改编策略。
3. scriptItem XML 能新增或更新剧本。
4. XML 不显示在普通聊天正文里。
5. 非法 XML 不写业务数据。
6. 保存失败有提示。
7. 不按 name 误覆盖剧本。
8. 页面不直接访问 SQLite。
```

## 9. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-F-011-012-001 | scriptItem 的唯一标识怎么来 | 建议 Agent 可带 episodeKey/id；没有时服务端生成临时唯一 key |

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
1. Agent 输出特定 XML 时，右侧工作区同步更新。
2. 骨架写骨架，策略写策略，剧本写剧本表。
3. XML 格式不对就不写，避免污染数据。

我不会做什么：
1. 不按名称乱覆盖剧本。
2. 不把 XML 当普通聊天内容显示。

确认规则：
用户确认后才执行；未确认前只改文档。
```
