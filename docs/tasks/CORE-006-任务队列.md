# CORE-006 任务队列

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`、`docs/features/M-009-任务中心.md`  
执行规则：已确认并完成代码实现

## 0. 快速理解

```txt
一句话：这一步做一个统一任务记录服务，后面图片、视频、音频、资产提取、导出都用它记录状态。
为什么现在做：后面很多功能都是异步生成，如果没有统一任务表和状态，任务中心、失败原因、取消能力都会乱。
做完后有什么用：业务只调用 task.create / task.succeed / task.fail / task.list，就能统一写任务、查任务、看失败原因。
这一步不碰什么：不做任务中心页面、不做模型调用、不做并发调度器、不做重试按钮。
```

## 1. 本次做什么

```txt
目标：实现 main/services/task 的任务记录基础能力。
只做：任务表、状态枚举、创建任务、更新状态、失败原因、列表、详情、分类、取消标记。
不做：任务中心 UI、模型生成、业务任务执行器、进度百分比、重试、暂停恢复、批量删除。
```

## 2. 参考项目怎么做

参考源码已经查完，结论如下。

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-app/src/utils/taskRecord.ts` | 创建任务写 `o_tasks`，返回 done 函数；成功调用 `done(1)`，失败调用 `done(-1, reason)` |
| `Toonflow-app/src/lib/initDB.ts` | `o_tasks` 字段：`id/projectId/taskClass/relatedObjects/model/describe/state/startTime/reason` |
| `Toonflow-app/src/utils/ai.ts` | `withTaskRecord` 包住图片/视频/音频生成；生成前写任务，成功写已完成，失败写生成失败和错误原因 |
| `routes/task/getTaskApi.ts` | 按 `taskClass/state/projectId/page/limit` 查询任务列表，按 id 倒序 |
| `routes/task/getTaskCategories.ts` | 从 `o_tasks.taskClass` groupBy 动态获取分类，不写死 |
| `routes/task/taskDetails.ts` | 按 `taskId` 查单条任务 |
| `routes/task/getProject.ts` | 从 `o_project` 取 `id/name` 给任务中心项目筛选 |
| `lib/fixDB.ts` | 软件异常退出时把生成中状态改成失败，并写“软件退出导致失败” |

Toonflow 任务状态：

| Toonflow state | 含义 |
|---|---|
| `进行中` | 任务创建后默认状态 |
| `已完成` | 模型或异步任务完成 |
| `生成失败` | 模型或异步任务失败 |

Toonflow 任务字段：

| 字段 | 含义 |
|---|---|
| `id` | 任务 ID |
| `projectId` | 项目 ID |
| `taskClass` | 任务分类，例如角色图生成、视频生成 |
| `relatedObjects` | 关联对象，字符串或 JSON 字符串 |
| `model` | 使用的模型名称 |
| `describe` | 任务描述 |
| `state` | 进行中、已完成、生成失败 |
| `startTime` | Date.now 毫秒时间戳 |
| `reason` | 失败原因 |

VT Studio 不照搬：

```txt
不使用 o_tasks 表名。
不使用 taskClass / relatedObjects / startTime 作为正式字段名。
不把页面筛选逻辑散落在页面里。
不让业务页面直接更新任务表。
```

## 3. 用户操作

本任务不涉及页面。

```txt
入口：无。
按钮/操作：无。
弹窗/表单：无。
成功反馈：无页面反馈，服务返回数据。
失败反馈：服务抛 VtError，IPC 后续统一返回 { code: 400, data: {}, msg }。
```

## 4. 要做什么功能

### 1. 创建任务

怎么做：
- 输入：`projectId?`、`category`、`relatedObjects?`、`modelName?`、`description?`。
- 输出：`taskId`、`status`、`startedAt`。
- 写什么数据：插入 `tasks` 表。
- 状态怎么变：初始状态写 `running`。
- 异常怎么处理：`category` 为空抛 `VtError(INVALID_PARAMS)`；`relatedObjects` 不能序列化时转字符串。
- 限制：只创建记录，不执行真实业务。

### 2. 标记任务成功

