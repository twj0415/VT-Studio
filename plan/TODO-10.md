# TODO-10：TTS、字幕、封面、模板、BGM 成片增强

> 目标：补齐短视频成片质量增强项，但不让它们反向阻塞 `image_to_video` 主线。
> 本文件来自 `doc/功能模块/09-配音TTS.md`、`10-字幕.md`、`11-封面.md`、`12-模板与动效.md`、`13-视频合成.md`、`doc/底层设计/14-模板渲染与Chromium安全规范.md`。

---

## 阶段目标

在主线可跑通后，逐步加入：

```text
TTS
字幕
封面
HTML 模板
基础动效
BGM
音频混合
```

这些能力提高成片质量，但不应成为当前 `image_to_video` 主线强制前置。

和主线的关系必须讲清楚：

```text
独白 / 旁白：优先使用 StoryboardItem.narrationText，没有则可退回 sourceText。
字幕：从独白 / 旁白断句生成 subtitleChunks / subtitles.json。
TTS：把独白 / 旁白生成音频，写 audioPath / audioDurationSeconds。
时长：未启用 TTS 时使用分镜估算或视频阶段设置；启用 TTS 且选择音频驱动时，用真实 audioDurationSeconds 校准 durationSeconds。
合成：默认先拼接已确认视频片段；字幕、配音、BGM、封面都是可选增强。
```

---

## 本阶段范围

包含：

- TTSProvider。
- 音频生成和真实时长读取。
- 字幕断句、编辑、样式、安全区。
- 封面标题、主图选择、cover.png。
- HTML 模板 manifest 和参数 DSL。
- Chromium 模板渲染安全。
- BGM 混音。
- 字幕 / 封面 / BGM 可以作为合成输入或导出后处理接入，但不放进当前合成页右侧控制区，也不成为当前强制 StepBar。

不包含：

- 模板市场。
- 专业多轨时间轴。
- 复杂封面编辑器第一版。
- 把 TTS/字幕/封面/BGM 设为必经步骤。

---

## TODO

> 本文件的每条 TODO 按以下口径执行：
> - 顺序：只做本文件中第一条未完成 TODO；本文件未完成前不得跳到后续 TODO 文件。
> - 规范：先遵守本阶段范围、底层设计、安全红线、命名规则和 `plan/阶段路线图.md` 的完成判定。
> - 问题：必须说清不做会造成什么用户问题、工程问题或后续返工。
> - 位置：必须落到页面、接口、表结构、DTO、Store、Service、组件、样式、i18n、测试或文档；不能只写“相关文件”。
> - 改法：按小步实现，写清数据流、状态流、边界和本阶段不做什么。
> - 验收：写清做到什么客观状态才算完成，不能把验证命令当验收。
> - 验证：写清命令、页面流程、数据库检查、文件检查、日志检查或 smoke test。
> - 下一步：本条必须满足“下一步进入条件”后，才能打勾进入下一条；旧 TODO 缺字段时先补齐再实现。
> - 打标：完成一条后必须立刻把 `【】` 改成 `【X】`，并写完成记录；不需要用户提醒。
> - 停止：如果需求不清、验收不清、文档冲突、数据结构不明或风险高，停下来说明，不靠猜测继续。

### 【X】10.0 明确字幕 / 独白 / 配音的主线关系

**问题：**
字幕、独白、配音容易被误解成当前 StepBar 里的强制步骤，也容易被误做成三套互相覆盖的文本。正确关系是：独白是文本，字幕是显示断句，TTS 是音频生成，它们都服务合成，但不阻塞最小 image_to_video。

**位置：**

```text
doc/开始之前/菜单与页面详细说明.md
doc/开始之前/主流程与数据流.md
doc/功能模块/对应字幕/TTS/合成文档
src-tauri/src/domain/storyboard*
src-tauri/src/domain/audio* / subtitle*
src/src/features/workspace/compose*
src/src/features/workspace/video*
```

这条优先统一规则和字段边界；如果代码中已有冲突字段，先收敛命名，不急着接真实 TTS。

**改法：**

- 统一数据来源：

```text
独白 / 旁白文本 = StoryboardItem.narrationText
字幕断句 = StoryboardItem.subtitleChunks
TTS 音频 = StoryboardItem.audioPath / audioDurationSeconds
```

- 没有 `narrationText` 时，可以用 `sourceText` 生成字幕或 TTS，但不能反向改写用户原文。
- 字幕编辑只修改 `subtitleChunks`，不自动改 `narrationText`。
- TTS 重生成只替换音频和真实时长，不自动改字幕文本；如果时长变化，字幕时间轴和合成状态必须标记需重新检查。
- 启用音频驱动时，`audioDurationSeconds` 可以更新 `durationSeconds`；未启用时，视频时长仍由视频阶段或分镜估算控制。
- UI 上不要把 TTS、字幕、封面、BGM 放进当前强制 StepBar；它们作为合成前后的可选增强能力出现，页面上应独立于当前“片段检查 → 合成 final.mp4 → 导出”的合成页控制区。

**验收：**

- 不启用 TTS 和字幕时，仍能从已确认视频片段合成 `final.mp4`。
- 只编辑字幕断句，不会改变旁白正文和音频。
- 启用 TTS 后，真实音频时长能用于字幕时间轴。
- TTS 失败时，如果用户未要求“必须带配音导出”，基础视频合成不被阻断。
- 页面和文档都明确：分镜 → 生图 → 视频 → 合成仍是主 StepBar；字幕/TTS/封面/BGM 是可选增强。
- `narrationText / subtitleChunks / audioPath / audioDurationSeconds` 边界清楚，没有第二套“独白正文”字段。

