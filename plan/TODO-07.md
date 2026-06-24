# TODO-07：真实生图、图生视频与 FFmpeg 合成

> 目标：把 TODO-03 的 Mock 主线替换为真实生图、真实图生视频和真实 FFmpeg 合成。  
> 本文件来自 `doc/功能模块/08-AI生图.md`、`14-AI视频.md`、`13-视频合成.md`、`doc/底层设计/13-FFmpegSidecar与媒体处理规范.md`。

---

## 阶段目标

跑通真实闭环：

```text
分镜 imagePrompt → 真实候选图 → 选择最终图 → 真实图生视频片段 → 确认片段 → FFmpeg 合成 final.mp4
```

前置要求：

```text
TODO-06 的 provider_models
TODO-06 的 workflow_presets
TODO-06 的 ImageInputPlan / VideoInputPlan
TODO-06 的 list_executable_media_options
```

本阶段不允许按某一家模型或 workflow 名称硬编码输入要求。

---

## 本阶段范围

包含：

- 真实 AI 生图。
- 真实图生视频。
- 基于 `ImageInputPlan / VideoInputPlan` 的输入收集、缺项提示和后端校验。
- 生成结果转存受控工作区。
- ImageCandidate / VideoSegment 入库。
- FFmpeg / FFprobe sidecar 检测。
- 片段格式检查。
- concat 合成 final.mp4。
- 合成错误码和基础恢复。

不包含：

- 字幕烧录。
- 封面模板。
- BGM 混音。
- 多轨时间轴。
- 高级转场。

---

## TODO

> 本文件的每条 TODO 按以下口径执行：
> - 顺序：只做本文件中第一条未完成 TODO；本文件未完成前不得跳到后续 TODO 文件。
> - 规范：先遵守本阶段范围、底层设计、安全红线、命名规则和 `plan/阶段路线图.md` 的完成判定。
> - 做什么：落地到具体文件、接口、页面、表结构、DTO、Store、Service、组件、样式、i18n、测试或文档。
> - 做到怎么样：UI、逻辑、样式、组件封装、多语言、状态、错误处理、安全、验证全部满足，才算完成。
> - 怎么做：按“改法”小步实现；不要引入本阶段明确排除的能力。
> - 打标：完成一条后必须立刻把 `【】` 改成 `【X】`，并写完成记录；不需要用户提醒。
> - 停止：如果需求不清、验收不清、文档冲突、数据结构不明或风险高，停下来说明，不靠猜测继续。

### 【】7.1 接入真实 AI 生图

**问题：**  
Mock 生图只能验证流程，不能产出真实图片。

**改法：**

基于每行：

```text
imagePrompt
negativePrompt
ImageInputPlan.required / optional
providerModelId 或 workflowPresetId
```

调用 Image Provider 或 WorkflowRegistry。

输出：

```text
ImageCandidate[]
imagePath
status
provider/model snapshot
generationContextSnapshot
```

**验证：**

- 每个分镜可真实生成候选图。
- 使用不同生图模型时，前端展示的参考图、姿态图、mask、workflow 参数来自 `ImageInputPlan`。
- 远程 URL 下载或复制到受控工作区。
- 入库只存相对路径。
- 用户可选择最终图。

**风险：**  
不要长期引用 Provider 返回的临时 URL。

---

### 【】7.2 实现生图失败重试和单行隔离

**问题：**  
批量生图时单行失败不能影响其他行保存。

**改法：**

每个 StoryboardItem 单独记录：

```text
status
lastErrorJson
retryCount
TaskStep / TaskAttempt
```

**验证：**

- 单行失败可重试。
- 成功行结果保留。
- 审核失败可进入提示词中性化重试。

**风险：**  
不要失败后整批回滚已成功候选图。

---

### 【】7.3 接入真实图生视频

**问题：**  
当前主线必须把已选图片生成视频片段。

**改法：**

输入必须来自：

```text
selectedImageId → ImageCandidate.imagePath
videoPrompt
durationSeconds
VideoInputPlan.required / optional
providerModelId 或 workflowPresetId
```

输出：

```text
VideoSegment
videoPath
status
selected=false
generationContextSnapshot
```