怎么做：
- 输入：`taskId`。
- 输出：更新后的任务。
- 写什么数据：`status=succeeded`、`finished_at=Date.now()`、`updated_at=Date.now()`、`error_reason=null`。
- 状态怎么变：只允许 `running -> succeeded`。
- 异常怎么处理：任务不存在抛 `VtError(TASK_NOT_FOUND)`；已失败/已取消/已成功不允许重复成功。
- 限制：不写业务结果文件，不写业务表状态。

### 3. 标记任务失败

怎么做：
- 输入：`taskId`、`error`。
- 输出：更新后的任务。
- 写什么数据：`status=failed`、`error_reason=normalizeUnknownError(error).message`、`finished_at`、`updated_at`。
- 状态怎么变：只允许 `running -> failed`。
- 异常怎么处理：任务不存在抛 `VtError(TASK_NOT_FOUND)`。
- 限制：完整 stack/detail 不写进任务表，只进日志。

### 4. 取消任务

怎么做：
- 输入：`taskId`、`reason?`。
- 输出：更新后的任务。
- 写什么数据：`status=cancelled`、`error_reason=reason || "任务已取消"`、`finished_at`、`updated_at`。
- 状态怎么变：只允许 `running -> cancelled`。
- 异常怎么处理：已成功/已失败的任务不允许取消，抛 `VtError(CONFLICT)`。
- 限制：Toonflow 没有统一取消状态，这是 VT Studio 增强；本任务只标记取消，不中断模型请求。

### 5. 判断任务是否已取消

怎么做：
- 输入：`taskId`。
- 输出：`boolean`。
- 读什么数据：查 `tasks.status`。
- 状态怎么变：不改状态。
- 异常怎么处理：任务不存在返回 `false` 或抛错需统一，建议抛 `VtError(TASK_NOT_FOUND)`。
- 限制：后续模型生成循环可以调用它判断是否停止。

### 6. 查询任务详情

怎么做：
- 输入：`taskId`。
- 输出：单条任务详情。
- 读什么数据：查 `tasks where id = taskId`。
- 状态怎么变：不改状态。
- 异常怎么处理：不存在抛 `VtError(TASK_NOT_FOUND)`。
- 限制：不做页面详情弹窗。

### 7. 查询任务列表

怎么做：
- 输入：`page`、`limit`、`projectId?`、`category?`、`status?`。
- 输出：`{ data, total }`。
- 读什么数据：查 `tasks`，按 `id desc` 排序。
- 状态怎么变：不改状态。
- 异常怎么处理：page/limit 不合法抛 `VtError(INVALID_PARAMS)`。
- 限制：项目名称 join 后续等项目表创建后再接；本任务先保留 `projectId` 字段筛选。

### 8. 获取任务分类列表

怎么做：
- 输入：无，后续可加 `projectId?`。
- 输出：分类数组。
- 读什么数据：`select category from tasks group by category`，过滤空值。
- 状态怎么变：不改状态。
- 异常怎么处理：数据库错误走 `VtError(DATABASE_ERROR)`。
- 限制：分类不写死，跟 Toonflow 一样从任务表动态读。

### 9. 更新任务描述和关联对象

怎么做：
- 输入：`taskId`、`description?`、`relatedObjects?`、`modelName?`。
- 输出：更新后的任务。
- 写什么数据：更新 `description/related_objects/model_name/updated_at`。
- 状态怎么变：不改状态。
- 异常怎么处理：任务不存在抛 `VtError(TASK_NOT_FOUND)`。
- 限制：只允许任务运行中更新，完成后不建议改；如果必须改，需要单独确认。

### 10. 启动时修复运行中任务

怎么做：
- 输入：无。
- 输出：修复数量。
- 写什么数据：把 `status=running` 的任务改成 `failed`，`error_reason="软件退出导致失败"`，写 `finished_at/updated_at`。
- 状态怎么变：`running -> failed`。
- 异常怎么处理：数据库错误走 `VtError(DATABASE_ERROR)`。
- 限制：参考 Toonflow 的 `fixDB.ts`，只修复任务表；业务表状态后续在对应业务模块处理。

## 5. 数据和状态

VT Studio 建议表名：

```txt
tasks
```

字段设计：