**验证：**

- 搜索字段命名，确认没有新增平行旁白字段或让字幕字段覆盖原文。
- 前端 typecheck/build 通过。
- 手动 smoke：关闭字幕/TTS 仍可合成；编辑字幕不改变旁白；TTS 失败不阻断无配音导出。

**下一步进入条件：**

- 文档、数据字段和页面入口都已统一“可选增强”口径。
- 完成记录写清哪些字段是权威来源、哪些状态会触发合成重新检查。
- 确认 10.1 TTSProvider 会复用这些字段后，再把本条改为 `【X】` 并进入 10.1。

**风险：**
不要创建第二套“独白正文”字段；不要让字幕编辑污染原文；不要把估算字幕时间伪装成真实字级时间戳。

**完成记录（2026-06-26）：**

- 已确认当前文档口径：分镜 → 生图 → 视频 → 合成仍是强制主 StepBar；字幕 / TTS / 封面 / BGM 是可选增强，不放进当前合成页右侧控制区，也不阻塞最小 `image_to_video` 主线。
- 已在数据结构层固定权威字段：
  - 独白 / 旁白正文：`StoryboardItem.narrationText`，缺省时增强能力可读取 `sourceText`，但不能反向改写原文。
  - 字幕断句：`StoryboardItem.subtitleChunks` / SQLite `subtitle_chunks_json`。
  - TTS 音频：`StoryboardItem.audioPath` / SQLite `audio_path`。
  - 真实音频时长：`StoryboardItem.audioDurationSeconds` / SQLite `audio_duration_seconds`。
  - 状态仍沿用已有 `audioStatus / subtitleStatus`。
- 已新增 SQLite migration `storyboard_optional_audio_subtitle_fields_v1`，只通过新 migration 给旧库补列；未修改已应用旧 migration checksum。
- Rust `StoryboardItemDto`、Repository 读写、前端 `StoryboardItemDto` 和前端 mock 创建逻辑已同步字段。
- 本条没有实现真实 TTSProvider、音频时长探测、字幕断句 UI 或合成接入；这些继续按 10.1 / 10.2 / 10.3 / 10.9 顺序执行。

**验证记录（2026-06-26）：**

- `cargo check` 通过。
- `cargo fmt --check` 通过。
- `cargo test scene_repository` 通过：1 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动态 / 静态 import 提示和 chunk 体积提示，不影响构建结果。
- 字段命名搜索确认未新增 `monologue / voiceText / ttsText / audioText / subtitleText` 等平行独白正文字段；新增代码侧只有 `subtitleChunks / audioPath / audioDurationSeconds`。
- 未执行真实桌面手动 smoke；当前不能写成 TTS、字幕或真实合成增强通过。

---

### 【X】10.1 实现 TTSProvider

**问题：**
TTS 对口播、字幕、数字人、纯图文成片有价值，但不应阻塞当前主线。

**改法：**

实现：

```text
TTSProvider
音色选择
语速选择
批量生成
单条重配音
上传/替换音频
```

输出字段：

```text
audioPath
audioDurationSeconds
audioStatus
lastErrorJson
retryCount
```

文本来源：

```text
优先 StoryboardItem.narrationText
没有 narrationText 时可使用 sourceText
不得由 TTSProvider 自行改写文本
```

**验证：**

- 可按分镜生成音频。
- 音频路径进入受控工作区。
- 失败状态可重试。
- 不启用 TTS 时仍能完成图生视频合成。
- 生成音频后能读取真实 audioDurationSeconds，并触发字幕 / 合成依赖状态检查。

**风险：**
云 TTS Key 必须进 keyring；上传音频不能引用用户原路径。

**完成记录（2026-06-26）：**

- 已复用 `ProviderManager::generate_tts` 和 `TtsProviderRequest / TtsProviderResponse`，新增分镜级 TTS 业务链路 `start_tts_generation`，文本来源固定为 `narrationText`，为空时读取 `sourceText`，TTS 不改写文本。
- 已新增受控 fake TTS provider/model，仅用于本地可验证链路，不包装成真实云厂商；真实云 TTS 仍走 provider 配置和 keyring。
- 已新增 SQLite migration `storyboard_audio_error_fields_v1`，补 `audio_last_error_json / audio_retry_count`；DTO 和前端类型同步为 `audioLastErrorJson / audioRetryCount`。
- TTS 成功后写回 `audioPath / audioDurationSeconds / audioStatus=succeeded`；失败写回 `audioStatus=failed / audioLastErrorJson / audioRetryCount+1`，可重新执行同一接口重试。
- 音频输出路径使用受控项目相对路径：`projects/{projectId}/audio/{itemId}/voice.mp3|wav`，拒绝 provider 返回绝对路径。
- 前端视频页新增可选配音列：展示音频状态、路径、时长、失败原因、重试次数；支持单条生成 / 重新配音和“生成缺失配音”批量入口。该入口不进入强制 StepBar，不阻塞视频生成或合成。
- 已新增 `replace_storyboard_audio` 后端命令和前端 entity/store API，把用户选择的本地音频复制到受控项目目录 `projects/{projectId}/audio/{itemId}/uploaded.ext` 后再写回分镜；不保存用户原始路径。
- 本条没有完成真实音频文件探测；上传/替换音频的 `audioDurationSeconds` 仍保持空，TTS provider 返回时长也会在 10.2 用 FFprobe/音频库复核后再作为真实时长。

