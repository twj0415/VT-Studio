# CORE-013 日志与诊断层

状态：已完成  
所属菜单：00-公共底层  
对应功能文档：`docs/features/00-公共底层.md`  
原则：先让启动日志正常人能看懂，再保留开发排查需要的详细信息

## 0. 快速理解

```txt
一句话：把终端里一堆看不懂的 console 改成清楚的启动状态，把详细对象和错误写进日志文件。
为什么现在做：后面任务、模型、导出、Socket 日志会越来越多，不先统一会越来越乱。
做完后有什么用：你跑 pnpm run dev 时能一眼看懂启动到哪一步，开发排查也能看 main.log。
这一步不碰什么：不做前端日志面板，不做日志上传，不改业务流程。
```

## 1. 本次做什么

```txt
目标：统一 main 侧日志入口，替换当前启动链路里散落的 console。
只做：主进程 logger、启动日志、本地服务日志、Socket 日志、数据库 seed 日志、服务错误日志、开发终端 UTF-8、第三方噪声降级、终端固定列宽、行间距和彩色分组。
不做：renderer.log、日志查看页面、日志清理设置、远程诊断上传。
```

## 2. 参考项目怎么做

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-app/src/*` | 未找到统一日志层，主要是零散输出 |
| `Toonflow-app/src/utils/taskRecord.ts` | 任务失败原因写数据库，不解决全局日志 |
| `Toonflow-app/src/utils/ai.ts`、`vendor/*` | 模型失败靠异常和任务记录排查，缺少统一诊断入口 |

结论：参考项目没有成熟日志方案，VT Studio 这里做底层增强，不照搬。

## 3. 用户操作

```txt
入口：无页面入口。
按钮/操作：无。
弹窗/表单：无。
成功反馈：运行 pnpm run dev 时终端显示中文短状态。
失败反馈：启动失败时终端显示哪一步失败，详细错误写 main.log。
```

## 4. 要做什么功能

### 1. 统一 logger

怎么做：
- 输入：level、scope、message、detail。
- 输出：终端短句 + `logs/main.log` 详细 JSON 行。
- 写什么数据：写入 runtime logs，不写项目根目录。
- 状态怎么变：无业务状态。
- 异常怎么处理：日志写入失败不能影响应用启动。
- 限制：业务模块不要继续散写 `console.info/error`。

### 2. 终端人能看懂

怎么做：
- 输入：启动链路状态。
- 输出：`[数据库] 已连接：10 张表，4 个迁移记录` 这种短句。
- 写什么数据：详细对象只写日志文件。
- 状态怎么变：无业务状态。
- 异常怎么处理：error/fatal 才刷终端错误。
- 限制：不在终端输出 stack、大 JSON、密钥、模型请求全文。

### 3. 详细日志落文件

怎么做：
- 输入：detail/error。
- 输出：`%TEMP%\VT Studio Dev\user-data\logs\main.log`。
- 写什么数据：time、level、scope、message、detail。
- 状态怎么变：无业务状态。
- 异常怎么处理：写文件失败直接吞掉，避免阻塞启动。
- 限制：token、apiKey、password、secret、authorization 要脱敏。

### 4. 替换当前启动链路

怎么做：
- 输入：文件系统初始化、数据库迁移、任务恢复、本地服务、Socket、窗口加载。
- 输出：终端中文短句。
- 写什么数据：详细目录、数据库信息、本地服务端口、Socket URL 写 main.log。
- 状态怎么变：无业务状态。
- 异常怎么处理：服务错误统一走 `logger.error`。
- 限制：第三方 Electron/Sass 警告只做降噪，不隐藏真正构建错误。

### 5. 修复 Windows 开发终端乱码

怎么做：
- 输入：用户执行 `pnpm run dev`。
- 输出：脚本先切到 UTF-8，再启动 electron-vite。
- 写什么数据：不写数据。
- 状态怎么变：无业务状态。
- 异常怎么处理：如果用户绕过 `pnpm run dev` 直接跑 electron-vite，仍可能看到乱码。
- 限制：这是 Windows 开发体验配置，生产程序不依赖它。

### 6. 降低第三方启动噪声

怎么做：
- 输入：Electron/Chromium cache、os_crypt、Sass legacy API 警告。
- 输出：Chromium 日志级别调高，Sass legacy-js-api 警告静默。
- 写什么数据：不写数据。
- 状态怎么变：无业务状态。
- 异常怎么处理：项目自己的错误仍走 logger，不被隐藏。
- 限制：真正影响启动的构建错误仍会显示。

### 7. 终端日志固定格式

怎么做：
- 输入：level、scope、message。
- 输出：`时间 级别 │ 模块名       │ 文案`。
- 写什么数据：文件日志不变，仍写 JSON 行。
- 状态怎么变：无业务状态。
- 异常怎么处理：error/fatal 仍走 stderr，warn 走 warn，其它走 info。
- 限制：颜色只用于终端，不写进 `main.log`。

### 8. 终端行间距

怎么做：
- 输入：终端可见日志。
- 输出：每条 VT Studio 日志后空一行，分隔线前后留空。
- 写什么数据：文件日志不变，不写空行。
- 状态怎么变：无业务状态。
- 异常怎么处理：无。
- 限制：只影响 VT Studio logger 输出，Vite/HMR 自己的输出不受控制。

## 5. 数据和状态

```txt
字段：time、level、scope、message、detail。
接口/能力：main/services/logger。
数据读写：追加写 runtime logs/main.log。
任务状态：不改变任务状态。
轮询/Socket：Socket 启停日志走 logger。
模型调用：供应商脚本 logger 只写文件 detail，后续模型任务再补 requestId。
删除影响：无。
```

## 6. VT Studio 怎么落

```txt
能力名：logger.info / logger.warn / logger.error / logger.detail / logger.fatal
调用链：main/service -> logger -> terminal + logs/main.log
需要新增：src/main/services/logger.ts
需要修改：启动入口、本地服务、窗口、Socket、seed、result、vendor-runner、package.json、electron.vite.config.ts、gpu 配置
```

## 7. 偏差

```txt
和 Toonflow 不同的地方：VT Studio 新增统一日志与脱敏规则。
原因：参考项目没有统一日志层，后续本地桌面排查必须可读、可定位。
是否写入 04：是，记录为 D-BASE-015。
```

## 8. 验收

```txt
1. pnpm run dev 终端显示中文短状态。
2. 主进程不再散落业务 console，只保留 logger 内部 console。
3. runtime logs/main.log 能写入详细日志。
4. token/apiKey/password/secret/authorization 不直接写明文。
5. typecheck 通过。
6. 控制台输出有固定列宽、级别颜色、启动分隔线和行间距。
```

## 9. 执行后记录

```txt
改了哪些文件：
- src/main/services/logger.ts
- src/main/index.ts
- src/main/app/server.ts
- src/main/app/window.ts
- src/main/services/socket/index.ts
- src/main/services/database/seed.ts
- src/main/services/result.ts
- src/main/services/model/vendor-runner.ts
- src/main/app/gpu.ts
- electron.vite.config.ts
- package.json
- docs/features/00-公共底层.md
- docs/tasks/CORE-013-日志与诊断层.md
- docs/03-执行进度.md
- docs/04-对齐验收与偏差记录.md

验证结果：
- `D:\software\nodejs\pnpm.cmd run typecheck` 通过。
- `D:\software\nodejs\pnpm.cmd run dev` 已验证：终端中文正常显示，项目启动日志为短状态，未再刷 Sass legacy-js-api 警告。
- 本轮继续补充：终端日志改为固定列宽、级别颜色、分隔线和行间距；`D:\software\nodejs\pnpm.cmd run typecheck` 通过。旧 dev 会话未响应重启输入，实际新格式等下一次重新运行 `pnpm run dev` 验证。

未完成事项：
- renderer.log、日志页面、日志清理设置、任务/模型 requestId 细化后续随对应功能补。

最终结论：可接受偏差。
```

## 10. 最后大白话

```txt
我这次准备怎么做：
1. 让 pnpm run dev 的项目日志变成中文短句，并避免 Windows 终端中文乱码。
2. 让控制台输出按“时间 / 级别 / 模块 / 内容”排列，并加空行方便扫读。
3. 把详细技术信息写进 logs/main.log。
4. 以后 main 侧日志都从 logger 走，不再到处 console。

我不会做什么：
1. 不做前端日志页面。
2. 不隐藏真正的构建错误。
3. 不改业务功能。
```
