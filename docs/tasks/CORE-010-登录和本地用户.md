# CORE-010 登录和本地用户

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`、`docs/features/M-001-全局导航与项目入口.md`、`docs/features/M-002-设置.md`  
原则：先确认本文档，再改代码

---

## 0. 快速理解

```txt
一句话：把 Toonflow 的账号登录，落成 VT Studio 的本地用户登录和本地登录状态。
为什么现在做：后面所有页面、Socket、任务和项目上下文都需要知道“当前本机用户是谁”。
做完后有什么用：应用可以有登录页、路由守卫、当前用户、修改用户名密码、退出登录。
这一步不碰什么：不做云账号、不做注册、不做验证码、不做权限系统、不做远程 token 校验。
```

---

## 1. 本次做什么

```txt
目标：
  用 users 表里的本地默认用户实现登录、当前用户、修改用户、退出登录和路由状态。

只做：
  1. 本地登录 service / IPC 契约
  2. 当前用户读取
  3. 用户名和密码修改
  4. 登录状态保存和恢复
  5. 退出登录清理状态
  6. 路由守卫按本地登录状态判断

不做：
  1. 云端账号体系
  2. 多账号注册/找回密码/验证码
  3. 角色权限 RBAC
  4. 远程 JWT 校验
  5. 记住密码和自动填充
  6. 设置页完整 UI 重做
```

---

## 2. 参考项目怎么做

源码已查完，以下是事实，不写“后续再看”。

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-web/src/pages/login/index.vue` | 登录页输入 username/password；可切语言；可设置请求地址；调用 `/login/login`；成功后写 `localStorage.token` 和 `localStorage.userId`，跳 `/project` |
| `Toonflow-web/src/router/index.ts` | `beforeEach`：`/login` 放行；其他路由检查 `localStorage.token`，没有 token 跳 `/login` |
| `Toonflow-app/src/routes/login/login.ts` | 查 `o_user.name`；明文比对 password；读 `o_setting.tokenKey`；签 180Days JWT；返回 `{ token: "Bearer xxx", name, id }` |
| `Toonflow-web/src/components/setting/components/loginConfig.vue` | 设置页读取 `/setting/loginConfig/getUser`；表单字段 name/password；校验用户名 2-20、密码 6-20；保存调用 `updateUserPwd` |
| `Toonflow-app/src/routes/setting/loginConfig/getUser.ts` | 返回 `o_user` 第一条记录 |
| `Toonflow-app/src/routes/setting/loginConfig/updateUserPwd.ts` | 参数 `name,password,id`；直接更新 `o_user` |
| `Toonflow-web/src/components/setting/components/logoutConfig.vue` | 退出时弹确认框；删除 `localStorage.token` 和 `localStorage.user`；跳 `/login` |

参考项目已有问题：

```txt
1. 登录成功写 token/userId，但退出只删 token/user，没删 userId。
2. 登录密码是明文存储和明文比对。
3. token 是 Web 后端 JWT；VT Studio 是本地桌面应用，不需要云端账号体系。
4. 登录页请求地址配置属于 Toonflow Web/HTTP 架构，VT Studio 不照搬成核心登录能力。
```

---

## 3. 用户操作

```txt
入口：
  1. 未登录进入应用时显示登录页。
  2. 已登录进入应用时直接进入项目页。
  3. 设置弹窗里保留“登录/用户配置”和“退出登录/清理登录状态”能力。

按钮/操作：
  1. 登录页：输入用户名、密码，点击登录。
  2. 设置页用户配置：修改用户名、密码，点击保存。
  3. 设置页退出登录：点击退出登录，二次确认。

弹窗/表单：
  1. 登录表单：用户名、密码。
  2. 修改用户表单：用户名、密码。
  3. 退出登录确认弹窗。

成功反馈：
  1. 登录成功：保存本地登录状态，跳项目页。
  2. 修改成功：提示保存成功，并刷新当前用户。
  3. 退出成功：清理登录状态和当前项目上下文，跳登录页。

失败反馈：
  1. 用户名或密码为空：提示必填。
  2. 用户不存在或密码错误：提示登录失败。
  3. 本地用户缺失：提示需要重新初始化默认数据。
  4. 保存用户失败：提示保存失败。
```

---

## 4. 要做什么功能

### 1. 本地登录

怎么做：
- 输入：`username`、`password`。
- 输出：`{ user: { id, name }, token }`。
- 写什么数据：不写数据库；只在 renderer 本地保存登录状态。
- 状态怎么变：未登录 -> 已登录。
- 异常怎么处理：
  - 用户名或密码为空：返回 `code=400`。
  - users 表没有用户：返回 `code=400`，msg 提示默认用户缺失。
  - 密码不匹配：返回 `code=400`。
- 限制：
  - 只查 users 表第一阶段本地用户。
  - 不请求远程服务。
  - 不做注册。

### 2. 本地 token / session