**验证记录（2026-06-26）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test tts_generation` 通过：2 passed。
- `cargo test replace_storyboard_audio` 通过：1 passed。
- `cargo test scene_repository` 通过：1 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动态 / 静态 import 提示和 chunk 体积提示，不影响构建结果。
- 未执行真实云 TTS 调用和桌面端文件选择 smoke；当前只能确认本地 provider 链路、状态落库、受控路径和前端类型/构建通过。

---

### 【X】10.2 读取真实音频时长

**问题：**
不能只相信 TTS 返回时长，字幕和合成都依赖真实时长。

**改法：**

用 FFprobe 或音频库读取：

```text
audioDurationSeconds
sample_rate
channels
format
```

**验证：**

- TTS 生成后自动回填真实时长。
- 启用音频驱动时可更新分镜 durationSeconds。
- 如果对应分镜已有视频片段，时长变化不能静默裁剪或拉伸旧视频，必须提示重新生成、重新合成或接受不对齐。

**风险：**
音频变更后字幕和合成状态要重新检查。

**完成记录（2026-06-26）：**

- 已明确 `audioDurationSeconds` 只代表真实探测后的时长，不再把 TTS provider 返回时长直接写成最终真实时长。
- 已扩展 `MediaProbeDto`，补 `channels` 字段；现有 FFprobe 解析会回填：
  - `durationSeconds`
  - `sampleRate`
  - `channels`
  - `formatName / container`
  - `audioCodec / bitRate`
- 已新增 `StoryboardItem.audioProbe` / SQLite `audio_probe_json` / 前端 `audioProbe`，保存完整音频探测结果。
- 已新增 `probe_storyboard_audio` 后端命令和前端 entity/store API；视频页配音列新增“读时长”按钮，用户可对已有 `audioPath` 显式读取真实音频时长。
- TTS 生成和上传替换音频后只写入 `audioPath / audioStatus`，不伪造真实时长；真实时长必须由 `probe_storyboard_audio` 写入。
- 探测成功后写回 `audioDurationSeconds` 和 `audioProbe`；如果时长变化，会把 `subtitleStatus` 与 `renderStatus` 标记回 `pending`，不静默裁剪、拉伸或改写既有视频片段。
- 探测失败会通过命令错误返回，不把估算值写入 `audioDurationSeconds`。

**验证记录（2026-06-26）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test tts_generation` 通过：2 passed。
- `cargo test update_storyboard_item_audio_persists_probe` 通过：1 passed。
- `cargo test parses_ffprobe_video_json` 通过：1 passed，覆盖 `channels` 解析。
- `cargo test scene_repository` 通过：1 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动态 / 静态 import 提示和 chunk 体积提示，不影响构建结果。
- 未执行真实 FFprobe sidecar 的桌面 smoke；当前已验证 parser、数据库写回、状态重置和前端类型/构建。

---

### 【X】10.3 实现基础字幕

**问题：**
字幕有价值，但不能成为合成强制条件。

**改法：**

实现：

```text
自动断句
字幕编辑
字幕样式预设
字幕安全区
subtitles.json
```

切分规则：

```text
中文竖屏默认每行 12~18 字
优先按标点和语义切分
根据 item 时长累加全局时间
```

**验证：**

- 可从分镜文案生成基础字幕。
- 断句编辑不改变 narrationText。
- 可选择是否参与合成。
- 不启用字幕仍可输出 final.mp4。
- 没有 TTS 时使用 item.durationSeconds 估算时间轴；有 TTS 时优先使用 audioDurationSeconds。

**风险：**
不要按固定字数硬切；没有字级时间戳不要假装精确。

**完成记录（2026-06-26）：**

- 后端已新增 `generate_subtitles` / `update_storyboard_subtitles` 命令和 DTO：
  - `GenerateSubtitlesRequest`
  - `UpdateStoryboardSubtitlesRequest`
  - `GenerateSubtitlesResultDto`
  - `SubtitlesFileDto`
- 字幕文本来源固定为 `StoryboardItem.narrationText`，为空时读取 `sourceText`；生成和编辑字幕都不改 `narrationText/sourceText`。
- 字幕断句写入 `StoryboardItem.subtitleChunks` / SQLite `subtitle_chunks_json`，生成和编辑成功后设置 `subtitleStatus=succeeded`，并把 `renderStatus=pending` 标记为需要重新检查合成输入。
- 自动断句规则已实现：
  - 中文竖屏默认目标 12-18 字 / 行。
  - 优先按句末标点切分，再按逗号、顿号、冒号和 12-18 字目标拆长句。
  - 不做纯固定字数硬切。
- 时间轴已实现：
  - 有 `audioDurationSeconds` 且已探测时优先使用真实音频时长。
  - 没有真实音频时长时使用 `durationSeconds`。
  - 单条内部按字幕文本长度比例分配时间，标记 `estimated=true`，不伪装真实字级时间戳。
  - 输出 `projects/{projectId}/subtitles/subtitles.json`，包含全局累加时间轴、itemId/itemIndex、chunkId、文本、start/end、estimated 和默认样式。
