# CORE-004 本地文件层

状态：已完成  
类型：基础任务  
所属模块：公共底层  
对应功能文档：`docs/features/00-公共底层.md`  
执行规则：已按用户确认执行；后续功能仍需先写 tasks 文档并确认

## 0. 快速理解

```txt
一句话：这一步是给整个软件定好“文件放哪、怎么读写、怎么防乱删”的底座。
为什么现在做：后面项目、素材、图片、音频、视频、导出都会用文件，不先统一后面会乱。
做完后有什么用：后续所有功能都走统一文件服务，不会把运行数据写到项目目录。
这一步不碰什么：不做页面、不做素材业务、不做剪映导出、不做模型生成。
```

## 1. 这一步只做什么

```txt
本任务只做“文件路径和本地文件读写基础层”。
它不做项目管理、不做素材中心、不做导出、不做图片生成、不做剪映草稿。
```

本任务要解决的问题：

```txt
后续项目、素材、图片、音频、视频、Agent、导出都会写文件。
如果现在不先统一路径规则，后面一定会出现：
1. 文件乱写到项目根目录。
2. 临时文件、数据库、素材混在一起。
3. renderer 直接传绝对路径读写文件。
4. 删除文件时误删非项目文件。
5. 导出、预览、模型生成各用一套路径。
```

## 2. 参考项目怎么做

参考源码：

```txt
D:\project\短视频\Toonflow-app-master\src\utils\getPath.ts
D:\project\短视频\Toonflow-app-master\src\utils\oss.ts
D:\project\短视频\Toonflow-app-master\src\app.ts
D:\project\短视频\Toonflow-app-master\src\utils\replaceUrl.ts
D:\project\短视频\Toonflow-app-master\src\routes\project\delProject.ts
D:\project\短视频\Toonflow-app-master\src\routes\assets\*
D:\project\短视频\Toonflow-app-master\src\routes\production\*
```

看到的事实：

| 参考位置 | Toonflow 事实 | VT Studio 怎么处理 |
|---|---|---|
| `utils/getPath.ts` | Electron 下统一放 `app.getPath("userData")/data` | 保留统一入口，但 VT Studio 直接使用已配置好的 `app.getPath("userData")` |
| `utils/getPath.ts` | 非 Electron 下放 `process.cwd()/data` | 不照搬；开发环境也不能写项目根目录 |
| `utils/getPath.ts` | 用 `is-path-inside` 防路径逃逸 | 必须保留同等能力 |
| `utils/oss.ts` | 文件统一写到 `getPath("oss")` | VT Studio 建议拆成更清楚的 `projects/assets/temp/exports/cache` |
| `utils/oss.ts` | 只接受相对路径，内部拼绝对路径 | 必须保留，不允许 renderer 传任意绝对路径写文件 |
| `utils/oss.ts` | 支持 `writeFile/getFile/deleteFile/deleteDirectory/fileExists` | VT Studio 本任务先封装这些基础能力 |
| `utils/oss.ts` | `getFileUrl/getSmallImageUrl` 返回本地 HTTP URL | VT Studio 先不启动 HTTP 静态服务；预览 URL 后续按 Electron 安全方案定 |
| `app.ts` | 启动时创建 `oss/skills/assets/web` 静态目录 | VT Studio 只创建基础目录，不提前暴露静态目录 |
| `replaceUrl.ts` | 把 `/oss/...` URL 转回相对文件路径 | VT Studio 后续如有预览 URL，也必须提供反解规则 |
| `project/delProject.ts` | 删除项目时删除 `${id}/` 文件夹 | VT Studio 删除项目时也要按项目归属删，但不在本任务做业务删除 |

参考项目的核心逻辑可以总结为：

```txt
getPath 负责确定根目录。
oss 负责所有文件读写。
业务表里保存相对路径。
页面拿到的是可预览 URL，不直接拿系统绝对路径。
```

VT Studio 要保留这个思路，但目录要更专业、更清楚。

