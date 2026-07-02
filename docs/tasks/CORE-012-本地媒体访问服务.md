# CORE-012 本地媒体访问服务

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`、`docs/features/M-006-资产中心.md`、`docs/features/M-008-生产工作台.md`、`docs/features/M-010-导出.md`  
原则：先确认本文档，再改代码

---

## 0. 快速理解

```txt
一句话：给本地图片、视频、音频做一个受控访问服务，让页面能安全预览媒体文件。
为什么现在做：资产中心、生产工作台、导出都会显示图片/视频/音频；没有统一服务，后面每个模块都会乱拼路径。
做完后有什么用：页面拿到的是受控 media URL，不是系统绝对路径；视频支持拖进度条，图片支持真缩略图。
这一步不碰什么：不做素材上传、不做图片生成、不做业务表、不做资产 UI、不做剪映导出。
```

---

## 1. 本次做什么

```txt
目标：
  在已有本地 HTTP server 上增加 /media 访问能力，统一处理本地图片、视频、音频预览。

只做：
  1. 媒体 URL 生成
  2. 媒体 URL 签名 token 校验
  3. 媒体文件读取
  4. MIME 判断
  5. HTTP Range 分段读取（视频/音频拖进度条）
  6. 图片缩略图生成和缓存
  7. URL 反解为受控 root + relativePath
  8. 访问失败时返回清晰错误

不做：
  1. 上传素材
  2. 删除素材
  3. 图片/视频/音频生成
  4. 写业务数据库
  5. 资产中心 UI
  6. 生产工作台 UI
  7. 导出逻辑
  8. 外网访问
```

---

## 2. 参考项目怎么做

源码已查完，以下是事实。

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-app/src/utils/oss.ts` | 文件存在 `getPath("oss")`；`getFileUrl(path)` 返回 `/oss/xxx` 或本地 HTTP 地址；`writeFile/getFile/deleteFile/deleteDirectory/fileExists` 都基于相对路径和 rootDir |
| `Toonflow-app/src/utils/oss.ts#getSmallImageUrl` | 缩略图生成逻辑整段注释掉了；当前实际只返回原图 URL 加 `?size=20`，不是真缩略图 |
| `Toonflow-app/src/utils/image.ts` | 用 sharp 实现 `resizeImage/ensureThumbnail`，支持固定尺寸和百分比缩放 |
| `Toonflow-app/src/utils/replaceUrl.ts` | 把 URL 或 `/oss/...` 转成相对文件路径，去 query，并阻止 `..` 路径穿越 |
| `Toonflow-app/src/routes/common/getBigImage.ts` | 前端传缩略图 URL，后端反解路径，再返回原图 URL |
| `Toonflow-app/src/routes/production/workbench/getFileUrl.ts` | 根据 storyboard/assets id 查 `filePath`，调用 `u.oss.getSmallImageUrl` 返回 URL 映射 |
| `Toonflow-app/src/routes/assets/uploadClip.ts` | 上传 base64 文件，写到 `/${projectId}/assets/${uuid}.${ext}`，再写资产和图片表 |

参考项目已有问题：

```txt
1. /oss 是裸本地 HTTP，没有 token 校验。
2. URL 用 localhost + 固定/环境端口，安全边界弱。
3. 缩略图是假实现，列表实际仍加载原图。
4. getSmallImageUrl 只给 URL 加 ?size=20，不会减少图片体积。
5. 部分路径反解和删除逻辑容易因为先删库再查路径而失效。
6. OSS 这个名字容易误解，实际是本地文件存储。
```

VT Studio 结论：

```txt
不照搬 OSS 命名。
不暴露裸 /oss。
统一使用 127.0.0.1 本地媒体服务 + token + 受控相对路径。
图片、视频、音频都走同一个媒体访问服务。
```

---

## 3. 用户操作

本任务是底层服务，没有单独页面。

```txt
入口：
  1. 资产中心后续显示图片、视频、音频。
  2. 生产工作台后续显示分镜图、资产图、视频候选、音频素材。
  3. 导出模块后续校验和预览素材。

按钮/操作：
  本任务不新增按钮。

弹窗/表单：
  本任务不新增弹窗。

成功反馈：
  页面能用 media URL 正常显示图片、播放视频/音频。

失败反馈：
  1. 文件不存在：显示素材缺失。
  2. token 无效或过期：显示无权限或刷新 URL。
  3. MIME 不支持：显示不支持的媒体类型。
  4. 缩略图生成失败：降级返回原图 URL。
```

---