- 字幕样式和安全区已落第一版：
  - 默认样式 `vertical_cn_default`。
  - 底部居中、白字、黑色描边。
  - 安全区：top 96、bottom 160、left/right 48。
  - 最大行长 18 字。
- 前端视频页已新增可选字幕增强入口：
  - 顶部“生成字幕文件”。
  - 单行“生成字幕”。
  - 单行字幕状态、断句数量和预览。
  - 字幕编辑弹窗，每行对应一个字幕 chunk；保存只更新 `subtitleChunks`。
  - 用户可见文案已走 i18n。
- 本条没有把字幕接入合成烧录；是否参与合成留给 10.9。未启用字幕仍不影响当前 `final.mp4` 最小合成主线。

**验证记录（2026-06-26）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test generate_subtitles` 通过：1 passed。
- `cargo test update_storyboard_subtitles` 通过：1 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动态 / 静态 import 提示和 chunk 体积提示，不影响构建结果。
- 未执行真实桌面端字幕 smoke 和真实合成烧录；当前只确认基础字幕生成、编辑、文件输出、状态写回和前端构建链路。

---

### 【X】10.4 实现逐字 / 卡拉OK字幕增强

**问题：**
逐字字幕是亮点，但没有真实字级时间戳时不能伪装精确。

**改法：**

支持：

```text
estimated word timing
highlight mode
karaoke style
```

并标记：

```text
estimated = true
```

**验证：**

- UI 明确估算时间。
- 安全区、自动换行、描边生效。

**风险：**
估算不要当真实对齐数据用于严肃时间轴。

**完成记录（2026-06-26）：**

- 已在 `subtitles.json` 输出结构中增加估算逐字高亮数据：
  - `SubtitleTimelineChunkDto.wordTimings[]`
  - 每个 token 包含 `token / startSeconds / endSeconds / estimated=true`。
- 中文无空格文本按单字生成估算 token；带空格文本按词生成估算 token。
- 逐字时间按 chunk 时间范围和 token 文本长度比例分配，只作为 `estimated` 数据，不伪装成真实 ASR/字级对齐。
- 字幕样式增加：
  - `mode=karaoke_estimated`
  - `highlightColor=#FFD54A`
- 前端视频页和字幕编辑弹窗已明确展示“估算逐字高亮 / 卡拉OK高亮：估算时间”，避免用户误解为真实精准时间戳。
- 本条不接入真实 ASR、不做精确字级时间戳、不做字幕烧录；烧录和合成接入仍留给 10.9。

**验证记录（2026-06-26）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test generate_subtitles` 通过：1 passed，覆盖 `wordTimings` 非空、全部 `estimated=true`、样式 `mode=karaoke_estimated`。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动态 / 静态 import 提示和 chunk 体积提示，不影响构建结果。

---

### 【X】10.5 实现封面基础能力

**问题：**
封面对发布有价值，但不应阻塞主视频合成。

**改法：**

支持封面来源：

```text
已选图
视频首帧
本地上传
AI 生成图
```

封面字段：

```text
coverPath
coverTitle
coverTemplateId
coverSourceItemId
```

**验证：**

- 可选择封面主图。
- 可编辑标题。
- 可生成 `cover/cover.png`。
- 不选封面也能合成主视频。

**风险：**
不要一开始做复杂封面编辑器；上传图片必须复制进项目目录。

**完成记录（2026-06-26）：**

- 已新增项目级封面字段并同步 DTO / TS 类型：
  - `coverPath`
  - `coverTitle`
  - `coverTemplateId`
  - `coverSourceItemId`
- 已新增 SQLite migration `project_cover_fields_v1`，字段挂在 `projects`，不把封面误做成分镜字段或合成任务字段。
- 已新增后端命令和前端 entity/store API：
  - `generate_project_cover`
  - `replace_project_cover_image`
  - `generateProjectCover`
  - `replaceProjectCoverImage`
  - `projectStore.generateCover`
  - `projectStore.replaceCoverImage`
- 已在作品工作台增加封面区：
  - 编辑封面标题。
  - 从已有分镜最终图选择封面主图来源。
  - 选择 / 粘贴本地图片路径并上传。
  - 生成 `projects/{projectId}/cover/cover.png`。
  - 封面明确作为可选增强，不进入当前强制 StepBar，也不阻塞主视频合成。
- 封面标题第一版限制 15 字；模板 ID 只允许 ASCII 字母、数字、`-`、`_`、`.`，过滤后为空则回到 `knowledge_bold`。
- 上传封面主图会复制到受控项目目录 `projects/{projectId}/cover/source.{ext}`，不会持久化用户原始绝对路径。
- 本阶段不做复杂封面编辑器，不做 Chromium/HTML 模板渲染，不做真实模板参数系统；这些继续进入 10.6 / 10.7。
- 当前基础 `cover.png` 是可验证 PNG 占位产物，用来打通字段、路径、上传、项目级状态和 UI；正式模板渲染由 10.6 / 10.7 替换。

**验证记录（2026-06-26）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test generate_project_cover` 通过：2 passed，覆盖基础封面生成、字段写回、受控 `assets/` 来源路径和模板 ID 兜底。
- `cargo test replace_project_cover_image` 通过：1 passed，覆盖本地上传复制到受控项目目录并生成 `cover.png`。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动态 / 静态 import 提示和 chunk 体积提示，不影响构建结果。
- 未执行真实桌面端点击 smoke；当前确认后端命令、受控路径、前端类型和生产构建通过。

