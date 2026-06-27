# MVP Smoke Test 清单

> 目标：固定主线验收口径，区分 Mock 闭环、桌面受控闭环和真实 Provider / FFmpeg 闭环，避免把“页面能打开”误写成“生产链路通过”。

## 执行前置

| 类型 | 前置条件 | 允许默认执行 | 说明 |
| --- | --- | --- | --- |
| 浏览器 Mock smoke | `pnpm --dir src dev` 可打开页面 | 是 | 只验证路由、页面状态和前端交互，不证明 Tauri、SQLite、文件系统或真实生成可用。 |
| 桌面受控 smoke | Tauri dev 可启动，使用 controlled fake Provider | 是 | 可验证 Tauri command、SQLite、workspace 文件写入、候选结果、任务历史和导出记录。 |
| 真实 Provider smoke | 用户显式提供密钥并确认费用 | 否 | 不得默认调用外部图片、视频、LLM、TTS 或工作流服务。 |
| 真实 FFmpeg smoke | `sidecars/ffmpeg.exe` 与 `sidecars/ffprobe.exe` 存在 | 否 | 没有 sidecar 时只能验证错误恢复，不能写成真实 final.mp4 通过。 |
| 外部目录导出 smoke | 有安全文件选择 / 授权 token 设计 | 否 | 当前默认导出到受控 `outputs/*`，不让前端传任意系统路径。 |

## 主线 Smoke 步骤

| # | 步骤 | 必查点 | 通过标准 | 记录字段 |
| --- | --- | --- | --- | --- |
| 1 | 打开桌面应用 | 应用启动、首页可见、无启动崩溃 | 进入“我的作品”，能看到创建入口和已有作品列表 / 空态 | app_version、启动方式 |
| 2 | 点击“开始创作” | 自动创建或进入草稿作品 | 新草稿有 project_id，Header / 工作台能显示作品标题 | project_id |
| 3 | 输入一段文字 | 长文本检测、智能切分建议、手动创建兜底 | 能得到可编辑的文案段落；内容创作页只确认 `sourceText / narrationText`，不提前伪造完整分镜 | 输入长度、切分段数 |
| 4 | 生成 / 编辑分镜 | StoryboardItem 结构、内联编辑、Inspector 辅助 | 每行能编辑文本、画面、角色、场景、提示词、时长、状态；Inspector 不重复编辑同一字段 | 分镜行数、失败行 |
| 5 | 行号快速跳转 | 长表格定位 | 输入目标行号后滚动 / 聚焦到对应行，越界有提示 | 目标行号 |
| 6 | 进入生图表格 | 下游状态、候选结果区域 | 每行展示生成入口、候选图、最终选择状态；没有 ETA 文案 | 候选列状态 |
| 7 | 预览最终提示词 | 规则、角色、场景、镜头字段组装 | 弹窗 / 面板展示实际发送给模型的 prompt；缺规则时明确显示未接入或缺配置，不伪造 | prompt 来源 |
| 8 | 每行生成候选图 | Provider 边界、文件路径、失败隔离 | controlled fake 写入 workspace 相对路径；单行失败不清空其他行；候选可预览、取消、确认 | provider 类型、候选数 |
| 9 | 每行选择最终图 | `selectedImageId` | 选择后 DB / 页面状态指向候选 ID，不回退到覆盖式 `imagePath` | selectedImageId |
| 10 | 进入视频阶段 | 图片依赖校验 | 未选择最终图的行被阻断并给恢复动作；已选择行可生成视频 | 阻断行数 |
| 11 | 生成 / 确认视频片段 | `selectedVideoSegmentId`、失败隔离 | controlled fake 写入 workspace 相对路径；可确认最终视频片段；失败行可重试 | segment_id |
| 12 | 进入合成阶段 | 合成页职责 | 页面只包含片段检查、合成控制、输出信息和导出操作；无字幕 / 封面 / BGM 混入入口 | 页面检查结果 |
| 13 | 生成最终视频 | FFmpeg sidecar 或受控错误 | 有 sidecar 时生成 `outputs/.../final.mp4`；无 sidecar 时返回可恢复错误，不能记为真实合成通过 | sidecar 状态、outputPath |
| 14 | 导出 final.mp4 | ExportRecord、安全路径 | 导出到受控 `outputs/user_exports/...`，导出记录含状态、时间、错误信息 | export_id |
| 15 | 打开输出目录 | 目录反查和 PathGuard | 只能打开本次 ExportRecord 的受控父目录 | 打开结果 |
| 16 | 回看历史 | 任务历史、导出记录 | 工作台内能回看任务状态、失败详情、导出记录和资源消耗摘要；无真实统计时显示未接入 | task_id、资源摘要 |

## 失败恢复 Smoke

| 场景 | 操作 | 期望 |
| --- | --- | --- |
| 未选择最终图进入视频 | 清空某行 `selectedImageId` 后进入视频 | 页面阻断该行，提示先选择最终图。 |
| Provider 单行失败 | 让 controlled fake 某一行返回错误 | 失败行显示错误和重试，其他行候选保留。 |
| 缺少 FFmpeg sidecar | 不放置 `sidecars/ffmpeg.exe` / `ffprobe.exe` 后合成 | 返回 `ffmpeg.sidecar_missing`，recoverAction 指向安装 / 检查 sidecar。 |
| final.mp4 缺失导出 | 删除或指向不存在的受控 output | 返回 `export.final_missing`，不写成功导出记录。 |
| 诊断包含疑似密钥 | 日志写入测试密钥片段后导出诊断包 | 返回 `diagnostic.secret_detected`，阻断导出。 |
| 项目包 Zip Slip | 导入包含 `../` entry 的项目包 | 返回 `import.zip_slip_detected`，不写入项目。 |

## 记录模板

```text
日期：
执行人：
分支 / 提交：
应用版本：
运行方式：浏览器 Mock / Tauri dev / Tauri build
Provider：controlled fake / real provider（名称）
是否确认真实费用：是 / 否 / 不适用
FFmpeg sidecar：存在 / 缺失

主线结果：
- 创建作品：
- 内容导入：
- 分镜：
- 生图：
- 视频：
- 合成：
- 导出：
- 历史回看：

失败恢复结果：
- Provider 单行失败：
- 缺少 sidecar：
- final.mp4 缺失：
- 诊断脱敏：
- Zip Slip：

结论：
- 通过 / 未通过 / 部分通过
- 未执行项及原因：
- 后续修复项：
```

## 不能写成通过的情况

- 只打开浏览器页面，不能写成桌面主线通过。
- controlled fake 生成成功，不能写成真实 Provider 通过。
- 没有 `ffmpeg.exe / ffprobe.exe`，不能写成真实 final.mp4 通过。
- 只跑 typecheck/build/cargo test，不能写成手动 smoke 通过。
- 没有检查任务历史、导出记录、错误恢复，不能写成 MVP 闭环通过。