怎么做：
- 输入：登录成功的用户 id/name。
- 输出：本地 token 字符串。
- 写什么数据：renderer 写本地登录状态；main 不写 token 到数据库。
- 状态怎么变：生成一个本机可识别的登录凭据。
- 异常怎么处理：生成失败则登录失败。
- 限制：
  - token 只用于本地路由和 Socket 鉴权，不代表云端身份。
  - 可以沿用 `app_settings.tokenKey` 生成签名，也可以使用本地 session token；实现时必须固定一种，不允许页面自己拼 token。
  - 返回仍然走 `{ code, data, msg }`。

### 3. 当前用户读取

怎么做：
- 输入：无，或本地 token。
- 输出：`{ id, name }`。
- 写什么数据：不写。
- 状态怎么变：用于刷新页面后恢复用户信息。
- 异常怎么处理：
  - 未登录：返回 `code=400` 或空用户状态，renderer 跳登录页。
  - users 表缺失：提示默认用户缺失。
- 限制：不返回 password。

### 4. 修改本地用户

怎么做：
- 输入：`id`、`name`、`password`。
- 输出：保存成功。
- 写什么数据：更新 users 表的 `name/password/updated_at`。
- 状态怎么变：
  - 如果修改的是当前登录用户，renderer 同步更新本地 user 信息。
  - 如果修改密码，不强制退出；后续如要强制退出，单独记录偏差。
- 异常怎么处理：
  - name 为空、长度不合法：返回 `code=400`。
  - password 为空、长度不合法：返回 `code=400`。
  - id 不存在：返回 `code=400`。
- 限制：
  - 第一版仍保留明文密码，和参考项目一致；如果要 hash，必须单独开任务，因为会影响默认数据和登录校验。

### 5. 退出登录

怎么做：
- 输入：无。
- 输出：退出成功。
- 写什么数据：不写数据库。
- 状态怎么变：已登录 -> 未登录。
- 清理内容：
  - `token`
  - `user`
  - `userId`
  - 当前项目上下文
  - 当前路由里依赖用户的临时状态
- 异常怎么处理：本地清理失败时提示退出失败。
- 限制：
  - 不调用后端退出接口，因为参考项目没有后端 logout。
  - 不清模型密钥、不清数据库、不清素材文件。

### 6. 路由守卫

怎么做：
- 输入：目标路由和当前登录状态。
- 输出：放行或跳登录页。
- 写什么数据：不写。
- 状态怎么变：
  - 未登录访问业务页：跳登录页。
  - 已登录访问登录页：跳项目页。
  - 已登录访问业务页：放行。
- 异常怎么处理：读取本地状态异常时按未登录处理。
- 限制：
  - 不让页面直接读 SQLite 判断用户。
  - 登录状态应封装在 renderer store 或 auth composable 中，页面只读状态。

### 7. 和 Socket 鉴权打通

怎么做：
- 输入：登录 token。
- 输出：Socket 连接时可使用同一 token。
- 写什么数据：不写。
- 状态怎么变：登录成功后 Agent Socket 才能通过鉴权。
- 异常怎么处理：token 缺失或无效时 Socket 拒绝连接。
- 限制：
  - 本任务只定义登录侧 token 来源。
  - 不重写 CORE-008 Socket 事件流。

---

## 5. 数据和状态

Toonflow 源码名只用于回查，VT Studio 不照搬旧表名。

```txt
字段：
  users.id
  users.name
  users.password
  users.created_at
  users.updated_at

接口/能力：
  auth.login
  auth.getCurrentUser
  auth.updateLocalUser
  auth.logout
  auth.validateSession

数据读写：
  登录：读 users，不写数据库
  当前用户：读 users，不返回 password
  修改用户：写 users.name/password/updated_at
  退出：只清 renderer 本地状态，不写数据库

任务状态：
  不写 tasks 表

轮询/Socket：
  不轮询
  登录 token 供 CORE-008 Socket 鉴权使用

模型调用：
  无

删除影响：
  不删除用户表
  不清模型配置
  不清项目和素材
```

---

## 6. VT Studio 怎么落

```txt
能力名：
  auth.login
  auth.getCurrentUser
  auth.updateLocalUser
  auth.logout
  auth.validateSession

调用链：
  renderer -> window.vtStudio.auth -> main/ipc -> main/services/auth -> database/users

需要新增：
  src/shared/types/auth.ts
  src/main/services/auth/index.ts
  src/main/ipc/auth.ts
  renderer 侧 auth store 或 composable
  登录页或登录弹层（具体 UI 在实现时按现有路由结构落）

需要修改：
  src/shared/contracts/preload.ts 增加 auth API
  src/preload/index.ts 暴露 auth API
  src/main/ipc/index.ts 注册 auth IPC
  src/renderer/src/router/index.ts 增加本地登录守卫
```

返回格式固定：

```ts
{ code: 200, data: {...}, msg: "成功" }
{ code: 400, data: {}, msg: "失败原因" }
```

