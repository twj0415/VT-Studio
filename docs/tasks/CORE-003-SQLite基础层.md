# CORE-003 SQLite 基础层

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`  
执行规则：只拆本文档；已获用户确认并完成实现

## 0. 这一步先做什么

```txt
本任务只处理 CORE-003。
它不是把 Toonflow 的全部业务表一次性建完。
它只建立 SQLite 的连接、路径、初始化、迁移、事务和基础技术表。
```

本任务解决的问题：

```txt
后续设置、项目、小说、剧本、资产、任务、模型都要读写 SQLite。
如果没有先定数据库文件位置、连接方式、迁移规则和事务入口，后面每个功能都会各写各的数据库逻辑。
```

## 1. 参考项目怎么做

参考源码：

```txt
D:\project\短视频\Toonflow-app-master\src\utils\db.ts
D:\project\短视频\Toonflow-app-master\src\lib\initDB.ts
D:\project\短视频\Toonflow-app-master\src\utils\getPath.ts
D:\project\短视频\Toonflow-app-master\package.json
```

看到的事实：

| 参考位置 | Toonflow 事实 | VT Studio 怎么处理 |
|---|---|---|
| `utils/db.ts` | 使用 `knex` + `better-sqlite3` | VT Studio 建议先直接用 `better-sqlite3`，少一层抽象，后续需要再评估 Knex |
| `utils/db.ts` | 数据库文件名是 `db2.sqlite` | VT Studio 使用更明确的 `vt-studio.sqlite` |
| `utils/getPath.ts` | Electron 下放 `app.getPath("userData")/data` | VT Studio 放 `app.getPath("userData")/database`，不写项目目录 |
| `utils/getPath.ts` | 非 Electron 时落到 `process.cwd()/data` | VT Studio 不照搬，开发环境也不能写项目 `data` |
| `initDB.ts` | 表不存在时创建，`forceInit` 时删表重建 | VT Studio 不默认删表；迁移必须可重复执行 |
| `initDB.ts` | 一次性定义所有业务表和初始化数据 | VT Studio 不在 CORE-003 一次性创建所有业务表，业务表跟随功能任务迁移 |
| `initDB.ts` | 没有迁移版本表 | VT Studio 必须增加 `schema_migrations` |
| `package.json` | 依赖 `better-sqlite3`、`knex`、`sqlite3`、`@rmp135/sql-ts` | VT Studio 本任务建议只引入 `better-sqlite3` 和类型依赖 |

不照搬：

```txt
不使用 o_ 表名前缀。
不使用 db2.sqlite 文件名。
不把数据库放到项目源码目录。
不在初始化时调用模型 embedding。
不在 CORE-003 建完整业务表。
不做 forceInit 删表重建入口。
不引入 Knex，除非后续确认确实需要。
```

## 2. 当前项目事实

已存在：

```txt
src/main/app/runtime.ts
src/main/ipc/handle.ts
src/shared/types/response.ts
src/shared/constants/status.ts
```

当前已有能力：

```txt
运行目录已经固定：
开发：%TEMP%\VT Studio Dev\user-data
生产：%LOCALAPPDATA%\VT Studio\user-data

IPC 返回已经统一：
{ code, data, msg }
```

当前缺少：

```txt
没有 SQLite 依赖。
没有数据库文件路径规则。
没有数据库连接单例。
没有迁移表。
没有迁移执行器。
没有事务工具。
没有数据库关闭逻辑。
没有数据库基础自检。
```

## 3. 本次目标

本次只做 SQLite 基础层：

```txt
1. 引入 SQLite 最小依赖。
2. 确定数据库文件位置。
3. 创建数据库目录和数据库文件。
4. 建立 main 侧数据库连接单例。
5. 建立 schema_migrations 迁移表。
6. 建立迁移执行器。
7. 建立事务工具。
8. 应用启动时初始化数据库。
9. 应用退出时关闭数据库。
10. 提供数据库基础信息读取函数，供后续数据库管理页面使用。
```

本次不做业务表。原因：

```txt
项目表、设置表、任务表、模型表、资产表都涉及具体功能语义。
这些表必须在对应功能任务中结合 features 文档和参考源码再定。
否则现在一次性定死，后面很容易重构。
```

## 4. 准备怎么改

建议新增依赖：

```txt
dependencies:
- better-sqlite3

