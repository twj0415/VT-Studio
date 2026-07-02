# CORE-002 IPC 契约层

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`  
执行规则：已按用户确认执行；后续功能仍需先写 tasks 文档并确认

## 0. 这一步先做什么

```txt
本任务只处理 CORE-002。
它不是把 features/00-公共底层.md 全部做完。
它只定 renderer 和 main 之间怎么调用、怎么返回、怎么报错。
用户已确认响应格式使用 `{ code, data, msg }`。
```

本任务解决的问题：

```txt
后续设置、项目、数据库、文件、任务、模型都会调用 main 能力。
如果不先定 IPC 契约，后面每个功能都会各写各的调用方式。
最终会出现页面直接碰 Node、错误格式不统一、API 命名混乱的问题。
```

## 1. 参考项目怎么做

参考源码：

```txt
D:\project\短视频\Toonflow-web-master\src\utils\axios.ts
D:\project\短视频\Toonflow-web-master\src\stores\setting.ts
D:\project\短视频\Toonflow-web-master\src\utils\useSocket.ts
D:\project\短视频\Toonflow-app-master\src\router.ts
D:\project\短视频\Toonflow-app-master\src\lib\responseFormat.ts
D:\project\短视频\Toonflow-app-master\src\middleware\middleware.ts
D:\project\短视频\Toonflow-app-master\src\routes\project\getProject.ts
D:\project\短视频\Toonflow-app-master\src\routes\setting\fileManagement\openFolder.ts
D:\project\短视频\Toonflow-app-master\src\socket\index.ts
```

看到的事实：

| 参考位置 | Toonflow 事实 | VT Studio 怎么处理 |
|---|---|---|
| `Toonflow-web/src/utils/axios.ts` | 前端统一用 axios，请求前设置 `baseURL`、`timeout`、`Authorization` | VT Studio 不用 axios 调本地服务，改成统一 `window.vtStudio` |
| `Toonflow-web/src/stores/setting.ts` | 默认 `baseUrl` 是 `http://localhost:10588/api` | VT Studio 本地能力不暴露 HTTP baseUrl，普通功能走 IPC |
| `Toonflow-app/src/router.ts` | 后端集中注册 169 个 `/api/*` route | VT Studio 不照搬 `/api/*`，按领域拆成 `project.create` 这类能力 |
| `responseFormat.ts` | 返回 `{ code, data, message }`，成功 `code=200`，失败 `code=400` | VT Studio 保留 `code/data/message` 思路，但字段改成 `{ code, data, msg }` |
| `middleware.ts` | route 可用 zod 校验 body/query/params | VT Studio 每个 IPC handler 必须有参数校验入口，本任务先定位置 |
| `project/getProject.ts` | route 直接查 SQLite，再 `send(success(data))` | VT Studio 后续必须是 `ipc -> services -> db`，页面不能碰 db |
| `fileManagement/openFolder.ts` | route 校验路径后用系统命令打开文件夹 | VT Studio 这类本地能力必须放 main，不暴露任意命令能力 |
| `socket/index.ts` | Agent 使用 `/api/socket/productionAgent`、`/api/socket/scriptAgent` | Agent 流式事件不放进 CORE-002，后续 CORE-008 单独做 |

不照搬：

```txt
不启动本地 Express。
不保留可配置请求地址作为本地能力入口。
不把 169 个 route 一次性迁移成 IPC。
不让 renderer 直接使用 axios 调本地服务。
不在本任务处理 Socket.IO / Agent 流式消息。
```

## 2. 当前项目事实

已存在：

```txt
src/preload/index.ts
src/shared/contracts/preload.ts
src/main/ipc/index.ts
src/main/ipc/app.ts
src/shared/types/app.ts
src/renderer/env.d.ts
src/renderer/src/stores/app.ts
```

当前已有能力：

```txt
window.vtStudio.app.getInfo()
preload 内部调用 ipcRenderer.invoke("app:get-info")
main/ipc/app.ts 注册 ipcMain.handle("app:get-info")
renderer store 会直接把返回值写入 appInfo
```

当前主要问题：