## 4. 要做什么功能

### 1. 媒体 URL 生成

怎么做：
- 输入：受控根目录 `project|cache|temp|exports`、相对路径、可选模式 `original|thumbnail`、可选缩略图尺寸 `small|list|detail`。
- 输出：`http://127.0.0.1:{port}/media/{encodedResource}?mode=...&size=...&expires=...&token=...`。
- 写什么数据：不写数据库。
- 状态怎么变：无。
- 异常怎么处理：
  - 相对路径为空：返回失败。
  - 路径越界：返回失败。
  - 本地服务未启动：返回失败。
- 限制：
  - 不返回系统绝对路径。
  - 不使用 localhost，固定 127.0.0.1。
  - URL 和 token 由 main 生成，renderer 不自己拼。
  - `encodedResource` 必须由 root + relativePath 编码生成，不能直接暴露 Windows 绝对路径。

### 2. 媒体 URL 签名 token 校验

怎么做：
- 输入：URL 中的 `encodedResource`、`mode`、`size`、`expires`、`token`。
- 输出：校验通过或拒绝。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：
  - token 缺失、无效、被篡改：返回 401。
  - token 过期：返回 401。
- 限制：
  - 媒体服务只服务本机，不对外网开放。
  - token 不等同云账号 token，只是本地媒体访问凭据。
  - 不复用 CORE-010 登录 token。
  - 第一版使用运行期 media secret 做 HMAC 签名。
  - token 必须绑定 root、relativePath、mode、size、expires，不能做全局通用 token。
  - renderer 即使拿到某个 media URL，也不能改 path 后复用同一个 token。

### 3. 路径安全和 URL 反解

怎么做：
- 输入：media URL、URL path 或旧 `/oss/...` 风格路径。
- 输出：受控 root + relativePath。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：
  - `..` 路径穿越：拒绝。
  - 绝对路径：拒绝。
  - 空路径：拒绝。
- 限制：
  - 只允许访问 CORE-004 管理的项目、缓存、临时目录、导出目录。
  - 不允许访问 database、logs、vendors 等非媒体目录。
  - 新 media URL 反解必须校验签名；旧 URL 兼容只做路径转换，不绕过新 URL 生成。

### 4. 媒体文件读取

怎么做：
- 输入：受控 root + relativePath。
- 输出：HTTP 响应流。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：
  - 文件不存在：404。
  - 不是文件：400。
  - 读取失败：500 或统一错误。
- 限制：
  - 不一次性把大视频读进内存。
  - 不把错误 detail 暴露给页面，绝对路径只进日志。
  - 图片、视频、音频都用 stream，不用 `readFile` 一次性读完整文件。

### 5. MIME 判断

怎么做：
- 输入：文件扩展名，必要时结合文件头。
- 输出：`Content-Type`。
- 支持类型：
  - 图片：jpg/jpeg/png/webp/gif/bmp/svg
  - 视频：mp4/webm/mov
  - 音频：mp3/wav/m4a/aac/ogg/webm
- 异常怎么处理：
  - 不支持的类型：返回 415。
- 限制：
  - 不因为扩展名伪装就执行任何文件。
  - SVG 要谨慎，第一版只作为 `image/svg+xml` 文件流返回，不内联到 DOM，不做脚本执行。

### 6. HTTP Range 分段读取

怎么做：
- 输入：`Range` header。
- 输出：206 Partial Content。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：
  - Range 不合法：返回 416。
  - 无 Range：返回完整文件流。
- 限制：
  - 视频和音频必须支持 Range，否则页面拖进度条体验会差。
  - 图片可以不使用 Range。

### 7. 图片缩略图

怎么做：
- 输入：原图 root + relativePath、缩略图尺寸。
- 输出：缩略图 media URL。
- 写什么数据：写缓存目录下的缩略图文件。
- 状态怎么变：
  - 缩略图不存在：生成并缓存。
  - 缩略图已存在：直接返回。
- 异常怎么处理：
  - 原图不存在：返回失败。
  - sharp 生成失败：降级返回原图 URL，并记录 warn。
- 限制：
  - 不能像参考项目一样只加 `?size=20`。
  - 缩略图保存到已有运行目录 `cache/thumbnails`，不污染项目素材目录。
  - 缩略图 key 要和原图路径、mtime、尺寸有关，避免原图更新后还用旧缩略图。

### 8. 原图 URL

怎么做：
- 输入：缩略图 URL 或 root + relativePath。
- 输出：原图 media URL。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：反解失败返回失败。
- 限制：
  - 对齐参考项目 `getBigImage` 的业务意图，但不沿用裸 `/oss`。

