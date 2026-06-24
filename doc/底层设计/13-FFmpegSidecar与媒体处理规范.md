# FFmpeg Sidecar 与媒体处理规范

> 这篇定义 FFmpeg / FFprobe 的打包、自检、命令封装、音画对齐、拼接、BGM 和错误处理。业务代码不得直接拼 FFmpeg 命令。

## 一、核心原则

```text
1. 桌面端使用 sidecar，不依赖用户系统安装 FFmpeg。
2. 所有命令参数使用 args 数组，禁止 shell 字符串拼接。
3. 所有输入输出路径必须来自 StorageService + PathGuard。
4. FFprobe 读取真实媒体信息，不能只相信 Provider 返回值。
5. 片段合成必须以 TTS 音频真实时长为主时钟。
6. 失败必须映射 AppError，不能只返回 stderr 字符串。
```

---

## 二、sidecar 布局

`resources/bin` 是打包源目录：

```text
resources/bin/ffmpeg.exe
resources/bin/ffprobe.exe
```

运行时由 StorageService 解析到 FileBucket `sidecars`：

```text
sidecars/ffmpeg.exe
sidecars/ffprobe.exe
```

启动自检：

```text
1. 文件存在。
2. 可执行。
3. `ffmpeg -version` 成功。
4. `ffprobe -version` 成功。
5. 版本号写入诊断信息。
```

缺失时：

```text
错误码：ffmpeg.not_found
用户提示：媒体合成组件缺失，请重新安装应用。
```

---

## 三、服务分层

```text
FfmpegService          业务入口
FfmpegCommandBuilder   只构造 args
FfprobeService         读取媒体信息
ProcessRunner          启动/取消/超时/日志截断
```

业务方法：

```text
probe_media
create_video_from_image_audio
merge_audio_video
concat_segments
mix_bgm
extract_frame
normalize_segment
```

---

## 四、媒体探测

`probe_media(path)` 返回：

```ts
interface MediaProbeDto {
  path: string
  mediaKind: MediaKind
  durationSeconds: number
  width?: number
  height?: number
  fps?: number
  videoCodec?: string
  audioCodec?: string
  sampleRate?: number
  bitRate?: number
  formatName?: string
}
```

规则：

```text
1. TTS 完成后必须 probe audio。
2. AI 视频完成后必须 probe video。
3. 最终导出后必须 probe final.mp4。
```

---

## 五、默认编码参数

第一版统一：

```text
container: mp4
video_codec: libx264
pix_fmt: yuv420p
fps: 30
crf: 18
preset: veryfast
audio_codec: aac
audio_bitrate: 192k
sample_rate: 44100
```

导出配置可覆盖，但必须经过 ExportConfig 校验。

---

## 六、图片 + 音频生成片段

输入：

```text
rendered_frame_path
或 image_path
audio_path
audio_duration_seconds
```

命令意图：

```text
ffmpeg
-loop 1
-t <audio_duration_seconds>
-i frame.png
-i audio.mp3
-r 30
-c:v libx264
-pix_fmt yuv420p
-c:a aac
-shortest
segments/scene_001.mp4
```

规则：

```text
1. 启用 TTS/纯图文成片时，时长可来自 ffprobe 的音频真实时长。
2. 图片转视频必须补足指定时长。
3. 输出 segment_path 写回 VideoSegment 或对应 Artifact；当前 image_to_video 主线优先使用 VideoSegment.video_path。
```

---

## 七、已有视频 + 音频对齐

用于 AI 视频或用户素材。

策略：

```text
视频短于音频：默认 freeze 最后一帧补齐
视频长于音频：超过 duration_tolerance 则裁切
默认 tolerance：0.3 秒
```

输出：

```text
segments/scene_xxx.mp4
```

规则：

```text
1. 不允许最终 segment 音画时长明显不一致。
2. 对齐后的时长写入 VideoSegment.duration_seconds 或 StoryboardItem.duration_seconds。
3. 原始 AI 视频路径保留在 VideoSegment.video_path。
```

---

## 八、片段拼接

拼接顺序：

```text
StoryboardItem.item_index ASC
```

优先：

```text
concat demuxer
```

失败降级：

```text
concat filter
```

filelist 临时文件规则：

```text
1. 写入任务 temp 目录，不写系统临时目录。
2. filelist 路径必须 SafePath。
3. 每个 segment 路径必须转为 FFmpeg 可识别路径。
4. 用完删除临时 filelist。
```

---

## 九、BGM 混音

配置：

```ts
interface BgmConfig {
  enabled: boolean
  assetId?: string
  relativePath?: string
  volume: number
  mode: 'loop' | 'once'
  fadeInSeconds?: number
  fadeOutSeconds?: number
}
```

默认值：

```text
volume = 0.2
mode = loop
fade_in = 0
fade_out = 2
```

规则：

```text
1. BGM 不能盖过人声。
2. BGM 文件必须来自 assets/bgm 或用户导入资产。
3. fade_out 必须按最终视频时长计算。
```

---

## 十、封面和尾帧提取

`extract_frame(video_path, time_seconds, output_path)`：

```text
1. 用于封面候选图。
2. 用于 AI 视频尾帧续接。
3. time_seconds 超出时长时自动 clamp。
```

---

## 十一、取消与进程清理

ProcessRunner 必须支持：

```text
1. cancel token。
2. timeout。
3. kill 子进程。
4. Windows 下清理子进程树。
5. 删除未完成临时文件。
```

取消映射：

```text
用户取消 → task step cancelled
进程异常退出 → ffmpeg.process_failed
超时 → ffmpeg.timeout
```

---

## 十二、日志和 stderr

规则：

```text
1. stderr 最多保存最后 200 行或 32KB。
2. 日志记录 args 摘要，不记录用户绝对路径。
3. 诊断包可包含脱敏后的 FFmpeg 日志。
```

---

## 十三、错误码

```text
ffmpeg.not_found
ffmpeg.probe_failed
ffmpeg.invalid_media
ffmpeg.process_failed
ffmpeg.timeout
ffmpeg.concat_failed
ffmpeg.audio_video_mismatch
ffmpeg.output_missing
```

---

## 十四、禁止事项

```text
1. 禁止 Command 直接拼 FFmpeg。
2. 禁止 shell=true 执行命令。
3. 禁止使用未经过 PathGuard 的路径。
4. 禁止把系统绝对路径写入数据库。
5. 禁止静默吞掉 stderr。
```
