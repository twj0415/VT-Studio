# F-010-003 WebAV mp4 导出

状态：等待用户确认  
所属菜单：M-010 导出  
对应功能文档：`docs/features/M-010-导出.md`  
原则：先确认本文档，再改代码

## 0. 快速理解

```txt
一句话：内置视频编辑器可以用 WebAV 把当前时间线合成为 mp4。
为什么现在做：这是参考项目已有的快速视频导出能力，但它不是剪映草稿。
做完后有什么用：用户可以把内置时间线快速导出成一个 mp4 文件。
这一步不碰什么：不写剪映草稿，不保存导出历史，不进入视频生成模型。
```

## 1. 本次做什么

```txt
目标：
  对齐 Toonflow videoPreview.exportVideo 的 WebAV mp4 导出。

只做：
  1. 导出按钮
  2. 导出前检查
  3. WebAV combinator 合成 mp4
  4. 下载或保存 mp4
  5. 导出错误提示

不做：
  1. 剪映草稿导出
  2. 导出历史
  3. 任务中心记录
  4. 后端渲染
```

## 2. 参考项目怎么做

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-web/src/views/production/components/workbench/editVideo/videoPreview.vue` | `exportVideo()` 创建 WebAV combinator，输出 chunks，Blob 下载 |
| `Toonflow-web/src/views/production/components/workbench/editVideo/index.vue` | 点击底部导出视频按钮调用 videoPreviewRef.exportVideo |
| `.../utils/transitionRenderers.ts` | 转场渲染 |
| `.../utils/filterEffect.ts` | 滤镜效果 |

参考项目关键事实：

```txt
1. avCanvas 不存在时报 avCanvasNotInit。
2. clipSpriteMap 为空时报 noExportContent。
3. 播放中导出会先暂停。
4. createCombinator 后读取 Uint8Array chunks，生成 video/mp4 Blob。
5. 下载名是 WebAV-export-${Date.now()}.mp4。
6. WebAV 内部大量使用微秒，vue-clip-track 使用秒，时间单位必须明确。
7. WebAV 导出没有任务中心记录，也没有导出历史。
```

## 3. 用户操作

```txt
入口：
  内置视频编辑器底部工具栏 -> 导出视频。

按钮/操作：
  点击导出视频。

弹窗/表单：
  可选保存路径弹窗。

成功反馈：
  mp4 文件保存或下载完成。

失败反馈：
  没有内容、画布未初始化、素材缺失、WebAV 合成失败。
```

## 4. 要做什么功能

### 1. 导出前检查

怎么做：
- 输入：AVCanvas、clipSpriteMap、时间线状态。
- 输出：可导出或错误。
- 写什么数据：不写。
- 状态怎么变：isExporting 校验前后更新。
- 异常怎么处理：没有内容或画布未初始化直接提示。
- 限制：播放中先暂停。

### 2. WebAV 合成

怎么做：
- 输入：当前 timeline sprites。
- 输出：mp4 chunks。
- 写什么数据：不写数据库。
- 状态怎么变：exporting -> succeeded/failed。
- 异常怎么处理：WebAV 抛错显示可读原因。
- 限制：时间单位统一转换，不能秒/微秒混用。

### 3. 保存 mp4

怎么做：
- 输入：mp4 Blob 或 buffer。
- 输出：文件路径或下载。
- 写什么数据：写用户选择目录或 downloads。
- 状态怎么变：导出完成。
- 异常怎么处理：写文件失败提示。
- 限制：这不是剪映草稿，不写剪映目录。

## 5. 数据和状态

```txt
字段：
  timeline tracks/clips
  clipSpriteMap
  avCanvas

接口/能力：
  export.webAvMp4 可选

数据读写：
  前端合成 Blob
  可选 main 保存文件

任务状态：
  参考项目不写 tasks；第一版可不写

轮询/Socket：
  无

模型调用：
  无

删除影响：
  无
```

## 6. VT Studio 怎么落

```txt
能力名：
  export.webAvMp4

调用链：
  renderer WebAV 编辑器 -> WebAV -> 可选 window.vtStudio.file.save

需要新增：
  WebAV 导出按钮和状态
  导出前素材检查

需要修改：
  时间单位转换工具
```

## 7. 偏差

```txt
和 Toonflow 不同的地方：
  导出前增加素材缺失校验和时间单位明确转换。

原因：
  参考项目合成中失败才报错，用户定位困难；时间单位混用风险高。

是否写入 04：
  不需要，属于健壮性增强。
```

## 8. 验收

```txt
1. 没有内容不能导出。
2. 画布未初始化不能导出。
3. 播放中导出会先暂停。
4. 有内容时能合成 mp4。
5. 导出文件名清楚。
6. WebAV 导出不等于剪映草稿。
7. 时间单位转换明确。
8. 失败有可读提示。
```

## 9. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-F-010-003-001 | WebAV mp4 导出是否写任务中心 | 建议第一版不写，除非后续发现合成耗时很长 |

## 10. 执行后记录

```txt
改了哪些文件：（完成后填写）
验证结果：（完成后填写）
未完成事项：（完成后填写）
最终结论：（完成后填写）
```

## 11. 最后大白话

```txt
我这次准备怎么做：
1. 内置编辑器当前时间线可以导出 mp4。
2. 导出前检查有没有内容和缺失素材。
3. 这只是 mp4 快速导出，不是剪映草稿。

我不会做什么：
1. 不写剪映草稿。
2. 不保存导出历史。
3. 不调用视频生成模型。

确认规则：
用户确认后才执行；未确认前只改文档。
```