## 3. 当前项目事实

已存在：

```txt
src/main/app/runtime.ts
src/main/services/database/*
src/main/ipc/*
src/shared/types/response.ts
src/shared/constants/status.ts
```

当前已有规则：

```txt
开发 userData：%TEMP%\VT Studio Dev\user-data
生产 userData：%LOCALAPPDATA%\VT Studio\user-data
数据库：userData/database/vt-studio.sqlite
IPC 返回：{ code, data, msg }
```

当前缺少：

```txt
没有统一文件服务。
没有项目目录规则。
没有素材目录规则。
没有临时目录规则。
没有导出目录规则。
没有安全拼路径工具。
没有路径逃逸校验。
没有受控删除工具。
没有文件存在、目录创建、文件复制基础能力。
没有文件层错误码。
```

## 4. 本次目标

本次只做本地文件基础层：

```txt
1. 统一 userData 下的业务文件目录。
2. 提供安全路径拼接和路径逃逸校验。
3. 提供目录创建、存在判断、文件复制、文件删除、目录删除基础能力。
4. 定义项目文件、素材文件、临时文件、导出文件的目录规则。
5. 保证 renderer/preload 不直接访问 fs/path。
6. 为后续项目管理、素材管理、模型生成、导出提供统一文件入口。
```

本任务不做 UI，也不做任何业务表。

## 5. 目录规则

基础根目录：

```txt
dev：%TEMP%\VT Studio Dev\user-data
prod：%LOCALAPPDATA%\VT Studio\user-data
```

建议目录：

```txt
user-data/
  database/
    vt-studio.sqlite
  projects/
    {projectId}/
      project.json
      source/
      assets/
        images/
        audio/
        video/
        clips/
        documents/
      generated/
        images/
        audio/
        video/
        storyboard/
      exports/
      temp/
  cache/
    thumbnails/
    model-test/
  temp/
  exports/
  logs/
```

目录含义：

| 目录 | 用途 | 本任务是否创建 |
|---|---|---|
| `database` | SQLite 文件 | 已由 CORE-003 管 |
| `projects` | 项目业务文件根目录 | 创建根目录 |
| `projects/{projectId}` | 单个项目目录 | 只提供解析函数，不创建具体项目 |
| `source` | 小说原文、导入文档等原始输入 | 只定规则 |
| `assets/images` | 用户导入或确认后的图片资产 | 只定规则 |
| `assets/audio` | 用户导入或确认后的音频资产 | 只定规则 |
| `assets/video` | 用户导入或确认后的视频素材 | 只定规则 |
| `assets/clips` | 片段素材 | 只定规则 |
| `generated/images` | 模型生成图片 | 只定规则 |
| `generated/audio` | 模型生成音频 | 只定规则 |
| `generated/video` | 模型生成视频 | 只定规则 |
| `generated/storyboard` | 分镜图 | 只定规则 |
| `projects/{projectId}/exports` | 项目内导出结果 | 只定规则 |
| `cache/thumbnails` | 缩略图缓存 | 创建根目录 |
| `cache/model-test` | 模型测试产物 | 创建根目录 |
| `temp` | 全局临时文件 | 创建根目录 |
| `exports` | 全局导出默认目录 | 创建根目录 |
| `logs` | 日志 | 创建根目录 |

说明：

```txt
项目创建功能后续才真正创建 projects/{projectId}。
素材功能后续才往 assets/generated 写文件。
导出功能后续才决定具体剪映草稿结构。
本任务只把目录规则和基础 API 定好。
```

## 6. 禁止写入的位置

明确禁止：

```txt
D:\project\vt-studio\.runtime
D:\project\vt-studio\data
D:\project\vt-studio\temp
D:\project\vt-studio\exports
D:\project\vt-studio\db.sqlite
D:\project\vt-studio\assets
D:\project\vt-studio\oss
```

也禁止：

