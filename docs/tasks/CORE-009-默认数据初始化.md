# CORE-009 默认数据初始化

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`  
原则：先确认本文档，再改代码

---

## 0. 快速理解

```txt
一句话：把应用第一次启动时必须有的那批"出厂设置"写进数据库。
为什么现在做：CORE-007 已经把表建好了，但表是空的。
              现在 Agent 读不到模型配置、设置页全空、打开项目校验模型必失败。
              不做这步，CORE-008 Agent 事件流测不通。
做完后有什么用：应用启动就有默认供应商、默认 Agent(17个)、默认提示词(4个)、
              默认 Skill(21个)、默认设置项——用户不用手动配就能看到正常界面。
这一步不碰什么：不做设置页面 UI，不做 Agent 对话，不改任何已有表结构。
```

---

## 1. 本次做什么

```txt
目标：把参考项目 initDB.ts 里所有 initData 写的默认数据，
      按 VT Studio 已有表名/字段名映射过来，可重复执行地写入。

只做：
  1. 默认用户（1条）
  2. 默认 app_settings（10条 setting 配置项）
  3. 默认供应商（初始 8 条 model_vendors；F-002-003 后扩展为 11 条入口）
  4. 默认 Agent 配置（17条 agent_model_configs）
  5. 默认提示词（4条 prompts）
  6. 默认 Skill 主文件（7条 main skill，o_skillAttribution 14条关联）
  7. 默认 Skill references（14条，含 description，embedding 按策略处理）
  8. 默认 skillAttribution（14条关联）
  9. 写入逻辑必须幂等（已存在则跳过，不重复插入）

不做：
  设置页面 UI、供应商代码文件写入、ONNX 模型下载、Agent 对话、任何业务功能
```

---

## 2. 参考项目怎么做

源码已查完，以下是事实。

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-app/src/lib/initDB.ts` | 统一初始化所有表结构和默认数据；用 `forceInit=true` 可强制重建（deleteAllData 接口调用）；默认数据紧跟建表执行 |
| `knex.schema.hasTable` | 先查表是否存在，不存在才建表并写默认数据；forceInit=true 时先删表再建 |
| `o_skillList.initData` | **直接调用 `getEmbedding`** 为每条有 description 的 skill 生成向量并插入；main skill embedding 为空字符串 |
| `o_skillAttribution` | skill 和所属 Agent 文件的关联表，需和 skillList 同批写入 |

### 默认数据清单（全部来自源码，不脑补）

**用户（1条）**
```txt
id=1, name=admin, password=admin123
VT Studio 映射：users 表
```

**app_settings（10条）**
```txt
tokenKey       → uuid 前8位随机生成（每次初始化不同）
messagesPerSummary → 10
shortTermLimit     → 5
summaryMaxLength   → 500
summaryLimit       → 10
ragLimit           → 3
deepRetrieveSummaryLimit → 5
modelOnnxFile  → ["all-MiniLM-L6-v2","onnx","model_fp16.onnx"]（JSON 字符串）
modelDtype     → fp16
switchAiDevTool → 0
```

**供应商 model_vendors（CORE-009 初始 8 条，均 enabled=0）**
```txt
toonflow / deepseek / atlascloud / volcengine / minimax / openai / klingai / vidu
input_values="{}", models="[]", enabled=0
```

F-002-003 完成后，默认供应商入口扩展为：

```txt
toonflow / deepseek / anthropic / gemini / atlascloud / volcengine / minimax / openai / klingai / vidu / comfyui
```

**Agent 配置 agent_model_configs（17条）**

主 Agent（4条）：
```txt
key=scriptAgent   name=剧本Agent   disabled=false
key=productionAgent name=生产Agent disabled=false
key=universalAi   name=通用AI      disabled=false
key=ttsDubbing    name=TTS配音     disabled=true（默认关闭）
```

子 Agent（13条，temperature=1, maxOutputTokens=0, disabled=false）：
```txt
scriptAgent:decisionAgent       剧本Agent:决策层
scriptAgent:supervisionAgent    剧本Agent:监督层
scriptAgent:storySkeletonAgent  剧本Agent:故事骨架
scriptAgent:adaptationStrategyAgent 剧本Agent:改编策略
scriptAgent:scriptAgent         剧本Agent:剧本生成
productionAgent:decisionAgent   生产Agent:决策层
productionAgent:supervisionAgent 生产Agent:监督层
productionAgent:deriveAssetsAgent 生产Agent:衍生资产
productionAgent:generateAssetsAgent 生产Agent:生成资产
productionAgent:directorPlanAgent 生产Agent:导演规划
productionAgent:storyboardGenAgent 生产Agent:分镜生成
productionAgent:storyboardPanelAgent 生产Agent:分镜面板
productionAgent:storyboardTableAgent 生产Agent:分镜表格
```

