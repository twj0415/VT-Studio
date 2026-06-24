# 配音 TTS

> 这是给 AI 开发用的功能说明书。读完这一篇，就能把“配音 TTS”模块从零写出来，不用再问。
> 配套阅读：Provider 与安全看 `底层设计/Provider与安全.md`；音频字段看 `底层设计/数据结构.md`；当前开发顺序看项目 `plan/阶段路线图.md`。

## 一、这个功能是什么

配音 TTS 模块负责把分镜文案转成音频文件，并读取真实音频时长。

它要做，但不是当前 `image_to_video` 主线的前置条件。

当前主线可以先走：

```text
输入文字 → 分镜 → 生图 → 图生视频 → 合成
```

启用 TTS 后，它作为增强步骤插入：

```text
分镜文案 → TTS 音频 → 字幕对齐 / 口播 / 纯图文模板成片 / 带声音合成
```

写回字段建议：

```text
StoryboardItem.audioPath
StoryboardItem.audioDurationSeconds
StoryboardItem.durationSeconds（启用音频驱动时可回填）
```

---

## 二、子功能清单

1. 选择音色和语速
2. 批量生成配音
3. 读取真实音频时长
4. 单条重配音
5. 上传 / 替换音频
6. ComfyUI TTS 或云 TTS 扩展

---

## 子功能 1：选择音色和语速

### 界面

- 音色下拉：中文男声、中文女声、英文男声等。
- 语速滑块：0.5x ~ 2.0x，默认 1.2x。
- 试听按钮。

### 系统逻辑

1. 读取默认 TTS 配置。
2. 用户选择 `voiceId` 和 `ttsSpeed`。
3. 保存到项目或任务参数。
4. 每条 StoryboardItem 默认使用该配置，单条可覆盖。

### 参考来源

- Pixelle `pixelle_video/tts_voices.py::EDGE_TTS_VOICES`。
- Pixelle `pixelle_video/tts_voices.py::speed_to_rate`。

### 坑提醒

- 字段命名要明确：`voice_id`、`tts_speed`、`audio_duration_seconds`，不要写成 voice/speed/dur 这种含糊名字。

---

## 子功能 2：批量生成配音

### 界面

- 分镜音频列表。
- 每条显示：文案、音色、状态、时长、试听。
- 按钮：生成全部配音 / 重试失败。

### 系统逻辑

1. 读取 StoryboardItem[]，筛选 `audioStatus=pending`。
2. 对每条 StoryboardItem 调 TTSProvider。
3. 输出文件路径：`audio/item_001.mp3`。
4. 写入 `audioPath`。
5. 读取真实音频时长。
6. 写入 `audioDurationSeconds`。
7. 如果当前 workflow 启用音频驱动时长，可同步覆盖 `durationSeconds`。
8. `audioStatus=succeeded`。
9. 失败写 `audioStatus=failed / lastErrorJson / retryCount`。

### 参考来源

- Pixelle `pixelle_video/services/tts_service.py::TTSService`。
- Pixelle `pixelle_video/services/tts_service.py::_call_local_tts`。
- Pixelle `pixelle_video/utils/tts_util.py::edge_tts`。
- Pixelle `pixelle_video/services/frame_processor.py::_step_generate_audio`。

### 坑提醒

- Toonflow 的 TTS 是空壳，不能抄。它的 `AiAudio.run` 静默吞错，供应商 TTS 多数未真正实现。
- TTS 失败不能静默跳过；但未启用 TTS 时，也不能阻塞 image_to_video 主线。

---

## 子功能 3：读取真实音频时长

### 系统逻辑

1. TTS 生成 mp3 后调用 FFprobe 或音频库读取时长。
2. 写 `audioDurationSeconds`。
3. 如果 workflow 需要音频驱动，同步覆盖 `durationSeconds`。
4. 更新 Storyboard.totalDurationSeconds。

### 参考来源

- Pixelle `pixelle_video/services/frame_processor.py::_step_generate_audio`。
- Pixelle `pixelle_video/utils/tts_util.py::get_audio_duration_seconds`。
- Pixelle `pixelle_video/services/video.py::_get_audio_duration_seconds`。

### 坑提醒

- 没有真实音频时长时，不要假装精确。
- 时长单位统一秒，字段名用 `_duration_seconds`。

---

## 子功能 4：单条重配音

### 界面

- 每条音频有试听、重生成、替换。
- 支持单条改音色/语速。

### 系统逻辑

1. 用户点击某条 StoryboardItem 重配音。
2. 重新生成音频。
3. 新音频成功后替换 `audioPath`。
4. 重新读取 durationSeconds。
5. 如果字幕或合成依赖音频，相关状态重置 pending。

### 坑提醒

- 音频变了，依赖音频的字幕/合成必须重新检查。

---

## 子功能 5：上传 / 替换音频

### 界面

- 上传 mp3/wav。
- 试听。
- 使用此音频。

### 系统逻辑

1. 文件经 PathGuard 复制到任务 audio 目录。
2. 读取时长。
3. 写入 `audioPath / audioDurationSeconds`。
4. 重置依赖音频的后续状态。

### 坑提醒

- 不要直接引用用户原路径。

---

## 子功能 6：ComfyUI TTS 或云 TTS 扩展

### 系统逻辑

默认 Edge TTS。同接口扩展：

- Azure TTS
- 火山 TTS
- 阿里云 TTS
- ElevenLabs
- ComfyUI TTS 工作流

### 参考来源

- Pixelle `pixelle_video/services/tts_service.py::TTSService.__call__`。
- Pixelle `pixelle_video/services/tts_service.py::_call_comfyui_workflow`。

### 坑提醒

- 云 TTS API Key 必须存 keyring。
- 不同 Provider 的语速、音色参数差异要封装在 Provider 内。

---

## 四、这个模块对外提供什么

写回 StoryboardItem 或音频资产：

```json
{
  "itemId": "item_001",
  "audioPath": "audio/item_001.mp3",
  "audioDurationSeconds": 4.82,
  "audioStatus": "succeeded"
}
```

后续：

- `10-字幕.md` 可用音频时长做字幕时间轴。
- `13-视频合成.md` 可在启用音频时混音。
- `15-数字人口播.md` 会强依赖声音和口播音频。