**验证：**

- 没有 selectedImageId 不能生成视频。
- 对首尾帧、参考图、ComfyUI / RunningHub 等不同模式，输入项必须来自 `VideoInputPlan`，不能写死只传 startFrame。
- 片段保存到受控工作区。
- 失败可重试。
- 用户确认后写 selectedVideoSegmentId。

**风险：**  
不同 Provider 的图生视频能力差异大，必须走能力矩阵和输入规划，不要硬编码某一家。

---

### 【】7.4 实现视频 Provider 能力校验

**问题：**  
视频模型对时长、比例、fps、输入图数量约束强。

**改法：**

后端校验：

```text
durationSeconds
resolution
aspectRatio
fps
inputImages
mode=image_to_video
```

**验证：**

- 超出模型能力后端拒绝。
- 前端禁用只是体验，后端才是最终校验。
- 错误码明确可恢复动作。

**风险：**  
不要让业务层按 modelName 写特殊逻辑。

---

### 【】7.5 实现 FFmpeg / FFprobe sidecar 检测

**问题：**  
开发环境能跑不代表发布版能合成。

**改法：**

实现：

```text
ffmpeg -version
ffprobe -version
sidecar path resolve
版本信息展示
缺失错误码
```

**验证：**

- sidecar 缺失不阻止打开应用，但禁止启动合成任务。
- 设置页或诊断页可看到 sidecar 状态。

**风险：**  
路径必须来自 PathGuard / SafePath。

---

### 【】7.6 实现 Ffprobe 媒体探测

**问题：**  
不能只相信 Provider 返回的时长和格式。

**改法：**

探测：

```text
container
codec
width
height
fps
duration
audio stream
video stream
```

**验证：**

- 每个 VideoSegment 合成前可被 ffprobe。
- 损坏片段有明确错误。
- 编码不一致可进入转码兜底。

**风险：**  
音画不同步和编码不一致是合成高频问题。

---

### 【】7.7 实现片段 concat 合成

**问题：**  
最终必须把多个视频片段合成一个视频。

**改法：**

合成规则：

```text
按 StoryboardItem.index 排序
读取 selectedVideoSegmentId
生成 filelist 到任务工作区
ffmpeg concat
输出 exports/final.mp4
```

默认编码：

```text
mp4
libx264
yuv420p
fps=30
crf=18
preset=veryfast
aac
192k
44100
```

**验证：**

- 得到可播放 final.mp4。
- 输出路径在受控工作区。
- CompositionTask.outputPath 写入相对路径。

**风险：**  
FFmpeg 参数必须数组传递，不能拼 shell 字符串。

---

### 【】7.8 实现转码兜底

**问题：**  
不同 Provider 输出视频编码、fps、分辨率可能不一致。

**改法：**

合成前检查，不一致时转码到统一规格：

```text
resolution
fps
codec
pixel format
audio format
```

**验证：**

- 不一致片段可被转码后合成。
- 转码失败有明确错误码。
- 临时文件位于任务工作区。

**风险：**  
临时文件不能写系统临时目录破坏 PathGuard 边界。

---

### 【】7.9 实现 FFmpeg 日志脱敏和截断

**问题：**  
FFmpeg stderr 可能包含路径、参数、用户文件名，需要控制日志大小和敏感信息。

**改法：**

规则：

```text
stderr 保存最后 200 行或 32KB
路径脱敏为 relative_path
SecretGuard 扫描
写 task.log / ffmpeg.log
```

**验证：**

- 日志不包含真实密钥。
- 日志不包含敏感绝对路径。
- 错误详情可定位失败原因。

**风险：**  
不要把完整 Provider 请求或用户隐私内容写进日志。

---

## 阶段完成标准

- 每个分镜能真实生成候选图。
- 每个分镜能基于已选图生成视频片段。
- 图片 / 视频真实生成前，前端按输入规划展示缺项，后端按能力矩阵和输入规划拒绝非法参数。
- 所有片段确认后能 FFmpeg 合成 final.mp4。
- final.mp4 可播放。
- 失败可定位、可重试，不破坏已完成产物。
- 所有产物路径受控，日志无密钥和敏感绝对路径。