### 9. 旧 URL 兼容

怎么做：
- 输入：旧格式 `/oss/...`、`/smallImage/...`、带 query 的 URL。
- 输出：受控相对路径或新 media URL。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：无法反解就返回空或失败。
- 限制：
  - 只用于迁移/兼容，新的业务数据不保存旧 URL。

### 10. 服务启动和关闭

怎么做：
- 输入：已有 local HTTP server。
- 输出：挂载 `/media` 处理器。
- 写什么数据：不写。
- 状态怎么变：随应用启动和退出。
- 异常怎么处理：
  - server 未启动：媒体 URL 生成失败。
  - 关闭时有请求：让 Node 正常 close，不强杀。
- 限制：
  - 不单独开第二个端口。
  - 只监听 127.0.0.1。
  - 和 CORE-008 Socket 共用同一个本地 server，但路由职责分开。

---

## 5. 数据和状态

```txt
字段：
  不新增业务字段。
  后续业务表保存 relativePath/filePath，不保存 media URL。

接口/能力：
  media.createUrl
  media.createThumbnailUrl
  media.resolveUrlToPath
  media.getOriginalUrl
  media.ensureThumbnail
  media.verifySignedUrl

数据读写：
  读项目/素材相对路径对应文件
  写 cache/thumbnails 缩略图

任务状态：
  不写 tasks 表。

轮询/Socket：
  不轮询，不新增 Socket。

模型调用：
  无。

删除影响：
  不删除原始素材。
  缩略图缓存可被清理，但本任务不做清理入口。
```

---

## 6. VT Studio 怎么落

```txt
能力名：
  media

调用链：
  renderer -> window.vtStudio.media.createUrl -> main/ipc -> media service -> local HTTP /media
  browser/video/img/audio -> http://127.0.0.1:{port}/media/... -> media request handler -> file stream

需要新增：
  src/main/services/media/index.ts
  src/main/services/media/url.ts
  src/main/services/media/request-handler.ts
  src/main/services/media/security.ts
  src/main/services/media/thumbnail.ts
  src/main/services/media/mime.ts
  src/shared/types/media.ts
  src/main/ipc/media.ts
  scripts/verify-core-012.mjs

需要修改：
  src/main/app/server.ts 挂载 media request handler
  src/main/ipc/index.ts 注册 media IPC
  src/shared/contracts/preload.ts 增加 media API
  src/preload/index.ts 暴露 media API
  src/main/services/file-system/index.ts 如需补导出类型
```

对外 IPC 返回仍固定：

```ts
{ code: 200, data: { url: "http://127.0.0.1:xxxx/media/..." }, msg: "成功" }
{ code: 400, data: {}, msg: "文件不存在" }
```

服务端 HTTP 响应：

```txt
200：完整文件
206：Range 分段
400：路径非法
401：token 无效
404：文件不存在
415：不支持的媒体类型
416：Range 不合法
```

---

## 7. 偏差

| 偏差 | 原因 | 是否写入 04 |
|---|---|---|
| 不照搬 `/oss` 裸访问，改为 `/media` + token | 防止任意本地文件被页面或外部访问 | 已有 `D-BASE-012` |
| 缩略图做真文件，不用 `?size=20` 假参数 | 列表性能需要真实缩略图 | 已有 `D-BASE-012` |
| 不使用 localhost，固定 127.0.0.1 | 避免 localhost 解析差异和外部代理影响 | 不新增偏差 |
| 不把 media URL 存数据库 | URL 带端口和 token，会过期；业务表应保存相对路径 | 不新增偏差 |

---

## 8. 验收

```txt
1. typecheck 通过。
2. build 通过。
3. 本地 server 只监听 127.0.0.1。
4. createUrl 返回 http://127.0.0.1:{port}/media/...，不返回绝对路径。
5. token 缺失、错误、过期或和 URL 参数不匹配时，/media 返回 401。
6. 路径带 ../、盘符、UNC、空字节时拒绝访问。
7. 访问不存在文件返回 404。
8. 图片文件返回正确 Content-Type。
9. 视频文件支持 Range，返回 206 和 Content-Range。
10. 音频文件支持 Range，返回 206 和 Content-Range。
11. 缩略图第一次访问会生成缓存文件。
12. 缩略图再次访问复用缓存。
13. 缩略图生成失败时降级原图，不导致页面崩溃。
14. 原图 URL 能从缩略图/旧 URL 反解。
15. 业务数据库不保存 media URL。
16. 不新增素材上传、资产表、生产表、导出逻辑。
17. verify-core-012 覆盖 createUrl、签名校验、Range、缩略图、路径逃逸和旧 URL 反解。
```