```txt
renderer 传一个绝对路径让 main 直接写。
业务代码自己 path.join(app.getPath("userData"), ...)。
业务代码自己 fs.rm(..., { recursive: true })。
页面保存数据库绝对路径、素材绝对路径。
把临时文件提交到项目目录。
```

业务表里后续应该保存：

```txt
projectId
fileKind
relativePath
mimeType
size
hash
createdAt
```

不建议保存：

```txt
C:\Users\...\xxx.png
D:\project\...\xxx.mp4
```

原因：

```txt
绝对路径换电脑、换用户、换安装目录后容易失效。
相对路径更适合迁移、导出、备份。
```

## 7. 路径安全规则

必须做：

```txt
1. 所有文件服务只接受“受控目录类型 + 相对路径”。
2. 相对路径要去掉开头的 / 和 \。
3. 统一处理 Windows 和 POSIX 分隔符。
4. path.resolve 后必须校验结果仍在允许根目录内。
5. 删除目录前必须校验目录在 userData 或项目目录内。
6. 不允许 `..` 逃逸到上级目录。
7. 不允许空路径触发删除根目录。
8. 不允许删除 database 目录。
```

路径逃逸示例：

```txt
../../Windows/System32
C:\Users\Twj\Desktop\a.png
/../../xxx
projects/abc/../../../database/vt-studio.sqlite
```

这些都必须拒绝。

## 8. 建议新增文件

建议新增：

```txt
src/main/services/file-system/paths.ts
src/main/services/file-system/safe-path.ts
src/main/services/file-system/directories.ts
src/main/services/file-system/operations.ts
src/main/services/file-system/project-paths.ts
src/main/services/file-system/index.ts
```

说明：

| 文件 | 作用 |
|---|---|
| `paths.ts` | 定义 userData 下各基础目录 |
| `safe-path.ts` | 安全拼路径、路径逃逸校验、相对路径规范化 |
| `directories.ts` | 初始化基础目录 |
| `operations.ts` | exists、ensureDir、copyFile、writeFile、deleteFile、deleteDirectory |
| `project-paths.ts` | 解析项目目录、项目素材目录、项目临时目录 |
| `index.ts` | 对外导出文件层能力 |

暂不新增：

```txt
不新增 file IPC。
不新增文件管理 UI。
不新增项目创建代码。
不新增素材上传代码。
不新增剪映导出代码。
不新增缩略图生成代码。
不新增本地 HTTP 静态服务。
```

## 9. 建议 API

底层 API：

```txt
getUserDataRoot()
getRuntimeDirectories()
initializeFileSystem()
normalizeRelativePath(relativePath)
safeJoin(root, relativePath)
assertInsideRoot(targetPath, rootPath)
ensureDirectory(path)
pathExists(path)
fileExists(path)
copyFileToManagedPath(sourcePath, targetRoot, targetRelativePath)
writeManagedFile(targetRoot, targetRelativePath, data)
readManagedFile(targetRoot, targetRelativePath)
deleteManagedFile(targetRoot, targetRelativePath)
deleteManagedDirectory(targetRoot, targetRelativePath)
```

项目路径 API：

```txt
resolveProjectRoot(projectId)
resolveProjectSourcePath(projectId, relativePath)
resolveProjectAssetPath(projectId, assetType, relativePath)
resolveProjectGeneratedPath(projectId, generatedType, relativePath)
resolveProjectTempPath(projectId, relativePath)
resolveProjectExportPath(projectId, relativePath)
```

目录类型建议：

```txt
assetType:
images
audio
video
clips
documents

generatedType:
images
audio
video
storyboard
```

返回值规则：

```txt
service 内部可以使用绝对路径。
返回给 renderer 的默认是相对路径或受控展示字段。
除非是打开文件夹这种系统动作，否则不把绝对路径暴露给页面。
```

## 10. 和预览 URL 的关系

参考项目：

```txt
Toonflow 用 /oss/xxx 和 /skills/xxx 做本地 HTTP 静态访问。
getSmallImageUrl 返回 /oss/xxx?size=20 或 http://localhost:端口/oss/xxx?size=20。
```

