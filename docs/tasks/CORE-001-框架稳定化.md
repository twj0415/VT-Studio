# CORE-001 框架稳定化

状态：已完成  
类型：基础任务  
所属模块：公共底层  
执行规则：已按用户确认执行；后续功能仍需先写 tasks 文档并确认

## 0. 这一步先做什么

```txt
本任务是正式开发的第一步。
用户已确认本文档，当前已完成 src 基础框架稳定化。
```

本任务解决的问题：

```txt
当前 src/main/index.ts 同时处理 userData、GPU 参数、窗口创建、renderer 加载、窗口事件、IPC 注册。
后续 SQLite、文件、模型、任务都要依赖 main/preload/ipc 的边界。
如果这一步不先稳住，后面业务功能会越做越乱。
```

## 1. 参考项目怎么做

参考源码：

```txt
D:\project\短视频\Toonflow-app-master\scripts\main.ts
D:\project\短视频\Toonflow-app-master\src\app.ts
D:\project\短视频\Toonflow-app-master\src\utils\getPath.ts
D:\project\短视频\Toonflow-app-master\package.json
```

看到的事实：

| 参考位置 | 事实 | VT Studio 怎么处理 |
|---|---|---|
| `scripts/main.ts` | Electron 主进程负责窗口创建、页面加载、启动流程 | 保留 Electron 主进程，但拆清楚启动、窗口、运行目录 |
| `scripts/main.ts` | 生产数据放 `app.getPath("userData")/data` | VT Studio 运行数据也放系统用户目录，不放项目源码目录 |
| `src/utils/getPath.ts` | Electron 下优先使用 `userData/data` | 本任务只确认 userData 根目录，业务文件层后续单独做 |
| `src/utils/getPath.ts` | 有路径逃逸检查 | 属于后续本地文件层，不塞进本任务 |
| `src/app.ts` | 启动时检查 data 可写 | 属于后续文件层，本任务不做业务文件检查 |
| `package.json` | 桌面开发有专门 dev 命令 | VT Studio 使用 `pnpm run dev` |

不照搬：

```txt
不启动 Toonflow 的 Express 本地服务。
不复制 Toonflow 的 data 目录。
不保留 toonflow:// 协议名。
不把 SQLite、模型、任务队列提前塞进本任务。
```

## 2. 当前项目事实

已存在：

```txt
src/main/index.ts
src/main/ipc/app.ts
src/preload/index.ts
src/shared/contracts/preload.ts
src/shared/types/app.ts
src/shared/constants/app.ts
```

当前已有能力：

```txt
preload 已暴露 window.vtStudio.app.getInfo
main/ipc 已有 app:get-info
renderer 已经可以通过 window.vtStudio 读取 appInfo
pnpm run dev 是当前开发启动命令
```

当前主要问题：

```txt
src/main/index.ts 职责太多。
userData 路径规则没有独立函数，后续容易被业务代码乱用。
窗口创建和 renderer 加载逻辑没有独立边界。
GPU 禁用参数直接堆在入口文件里。
IPC 注册只有 app 一个模块，缺统一 registerIpc 入口。
```

## 3. 本次目标

本次只做基础框架稳定化：

```txt
1. 保持 pnpm run dev 能打开 Electron 桌面窗口。
2. 确认 runtime/userData 不写进项目目录。
3. 把 src/main/index.ts 精简成启动编排。
4. 把窗口创建、运行目录、GPU 参数、IPC 注册拆到清楚的小文件。
5. 保持 renderer -> window.vtStudio -> main/ipc 的边界。
6. 保留一个最小 app health/info API，方便后续验证 IPC 正常。
```

本次不追求“大架构”，只把已经混在一起的基础能力拆开。

## 4. 准备怎么改

建议改动范围：

```txt
src/main/index.ts
src/main/app/runtime.ts
src/main/app/gpu.ts
src/main/app/window.ts
src/main/ipc/index.ts
src/main/ipc/app.ts
src/shared/types/app.ts
src/shared/contracts/preload.ts
src/preload/index.ts
```

说明：

| 文件 | 作用 | 是否必须 |
|---|---|---|
| `src/main/index.ts` | 只保留 app.whenReady、注册 IPC、创建窗口、生命周期 | 必须 |
| `src/main/app/runtime.ts` | 计算并设置 userData 路径 | 必须 |
| `src/main/app/gpu.ts` | 管理 GPU 禁用参数 | 可选，但建议 |
| `src/main/app/window.ts` | 创建 BrowserWindow、加载 renderer、外链拦截 | 必须 |
| `src/main/ipc/index.ts` | 统一注册 IPC 模块 | 必须 |
| `src/main/ipc/app.ts` | 保留 app:get-info，可补充 isDev/userDataPath | 必须 |
| `src/shared/types/app.ts` | AppInfo 类型同步 | 需要时改 |
| `src/shared/contracts/preload.ts` | preload API 类型同步 | 需要时改 |
| `src/preload/index.ts` | 只同步 API 暴露，不加业务能力 | 需要时改 |