---

## 9. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-CORE-012-001 | 媒体访问是否统一走本地 HTTP `/media`，不使用 Electron protocol | 已确认：统一走本地 HTTP `/media` |
| C-CORE-012-002 | 是否固定 127.0.0.1 + token 校验 | 已确认：固定 127.0.0.1 + token |
| C-CORE-012-003 | 缩略图是否做真实缓存文件，不照搬 `?size=20` | 已确认：做真实缩略图缓存 |
| C-CORE-012-004 | 数据库是否只保存相对路径，不保存 media URL | 已确认：业务数据只保存相对路径 |
| C-CORE-012-005 | 本任务是否不做上传/删除/生成，只做访问和缩略图 | 已确认：本任务不做上传/删除/生成 |
| C-CORE-012-006 | token 是否采用“按 URL 签名”的短期 token，而不是全局通用 token | 已确认：采用 URL 级签名 token |

---

## 10. 执行后记录

```txt
改了哪些文件：
  1. 新增 src/shared/types/media.ts。
  2. 新增 src/main/services/media/path.ts、security.ts、mime.ts、thumbnail.ts、url.ts、request-handler.ts、index.ts。
  3. 修改 src/main/app/server.ts，挂载 /media HTTP handler。
  4. 新增 src/main/ipc/media.ts，并修改 src/main/ipc/index.ts 注册 media IPC。
  5. 修改 src/shared/contracts/preload.ts 和 src/preload/index.ts，暴露 window.vtStudio.media。
  6. 新增 scripts/verify-core-012.mjs。

验证结果：
  1. node scripts\verify-core-012.mjs 通过。
  2. corepack pnpm@10.24.0 run typecheck 通过。
  3. node scripts\verify-f-002-017.mjs 通过。
  4. node scripts\verify-f-002-004.mjs 通过。
  5. corepack pnpm@10.24.0 run build 通过。

未完成事项：
  1. 不做素材上传、删除、生成，后续归资产和生产模块。
  2. 不做缩略图缓存清理入口，后续可在文件管理或缓存清理任务中处理。
  3. 当前只提供底层 media API，资产中心/生产工作台页面后续接入时必须使用 thumbnail URL 和 media URL。

最终结论：
  CORE-012 已完成。本地媒体访问服务已支持 127.0.0.1 /media、URL 级签名 token、受控 root、Range 分段读取、真实缩略图缓存、旧 /oss 反解和 preload API。
```

---

## 11. 最后大白话

```txt
我这次准备怎么做：
1. 在本地 127.0.0.1 服务上加一个 /media 地址。
2. 页面以后看图片、视频、音频，都拿 /media URL，不拿 C:\xxx 这种绝对路径。
3. URL 带 token，没 token 或 token 错就不给看。
4. 视频和音频支持 Range，这样能拖进度条。
5. 图片列表用真正缩略图，不再加载原图假装小图。
6. 数据库还是只存相对路径，不存临时 URL。

我不会做什么：
1. 不做素材上传。
2. 不做素材删除。
3. 不做图片/视频/音频生成。
4. 不做资产中心页面。
5. 不做生产工作台页面。
6. 不做剪映导出。

确认规则：
用户确认本文档后才执行；未确认前只整理 task 文档，不写业务代码。
```

## 12. 本轮补充

```txt
参考项目 imageOptimizer 是前端图片性能插件，不属于媒体服务本身。
但 CORE-012 做缩略图后，资产中心/生产工作台必须优先用 thumbnail URL。
列表图片仍要设置 loading=lazy、decoding=async，避免一次性加载大图。
```

## 13. 补充收口点

这一节把媒体服务的安全和复用规则写死，避免后续资产、生产、导出各自拼文件路径。

### 13.1 依赖谁

```txt
CORE-004：受控根目录、路径标准化、路径逃逸校验。
CORE-008：已有本地 HTTP server，媒体路由挂在同一个 127.0.0.1 服务上。
CORE-010：可复用本地登录 token，但 media token 语义必须独立说明。
CORE-013：所有访问失败、路径非法、缩略图失败写统一日志，终端只给短提示。
```

### 13.2 被谁依赖