VT Studio 本任务不直接照搬本地 HTTP。

本任务只确定：

```txt
文件真实存在哪里。
业务表保存什么相对路径。
如何安全读写。
```

预览 URL 后续在素材/图片/视频相关任务里再定：

```txt
方案 A：main 读取文件转 blob/base64 给 renderer。
方案 B：注册 Electron protocol，例如 vtstudio-file://。
方案 C：本地 HTTP 静态服务。
```

专业建议：

```txt
优先考虑 Electron protocol。
它比本地 HTTP 更贴近桌面程序，也减少端口占用和跨域问题。
但这属于预览能力，不放在 CORE-004 里直接做。
```

## 11. 和导出的关系

用户关心的导出问题先定边界：

```txt
剪映草稿导出不在 CORE-004 做。
CORE-004 只保证后续导出任务能拿到受控文件路径。
```

后续 `F-010-003 Windows 剪映专业版草稿导出` 会单独处理：

```txt
1. 剪映草稿目录结构。
2. 素材复制还是引用原路径。
3. draft_content.json / draft_meta_info.json 等文件写法。
4. 音视频轨道映射。
5. 素材缺失校验。
6. 导出目标选择。
```

本任务只提供：

```txt
resolveProjectExportPath()
ensureDirectory()
copyFileToManagedPath()
safeJoin()
```

## 12. 和模型生成的关系

参考项目：

```txt
图片/视频/音频模型生成后调用 ai.save(path)。
ai.save 内部再走 u.oss.writeFile(path, result)。
```

VT Studio 后续应该保持：

```txt
模型服务生成结果 -> 文件服务写入 managed path -> 数据库保存相对路径 -> 页面拿预览地址
```

本任务不调用模型，不做 ComfyUI，不做 SDK 封装。

但本任务要给后续模型层留好目录：

```txt
generated/images
generated/audio
generated/video
generated/storyboard
cache/model-test
```

## 13. 错误码建议

需要补充文件类错误码：

```txt
FILE_PATH_INVALID：路径非法
FILE_PATH_ESCAPE：路径逃逸
FILE_NOT_FOUND：文件不存在
FILE_ALREADY_EXISTS：文件已存在
FILE_READ_FAILED：文件读取失败
FILE_WRITE_FAILED：文件写入失败
FILE_DELETE_FAILED：文件删除失败
DIRECTORY_CREATE_FAILED：目录创建失败
DIRECTORY_NOT_EMPTY：目录非空
UNSUPPORTED_FILE_TYPE：不支持的文件类型
```

返回格式继续使用：

```txt
成功：{ code: 200, data: {}, msg: "成功" }
失败：{ code: 400, data: {}, msg: "文件路径非法" }
```

多语言规则：

```txt
code 用于程序判断。
msg 用于当前语言展示。
底层 error 可以保留 detail 给日志，不直接甩给页面。
```

## 14. 本次不做什么

```txt
不做项目新增/编辑/删除。
不创建 projects/{projectId} 具体项目目录。
不导入小说。
不上传素材。
不生成图片。
不生成音频。
不生成视频。
不生成缩略图。
不做文件预览 URL。
不做打开文件夹 IPC。
不做数据库记录。
不做剪映草稿导出。
不做 ComfyUI 文件输入输出。
不做清理缓存按钮。
```

原因：

```txt
这些都是业务功能。
本任务只铺底层，避免每个业务功能都自己写 fs/path。
```

## 15. 风险和处理

| 风险 | 处理 |
|---|---|
| 路径规则太粗，后续素材和导出混乱 | 先把 projects/assets/generated/exports/temp 分清楚 |
| renderer 传绝对路径导致越权 | 文件服务只接受受控根目录 + 相对路径 |
| 删除目录误删数据库或项目外文件 | 删除前强制 assertInsideRoot，并禁止空相对路径 |
| 后续预览方式没定 | 本任务先不做预览 URL，只保留相对路径和安全读取能力 |
| 文件写到项目源码目录 | 所有根目录只来自 `app.getPath("userData")` |
| 一开始就做太多业务 | 本任务不碰项目、素材、导出、模型 |