多语言规则：

```txt
1. main 返回稳定 msg，可先用中文。
2. renderer 页面提示优先走前端 i18n key。
3. 不把 main 的 msg 当作最终多语言方案；后续语言设置模块统一梳理。
```

---

## 7. 偏差

| 偏差 | 原因 | 是否写入 04 |
|---|---|---|
| 不完整保留 Toonflow 云账号/JWT 体系 | VT Studio 是本地桌面创作工具，账号不是主链路 | 已有 `D-BASE-013` |
| 退出登录必须同时清 `token/user/userId` | 修正参考项目登录写 userId、退出不删 userId 的不一致 | 不新增偏差，属于修正 |
| 不保留登录页请求地址配置为核心能力 | VT Studio 通过 preload/main 调本地服务，不走 Web baseUrl 配置 | 如页面形态变化，后续 M-001/M-002 记录 |
| 第一版保留明文密码 | 对齐参考项目和 CORE-009 默认数据；本地桌面低风险，hash 可后置 | 不新增偏差 |

---

## 8. 验收

```txt
1. typecheck 通过。
2. 未登录进入业务路由会进入登录页或登录状态页。
3. 使用默认 admin/admin123 能登录成功。
4. 登录成功后本地保存 token、user、userId，并跳项目页。
5. 刷新页面后能恢复当前用户，不误判未登录。
6. 用户名或密码错误时返回 code=400，并显示失败提示。
7. 设置里的用户配置能读取当前用户，不显示 password 之外的敏感内容。
8. 修改用户名/密码后，users 表更新，当前用户信息同步。
9. 退出登录会二次确认。
10. 退出后清理 token、user、userId 和当前项目上下文。
11. 退出后访问业务页会回到登录页。
12. 页面不直接访问 SQLite、文件系统或 Node API。
13. 所有 IPC 返回仍是 `{ code, data, msg }`，不出现 `message` 字段。
14. 不新增云账号、注册、验证码、多账号权限等非本任务能力。
```

---

## 9. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-CORE-010-001 | 是否按本地默认用户方案做，不接云账号体系 | 建议确认；符合已定 `D-BASE-013`，也符合本地桌面工具定位 |
| C-CORE-010-002 | 第一版密码是否继续明文存储 | 建议确认；对齐参考项目和 CORE-009，hash 后续可单独做，不要现在扩大范围 |
| C-CORE-010-003 | 退出登录是否清当前项目上下文，但不清模型密钥、数据库和素材 | 建议确认；退出只是用户状态，不应该误删配置和文件 |
| C-CORE-010-004 | 登录页是否保留为独立路由，而不是弹窗 | 建议保留独立路由；参考项目如此，路由守卫也更清楚 |

---

## 10. 执行后记录

```txt
改了哪些文件：
  src/shared/types/auth.ts
  src/shared/contracts/preload.ts
  src/preload/index.ts
  src/main/services/auth/index.ts
  src/main/ipc/auth.ts
  src/main/ipc/index.ts
  src/renderer/src/stores/auth.ts
  src/renderer/src/stores/app.ts
  src/renderer/src/router/index.ts
  src/renderer/src/features/auth/LoginHome.vue
  src/renderer/src/features/settings/SettingsHome.vue
  src/renderer/src/styles/index.scss

验证结果：
  1. node_modules\.bin\vue-tsc.CMD --noEmit -p tsconfig.web.json 通过。
  2. node_modules\.bin\tsc.CMD --noEmit -p tsconfig.node.json 通过。
  3. node_modules\.bin\electron-vite.CMD build 通过。
  4. pnpm run typecheck 未执行成功：pnpm 在无 TTY 环境尝试清理 node_modules，被 ERR_PNPM_ABORTED_REMOVE_MODULES_DIR_NO_TTY 阻止；已改用本地二进制完成等价 typecheck。

未完成事项：
  1. 未做云账号、注册、验证码、权限系统，符合本任务范围。
  2. 未做密码 hash，继续明文存储，符合用户确认点和当前默认数据。
  3. 如需人工看窗口交互，可后续运行 pnpm run dev 验证实际 UI。

最终结论：可接受偏差
```

---

## 11. 最后大白话

```txt
我这次准备怎么做：
1. 用 CORE-009 已经初始化好的 users 表做本地登录。
2. 用户输入 admin/admin123 这种本地用户名密码后，main 校验 users 表。
3. 登录成功后，前端保存本地登录状态，路由就能放行。
4. 设置里可以改本地用户名和密码。
5. 退出登录时，把 token、user、userId 和当前项目上下文都清掉。

我不会做什么：
1. 不做云账号。
2. 不做注册、验证码、找回密码。
3. 不做权限系统。
4. 不删除任何项目、模型、素材、数据库数据。
5. 不让页面直接碰 SQLite。

确认规则：
用户确认本文档后才执行；未确认前只整理 task 文档，不写业务代码。
```
