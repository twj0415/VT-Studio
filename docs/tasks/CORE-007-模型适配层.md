# CORE-007 模型适配层

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`、`docs/features/M-002-设置.md`  
原则：已确认并完成代码实现

## 0. 快速理解

```txt
一句话：这一步做 main 侧统一模型服务，让文本、图片、视频、TTS、Agent 都从同一个入口调用模型。
为什么现在做：后面的设置、项目、剧本 Agent、资产生成、视频生成都会用模型；如果这里不先定好，后面一定会到处重复封装。
做完后有什么用：业务只传模型 key 或 vendorId:modelName，就能解析供应商、找到模型、调用文本/图片/视频/TTS，并拿到统一错误。
这一步不碰什么：不做设置页面 UI，不做 Agent 对话流，不做业务生成页面，不做 ComfyUI 专用页面。
```

## 1. 本次做什么

```txt
目标：实现 main/services/model 的底层模型适配能力。
只做：模型类型、供应商协议、供应商代码加载、模型列表、模型详情、Agent 模型解析、文本/图片/视频/TTS 调用入口、模型测试入口的 service 方案。
不做：供应商设置页面、模型映射页面、提示词管理页面、Agent Socket/IPC 事件流、业务任务生成流程、项目模型配置 UI。
```

## 2. 参考项目怎么做

参考源码已经查完，结论如下。

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-app/src/utils/ai.ts` | 统一入口 `u.Ai.Text/Image/Video/Audio`；支持 Agent key 和 `vendorId:modelName`；文本走 `ai` SDK；图片/视频/音频走供应商函数；可包 `taskRecord` |
| `Toonflow-app/src/utils/vendor.ts` | 从 vendor 目录读 `${id}.ts`；用 `sucrase` 转 TS；从 `o_vendorConfig.models` 合并手动模型；按 `modelName` 去重 |
| `Toonflow-app/src/utils/vm.ts` | 用 `vm2` 跑供应商代码；沙盒暴露 `axios/FormData/jsonwebtoken/fetch/pollTask/zipImage/zipImageResolution/mergeImages/urlToBase64`，以及 `createOpenAI/createDeepSeek/createZhipu/createQwen/createAnthropic/createOpenAICompatible/createXai/createMinimax/createGoogleGenerativeAI` |
| `routes/modelSelect/getModelList.ts` | 只查启用的供应商；按 `text/image/video/all` 过滤；返回供应商分组需要的数据 |
| `routes/modelSelect/getModelDetail.ts` | 入参 `modelId=vendorId:modelName`，拆分后从供应商模型列表取详情 |
| `routes/project/getModelDetails.ts` | 根据 `scriptAgent/productionAgent` 找 Agent 当前模型，再取模型详情 |
| `routes/setting/vendorConfig/getVendorList.ts` | 返回供应商列表、动态 inputs、inputValues、合并后的 models、vendor code、说明、作者、版本 |
| `routes/setting/vendorConfig/addVendor.ts` | 添加供应商时校验 TS 代码必须导出 `vendor/textRequest/imageRequest/videoRequest`，并校验 vendor schema |
| `routes/setting/vendorConfig/updateCode.ts` | 更新供应商代码；重新校验导出和 schema；更新基础模型列表并写入代码文件 |
| `routes/setting/vendorConfig/updateVendorInputs.ts` | 保存供应商动态参数，例如 apiKey/baseUrl |
| `routes/setting/vendorConfig/enableVendor.ts` | 启用/禁用供应商 |
| `routes/setting/vendorConfig/addVendorModel.ts` | 手动添加模型到 `o_vendorConfig.models` |
| `routes/setting/vendorConfig/delVendorModel.ts` | 只能删除手动添加的模型；基础模型不允许删除 |
| `routes/setting/vendorConfig/deleteVendor.ts` | 删除供应商，删除 vendor TS 文件，并清空相关 Agent 的 vendorId/model |
| `routes/setting/vendorConfig/modelTest/textTest.ts` | 文本模型测试：调用 `u.Ai.Text(vendorId:modelName).invoke`，返回 thinking/content |
| `routes/setting/vendorConfig/modelTest/imageTest.ts` | 图片模型测试：prompt + 可选图片参考，固定 `size=1K`、`aspectRatio=16:9`，保存测试图片 |
| `routes/setting/vendorConfig/modelTest/videoTest.ts` | 视频模型测试：按模型第一个 duration/resolution，带 prompt 和参考图/视频/音频，保存测试视频 |
| `routes/setting/agentDeploy/*` | Agent 模型配置支持简易/高级模式；配置字段有 `model/modelName/vendorId/temperature/maxOutputTokens` |
| `Toonflow-web/src/components/modelSelect.vue` | 模型选择按供应商分组；选中值是 `vendorId:modelName`；无模型时跳转设置里的模型服务 |
| `Toonflow-web/src/components/setting/components/vendorConfig.vue` | 供应商设置页包含添加供应商、启用、动态参数、添加/编辑/删除模型、测试模型、编辑代码、文件/链接/代码导入 |