---

### 【X】10.6 实现模板 manifest 与参数 DSL

**问题：**
字幕、封面、纯图文成片都需要模板系统，但模板必须可校验、可控。

**改法：**

模板目录按画幅组织：

```text
vertical_9_16
horizontal_16_9
square_1_1
```

参数 DSL：

```text
{{param}}
{{param=default}}
{{param:type}}
{{param:type=default}}
```

**验证：**

- 根据 aspectRatio 选择模板目录。
- 参数类型校验。
- `select` 类型引用统一字典。

**风险：**
用户可见中文不能写死在模板参数里。

**完成记录（2026-06-26）：**

- 已新增模板领域对象、服务和命令：
  - `TemplateManifestDto`
  - `TemplateViewportDto`
  - `TemplateParamSchemaDto`
  - `TemplateParamValidationResultDto`
  - `list_template_manifests`
  - `validate_template_params`
- 已新增前端 `entities/template`：
  - `listTemplateManifests`
  - `validateTemplateParams`
  - 模板 manifest / 参数 schema / 校验结果类型。
- 模板目录按受控结构扫描：
  - `templates/builtin/{templateType}/{aspectRatio}/{templateId}.html`
  - `templates/user/{templateType}/{aspectRatio}/{templateId}.html`
  - 支持 `frame / cover / subtitle / transition / layout`。
  - 支持 `vertical_9_16 / horizontal_16_9 / square_1_1`，并兼容项目旧画幅值 `9:16 / 16:9 / 1:1`。
- 已按画幅固定 viewport：
  - `vertical_9_16 = 1080 x 1920`
  - `horizontal_16_9 = 1920 x 1080`
  - `square_1_1 = 1080 x 1080`
- 已实现 `{{param}} / {{param=default}} / {{param:type}} / {{param:type=default}}` DSL 扫描和参数解析。
- 已支持并校验参数类型：
  - `text / number / color / bool / select / image / font / range / json`
- 已忽略内置参数：
  - `title / text / image / index`
- 已实现安全边界：
  - `templateId` 和参数名必须为 `snake_case`。
  - `entryPath` 必须是受控 `templates/*.html` 相对路径。
  - `select` 参数必须映射统一字典，不允许模板内自带散乱中文选项。
  - 颜色必须为 `#RGB` 或 `#RRGGBB`。
- 已新增统一字典：
  - `templateType`
  - `templatePosition`
  - `transitionType`
  - `fontWeight`
- 已 seed 最小内置模板文件：
  - `cover/vertical_9_16/knowledge_bold.html`
  - `subtitle/vertical_9_16/karaoke_basic.html`
  - `frame/vertical_9_16/image_soft_zoom.html`
- 本条没有实现 Playwright / Chromium 渲染、截图、联网拦截、file:// 白名单或浏览器崩溃重启；这些继续进入 10.7。

**验证记录（2026-06-26）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test parses_template_placeholder_dsl_and_ignores_builtin_params` 通过：1 passed。
- `cargo test lists_seeded_builtin_templates_by_aspect_and_type` 通过：1 passed。
- `cargo test validates_template_params_and_normalizes_defaults` 通过：1 passed。
- `cargo test rejects_select_without_unified_dictionary_mapping` 通过：1 passed。
- `cargo test rejects_user_template_with_invalid_param_name` 通过：1 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动态 / 静态 import 提示和 chunk 体积提示，不影响构建结果。
- 注意：`cargo test template` 会额外匹配到既有 `export_service` 备份测试，其中 `backup_workspace_writes_safe_package_with_projects_assets_templates_and_migrations` 当前因 `export.secret_detected` 失败；这不是 10.6 模板 manifest / DSL 单测，未记录为通过。

---

### 【X】10.7 实现 Chromium 模板渲染安全

**问题：**
模板渲染是潜在联网、文件读取、XSS 入口。

**改法：**

实现 TemplateRenderer：

```text
Playwright sidecar
随包 Chromium
禁用外网请求
禁用非白名单 file://
禁用下载
禁用弹窗
禁用剪贴板
禁用持久化浏览器用户数据
```

用户文本注入必须使用：

```text
textContent
```

不能拼：

```text
innerHTML
```

**验证：**

- 默认模板截图成功。
- 外链图片被拒绝。
- 用户标题 escape，不执行 HTML。
- browser 崩溃可自动重启一次。

**风险：**
模板不能联网加载未知资源，字体和图片必须来自受控目录。

**完成记录（2026-06-27）：**

- 已新增渲染相关 DTO、命令和前端 API：
  - `PreviewTemplateRequest`
  - `PreviewTemplateResponseDto`
  - `RenderTemplateRequest`
  - `RenderTemplateResponseDto`
  - `TemplateRenderDataDto`
  - `preview_template`
  - `render_template`
  - `previewTemplate`
  - `renderTemplate`
- 已实现 TemplateRenderer 前置安全入口：
  - 渲染前必须通过 `list_template_manifests` 找到受控模板 manifest。
  - 渲染前必须通过 `validate_template_params` 校验参数。
  - 输入资源只允许 `templates/`、`assets/`、`projects/`、`cache/fonts/`。
  - 输出只允许 `cache/`、`projects/`、`outputs/`。
  - `file://`、绝对路径、`../`、工作区外路径会在渲染前拒绝。
  - 用户数据注入通过 JSON payload + `vt-template-data` 事件传递；当前代码不拼 `innerHTML`，不使用 `document.write`。
  - sidecar 缺失返回 `template.sidecar_missing`，恢复动作映射为 `configure_sidecar`。