## 16. 验收标准

确认执行后必须满足：

```txt
1. pnpm run typecheck 通过。
2. pnpm run build 通过。
3. pnpm run dev 能启动。
4. userData 下自动存在基础目录：projects/cache/temp/exports/logs。
5. 项目根目录没有 .runtime、data、temp、exports、oss、assets 等运行目录。
6. safeJoin 能拒绝 ../ 路径逃逸。
7. 删除文件/目录能力不能删除 userData 根目录、database 目录和项目外目录。
8. renderer/preload 仍不能直接访问 fs/path。
9. 没有新增业务表。
10. 没有新增业务页面。
```

## 17. 用户确认点

请确认是否按这个范围执行：

```txt
确认后才开始写代码。
不确认就只继续调整本文档。
```

需要确认的范围：

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-CORE-004-001 | 本地文件基础层目录是否按 `projects/cache/temp/exports/logs` 规划 | 建议确认，结构简单，后续不容易乱 |
| C-CORE-004-002 | 项目文件是否放 `projects/{projectId}`，素材跟随项目目录 | 建议确认，迁移和删除项目更清楚 |
| C-CORE-004-003 | 业务表后续只保存相对路径，不保存系统绝对路径 | 建议确认，便于迁移和备份 |
| C-CORE-004-004 | 本任务暂不做预览 URL，本地预览后续优先评估 Electron protocol | 建议确认，不要在基础层过早引入本地 HTTP |
| C-CORE-004-005 | 本任务不做剪映导出，只给导出准备安全路径能力 | 建议确认，导出逻辑要单独分析参考项目和剪映结构 |

## 18. 执行后记录

```txt
实际新增文件：
src/main/services/file-system/paths.ts
src/main/services/file-system/safe-path.ts
src/main/services/file-system/directories.ts
src/main/services/file-system/operations.ts
src/main/services/file-system/project-paths.ts
src/main/services/file-system/index.ts

实际修改文件：
src/main/index.ts
src/shared/constants/status.ts
docs/tasks/CORE-004-本地文件层.md
docs/03-执行进度.md

实现结果：
1. 新增 main 侧文件系统基础层。
2. 应用启动时初始化 userData 下 projects/cache/thumbnails/cache/model-test/temp/exports/logs。
3. 提供 getRuntimeDirectories/getUserDataRoot。
4. 提供 normalizeRelativePath/safeJoin/assertInsideRoot。
5. 提供 ensureDirectory/pathExists/fileExists/copyFileToManagedPath/writeManagedFile/readManagedFile/deleteManagedFile/deleteManagedDirectory。
6. 提供 resolveProjectRoot/Source/Asset/Generated/Temp/Export 路径解析。
7. 删除能力禁止删除受控根目录和 database 目录。
8. renderer/preload 未暴露 fs/path，也未新增 file IPC。
9. 未新增业务表、页面、素材业务、导出业务、预览 URL。

补充错误码：
FILE_PATH_ESCAPE
FILE_ALREADY_EXISTS
FILE_READ_FAILED
FILE_WRITE_FAILED
FILE_DELETE_FAILED
DIRECTORY_CREATE_FAILED
DIRECTORY_NOT_EMPTY
UNSUPPORTED_FILE_TYPE

验证命令：
D:\software\nodejs\pnpm.cmd run typecheck
D:\software\nodejs\pnpm.cmd run build

验证结果：
typecheck：通过。
build：通过。
dev：本轮未完成启动验证；sandbox 审批服务拒绝启动 Electron dev 进程，不做绕过。

未完成事项：
无业务未完成。后续预览 URL、打开文件夹、素材导入、剪映导出必须单独建 task。

是否有偏差：
无业务偏差。

是否更新 03：
是。

最终结论：
通过。
```