| 字段 | 类型 | 来源/说明 |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | 任务 ID |
| `project_id` | INTEGER NULL | 对应 Toonflow `projectId` |
| `category` | TEXT NOT NULL | 对应 Toonflow `taskClass` |
| `related_objects` | TEXT NULL | 对应 Toonflow `relatedObjects`，JSON 字符串 |
| `model_name` | TEXT NULL | 对应 Toonflow `model` |
| `description` | TEXT NULL | 对应 Toonflow `describe` |
| `status` | TEXT NOT NULL | running/succeeded/failed/cancelled |
| `started_at` | INTEGER NOT NULL | 对应 Toonflow `startTime` |
| `finished_at` | INTEGER NULL | VT Studio 增强，便于统计耗时 |
| `error_reason` | TEXT NULL | 对应 Toonflow `reason` |
| `created_at` | INTEGER NOT NULL | VT Studio 标准字段 |
| `updated_at` | INTEGER NOT NULL | VT Studio 标准字段 |

状态映射：

| Toonflow | VT Studio 内部 | 展示文案 |
|---|---|---|
| `进行中` | `running` | 进行中 |
| `已完成` | `succeeded` | 已完成 |
| `生成失败` | `failed` | 生成失败 |
| 源码未找到 | `cancelled` | 已取消 |

说明：

```txt
cancelled 是 VT Studio 增强，参考项目没有统一任务取消状态。
如果实现 cancelled，需要写入 04 偏差记录。
```

## 6. VT Studio 怎么落

能力名：

```txt
task.create
task.succeed
task.fail
task.cancel
task.isCancelled
task.detail
task.list
task.categories
task.updateMeta
task.recoverRunningTasks
```

调用链：

```txt
业务 service -> main/services/task
后续页面 -> window.vtStudio.task.* -> main/ipc -> main/services/task
```

建议新增：

```txt
src/main/services/task/constants.ts
src/main/services/task/types.ts
src/main/services/task/migrations.ts
src/main/services/task/mapper.ts
src/main/services/task/service.ts
src/main/services/task/index.ts
```

建议修改：

```txt
src/main/index.ts
src/shared/constants/status.ts（需要时）
docs/tasks/CORE-006-任务队列.md
docs/03-执行进度.md
docs/04-对齐验收与偏差记录.md（如确认 cancelled）
```

本任务暂不新增：

```txt
不新增 task IPC。
不新增任务中心页面。
不新增项目表 join。
不接模型服务。
```

## 7. 具体执行步骤

1. 在 `src/main/services/task/constants.ts` 定义 `TASK_STATUS`：`running/succeeded/failed/cancelled`。
2. 在 `types.ts` 定义 `TaskRecord`、`CreateTaskInput`、`ListTasksInput`、`TaskListResult`。
3. 在 `migrations.ts` 定义创建 `tasks` 表的 SQL，字段按本文档第 5 节。
4. 把任务迁移接入现有数据库迁移执行器，确保启动时创建表。
5. 在 `mapper.ts` 写数据库 row 到业务对象的映射，避免字段散落。
6. 在 `service.ts` 写 `createTask`，插入 running 任务。
7. 在 `service.ts` 写 `succeedTask`，只允许 running 变 succeeded。
8. 在 `service.ts` 写 `failTask`，用 `normalizeUnknownError(error).message` 写失败原因。
9. 在 `service.ts` 写 `cancelTask`，只标记 cancelled，不中断底层模型请求。
10. 在 `service.ts` 写 `isTaskCancelled`，给后续长任务轮询使用。
11. 在 `service.ts` 写 `getTaskDetail`，查不到抛 `VtError(TASK_NOT_FOUND)`。
12. 在 `service.ts` 写 `listTasks`，支持分页和 projectId/category/status 筛选。
13. 在 `service.ts` 写 `getTaskCategories`，从任务表 groupBy 动态取分类。
14. 在 `service.ts` 写 `recoverRunningTasks`，启动时把 running 改 failed。
15. 在 `src/main/index.ts` 启动数据库迁移后调用 `recoverRunningTasks`。
16. 跑 `D:\software\nodejs\pnpm.cmd run typecheck`。
17. 跑 `D:\software\nodejs\pnpm.cmd run build`。

## 8. 偏差