目录目标：

```txt
src/main/
  index.ts
  app/
    runtime.ts
    gpu.ts
    window.ts
  ipc/
    index.ts
    app.ts
```

不新增：

```txt
不新增 db 目录
不新增 files 目录
不新增 models 目录
不新增 tasks 目录
不新增业务 service 空壳
```

## 5. 运行目录规则

必须保持：

```txt
开发环境 userData：%TEMP%\VT Studio Dev\user-data
生产环境 userData：%LOCALAPPDATA%\VT Studio\user-data
```

禁止：

```txt
D:\project\vt-studio\.runtime
D:\project\vt-studio\data
D:\project\vt-studio\temp
```

本任务只处理 Electron `app.setPath("userData")`。  
项目业务目录、素材目录、导出目录、缓存目录后续在本地文件层任务里做。

## 6. IPC 和 preload 边界

本次只保留基础 API：

```txt
window.vtStudio.app.getInfo()
```

可以补充返回：

```txt
name
version
platform
isDev
userDataPath
```

不能做：

```txt
不能让 renderer 直接拿 Node API。
不能让 renderer 直接读取文件系统。
不能暴露任意路径读写 API。
不能提前加入 SQLite、模型、任务 IPC。
```

## 7. 本次不做什么

```txt
不做设置页面。
不做项目管理。
不做 SQLite。
不做数据库迁移。
不做文件管理。
不做任务队列。
不做模型适配。
不做 ComfyUI。
不做剪映导出。
不改业务页面。
不美化 UI。
```

如果执行中发现必须改页面才能验证，只允许做最小显示，不允许顺手做业务功能。

## 8. 风险和处理

| 风险 | 处理 |
|---|---|
| Electron 窗口不显示 | 保留 `ready-to-show`、`did-finish-load`、`did-fail-load` 的 show/focus 兜底 |
| userData 仍写到项目目录 | typecheck 后运行 dev，并检查项目根目录没有 `.runtime/data/temp` |
| 拆太多文件导致复杂 | 只拆 runtime/gpu/window/ipc，不提前建业务层 |
| preload 类型不同步 | 修改 `shared/contracts/preload.ts` 和 `preload/index.ts` 同步 |
| 外链乱打开 | 继续通过主进程 `shell.openExternal` 处理 |

## 9. 验收标准

确认执行后必须满足：

```txt
1. pnpm run typecheck 通过。
2. pnpm run dev 能打开 Electron 桌面窗口。
3. renderer 仍能调用 window.vtStudio.app.getInfo。
4. src/main/index.ts 只做启动编排，不再堆窗口和 runtime 细节。
5. 开发运行数据不写入 D:\project\vt-studio\.runtime。
6. 没有新增业务功能。
7. 没有创建空的大型目录结构。
```

## 10. 用户确认点

请确认是否按这个范围执行：

```txt
确认后我才开始改代码。
如果不确认，本任务保持等待状态。
```

需要你确认的范围：

| 编号 | 确认点 | 建议 |
|---|---|---|
| C-CORE-001 | 是否先做 CORE-001 框架稳定化 | 建议做，这是后续所有功能的底座 |
| C-CORE-002 | 是否允许拆出 `runtime/gpu/window/ipc` 这几个小文件 | 建议允许，范围小且清楚 |
| C-CORE-003 | `app.getInfo` 是否补充 `isDev/userDataPath` 用于验证 | 建议补充，方便确认 runtime 没写错 |

## 11. 执行后记录

```txt
实际新增文件：
- src/main/app/runtime.ts
- src/main/app/gpu.ts
- src/main/app/window.ts
- src/main/ipc/index.ts

实际修改文件：
- src/main/index.ts
- src/main/ipc/app.ts
- src/shared/types/app.ts

验证命令：
- pnpm run typecheck
- pnpm run dev

验证结果：
- pnpm run typecheck 通过。
- pnpm run dev 已启动 Electron 开发环境。
- renderer dev server 使用 http://localhost:5174/，因为 5173 已被占用。
- 项目根目录未生成 .runtime、data、temp。
- PowerShell 启动 pnpm run dev 时遇到系统会话错误 CreateProcessAsUserW 1312；改用 cmd.exe 启动成功。
- Electron 日志有 Windows cache / os_crypt 访问警告，但 dev 会话未退出。

未完成事项：
- 本任务不做 SQLite、文件业务层、任务队列、模型适配、ComfyUI、剪映导出。
- 后续从 CORE-002 IPC 契约层开始时，仍需先创建或优化对应 tasks 文档并确认。

是否有偏差：
- 无业务语义偏差。
- 无运行目录偏差。

是否更新 03：
- 是。

是否需要更新 01/02/04：
- 01/02 不需要。
- 04 不需要；本次没有改变参考项目业务语义。

最终结论：通过
```
