# ACCEPTANCE

## 完成层级

```text
ui              只有页面和交互壳
tauri_sqlite    走 Tauri command 并写入 SQLite
controlled_fake 使用受控 fake provider 证明流程
real_provider   调用真实外部 Provider 并产生真实结果
real_ffmpeg     调用真实 ffmpeg/ffprobe 并产生可播放文件
release_smoke   发布包环境验证通过
```

不能把低层级写成高层级。

当前项目特别规则：

```text
OpenAI-compatible LLM /chat/completions 真实请求成功后，LLM 连接测试和 LLM 分镜可标记 real_provider。
OpenAI-compatible image /images/generations 真实请求成功并写入受控 workspace 后，生图可标记 real_provider。
OpenAI-compatible TTS /audio/speech 真实请求成功并写入受控 workspace 后，配音可标记 real_provider。
OpenAI-compatible VLM /chat/completions 真实请求成功且输入图片来自受控 assets/ 后，图片理解可标记 real_provider。
video / workflow 仍走未实现 adapter 时，不能标记 real_provider。
没有真实 ffmpeg/ffprobe sidecar 和可播放 final.mp4 之前，不能标记 real_ffmpeg。
任务中心已有 /tasks 路由和页面，已展示步骤错误、尝试历史、Artifact 和基础资源摘要；没有供应商价格表和真实计费回执前不能展示费用金额。
```

## 通用验收

1. 功能入口存在。
2. 主按钮、批量按钮、行内按钮、危险按钮明确。
3. 状态机覆盖空态、运行中、失败、成功、可重试。
4. 数据读取来源明确。
5. 数据写回字段明确。
6. 调用的 Tauri command 明确。
7. 失败能定位到步骤、任务和必要的行号。
8. 日志脱敏。
9. 路径安全。
10. 验证命令或 smoke 步骤已记录。

## 生成类验收

1. 不直接从 Vue 页面请求 Provider。
2. 真实密钥只通过 keyring 读取。
3. 任务进入 Task / TaskStep / Attempt / Artifact。
4. 失败可重试。
5. 取消后保留已完成产物。
6. 候选结果追加，不覆盖历史。
7. 用户最终确认写回 selected 字段。
8. 普通“测试连接”不得触发计费型生成；真实生成测试必须显式确认。

## 不能算完成

1. 只有 UI，没有 Tauri/SQLite 写回。
2. 浏览器 mock 成功，但桌面 Tauri 没验证。
3. controlled fake 成功，但写成真实 Provider。
4. Provider 配置能保存，但不能测试。
5. 生成按钮能点，但失败后没有任务记录。
6. 候选被新结果覆盖，无法撤回。
7. 密钥、绝对路径或完整请求头进入日志、导出包、诊断包。
8. final.mp4 只写了记录，没有真实文件或无法播放。

## 验证记录格式

```text
任务：
完成层级：
改动文件：
验证命令：
smoke 步骤：
未执行项：
风险：
下一步：
```