devDependencies:
- @types/better-sqlite3
```

使用命令：

```txt
pnpm add better-sqlite3
pnpm add -D @types/better-sqlite3
```

说明：

```txt
安装依赖需要网络权限。
执行实现时如果 sandbox 拦截，需要申请批准。
```

建议改动范围：

```txt
package.json
pnpm-lock.yaml
src/main/index.ts
src/main/services/database/path.ts
src/main/services/database/connection.ts
src/main/services/database/migrations.ts
src/main/services/database/transaction.ts
src/main/services/database/info.ts
src/main/services/database/index.ts
src/shared/constants/status.ts
```

说明：

| 文件 | 作用 | 是否必须 |
|---|---|---|
| `src/main/services/database/path.ts` | 计算数据库目录和文件路径 | 必须 |
| `src/main/services/database/connection.ts` | 创建/复用/关闭 SQLite 连接 | 必须 |
| `src/main/services/database/migrations.ts` | 创建 `schema_migrations` 并执行迁移 | 必须 |
| `src/main/services/database/transaction.ts` | 统一事务入口 | 必须 |
| `src/main/services/database/info.ts` | 读取数据库路径、大小、表数量等基础信息 | 建议 |
| `src/main/services/database/index.ts` | 对外导出数据库基础能力 | 必须 |
| `src/main/index.ts` | 启动时初始化数据库，退出时关闭 | 必须 |
| `src/shared/constants/status.ts` | 补充数据库错误码文案 | 需要时改 |

不新增：

```txt
不新增 database IPC。
不新增设置页数据库管理 UI。
不新增 project/script/assets 等业务表。
不新增空的 repository 层。
不新增 ORM 复杂分层。
```

## 5. 数据库路径规则

必须保持：

```txt
开发数据库：%TEMP%\VT Studio Dev\user-data\database\vt-studio.sqlite
生产数据库：%LOCALAPPDATA%\VT Studio\user-data\database\vt-studio.sqlite
```

禁止：

```txt
D:\project\vt-studio\.runtime
D:\project\vt-studio\data
D:\project\vt-studio\temp
D:\project\vt-studio\db.sqlite
```

路径处理要求：

```txt
数据库目录不存在时创建。
数据库文件不存在时创建。
路径必须来自 app.getPath("userData")。
不能接受 renderer 传入的任意数据库路径。
不能在页面层拼接数据库路径。
```

## 6. 基础表和迁移

本任务只建技术表：

```txt
schema_migrations
```

建议字段：

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | TEXT | 迁移 ID，例如 `0001_create_schema_migrations` |
| `name` | TEXT | 迁移名称 |
| `checksum` | TEXT | 迁移内容校验，后续用于排查 |
| `applied_at` | INTEGER | 应用时间戳 |

迁移规则：

```txt
迁移必须按顺序执行。
已执行迁移不能重复执行。
写迁移记录必须和迁移 SQL 在同一事务中完成。
迁移失败必须回滚。
迁移失败必须抛出明确错误，最终经 IPC 返回 { code, data, msg }。
```

为什么不建全部业务表：

```txt
Toonflow 的 o_project、o_script、o_assets 等表字段很多，并且有旧命名和拼写错误。
VT Studio 已确定不能照搬 o_ 前缀和拼写错误。
业务表应在具体功能任务里逐项对齐字段和行为，再用 migration 新增。
```

## 7. 连接和事务

连接规则：

```txt
main 进程中只维护一个数据库连接。
renderer 不允许访问数据库连接。
preload 不暴露数据库对象。
服务层通过 database/index.ts 获取受控能力。
应用退出时关闭连接。
```

事务规则：

```txt
写操作默认优先使用事务。
多表写入必须使用事务。
初始化和迁移必须使用事务。
事务失败必须回滚。
```

建议 API：

```txt
initializeDatabase()
getDatabase()
closeDatabase()
runMigrations()
withTransaction(fn)
getDatabaseInfo()
```

## 8. 本次不做什么

```txt
不做项目管理。
不做设置页面。
不做数据库管理页面。
不做导入导出。
不做清空数据库。
不做任务队列。
不做模型配置。
不做初始化 Toonflow 默认供应商。
不做初始化 Agent 配置。
不做初始化提示词。
不做 Skill embedding。
不做 renderer 数据库请求封装。
```

## 9. 风险和处理

| 风险 | 处理 |
|---|---|
| better-sqlite3 是原生依赖，安装或构建可能失败 | 实现时先安装依赖并运行 typecheck/build；失败时再评估 sqlite3 或其他方案 |
| 一次性建完整业务表导致后面返工 | 本任务只建迁移表，业务表跟随功能任务 |
| 数据库写到项目目录 | 路径只来自 `app.getPath("userData")/database` |
| 迁移失败导致半初始化 | 迁移和记录写入必须在事务内 |
| 连接被多个地方重复创建 | 用 connection 单例，不允许页面或业务文件 new Database |
| 和 Toonflow 表名不同 | 这是命名修正，业务语义后续逐表对齐 |

## 10. 验收标准

确认执行后必须满足：

```txt
1. pnpm run typecheck 通过。
2. 如果依赖安装成功，pnpm run build 至少能通过 main/preload/renderer 构建。
3. pnpm run dev 能启动，不因为 SQLite 初始化报错退出。
4. 数据库文件创建在 userData/database/vt-studio.sqlite。
5. 项目根目录没有 .runtime、data、temp、db.sqlite。
6. schema_migrations 表存在。
7. 重复启动不会重复执行已完成迁移。
8. renderer 仍不能直接访问 SQLite。
9. 没有新增业务表和业务 IPC。
```

## 11. 用户确认点

请确认是否按这个范围执行：

```txt
确认后才开始改代码和安装依赖。
不确认就只继续调整本文档。
```

需要你确认的范围：

| 编号 | 确认点 | 建议 |
|---|---|---|
| C-CORE-003-001 | SQLite 驱动是否先用 `better-sqlite3`，不引入 Knex | 建议确认，结构更简单，后续不够再加 Knex |
| C-CORE-003-002 | 数据库路径是否使用 `userData/database/vt-studio.sqlite` | 建议确认，清晰且不污染项目目录 |
| C-CORE-003-003 | 本任务是否只建 `schema_migrations`，业务表后续跟随功能任务创建 | 建议确认，避免没分析功能就定死表结构 |
| C-CORE-003-004 | 是否允许安装 `better-sqlite3` 和 `@types/better-sqlite3` | 建议确认，这是实现 SQLite 基础层的前提 |

## 12. 执行后记录

实际新增文件：

```txt
src/main/services/database/path.ts
src/main/services/database/connection.ts
src/main/services/database/migrations.ts
src/main/services/database/transaction.ts
src/main/services/database/info.ts
src/main/services/database/index.ts
.npmrc
```

实际修改文件：

```txt
package.json
pnpm-lock.yaml
pnpm-workspace.yaml
src/main/index.ts
src/shared/constants/status.ts
docs/tasks/CORE-003-SQLite基础层.md
docs/03-执行进度.md
```

安装依赖：

```txt
dependencies:
- better-sqlite3