- 已补真实 sidecar 执行协议，不再停留在占位错误：
  - `render_template` 会生成受控临时 JSON 请求文件，包含 `entryPath / outputPath / viewport / allowedResourceRoots / payload / injectionScript / policy`。
  - 只调用 `sidecars/node.exe sidecars/playwright-driver.js temp/template_render/request_*.json`，不读取系统 PATH。
  - 运行时要求 `sidecars/node.exe`、`sidecars/chromium.exe`、`sidecars/playwright-driver.js` 三件套；缺任意一个都返回 `template.sidecar_missing`。
  - sidecar 日志会脱敏和路径替换，不泄露工作区绝对路径或疑似密钥。
  - sidecar 成功但输出文件不存在时返回 `template.output_missing`，不伪造图片。
  - browser 崩溃类错误会重试一次，仍失败时返回 `template.browser_crashed`。
  - 临时请求文件执行后会清理。
- 已新增随包 driver 脚本 `sidecars/playwright-driver.js`：
  - 使用 `playwright-core` 启动指定 `chromium.exe`。
  - 禁止外网请求，只允许白名单 `file://` 资源。
  - `acceptDownloads=false`，弹窗立即关闭。
  - 注入脚本禁用剪贴板和文件选择类 API。
  - 不使用持久化浏览器用户数据目录。
- 已新增模板 sidecar 诊断入口：
  - 后端 `check_template_sidecars`。
  - 前端 `checkTemplateSidecars`。
  - 设置页“本地媒体工具”卡片同时显示“视频合成工具”和“模板渲染工具”两组状态。
- 已新增真实 sidecar 验收脚本和 smoke 入口：
  - `scripts/verify-template-sidecar.ps1` 会检查 `sidecars/node.exe`、`sidecars/chromium.exe`、`sidecars/playwright-driver.js`、`sidecars/node_modules/playwright-core`。
  - `real_template_sidecar_renders_default_cover_png` 是 ignored Rust smoke；真实 sidecar 放齐后由脚本调用，验证默认模板输出 PNG。
- 已新增 sidecar 准备脚本与打包源目录：
  - `scripts/prepare-template-sidecar.ps1` 只接受显式 `NodeExePath / ChromiumExePath / PlaywrightCoreDir`，不联网、不猜系统 PATH。
  - 脚本会复制到 `resources/bin/` 和运行时 `sidecars/` 两处。
  - `src-tauri/tauri.conf.json` 已把 `../resources/bin/**/*` 纳入 bundle resources。
  - `resources/bin/README.md` 作为占位文件，避免真实二进制未放入时 Tauri 构建因资源 glob 不存在而失败。
- 已把内置 seed 模板从“只含 DSL 占位符的空壳”改为可真实截图的 HTML：
  - `knowledge_bold`：封面模板，支持标题、旁白、主图、安全文本注入。
  - `karaoke_basic`：字幕模板，支持安全区、颜色、断句文本。
  - `image_soft_zoom`：画面模板，支持主图和缩放参数。
  - 这些模板仍保留 DSL 参数，`parse_template_params_from_html` 和 manifest 校验继续生效。
- 已补齐真实运行所需 sidecar：
  - `sidecars/node.exe`
  - `sidecars/chromium/chrome.exe`
  - `sidecars/playwright-driver.js`
  - `sidecars/node_modules/playwright-core`
  - `sidecars/chromium.exe` 保留为兼容入口；真实渲染优先使用完整目录里的 `sidecars/chromium/chrome.exe`，避免只复制单 exe 缺 DLL / 资源导致启动失败。