```txt
F-006-006 素材预览：图片、视频、音频都必须走 media URL。
F-006-008/F-006-009 图片生成结果预览：保存相对路径，展示走 media URL。
F-007 角景音频绑定：角色图、场景图、音频试听走 media URL。
F-008 生产工作台：分镜图、资产图、视频候选、音频素材走 media URL。
F-010 导出：素材校验读取文件路径，预览展示仍走 media URL。
```

### 13.3 受控根目录

```txt
允许访问：
  project：项目业务目录
  cache：缓存目录
  thumbnails：缩略图缓存目录
  temp：临时媒体目录
  exports：导出预览目录

禁止访问：
  database
  logs
  vendors
  任意绝对路径
  项目源码目录
  用户未显式选择且不在受控根下的目录
```

media URL 不保存到数据库：

```txt
数据库保存 root + relative_path，或保存业务相对路径。
renderer 展示前调用 media.createUrl/createThumbnailUrl。
端口变化、token 变化、开发/生产 userData 变化都不能影响业务数据。
```

### 13.4 token 规则

```txt
media secret 由 main 在本次应用运行期生成。
renderer 只能通过 window.vtStudio.media 获取 URL，不能自己拼 token。
token 由 root、relativePath、mode、size、expires 计算签名。
token 至少要绑定本次应用运行实例，应用重启后旧 token 失效。
token 过期或无效返回 401。
token 不写数据库，不写日志。
```

专业建议：

```txt
第一版使用“应用运行期 media secret + URL 级 HMAC 签名”：应用启动生成 secret，退出失效。
不做长生命周期 token，不做云端权限。
原因：这是本地桌面预览服务，不是对外资源服务；URL 级签名比全局 token 更能防止路径被篡改。
```

### 13.5 路径编码和反解

```txt
createUrl 输入必须是 root + relativePath，不接受任意绝对路径。
root 第一版只允许 project、cache、temp、exports。
thumbnail 不作为业务 root，缩略图由 mode=thumbnail 映射到 cache/thumbnails。
URL path 必须 encode，不能直接拼中文、空格、反斜杠。
resolveUrlToPath 必须去掉 query/hash。
旧 /oss 和 ?size=20 只用于兼容，不作为新数据写入。
发现 ..、盘符、UNC 路径、空字节，直接拒绝。
```

### 13.6 Range 规则

```txt
视频和音频必须支持 Range。
Range 合法：返回 206、Content-Range、Accept-Ranges、Content-Length。
Range 缺失：返回 200 完整流。
Range 超界或格式非法：返回 416。
不能一次性 readFile 大视频。
```

### 13.7 缩略图规则

```txt
缩略图只针对图片。
缩略图目录：cache/thumbnails。
缩略图 key 必须包含 root、relativePath、mtime、size，避免原图更新后缓存不刷新。
默认尺寸至少支持 small/list/detail 三档，具体像素由 service 常量统一定义。
生成失败：记录 warn，返回原图 URL，不让页面崩。
删除原图后：缩略图可以残留在缓存，后续缓存清理任务再处理；本任务不做清理入口。
```

### 13.8 MIME 和 SVG

```txt
MIME 先按扩展名判断，必要时可补文件头检查。
不支持的类型返回 415。
SVG 第一版建议默认不内联、不执行脚本，只作为 image/svg+xml 文件流返回。
如果后续要清洗 SVG，单独做安全增强任务。
```

### 13.9 服务契约

```txt
media.createUrl
  输入：root, relativePath, expiresInSeconds?
  输出：{ url }
  失败：root 不允许、路径非法、服务未启动

media.createThumbnailUrl
  输入：root, relativePath, size, expiresInSeconds?
  输出：{ url, fallback }
  失败：原图不存在、路径非法

media.resolveUrlToPath
  输入：url
  输出：{ root, relativePath, mode, size? }
  失败：无法反解、签名无效、路径非法

media.getOriginalUrl
  输入：url 或 root + relativePath
  输出：{ url }
  失败：无法反解、文件不存在

media.verifySignedUrl
  输入：encodedResource, mode, size, expires, token
  输出：{ root, relativePath, mode, size? }
  失败：签名无效、过期、参数被篡改
```

### 13.10 严格不允许遗漏

```txt
不能返回 C:\xxx 绝对路径给 renderer。
不能监听 0.0.0.0。
不能使用裸 /oss 无 token 访问。
不能使用全局通用 token 允许任意改 path。
不能把 media URL 入库。
不能让业务页面自己拼 http://127.0.0.1:port/media。
不能读取 database/logs/vendors。
不能一次性读取大视频到内存。
不能把缩略图写进项目素材目录。
```