devDependencies:
- @types/better-sqlite3
```

额外环境处理：

```txt
1. 项目级 .npmrc 增加 registry、electron_mirror、electron_builder_binaries_mirror。
2. 项目级 .npmrc 增加 store-dir=.pnpm-store，避免继续使用损坏的全局 pnpm store。
3. pnpm-workspace.yaml 允许 better-sqlite3 build script。
4. better-sqlite3 按 Electron 34.0.2 runtime 重新 rebuild，生成 native binding。
```

实现结果：

```txt
1. 数据库路径固定为 app.getPath("userData")/database/vt-studio.sqlite。
2. main 进程维护 SQLite 单例连接。
3. 应用启动时执行 runMigrations()。
4. 应用退出前 closeDatabase()。
5. 只创建 schema_migrations 技术表。
6. 提供 withTransaction(fn) 事务入口。
7. 提供 getDatabaseInfo() 基础信息读取。
8. renderer/preload 未暴露 SQLite、文件路径或数据库对象。
9. 未新增业务表，未新增 database IPC，未新增 UI。
```

验证命令：

```txt
D:\software\nodejs\pnpm.cmd run typecheck
D:\software\nodejs\pnpm.cmd run build
D:\software\nodejs\pnpm.cmd run dev
```

验证结果：

```txt
typecheck：通过。
build：通过。
dev：通过，Electron 主进程成功初始化 SQLite。
数据库文件：C:\Users\Twj\AppData\Local\Temp\VT Studio Dev\user-data\database\vt-studio.sqlite
schema_migrations：已创建，migrationCount = 1。
SQLite version：3.53.2。
```

未完成事项：

```txt
无。业务表、数据库管理页面、导入导出、清库、任务队列等按后续功能任务处理。
```

是否有偏差：

```txt
无业务偏差。
环境处理有调整：为 better-sqlite3 native build 增加 pnpm build 许可和项目级缓存/镜像配置。
```

是否更新 03：

```txt
是。
```

最终结论：

```txt
通过。
```