## 3. 用户操作

本任务是底层 service，不直接做页面。

```txt
入口：无页面入口。
按钮/操作：无页面按钮。
弹窗/表单：无页面弹窗。
成功反馈：service 返回模型列表、模型详情、测试结果或生成结果。
失败反馈：service 抛 VtError，后续 IPC 统一返回 { code: 400, data: {}, msg }。
```

说明：

```txt
设置页里的按钮和弹窗已经在参考项目查清楚，但它们属于后续 M-002 设置任务。
CORE-007 只把这些页面后面要调用的 main 侧能力先打好。
```

## 4. 要做什么功能

### 1. 定义模型类型和供应商协议

怎么做：
- 输入：无。
- 输出：TypeScript 类型。
- 写什么数据：不写数据。
- 状态怎么变：无。
- 异常怎么处理：无运行异常。
- 限制：只定义协议，不做 UI。

必须覆盖：

```txt
TextModel：name/modelName/type=text/think
ImageModel：name/modelName/type=image/mode
VideoModel：name/modelName/type=video/mode/audio/durationResolutionMap
TTSModel：name/modelName/type=tts/voices
VendorInput：key/label/type(text,password,url)/required/placeholder
VendorManifest：id/author/description/name/icon/inputs/inputValues/models/version
ModelId：vendorId:modelName
```

### 2. 读取供应商代码

怎么做：
- 输入：`vendorId`。
- 输出：供应商 TS 代码字符串。
- 读什么数据：从 userData 受控 vendor 目录读取 `${vendorId}.ts`。
- 状态怎么变：无。
- 异常怎么处理：文件不存在抛 `VtError(MODEL_VENDOR_NOT_FOUND)`。
- 限制：不能从 renderer 读文件；不能读项目根目录里的临时代码。

VT Studio 落点：

```txt
vendor 代码目录放在 userData/vendors。
不放 D:\project\vt-studio\data/vendor。
文件路径必须走现有 file-system 安全边界。
```

### 3. 编译和运行供应商代码

怎么做：
- 输入：供应商 TS 代码。
- 输出：`vendor/textRequest/imageRequest/videoRequest/ttsRequest`。
- 写什么数据：不写数据。
- 状态怎么变：无。
- 异常怎么处理：编译失败、导出缺失、schema 不合法都抛 `VtError(MODEL_VENDOR_INVALID)`。
- 限制：供应商代码只能在 main 侧受控执行，renderer 不能执行。

专业建议：

```txt
Toonflow 用 sucrase + vm2 跑动态 TS。
VT Studio 可以保留动态供应商能力，但 CORE-007 先把运行器封装成 model/vendor-runner。
后续如果动态代码安全风险太高，可以把内置供应商改成静态 adapter，第三方供应商再单独确认沙盒策略。
```

### 4. 合并模型列表

怎么做：
- 输入：`vendorId`。
- 输出：合并后的模型数组。
- 读什么数据：供应商代码里的 `vendor.models` + 数据库里该供应商手动添加的 `models`。
- 状态怎么变：无。
- 异常怎么处理：供应商不存在、代码无效、models 不是数组，抛模型错误。
- 限制：按 `modelName` 去重，手动模型覆盖同名基础模型。

参考项目逻辑：

```txt
const combined = [...vendor.vendor.models, ...JSON.parse(db.models)]
Map 使用 modelName 去重
```