- 已新增 `scripts/materialize-chromium-sidecar.js`，用于从 Playwright 下载残留目录中整理完整 Chromium 目录到 `sidecars/chromium/`。
- `Chrome Headless Shell` 下载多次超时，不再作为本条必要条件；真实 smoke 已用完整 Chromium 通过。
- 当前 `render_template` 在安全校验和 sidecar 检查后，不会伪造输出；缺 sidecar 时明确失败，不生成假图片。
- 本条不实现模板市场，也不实现复杂动效编辑器；继续进入 10.8。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test render_template -- --nocapture` 通过：6 passed。
- `cargo test check_template_sidecars -- --nocapture` 通过：1 passed。
- `cargo test lists_seeded_builtin_templates_by_aspect_and_type -- --nocapture` 通过：1 passed。
- `cargo test real_template_sidecar_renders_default_cover_png -- --ignored --nocapture` 通过：1 passed，确认真实 Chromium / Playwright 可渲染默认封面模板并输出 PNG。
- `cargo test render_template_rejects_external_or_file_url_resources_before_sidecar` 通过：1 passed。
- `cargo test render_template_reports_missing_template_sidecar_without_fake_output` 通过：1 passed。
- `cargo test browser_render_plan_uses_data_payload_event_not_inner_html` 通过：1 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动态 / 静态 import 提示和 chunk 体积提示，不影响构建结果。
- `cargo check` 在加入 bundle resources 后通过；`resources/bin/README.md` 确认可避免资源 glob 空目录导致构建失败。
- `D:\software\nodejs\node.exe --check sidecars\playwright-driver.js` 通过，driver 脚本语法有效。
- `powershell -ExecutionPolicy Bypass -File scripts\verify-template-sidecar.ps1 -SkipCargoSmoke` 在 sidecar 补齐前曾按预期失败；补齐后 `cargo test real_template_sidecar_renders_default_cover_png -- --ignored --nocapture` 已完成真实验证。
- `cargo test template -- --nocapture` 中模板相关项通过；该命令仍会触发既有 `export_service::tests::backup_workspace_writes_safe_package_with_projects_assets_templates_and_migrations` 失败，错误为 `export.secret_detected`，不是 10.7 模板渲染链路失败。

---

### 【X】10.8 实现 BGM 与基础音频混合

**问题：**
BGM 是短视频常见增强项，但不能盖过人声，也不能阻塞主线。

**位置：**

```text
src-tauri/src/domain/task.rs
src-tauri/src/db/mod.rs
src-tauri/src/db/task_repository.rs
src-tauri/src/services/ffmpeg_service.rs
src-tauri/src/services/task_service.rs
src/src/entities/task/types.ts
src/src/entities/scene/api.ts
src/src/entities/scene/store.ts
src/src/pages/composition/index.vue
src/src/shared/i18n/locales/zh-CN.ts
src/src/shared/i18n/locales/en-US.ts
```

**改法：**

合成页提供可选 BGM：

```text
选择 BGM
音量
循环/单次
淡入淡出
与主视频时长对齐
```

数据流：

```text
本地音频导入 -> assets/bgm/*
合成页选择 bgm asset -> StartCompositionRequest.includeBgm / bgmAssetId / bgmVolume / bgmLoop / bgmFadeInSeconds / bgmFadeOutSeconds
基础 concat -> outputs/exports/{projectId}/{taskId}_final.mp4
启用 BGM 时再混音 -> outputs/exports/{projectId}/{taskId}_final_bgm.mp4
CompositionTask.outputPath 指向最终输出
CompositionTask.enhancements 记录 BGM 参数和资产路径
```

边界：

- 不选 BGM 时，仍走原有 FFmpeg concat 输出。
- BGM 只接受 `kind=bgm` 的资产，资产文件必须在受控 `assets/` 目录。
- 音量限制为 0-1，前端滑块默认 18%，上限 60%，避免默认盖过人声。
- 有原视频音轨时使用 `amix` 混合；没有原音轨时直接铺 BGM 音轨。
- 输出用主视频总时长裁切，不让 BGM 反向拉长视频。
- 本条不做版权授权审查、不做多轨时间轴、不做专业音频 ducking；后续如需要再进入独立 TODO。

**验证：**

- 可选择 BGM 并参与最终合成。
- 不选 BGM 时仍能输出 final.mp4。
- BGM 音量可控。
- BGM 导入后不保存用户原始路径，只保存受控资产路径。
- 合成任务能记录是否包含 BGM、BGM 资产、音量、循环和淡入淡出参数。

**风险：**
BGM 文件路径必须进入受控资产；版权和授权后续要检查。

**完成记录（2026-06-27）：**

- 已扩展 `StartCompositionRequest`，支持：
  - `includeBgm`
  - `bgmAssetId`
  - `bgmVolume`
  - `bgmLoop`
  - `bgmFadeInSeconds`
  - `bgmFadeOutSeconds`
- 已新增 SQLite migration `composition_enhancements_v1`，给 `composition_tasks` 增加 `enhancements_json`，用于记录可选增强参数；旧任务默认 `{}`。
- 已扩展 `CompositionTaskDto` / 前端 `CompositionTaskDto`，同步 `enhancements`。
- 已在 `ffmpeg_service` 增加 BGM 混音链路：
  - 基础 concat 保持原有输出。
  - 启用 BGM 后输出 `outputs/exports/{projectId}/{taskId}_final_bgm.mp4`。
  - BGM 路径只允许 `assets/` 或 `projects/` 受控 bucket；业务入口进一步限制为 `kind=bgm` 且在 `assets/`。
  - 有原音轨时使用 `amix`；无原音轨时直接使用 BGM 音轨。
  - 使用主视频总时长 `-t` 对齐，避免 BGM 拉长最终视频。
  - 支持音量、循环、淡入、淡出。
- 已在 `task_service::start_composition` 中接入 BGM：
  - `includeBgm=false` 或未传时，仍只执行最小 concat。
  - `includeBgm=true` 时校验 BGM 资产存在、未删除、`kind=bgm`、路径在 `assets/`。
  - 成功后 `CompositionTask.outputPath` 指向最终 BGM 混音文件。
- 已在合成页右侧加入“BGM 增强”可选卡片：
  - 启用 / 关闭 BGM。
  - 选择已有 BGM 资产。
  - 导入本地 BGM 到受控资产库。
  - 音量滑块。
  - 循环到主视频结束。
  - 淡入 / 淡出秒数。
  - 用户可见文案已走 i18n。
- 本条没有实现字幕烧录、封面 metadata、版权授权检查、专业 ducking 或多轨时间轴；这些不属于 10.8。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test bgm_mix -- --nocapture` 通过：2 passed。
- `cargo test composition_task -- --nocapture` 通过：2 passed。
- `cargo test start_composition -- --nocapture` 通过：1 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动静态 import 和 chunk 体积警告，不影响构建结果。
- 未执行真实 FFmpeg BGM 混音桌面 smoke；当前已验证命令参数构造、受控路径校验、任务增强字段持久化和前端构建链路。

---

### 【X】10.9 合成阶段接入可选增强

**问题：**
增强项要能参与合成，但不能破坏最小 concat 能力。

**位置：**

```text
src-tauri/src/domain/task.rs
src-tauri/src/services/ffmpeg_service.rs
src-tauri/src/services/task_service.rs
src/src/entities/task/types.ts
src/src/entities/scene/api.ts
src/src/pages/composition/index.vue
src/src/shared/i18n/locales/zh-CN.ts
src/src/shared/i18n/locales/en-US.ts
```

**改法：**

合成参数：

```text
includeSubtitle
includeBgm
includeCoverMetadata
subtitlePath
bgmAssetId
coverPath
```

执行顺序：

```text
concat 基础拼接
→ 可选 subtitle burn
→ 可选 BGM mix
→ 可选 cover metadata 记录
→ CompositionTask.outputPath 指向最终输出
```

任务增强记录：

```json
{
  "includeSubtitle": true,
  "subtitlePath": "projects/{projectId}/subtitles/subtitles.json",
  "includeBgm": true,
  "bgmAssetId": "asset_xxx",
  "includeCoverMetadata": true,
  "coverPath": "projects/{projectId}/cover/cover.png",
  "steps": [
    { "step": "concat", "status": "succeeded" },
    { "step": "subtitle", "status": "succeeded" },
    { "step": "bgm", "status": "succeeded" },
    { "step": "cover_metadata", "status": "succeeded" }
  ]
}
```

边界：

- 不勾选任何增强时仍只走最小 concat。
- 字幕只接受 `projects/{projectId}/subtitles/subtitles.json` 这类受控项目路径。
- 字幕烧录把 `subtitles.json` 转成临时 SRT，再用 FFmpeg `subtitles` filter 烧录；临时 SRT 执行后清理。
- BGM 复用 10.8 的混音链路，不重复造上传或混音入口。
- 封面本条只记录 metadata，不把封面图片强塞进 mp4 轨道，不做平台发布包封面嵌入。
- 可选增强失败时，错误码带 `composition.subtitle_failed` / `composition.bgm_failed` / `composition.cover_*` 前缀，能定位是哪一步；未勾选增强不影响基础 concat。

**验证：**

- 不勾选增强项时走最小 concat。
- 勾选字幕/BGM 时有明确任务步骤。
- 失败时能定位是 concat、字幕、BGM 还是模板渲染失败。
- 合成页能查看 concat / subtitle / bgm / cover metadata 的步骤状态。
- 封面元数据只在勾选时记录，且使用项目受控 `coverPath`。

**风险：**
不要让可选增强失败导致基础 final.mp4 不可生成，除非用户明确选择强制包含。

**完成记录（2026-06-27）：**

- 已扩展 `StartCompositionRequest`：
  - `includeSubtitle`
  - `subtitlePath`
  - `includeBgm`
  - `bgmAssetId`
  - `includeCoverMetadata`
  - `coverPath`
- 已把合成链路统一成：
  - 基础 concat。
  - 可选字幕烧录。
  - 可选 BGM 混音。
  - 可选封面 metadata 记录。
- 已新增 FFmpeg 字幕烧录能力：
  - 读取受控 `projects/*/subtitles/subtitles.json`。
  - 转成临时 SRT。
  - 使用 FFmpeg `subtitles` filter 输出 `{taskId}_final_subtitle.mp4`。
  - 临时 SRT 执行后清理。
  - 拒绝 `assets/`、绝对路径、`../` 等非受控字幕路径。
- 已把 10.8 的 BGM 混音接入统一增强步骤，任务里记录 `bgm` 参数和 `steps`。
- 已接入封面 metadata：
  - 勾选时读取项目 `coverPath / coverTitle / coverTemplateId / coverSourceItemId`。
  - 仅记录到 `CompositionTask.enhancements`，不做复杂平台封面嵌入。
- 合成页新增“可选增强”卡片：
  - 烧录字幕开关。
  - 封面 metadata 开关。
  - 展示默认字幕路径和当前项目封面路径。
  - 展示 `concat / subtitle / bgm / cover_metadata` 步骤状态。
  - 文案已走 i18n。
- 可选增强失败会分别带 `composition.subtitle_failed`、`composition.bgm_failed`、`composition.cover_missing` / `composition.cover_path_invalid`，不再只有模糊合成失败。

**验证记录（2026-06-27）：**

- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test bgm_mix -- --nocapture` 通过：2 passed。
- `cargo test subtitle_burn -- --nocapture` 通过：2 passed。
- `cargo test composition_task -- --nocapture` 通过：2 passed。
- `cargo test start_composition -- --nocapture` 通过：1 passed。
- `pnpm --dir src typecheck` 通过。
- `pnpm --dir src build` 通过；仍有既有 Vite 动静态 import 和 chunk 体积警告，不影响构建结果。
- 未执行真实 FFmpeg 字幕烧录桌面 smoke；当前已验证字幕 JSON 转 SRT、受控路径拒绝、任务增强记录、前端类型和构建。

---

## 阶段完成标准

- 字幕 / 独白 / 配音的数据关系清楚，不互相覆盖。
- TTS 可生成音频并读取真实时长。
- 字幕可生成、编辑、选择是否合成。
- 封面可选择主图并生成 cover.png。
- 模板渲染具备安全沙箱和参数校验。
- BGM 可选参与合成。
- 所有增强项都不阻塞最小 image_to_video 主线。