```txt
没有统一 Response 类型。
没有统一 IPC handler 包装。
没有统一 code/msg 规则。
没有统一参数校验位置。
没有写清楚能力名和 IPC channel 的关系。
后续如果直接加业务 IPC，会很快变乱。
```

## 3. 本次目标

本次只做 IPC 契约层：

```txt
1. 定义 VT Studio 统一返回结构 `{ code, data, msg }`。
2. 定义 window.vtStudio 的公开 API 命名规则。
3. 定义 ipcMain.handle 的注册包装方式。
4. 定义 preload 只能暴露白名单方法。
5. 把现有 app.getInfo 改成走统一契约。
6. 保持 renderer 不能直接访问 Node、SQLite、文件系统、模型 SDK。
```

本次不是做业务功能，只是给后面的业务功能立规矩和最小代码支撑。

## 4. 准备怎么改

建议改动范围：

```txt
src/shared/types/response.ts
src/shared/constants/status.ts
src/shared/contracts/preload.ts
src/main/ipc/handle.ts
src/main/ipc/app.ts
src/preload/index.ts
src/renderer/src/stores/app.ts
```

说明：

| 文件 | 作用 | 是否必须 |
|---|---|---|
| `src/shared/types/response.ts` | 定义 `VtResponse<T>` | 必须 |
| `src/shared/constants/status.ts` | 定义状态码和中英文兜底文案 | 必须 |
| `src/main/ipc/handle.ts` | 统一注册 IPC，捕获异常并返回 `{ code, data, msg }` | 必须 |
| `src/main/ipc/app.ts` | 把 `app:get-info` 接入统一 handler | 必须 |
| `src/preload/index.ts` | 只暴露白名单 API，不暴露 ipcRenderer | 必须 |
| `src/shared/contracts/preload.ts` | 同步 `window.vtStudio` 类型 | 必须 |
| `src/renderer/src/stores/app.ts` | 适配 `app.getInfo` 的 Result 返回 | 必须 |

目录目标：

```txt
src/main/ipc/
  index.ts
  app.ts
  handle.ts

src/shared/types/
  app.ts
  response.ts

src/shared/constants/
  app.ts
  status.ts

src/shared/contracts/
  preload.ts
```

不新增：

```txt
不新增 project IPC。
不新增 setting IPC。
不新增 database IPC。
不新增 file IPC。
不新增 task IPC。
不新增 model IPC。
不新增 agent IPC。
不新增 services 空壳。
```

## 5. 契约怎么定

统一返回结构：

```txt
成功：{ code: 200, data: {}, msg: "成功" }
失败：{ code: 400, data: {}, msg: "失败原因" }
```

建议类型：

```txt
VtResponse<T> = { code: number; data: T; msg: string }
无数据时 data 使用 {}
```

能力命名：

| 层 | 写法 | 示例 |
|---|---|---|
| 文档能力名 | `domain.action` | `project.create` |
| preload 方法 | `window.vtStudio.domain.action()` | `window.vtStudio.project.create()` |
| IPC channel | `domain:action` | `project:create` |
| 当前 app 能力 | `app.getInfo` | `app:get-info` |

状态码规则：

```txt
200：成功
202：异步任务已创建
400：普通失败
40001：参数错误
401：未登录/登录失效
403：无权限
404：数据不存在
409：状态冲突
500：系统异常
600xx：文件错误
700xx：模型错误
800xx：任务错误
900xx：Agent 错误
100xxx：导出错误
```

本任务只落最小状态码和多语言兜底文案表，不把所有业务错误一次性用完。

多语言规则：

```txt
接口仍只返回 msg。
msg 从统一状态文案表生成，不能在业务里到处手写。
当前项目还没接入完整 i18n，因此本任务先支持 zh-CN 和 en 的状态文案表。
后续 F-002-002 语言设置接入 vue-i18n 后，再把语言来源接入响应工具。
```

## 6. 调用边界

必须保持：

```txt
renderer -> window.vtStudio -> preload 白名单 -> main/ipc -> main/services
```

本任务允许：

```txt
renderer 调用 window.vtStudio.app.getInfo()
preload 调用 ipcRenderer.invoke(固定 channel)
main/ipc 使用统一 handle 包装返回 VtResponse
```