**提示词 prompts（4条）**
```txt
name=事件提取         type=eventExtraction
name=剧本资产提取     type=scriptAssetExtraction
name=视频提示词生成   type=videoPromptGeneration
name=音色绑定         type=audioBindPrompt
```
（data 字段内容很长，直接从参考项目 initDB.ts 第 348-370 行抄录，不修改）

**Skill（21条 + 14条关联）**

7条 main skill（embedding 为空字符串，state=-1）：
```txt
production_agent_decision.md
production_agent_execution.md
production_agent_supervision.md
script_agent_decision.md
script_agent_execution.md
script_agent_supervision.md
universal_agent.md
```

14条 references skill（有 description，state 依 embedding 是否生成而定）：
```txt
references/event_extract.md
references/novel_character_extract.md
references/novel_props_extract.md
references/novel_scene_extract.md
references/video_dialogue_extract.md
references/pipeline.md
references/adaptation_format.md
references/event_format.md
references/script_format.md
references/skeleton_format.md
references/quality_criteria.md
references/plan.md
references/derive_assets_extraction.md
references/storyboard_generation.md
```

skillAttribution（14条 skill→agent 归属关联），详见参考项目 initDB.ts 第 948-1005 行，全部照映射规则写入 VT Studio 对应表。

---

## 3. 用户操作

```txt
入口：无页面入口，由迁移脚本或应用启动时自动执行。
按钮/操作：无。
弹窗/表单：无。
成功反馈：服务启动日志输出初始化完成。
失败反馈：启动时报错并阻止服务启动。
```

---

## 4. 要做什么功能

### 1. 幂等初始化入口

怎么做：
- 输入：无（迁移脚本或服务启动时调用）。
- 输出：各表默认数据已写入。
- 写什么数据：见第 2 节完整清单。
- 状态怎么变：仅写入，不修改已有数据。
- 异常怎么处理：任何一步失败抛错，服务启动中断，日志打出失败原因。
- 限制：写入前先检查是否已存在（按主键或唯一键），存在则跳过，不报错、不覆盖。

### 2. 默认用户写入

怎么做：
- 输入：无，硬编码 id=1, name=admin, password=admin123。
- 输出：users 表有一条 id=1 的默认用户。
- 写什么数据：users 表。
- 异常：已存在则跳过。
- 限制：密码不加密（对齐参考项目；CORE-010 登录降级方案可后续改）。

### 3. 默认设置项写入（app_settings）

怎么做：
- 输入：无，硬编码 10 个 key-value。
- 输出：app_settings 表写入 10 条。
- 特殊：tokenKey 的 value 是启动时随机生成的 uuid 前 8 位，**不能硬编码固定值**。
- 异常：已存在则跳过（不覆盖已有 tokenKey）。

### 4. 默认供应商写入（model_vendors）

怎么做：
- 输入：无，CORE-009 初始硬编码 8 条；F-002-003 后 seed 扩展到 11 条。
- 输出：model_vendors 表写入默认供应商，全部 enabled=0。
- 写什么数据：id/input_values="{}" /models="[]"/enabled=0。
- 异常：id 已存在则跳过。
- 限制：不写供应商代码文件（vendor TS 文件属于 M-002 设置功能的范畴）。

### 5. 默认 Agent 配置写入（agent_model_configs）

怎么做：
- 输入：无，硬编码 17 条。
- 输出：agent_model_configs 表写入 17 条。
- 写什么数据：key/name/description/model_id(空)/vendor_id(null)/temperature/max_output_tokens/disabled。
- 异常：key 已存在则跳过。
- 限制：model_id 和 vendor_id 均为空，等用户在设置页配置。

### 6. 默认提示词写入（prompts 表）

怎么做：
- 输入：无，硬编码 4 条，data 内容完整从参考项目抄录。
- 输出：prompts 表写入 4 条。
- 写什么数据：name/type/data/use_data。
- 异常：type 已存在则跳过。
- 限制：data 内容不删改，不翻译，和参考项目一致。

### 7. 默认 Skill 写入（skill_list 表 + skill_attributions 表）

怎么做：
- 输入：无，硬编码 21 条 skill + 14 条 attribution。
- 输出：skill_list 和 skill_attributions 写入完成。
- Skill embedding 策略（关键）：
  - main skill（7条）：description 为空，embedding 为空字符串，state=-1，直接插入，不调 embedding。
  - references skill（14条）：有 description，**尝试调 getEmbedding 生成向量**；
    - 如果 ONNX 模型文件已存在（userData/models/ 下已下载），则生成 embedding，state=1；
    - 如果模型文件不存在，则 embedding 为空字符串，state=-1，**不阻塞启动，只打 warn 日志**。
    - CORE-011 完成后，设置页"重新初始化 Skill"功能可补生成 embedding。
- 异常：id 已存在则跳过。
- 限制：skill MD5 和 ID 和参考项目保持一致（便于 skill 文件比对和升级）。

---

## 5. 数据和状态

VT Studio 表名映射（参考项目 → VT Studio 正式名）：

