# CORE-005 Result 和错误

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`  
执行规则：已按用户确认执行；后续功能仍需先写 tasks 文档并确认

## 0. 快速理解

```txt
一句话：这一步是把“成功怎么返回、失败怎么返回、错误怎么记录”统一起来。
为什么现在做：后面业务越来越多，如果每个服务自己 throw、自己拼 msg，页面和任务中心会越来越乱。
做完后有什么用：后续服务只抛统一错误，IPC 统一转成 { code, data, msg }，日志保留 detail。
这一步不碰什么：不做业务页面、不做业务表、不做任务队列、不改模型调用逻辑。
```

## 1. 这一步只做什么

```txt
本任务只做 Result 和错误基础层。
它不是业务功能，也不是重写所有服务。
```

本任务要解决的问题：

```txt
1. 服务层现在没有统一错误对象。
2. IPC 层有 VtIpcError，但名字偏 IPC，不适合 services 直接依赖。
3. 失败返回只有 code/msg，没有统一 detail 记录规则。
4. 文件、数据库、模型、任务后续都会抛错，必须先有统一入口。
5. 多语言 msg 必须基于内部错误类型映射，不允许每个服务随便写死一套。
```

## 2. 参考项目怎么做

参考源码：

```txt
D:\project\短视频\Toonflow-app-master\src\lib\responseFormat.ts
D:\project\短视频\Toonflow-app-master\src\utils\error.ts
D:\project\短视频\Toonflow-app-master\src\middleware\middleware.ts
D:\project\短视频\Toonflow-app-master\src\err.ts
D:\project\短视频\Toonflow-app-master\src\app.ts
D:\project\短视频\Toonflow-app-master\src\routes\*
```

看到的事实：

| 参考位置 | Toonflow 事实 | VT Studio 怎么处理 |
|---|---|---|
| `lib/responseFormat.ts` | 成功 `{ code: 200, data, message: "成功" }` | VT Studio 保留 code/data 语义，但字段用用户确认的 `msg` |
| `lib/responseFormat.ts` | 失败 `{ code: 400, data, message }` | VT Studio 保留对外失败 `code: 400`，细分错误只放内部 |
| `utils/error.ts` | `normalizeError(error)` 处理 Axios/Error/非 Error | VT Studio 要保留“错误归一化”能力 |
| `middleware.ts` | zod 参数错误返回 `{ message: "参数错误", errors }` | VT Studio 后续参数错误也要转成 `{ code, data, msg }` |
| `err.ts` | 监听 `unhandledRejection` 和 `uncaughtException` 并打印序列化错误 | VT Studio 建议本任务先做日志工具，不强行全局吞异常 |
| `app.ts` | 401/404/500 直接返回不同结构 | VT Studio 不照搬；IPC 必须统一格式 |
| `routes/*` | route 中混用 `success()`、`error()`、直接 `{ message }`、直接字符串、throw Error | VT Studio 不照搬混乱写法 |

Toonflow 的核心可用思路：

```txt
成功和失败都有 code/data/message。
业务错误可以带中文 message。
第三方错误要 normalize 后取 message。
未捕获异常要记录堆栈和序列化详情。
```

VT Studio 已确认格式：

```txt
成功：{ code: 200, data: {}, msg: "成功" }
失败：{ code: 400, data: {}, msg: "错误原因" }
```

注意：

```txt
对外响应 code 只表达成功/失败：成功 200，失败 400。
内部错误分类不能放到响应 code 里。
VT Studio 不使用 message 字段。
VT Studio 不使用 ok/error 外层结构。
VT Studio 不允许同一项目里混用 message/msg。
```

## 3. 当前项目事实

已存在：

```txt
src/shared/types/response.ts
src/shared/constants/status.ts
src/main/ipc/handle.ts
src/main/services/database/*
src/main/services/file-system/*
```

当前已有能力：

```txt
VtResponse<T> = { code, data, msg }
VT_STATUS 已有通用、文件、数据库、模型、任务、Agent、导出内部状态码
getStatusMsg(statusCode, locale) 已有中英文文案
handleIpc() 可以 catch 错误并返回失败响应
VtIpcError 可以指定 code 和 msg
```

当前不足：

```txt
VtIpcError 放在 ipc/handle.ts，services 不应该依赖 ipc。
没有 VtError 作为全局服务错误。
没有 normalizeUnknownError。
没有 detail/logDetail 规范。
没有服务层 Result 工具。
没有参数校验错误的统一表达。
没有判断错误来源的工具，比如 isVtError。
没有把错误转换成 VtResponse 的独立函数。
```

## 4. 本次目标

本次只做错误和 Result 基础层：

```txt
1. 新增 shared/main 可共用的错误类型。
2. 固定服务层抛错方式。
3. 固定 IPC 层捕获错误后如何转成对外 { code: 400, data: {}, msg }。
4. 固定 unknown error 如何归一化。
5. 固定 detail 只进日志，不直接返回页面。
6. 保留多语言能力：msg 默认从内部 statusCode 取当前语言文案，可被业务明确覆盖。
7. 把 VtIpcError 从 IPC 专用概念升级为通用 VtError 或兼容导出。
```

本任务不做业务错误逐个改造。

## 5. 建议新增/修改文件

建议新增：

```txt
src/shared/errors/vt-error.ts
src/shared/errors/normalize.ts
src/shared/errors/index.ts
src/shared/types/result.ts
src/main/services/result.ts
```

建议修改：

```txt
src/main/ipc/handle.ts
src/shared/types/response.ts
src/shared/constants/status.ts
docs/tasks/CORE-005-Result和错误.md
docs/03-执行进度.md
```

说明：

| 文件 | 作用 |
|---|---|
| `vt-error.ts` | 定义 `VtError`，包含 code/msg/detail/cause |
| `normalize.ts` | 把 unknown error 归一化为可记录结构 |
| `result.ts` | 定义服务层 Result 类型，必要时使用 |
| `main/services/result.ts` | main 侧创建成功/失败响应、错误转响应 |
| `ipc/handle.ts` | 使用统一错误工具，不再在 IPC 内定义业务错误 |

## 6. 建议结构

错误对象：

```ts
class VtError extends Error {
  statusCode: VtStatusCode
  errorKey?: string
  detail?: unknown
  cause?: unknown
}
```

归一化错误：

```txt
name
message
statusCode
errorKey
status
stack
detail
cause
```

返回结构继续固定：

```txt
VtResponse<T>:
code: 200 | 400
data: T
msg: string
```

服务层 Result 可选结构：

```txt
成功：{ success: true, data }
失败：{ success: false, error: VtError }
```

专业建议：

```txt
普通同步/异步服务优先直接 return data 或 throw VtError。
复杂批处理、任务内部状态可用 Result。
不要强制所有 service 都包 Result，否则代码会啰嗦。
```

## 7. 对外 code 和内部错误分类

对外响应 code 固定：

| 场景 | 对外 code | data | msg |
|---|---|---|
| 成功 | `200` | 业务数据 | `成功` 或业务成功文案 |
| 失败 | `400` | `{}` | 内部错误类型映射出的文案或业务错误文案 |

内部错误分类：

| 场景 | 内部 statusCode | msg 来源 |
|---|---|---|
| 普通失败 | `VT_STATUS.FAIL` | 业务 msg 或默认文案 |
| 参数错误 | `VT_STATUS.INVALID_PARAMS` | 参数错误 |
| 未登录 | `VT_STATUS.UNAUTHORIZED` | 登录已失效 |
| 无权限 | `VT_STATUS.FORBIDDEN` | 没有操作权限 |
| 数据不存在 | `VT_STATUS.NOT_FOUND` | 数据不存在 |
| 状态冲突 | `VT_STATUS.CONFLICT` | 当前状态不允许操作 |
| 系统异常 | `VT_STATUS.SYSTEM_ERROR` | 系统异常 |
| 文件错误 | `VT_STATUS.FILE_*` | 文件类文案 |
| 数据库错误 | `VT_STATUS.DATABASE_*` | 数据库类文案 |
| 模型错误 | `VT_STATUS.MODEL_*` | 模型类文案 |
| 任务错误 | `VT_STATUS.TASK_*` | 任务类文案 |
| Agent 错误 | `VT_STATUS.AGENT_*` | Agent 类文案 |
| 导出错误 | `VT_STATUS.EXPORT_*` | 导出类文案 |

原则：

```txt
页面展示 msg。
页面只用 code 判断成功/失败。
内部程序判断 statusCode/errorKey。
日志排查 detail/statusCode/errorKey。
业务记录失败原因时写 normalize 后的 message。
```

## 8. 多语言规则

当前已有：

```txt
getStatusMsg(statusCode, locale)
DEFAULT_LOCALE = zh-CN
```

本任务继续保持：

```txt
1. 对外 code 只判断成功/失败。
2. msg 默认从内部 statusCode + locale 取。
3. 业务可以覆盖 msg，但必须明确原因。
4. detail 不做用户展示，不要求多语言。
5. 后续语言设置接入后，IPC 可从设置读取 locale。
```

本任务暂不做：

```txt
不接入设置里的语言配置。
不做前端 i18n 页面。
不翻译所有业务错误。
```

## 9. 和 IPC 的关系

当前：

```txt
handleIpc(channel, handler)
handler 成功返回 data
catch VtIpcError 当前会把 error.code 直接放进响应 code
catch unknown 返回 SYSTEM_ERROR
```

现状问题：

```txt
这会导致失败响应 code 可能变成 40001/60001/70001。
它和前面已确认的“成功 200、失败 400”规则冲突。
本任务要修正为：VtError.statusCode 只用于内部分类，对外失败 code 固定 400。
```

本任务后建议：

```txt
handleIpc(channel, handler)
handler 成功返回 data
catch VtError -> { code: 400, data: {}, msg }
catch unknown -> normalizeUnknownError + { code: 400, data: {}, msg: "系统异常" }
console.error 打印 channel + normalized detail
```

不改变 renderer 调用格式。

## 10. 和任务失败原因的关系

任务队列还没做，但规则要先定：

```txt
任务失败 reason/error_reason 写 normalizeUnknownError(error).message。
不要把完整 stack 写进数据库业务字段。
完整 detail 进日志。
模型供应商返回的 message 要保留为失败原因。
```

后续 CORE-006 任务队列必须复用本任务工具。

## 11. 本次不做什么

```txt
不做任务队列。
不做模型适配。
不做参数校验库。
不批量改所有 service。
不新增业务 IPC。
不新增页面。
不新增数据库表。
不接入设置语言。
不做日志文件落盘。
```

说明：

```txt
日志文件落盘以后可以放到日志服务或开发者配置里做。
本任务只保证 console/error detail 结构清楚。
```

## 12. 风险和处理

| 风险 | 处理 |
|---|---|
| 又出现 VtIpcError、VtServiceError 多套错误 | 统一成 `VtError`，IPC 只负责转换 |
| 页面拿到 detail 泄露路径或 API Key | detail 只进日志，不返回 renderer |
| 所有 service 强制 Result 后代码啰嗦 | 普通服务 return/throw，复杂任务再用 Result |
| 业务 msg 写死导致多语言混乱 | 默认按内部 statusCode 取 msg，业务覆盖必须少用 |
| 把 Toonflow 的 message 字段带进来 | VT Studio 固定 `msg`，不新增 `message` |
| 内部错误码污染对外响应 code | 对外响应 code 固定 200/400，内部状态使用 statusCode/errorKey |

## 13. 验收标准

确认执行后必须满足：

```txt
1. pnpm run typecheck 通过。
2. pnpm run build 通过。
3. IPC 返回仍然是 { code, data, msg }。
4. 成功响应 code 固定 200。
5. 失败响应 code 固定 400。
6. 内部 statusCode/errorKey 不返回 renderer。
7. VtError 可在 services 中使用，不依赖 ipc 目录。
8. unknown error 能归一化，至少有 name/message/stack/detail。
9. VtError 的 detail 不返回 renderer。
10. handleIpc 仍能兼容现有 app.getInfo。
11. 不新增业务页面。
12. 不新增业务表。
13. 不改变已有状态码数值语义，只把它们作为内部 statusCode。
```

## 14. 用户确认点

请确认是否按这个范围执行：

```txt
确认后才开始写代码。
不确认就只继续调整本文档。
```

需要确认的范围：

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-CORE-005-001 | 是否把 `VtIpcError` 升级为通用 `VtError`，services 也用它 | 已按推荐确认 |
| C-CORE-005-002 | 是否继续固定返回 `{ code, data, msg }`，成功 code=200，失败 code=400，不使用 `message` | 已按推荐确认 |
| C-CORE-005-003 | 是否允许 detail 只进日志，不返回页面 | 已按推荐确认 |
| C-CORE-005-004 | service 是否优先 return/throw，不强制所有函数包 Result | 已按推荐确认 |
| C-CORE-005-005 | 本任务是否不接入前端语言设置，只保留 locale 能力 | 已按推荐确认 |

## 15. 执行后记录

```txt
实际新增文件：
src/shared/errors/vt-error.ts
src/shared/errors/normalize.ts
src/shared/errors/index.ts
src/shared/types/result.ts
src/main/services/result.ts

实际修改文件：
src/main/ipc/handle.ts
src/shared/types/response.ts
docs/tasks/TEMPLATE-功能执行文档模板.md
docs/00-项目规范.md
docs/01-参考功能总表.md
docs/tasks/CORE-005-Result和错误.md
docs/03-执行进度.md

实现结果：
1. 新增通用 VtError，services 和 IPC 可共用。
2. VtError 使用 statusCode/errorKey/detail/cause，不把内部 statusCode 当对外 code。
3. 新增 normalizeUnknownError，统一归一化 VtError、Error、普通对象、原始值。
4. 新增 main/services/result.ts，统一创建成功/失败响应、错误转响应、日志输出。
5. handleIpc 改为 catch 后统一 logServiceError + errorToResponse。
6. 对外 VtResponse code 类型收紧为 200 | 400。
7. 保留 VtIpcError 兼容导出，但实际指向 VtError。
8. detail 只进日志，不返回 renderer。
9. 未新增业务页面、业务 IPC、业务表。

验证命令：
D:\software\nodejs\pnpm.cmd run typecheck
D:\software\nodejs\pnpm.cmd run build

验证结果：
typecheck：通过。
build：通过。

未完成事项：
无。后续任务队列、模型适配、语言设置接入按单独任务处理。

是否有偏差：
无业务偏差。对外 code 固定 200/400 是已确认项目规则。

是否更新 03：
是。

最终结论：
通过。
```

## 16. 最后大白话

```txt
我这次准备怎么做：
1. 把错误统一成一套 VtError，services 和 IPC 都用同一套。
2. 服务层以后可以 throw VtError 或普通 Error。
3. IPC 统一接住错误，成功返回 code 200，失败返回 code 400。
4. 内部细分错误只用于日志和 msg 映射，不放到对外 code。
5. detail 只写日志，不返回页面。

我不会做什么：
1. 不做页面。
2. 不做业务功能。
3. 不接入语言设置页面。
4. 不做任务队列。
5. 不改模型调用逻辑。

确认规则：
用户已确认按推荐执行。
```