### 5. 获取启用供应商的模型选择列表

怎么做：
- 输入：`type=text/image/video/all`。
- 输出：供应商分组模型列表。
- 读什么数据：只读 `enabled=1` 的供应商。
- 状态怎么变：无。
- 异常怎么处理：没有启用供应商时返回空数组，不建议抛 404。
- 限制：保持参考项目语义，`all` 不包含 video。

输出结构建议：

```txt
[
  {
    vendorId,
    vendorName,
    label: model.name,
    value: model.modelName,
    modelId: "vendorId:modelName",
    type
  }
]
```

### 6. 获取模型详情

怎么做：
- 输入：`modelId=vendorId:modelName`。
- 输出：单个模型完整配置。
- 读什么数据：拆分 `vendorId/modelName` 后从合并模型列表查。
- 状态怎么变：无。
- 异常怎么处理：格式错误抛 `INVALID_PARAMS`；找不到抛 `MODEL_NOT_FOUND`。
- 限制：不从页面传整份模型对象，必须按 `modelId` 回查。

### 7. 解析 Agent 模型

怎么做：
- 输入：Agent key，例如 `scriptAgent`、`scriptAgent:decisionAgent`、`productionAgent:storyboardGenAgent`，或直接传 `vendorId:modelName`。
- 输出：最终 `vendorId:modelName` 和 Agent 配置。
- 读什么数据：`agentUseMode` 设置、Agent 部署配置表。
- 状态怎么变：无。
- 异常怎么处理：未配置模型抛 `MODEL_NOT_CONFIGURED`。
- 限制：不能在业务代码里自己拼 fallback。

参考项目规则：

```txt
agentUseMode=1：高级配置，按完整 key 查 Agent 配置。
agentUseMode=0：简易配置，子 Agent 退回主 Agent，例如 scriptAgent:decisionAgent -> scriptAgent。
agentUseMode 缺失：先查完整 key，没配再回退主 Agent。
```

### 8. 调用文本模型

怎么做：
- 输入：`modelKey`、`messages/prompt`、`tools?`、`think?`、`thinkLevel?`。
- 输出：`text`、`reasoningText/thinking`、原始 usage 可选。
- 读什么数据：供应商配置、Agent 配置、动态 inputValues。
- 状态怎么变：无。
- 异常怎么处理：供应商未启用、缺 apiKey、模型未找到、调用失败都转 VtError。
- 限制：不在 CORE-007 做 Agent 流式 UI，但 service 要支持 invoke 和 stream 两种能力。

参考项目细节：

```txt
文本模型由供应商 textRequest 返回 ai-sdk model。
generateText/streamText 注入 model。
如果有 tools，stopWhen = stepCountIs(Object.keys(tools).length * 50)。
Agent 配置里的 temperature/maxOutputTokens 会传给 generateText/streamText。
stream 会加 extractReasoningMiddleware，读取 reasoning_content。
switchAiDevTool=1 时会加 devToolsMiddleware。
```

### 9. 调用图片模型

怎么做：
- 输入：`modelKey`、`prompt`、`referenceList?`、`size`、`aspectRatio`、`task?`。
- 输出：base64 或保存后的文件路径由调用方决定。
- 读什么数据：供应商配置、模型详情。
- 状态怎么变：如传 task，创建/成功/失败任务状态。
- 异常怎么处理：调用失败写任务失败原因，并抛 VtError。
- 限制：本层只返回结果，不决定业务图片放在哪个资产目录。

参考项目细节：

```txt
ImageConfig.prompt 必填。
referenceList 只支持 image。
size 使用 "1K" | "2K" | "4K"。
aspectRatio 使用 "16:9" 这类字符串。
供应商返回 http 时会下载转 base64。
vendor version < 2.0 时，把 referenceList 转旧字段 imageBase64。
```

### 10. 调用视频模型

怎么做：
- 输入：`modelKey`、`prompt`、`duration`、`resolution`、`aspectRatio`、`referenceList?`、`audio?`、`mode`、`task?`。
- 输出：base64 或保存后的文件路径由调用方决定。
- 读什么数据：供应商配置、模型详情。
- 状态怎么变：如传 task，创建/成功/失败任务状态。
- 异常怎么处理：失败写任务失败原因。
- 限制：不在本层生成分镜、不决定时间线。