| 参考项目表 | VT Studio 表 | 说明 |
|---|---|---|
| `o_user` | `users` | 默认 admin/admin123 |
| `o_setting` | `app_settings` | key-value 设置项 |
| `o_vendorConfig` | `model_vendors` | 供应商，enabled=0 |
| `o_agentDeploy` | `agent_model_configs` | 17 个 Agent，model 为空 |
| `o_prompt` | `prompts` | 4 个默认提示词 |
| `o_skillList` | `skill_list` | 21 个 Skill |
| `o_skillAttribution` | `skill_attributions` | 14 条归属关联 |

任务状态：无任务队列，初始化不写 tasks 表。  
模型调用：Skill embedding 生成调 `getEmbedding`（本地 ONNX），失败不阻塞。  
删除影响：`other/deleteAllData` 对应的强制重建能力，CORE-009 的幂等写入复用同一套逻辑。

---

## 6. VT Studio 怎么落

```txt
能力名：database.initDefaultData（内部调用，不暴露给 renderer）
调用时机：服务启动时，在 CORE-003 SQLite 迁移之后执行
调用链：main 启动 -> migrations -> initDefaultData
```

建议新增或修改：
```txt
src/main/services/database/seed.ts          （新增，写入所有默认数据）
src/main/services/database/migrations.ts    （修改，启动时调 seed）
src/main/services/model/migrations.ts       （已有，确认 model_vendors/agent_model_configs 迁移已包含）
docs/tasks/CORE-009-默认数据初始化.md            （本文件）
docs/03-执行进度.md                         （完成后更新）
docs/04-对齐验收与偏差记录.md              （如有偏差更新）
```

---

## 7. 偏差

| 偏差 | 原因 | 是否写入 04 |
|---|---|---|
| 表名全部使用 VT Studio 正式命名，不用 o_ 前缀 | 遵守项目规范 | 不算业务偏差 |
| Skill embedding 生成改为"模型存在则生成，不存在则跳过"，参考项目是直接调（失败即报错） | 本地工具第一次启动不可能已有 ONNX 模型，必须容错；CORE-011 完成后可补全 | 不算业务偏差，记录在验收项 |
| tokenKey 仍随机生成，不硬编码 | 安全要求 | 不算业务偏差 |

---

## 8. 验收

```txt
1. typecheck 通过。
2. build 通过。
3. 首次启动后，users 表有 id=1 admin 用户。
4. app_settings 有 10 条，tokenKey 非空，modelOnnxFile 可被 JSON.parse 还原为数组。
5. model_vendors 有默认供应商记录，全部 enabled=0；F-002-003 后应包含 11 个默认入口。
6. agent_model_configs 有 17 条，key 值和参考项目一致，主 Agent disabled 均为 false，ttsDubbing disabled=true。
7. prompts 有 4 条，type 值和参考项目一致，data 不为空。
8. skill_list 有 21 条；main skill embedding 为空；references skill 若 ONNX 模型存在则 state=1，否则 state=-1 且启动不报错。
9. skill_attributions 有 14 条。
10. 重复启动（幂等验证）：二次启动不重复插入，数量保持不变。
11. 没有把参考项目旧表名/字段名用作 VT Studio 正式命名。
```

---

## 9. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-CORE-009-001 | 默认用户密码 admin123 是否保留明文存储 | 建议先保留（对齐参考项目，CORE-010 登录降级后密码校验简化）；若需要 hash 后续可单独改 |
| C-CORE-009-002 | Skill embedding 生成策略：ONNX 模型不存在时跳过（state=-1）而非报错 | 建议此方案；CORE-011 完成后设置页提供"重新生成 Skill 向量"入口补全 |
| C-CORE-009-003 | 是否现在就把提示词 data 内容完整写入（内容较长，约 8000 字） | 建议写入，这些是 Agent 运行的核心提示词，缺了 Agent 功能直接受损 |

---

## 10. 执行后记录

```txt
改了哪些文件：（完成后填写）
验证结果：（完成后填写）
未完成事项：（完成后填写）
最终结论：（完成后填写）
```

---

## 11. 最后大白话

```txt
我这次准备怎么做：
1. 新建一个 seed.ts，把参考项目 initDB.ts 里所有 initData 的默认数据，
   按 VT Studio 的表名和字段名翻译一遍，幂等地写进去。
2. 供应商、Agent、提示词、setting 直接写，不调任何外部服务。
3. Skill 里的 embedding 用"模型存在就生成，不存在就跳过"的策略，
   不让 ONNX 模型文件不存在这件事阻断应用启动。
4. 在 migrations 启动流程里调这个 seed，开发和生产都自动执行。

我不会做什么：
1. 不做设置页面。
2. 不写供应商的 TS 代码文件（那是 M-002 的活）。
3. 不下载 ONNX 模型文件（那是 CORE-011 的活）。
4. 不改任何已有的表结构。
5. 不修改用户已经改过的设置（幂等：存在就跳过）。

确认规则：
用户确认本文档后才执行；未确认前只改文档，不碰代码。
```