本任务禁止：

```txt
renderer 直接 import electron
renderer 直接 import fs/path
renderer 直接访问 SQLite
renderer 直接调用模型 SDK
preload 暴露 ipcRenderer
preload 暴露任意 invoke(channel, payload)
main/ipc 写业务逻辑
main/ipc 暴露任意文件路径读写
```

## 7. 本次不做什么

```txt
不做设置页面。
不做项目管理。
不做 SQLite。
不做文件管理。
不做任务队列。
不做模型适配。
不做 Agent Socket / IPC event。
不迁移 Toonflow 的 169 个 route。
不处理请求地址设置。
不引入新依赖。
```

如果执行中发现必须加业务能力才能验证，说明范围偏了，必须停下来重新确认。

## 8. 风险和处理

| 风险 | 处理 |
|---|---|
| 把 IPC 契约做成复杂框架 | 只做 `VtResponse`、状态码文案表和 `handleIpc`，不做自动代码生成 |
| 直接暴露通用 invoke | 禁止；preload 只能暴露白名单方法 |
| Response 改动影响现有 app store | 同步修改 `loadAppInfo`，失败时保留 `appInfo=null` |
| 和 Toonflow `{ code,data,message }` 不一致 | 只把 `message` 改成 `msg`，主结构保持一致 |
| Agent Socket 混进来 | 不处理；后续 CORE-008 单独做事件契约 |

## 9. 验收标准

确认执行后必须满足：

```txt
1. pnpm run typecheck 通过。
2. window.vtStudio.app.getInfo 仍能调用。
3. app.getInfo 返回统一 `{ code, data, msg }`。
4. main IPC 异常会变成 `{ code:500,data:{},msg:"系统异常" }`。
5. preload 没有暴露 ipcRenderer 或任意 invoke。
6. 没有新增任何业务 IPC。
7. 没有新增空的大型目录结构。
8. 没有改变运行目录规则。
```

## 10. 用户确认点

用户已确认按这个范围执行：

```txt
响应格式使用 { code, data, msg }。
结合参考项目 code/data/message 思路，但 VT Studio 字段使用 msg。
msg 由统一状态文案表生成，先支持 zh-CN 和 en。
```

需要你确认的范围：

| 编号 | 确认点 | 建议 |
|---|---|---|
| C-CORE-002-001 | 是否把 `app.getInfo` 也改成统一 `VtResponse<AppInfo>` | 已确认 |
| C-CORE-002-002 | 是否使用 `{ code, data, msg }` 作为唯一响应格式 | 已确认 |
| C-CORE-002-003 | 本任务是否只做 IPC 契约，不做任何业务 IPC | 已确认 |

## 11. 执行后记录

```txt
实际新增文件：
- src/shared/types/response.ts
- src/shared/constants/status.ts
- src/main/ipc/handle.ts

实际修改文件：
- src/main/ipc/app.ts
- src/shared/contracts/preload.ts
- src/renderer/src/stores/app.ts
- docs/tasks/CORE-002-IPC契约层.md
- docs/03-执行进度.md

验证命令：
- pnpm run typecheck

验证结果：
- pnpm run typecheck 通过。
- app.getInfo 已改为返回 { code, data, msg }。
- IPC 异常会统一返回 { code: 500, data: {}, msg: "系统异常" }。
- preload 仍只暴露 window.vtStudio.app.getInfo，没有暴露 ipcRenderer 或通用 invoke。
- 没有新增任何业务 IPC。

未完成事项：
- 本任务不接入完整 vue-i18n；后续 F-002-002 语言设置再做。
- 本任务不做 project/setting/database/file/task/model/agent 业务 IPC。
- 后续 CORE-003 SQLite 基础层仍需先创建任务文档并确认。

是否有偏差：
- 无业务语义偏差。
- 响应字段从 Toonflow 的 message 改为 msg，是用户确认后的格式调整，不影响业务语义。

是否更新 03：
- 是。

是否需要更新 01/02/04：
- 01/02 不需要。
- 04 不需要；本次没有改变参考项目业务语义。

最终结论：通过
```