必须支持的 mode：

```txt
singleImage
startEndRequired
endFrameOptional
startFrameOptional
text
videoReference:n
imageReference:n
audioReference:n
```

参考项目细节：

```txt
VideoModel.audio 支持 true/false/"optional"。
durationResolutionMap 是 [{ duration: number[], resolution: string[] }]。
视频测试默认拿第一组 duration/resolution 的第一个值。
referenceList 可以混合 image/video/audio。
供应商返回 http 时会下载转 base64。
```

### 11. 调用音频/TTS 模型

怎么做：
- 输入：`modelKey`、TTS 参数。
- 输出：base64 或保存后的文件路径由调用方决定。
- 读什么数据：供应商配置、模型详情。
- 状态怎么变：如传 task，创建/成功/失败任务状态。
- 异常怎么处理：失败写任务失败原因。
- 限制：参考项目页面未完整开放 TTS 模型编辑，本层只保留协议和调用入口。

### 12. 模型测试能力

怎么做：
- 输入：模型类型、vendorId、modelName、测试参数。
- 输出：文本内容、图片预览路径、视频预览路径。
- 写什么数据：测试文件写到 userData/cache 或 temp，不写项目目录。
- 状态怎么变：普通测试不写任务队列。
- 异常怎么处理：失败返回模型调用失败原因。
- 限制：这是 service 能力，设置页面后续再接 UI。

参考项目测试参数：

```txt
text：messages，返回 thinking/content。
image：prompt + 可选 imageBase64，固定 size=1K、aspectRatio=16:9。
video：mode、prompt、images/videos/audios，默认取模型第一组 duration/resolution。
```

### 13. 供应商动态参数注入

怎么做：
- 输入：运行供应商函数前读取 `inputValues`。
- 输出：供应商运行时对象里已填入 apiKey/baseUrl 等参数。
- 读什么数据：供应商配置表。
- 状态怎么变：无。
- 异常怎么处理：required 参数为空时抛 `MODEL_API_KEY_MISSING` 或 `MODEL_VENDOR_INPUT_MISSING`。
- 限制：密钥不返回 renderer；日志不打印密钥。

参考项目逻辑：

```txt
Object.assign(running.vendor.inputValues, JSON.parse(vendorConfigData.inputValues ?? "{}"))
running.vendor.models = modelList
```

### 14. ComfyUI 和不同协议扩展

怎么做：
- 输入：供应商 adapter 自己定义 inputValues 和模型 mode。
- 输出：统一的 text/image/video/tts 调用结果；ttsRequest 返回的是 audio base64。
- 读什么数据：同供应商配置。
- 状态怎么变：同任务队列。
- 异常怎么处理：同模型调用失败。
- 限制：CORE-007 不硬编码 ComfyUI 专用逻辑。

专业建议：

```txt
ComfyUI、OpenAI Compatible、火山、可灵、Vidu、RunningHub 这类差异，都应该通过供应商 adapter 处理。
核心层只认统一协议：textRequest/imageRequest/videoRequest/ttsRequest。
这样后续要换协议，不需要改业务页面。
```

## 5. 数据和状态

本任务建议先建底层所需表或 schema，后续设置页面复用。

```txt
供应商表：model_vendors
字段：id/name/author/description/icon/version/code_path/input_values/models/enabled/created_at/updated_at

Agent 配置表：agent_model_configs
字段：id/key/name/description/model_label/model_id/vendor_id/temperature/max_output_tokens/disabled/created_at/updated_at

设置项：agent_use_mode、switch_ai_dev_tool
```

字段映射：

| 业务含义 | Toonflow 源码名 | VT Studio 建议名 |
|---|---|---|
| 供应商配置 | `o_vendorConfig` | `model_vendors` |
| 动态参数 | `inputValues` | `input_values` |
| 手动模型 | `models` | `models` |
| 启用状态 | `enable` | `enabled` |
| Agent 配置 | `o_agentDeploy` | `agent_model_configs` |
| Agent key | `key` | `key` |
| 展示模型名 | `model` | `model_label` |
| 实际模型 ID | `modelName` | `model_id` |
| 供应商 ID | `vendorId` | `vendor_id` |
| 温度 | `temperature` | `temperature` |
| 最大输出 | `maxOutputTokens` | `max_output_tokens` |