| 偏差 | 原因 | 是否写入 04 |
|---|---|---|
| 新增 `cancelled` 状态 | Toonflow 源码未找到统一任务取消状态；VT Studio 后续长任务需要统一取消标记 | 需要 |
| 新增 `finished_at/created_at/updated_at` | Toonflow 只有 `startTime`；VT Studio 需要更完整的任务生命周期 | 需要 |
| 表名从 `o_tasks` 改为 `tasks` | 项目规则要求不照搬 `o_` 前缀 | 不算业务偏差 |

## 9. 验收

```txt
1. typecheck 通过。
2. build 通过。
3. 启动迁移后存在 tasks 表。
4. createTask 能写入 running 任务。
5. succeedTask 能把 running 改 succeeded。
6. failTask 能把 running 改 failed，并写 error_reason。
7. cancelTask 能把 running 改 cancelled。
8. 已完成/已失败/已取消任务不能重复完成或取消。
9. listTasks 支持 page/limit/projectId/category/status。
10. getTaskCategories 从 tasks 表动态读取分类。
11. recoverRunningTasks 能把 running 改 failed。
12. renderer 仍不能直接访问数据库。
13. 不新增页面，不新增业务 IPC。
```

## 10. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-CORE-006-001 | 是否新增 `cancelled` 状态 | 已按建议执行；后续长任务取消需要，已写入 04 |
| C-CORE-006-002 | 是否新增 `finished_at/created_at/updated_at` | 已按建议执行；任务生命周期更完整，已写入 04 |
| C-CORE-006-003 | 本任务是否先不做 task IPC 和页面 | 已按建议执行；本次只做底层 service |
| C-CORE-006-004 | 启动时是否把 running 任务修复为 failed | 已按建议执行；参考 Toonflow fixDB，避免重启后一直卡“进行中” |
| C-CORE-006-005 | 项目名称 join 是否等项目表任务再接 | 已按建议执行；项目表还没建，本任务只按 project_id 筛选 |

## 11. 执行后记录

```txt
已新增：
src/main/services/task/constants.ts
src/main/services/task/types.ts
src/main/services/task/migrations.ts
src/main/services/task/mapper.ts
src/main/services/task/service.ts
src/main/services/task/index.ts

已修改：
src/main/services/database/migrations.ts
src/main/index.ts
src/shared/constants/status.ts
docs/03-执行进度.md
docs/04-对齐验收与偏差记录.md

实际完成：
1. 新增 tasks 表迁移，字段包含 project_id/category/related_objects/model_name/description/status/started_at/finished_at/error_reason/created_at/updated_at。
2. 新增任务状态 running/succeeded/failed/cancelled。
3. 新增 createTask/succeedTask/failTask/cancelTask/isTaskCancelled/getTaskDetail/listTasks/getTaskCategories/updateTaskMeta/recoverRunningTasks。
4. 任务完成、失败、取消只允许从 running 流转，重复操作会抛任务状态冲突。
5. 启动时在数据库迁移后调用 recoverRunningTasks，把 running 改 failed。
6. 新增 TASK_STATUS_CONFLICT 状态码文案，内部用于细分错误；对外 IPC 仍统一返回 code 400。

验证：
1. D:\software\nodejs\pnpm.cmd run typecheck 通过。
2. D:\software\nodejs\pnpm.cmd run build 通过。

未做：
1. 未新增任务 IPC。
2. 未新增任务中心页面。
3. 未接项目表名称 join。
4. 未实现真实模型请求中断。
```

## 12. 最后大白话

```txt
我这次准备怎么做：
1. 建一个 tasks 表，用来统一记录所有异步任务。
2. 做 task service，提供创建、成功、失败、取消、详情、列表、分类、启动修复这些操作。
3. 任务失败时统一写失败原因，任务列表能按项目、分类、状态筛选。
4. 启动时把上次没跑完的 running 任务改成 failed，避免一直卡住。

我不会做什么：
1. 不做任务中心页面。
2. 不做模型调用。
3. 不做任务重试。
4. 不做暂停/恢复。
5. 不做任务进度百分比。
6. 不做业务表状态同步。

确认规则：
用户确认后才执行；未确认前只改文档。
```