任务状态：

```txt
模型测试：不进任务队列。
业务生成：调用方传 task 信息时，模型服务负责成功/失败写任务。
取消：长轮询/视频生成后续要定取消检查点，本任务只保留能力入口。
```

## 6. VT Studio 怎么落

能力名：

```txt
model.getEnabledModelList
model.getModelDetail
model.resolveModelKey
model.invokeText
model.streamText
model.generateImage
model.generateVideo
model.generateAudio
model.testText
model.testImage
model.testVideo
model.loadVendor
model.validateVendor
model.getVendorModelList
```

调用链：

```txt
业务 service -> main/services/model
后续页面 -> window.vtStudio.model.* -> main/ipc -> main/services/model
```

建议新增：

```txt
src/main/services/model/types.ts
src/main/services/model/constants.ts
src/main/services/model/migrations.ts
src/main/services/model/vendor-schema.ts
src/main/services/model/vendor-storage.ts
src/main/services/model/vendor-runner.ts
src/main/services/model/model-resolver.ts
src/main/services/model/text.ts
src/main/services/model/media.ts
src/main/services/model/test.ts
src/main/services/model/index.ts
```

建议修改：

```txt
src/main/services/database/migrations.ts
src/shared/constants/status.ts
docs/tasks/CORE-007-模型适配层.md
docs/03-执行进度.md
docs/04-对齐验收与偏差记录.md（如采用动态供应商沙盒）
```

依赖建议：

```txt
如果保留参考项目动态供应商代码能力，需要安装：
sucrase
vm2 或更安全的替代沙盒
ai
@ai-sdk/openai
@ai-sdk/deepseek
@ai-sdk/anthropic
@ai-sdk/openai-compatible

图片压缩、拼图等能力可以后续需要时再加 sharp。
```

## 7. 偏差

| 偏差 | 原因 | 是否写入 04 |
|---|---|---|
| 表名不使用 `o_vendorConfig/o_agentDeploy` | 项目规则要求不保留旧命名 | 不算业务偏差 |
| CORE-007 不直接做供应商设置页面 | 先做底层，页面属于 M-002 设置任务 | 不算业务偏差 |
| ComfyUI 不硬编码到核心层 | 不同协议应通过 adapter 扩展，避免核心层被供应商绑死 | 需要 |
| 动态供应商代码如果继续保留，需要沙盒限制和风险提示 | 参考项目支持编辑/导入供应商代码，但这是高风险能力 | 需要 |

## 8. 验收

```txt
1. typecheck 通过。
2. build 通过。
3. 能保存/读取供应商代码到 userData/vendors。
4. 能校验 vendor/textRequest/imageRequest/videoRequest 导出。
5. 能合并基础模型和手动模型，并按 modelName 去重。
6. 能只列出启用供应商的 text/image/video/all 模型。
7. modelId=vendorId:modelName 能查到模型详情。
8. Agent key 能按 agentUseMode 解析到最终 modelId。
9. 文本调用能使用 temperature/maxOutputTokens。
10. 图片调用支持 prompt/referenceList/size/aspectRatio。
11. 视频调用支持 duration/resolution/aspectRatio/referenceList/audio/mode。
12. 测试文件不写项目目录，只写 cache/temp。
13. renderer 仍不能直接调用 SDK、执行供应商代码、读取 apiKey。
14. 失败时返回明确原因，不只返回“失败”。
```

## 9. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-CORE-007-001 | 是否保留 Toonflow 的动态供应商 TS 代码能力 | 已按建议执行；只在 main 侧 vm2 沙盒执行，已写入 04 |
| C-CORE-007-002 | ComfyUI 是否作为供应商 adapter，而不是核心层硬编码 | 已按建议执行；核心层只认统一 adapter 协议，已写入 04 |
| C-CORE-007-003 | CORE-007 是否只做 service，不做设置页面 UI | 已按建议执行；设置页面后续按 M-002 单独做 |
| C-CORE-007-004 | 模型测试是否不写任务队列 | 已按建议执行；测试是即时操作，业务生成才写任务 |
| C-CORE-007-005 | 供应商代码目录是否放 userData/vendors | 已按建议执行；不放项目目录 |
| C-CORE-007-006 | 是否现在安装模型层依赖 | 已按建议执行；使用参考项目同代 `ai@6` 和 `@ai-sdk/*`，兼容 Node 20 |

## 10. 执行后记录

```txt
已新增：
src/main/services/model/constants.ts
src/main/services/model/types.ts
src/main/services/model/migrations.ts
src/main/services/model/validation.ts
src/main/services/model/storage.ts
src/main/services/model/vendor-runner.ts
src/main/services/model/vendor-service.ts
src/main/services/model/resolver.ts
src/main/services/model/text.ts
src/main/services/model/media.ts
src/main/services/model/test.ts
src/main/services/model/index.ts

已修改：
src/main/services/file-system/paths.ts
src/main/services/file-system/directories.ts
src/main/services/database/migrations.ts
src/shared/constants/status.ts
package.json
pnpm-lock.yaml
docs/03-执行进度.md
docs/04-对齐验收与偏差记录.md

实际完成：
1. 新增 userData/vendors 供应商代码目录，不写项目根目录。
2. 新增 model_vendors、agent_model_configs、app_settings 三张底层表。
3. 新增供应商协议校验，覆盖 vendor、inputs、text/image/video/tts 模型 schema。
4. 新增 sucrase + vm2 动态供应商运行器。
5. 新增供应商代码添加、更新、读取、参数更新、启用禁用、模型列表合并。
6. 新增启用模型列表、模型详情、Agent key 解析。
7. 新增文本 invoke/stream，支持 Agent temperature/maxOutputTokens 和 reasoning middleware。
8. 新增图片、视频、音频生成入口，支持 task 记录成功/失败。
9. 新增文本、图片、视频测试 service，测试文件写入 userData/cache/model-test。
10. 新增模型相关状态码：供应商不存在、供应商无效、模型不存在、供应商参数缺失。

回头审查修正：
1. 参考项目供应商语音模型类型是 type=tts，不是 type=audio；已修正。
2. ReferenceList 每项必须带 sourceType="base64"；已修正。
3. vendor 脚本必须导出 ttsRequest；已补强校验。
4. TTSModel 必须有 voices: { title, voice }[]；已补 schema。
5. 供应商沙盒必须补齐 axios/FormData/jsonwebtoken/zipImage/zipImageResolution/mergeImages/createZhipu/createQwen/createXai/createMinimax。

依赖处理：
1. 初次安装的 ai@7 要求 Node >=22，不适合当前 Node 20.19 开发环境。
2. 已降到参考项目同代：ai@6、@ai-sdk/openai@3、@ai-sdk/deepseek@2、@ai-sdk/anthropic@3、@ai-sdk/openai-compatible@2、@ai-sdk/google@3。
3. 新增 sucrase、vm2、@ai-sdk/devtools。

验证：
1. D:\software\nodejs\pnpm.cmd run typecheck 通过。
2. D:\software\nodejs\pnpm.cmd run build 通过。

未做：
1. 未做供应商设置页面 UI。
2. 未做模型映射页面。
3. 未内置默认 Toonflow 供应商数据。
4. 未做 Agent 对话流。
5. 未做业务生成流程。
6. 未硬编码 ComfyUI；后续走供应商 adapter。
```

## 11. 最后大白话

```txt
我这次准备怎么做：
1. 做一个统一模型服务，所有模型调用都从 main/services/model 走。
2. 先把供应商、模型、Agent key、模型测试、图片视频音频调用这些底层能力定住。
3. 供应商可以像参考项目一样用 TS adapter，但必须在 main 侧受控执行，不能让页面碰 SDK 和密钥。
4. ComfyUI 这类协议不写死在核心里，后续通过供应商 adapter 接进去。

我不会做什么：
1. 不做设置页面。
2. 不做 Agent 对话窗口。
3. 不做业务生成流程。
4. 不把 apiKey 返回给页面。
5. 不把测试文件写到项目目录。

确认规则：
用户确认后才执行；未确认前只改文档。
```
