# Provider、任务与安全

## Provider 分层

```text
Provider
ProviderModel
WorkflowPreset
CreativeRule
VideoPack
ProjectRuntimeConfig
TaskStep snapshot
```

## Provider

只保存连接信息：

```text
providerKind
vendor
displayName
baseUrl
authType
keyAlias
enabled
```

不能保存：

```text
modelName
workflow nodeMap
workflow outputMap
真实 API Key
```

## ProviderModel

保存 API 模型能力：

```text
modelName
abilityTypes
inputModalities
outputModalities
limits
featureFlags
enabled
```

## WorkflowPreset

保存 ComfyUI / RunningHub / 本地 workflow：

```text
workflowKey
workflowId
workflowVersion
paramSchema
nodeMap
outputMap
limits
enabled
```

未注册或 disabled 的 preset 不能进入业务页选择器。

## CreativeRule

保存：

```text
prompt_body
output_schema
params_schema
test_case
```

规则不直接调用 Provider，不直接写业务表。

## VideoPack

只保存默认策略组合：

```text
默认画幅
默认时长
默认分镜数
规则引用
推荐 ProviderModel / WorkflowPreset 引用
素材引用
```

不保存 API Key、Provider 连接、prompt 正文副本、候选产物、任务日志。

## 测试模式

```text
dry_run        不产生高成本媒体，用于校验配置和输入
real_generate 真实调用外部 Provider，必须二次确认
```

真实调用成功必须满足：

```text
真实请求发出
真实响应成功
产物下载到受控目录
数据库写入候选或任务结果
任务记录可查看
日志脱敏
失败可重试
```

## 当前代码状态

```text
ProviderManager 已存在。
Provider / ProviderModel / WorkflowPreset / dry_run / real_generate 的配置和测试入口已有基础。
OpenAI-compatible LLM 已实现真实 /chat/completions 请求。
OpenAI-compatible image / tts / vlm 已有最小真实 adapter，Provider 返回 bytes 后由 Service 写入受控 workspace。
video / workflow 已有通用 HTTP 异步轮询 adapter；workflow 另有 ComfyUI 最小 adapter。
LLM 失败会脱敏并映射 provider.auth_failed / provider.rate_limited / provider.timeout / provider.invalid_response / provider.output_missing 等错误。
ProviderAdapterPlaceholder 仍会拒绝 dummy / mock / controlled_fake，不允许伪装真实成功。
未做真实 key 和真实产物 smoke 的能力不能标 real_provider。
```

## 任务中心

任务中心是一级入口，不是附属历史页。

页面区域：

```text
顶部：筛选、搜索、状态统计
左侧：任务列表
中间：任务步骤和行级结果
右侧：错误详情、Artifact、脱敏日志、恢复建议
底部：批量操作和已用时
```

状态：

```text
pending
running
failed
cancelled
succeeded
```

说明：

```text
当前代码枚举是 pending / running / succeeded / failed / cancelled。
waiting_review、partial_failed、queued 等只能作为产品目标，不能在前端直接写死判断；要使用前必须先扩展 shared enums、Rust domain、数据库和字典。
```

操作：

```text
打开作品
跳到步骤
跳到分镜行
重试失败项
重试整个任务
取消任务
继续任务
打开输出目录
导出诊断包
复制错误码
```

显示规则：

```text
显示总数、成功数、失败数、当前步骤、已用时
不显示预计剩余时间
ProgressEvent 只做实时展示，刷新后以 SQLite 为准
失败必须显示错误码、脱敏错误、恢复建议
行级生成失败必须显示分镜行号和 itemId
资源消耗没有真实统计时显示“未接入统计”
```

## 存储安全

1. 真实密钥只进 keyring。
2. 数据库只保存 keyAlias。
3. 媒体文件进入受控工作区。
4. 数据库只保存相对路径。
5. 读取、复制、导出必须经过 StorageService / PathGuard。
6. 禁止 zip slip。
7. 日志、导出包、诊断包不能包含 API Key、Bearer Token、完整请求头、本机绝对路径。

## 导出和诊断

支持：

```text
导出 final.mp4
导出项目包
导出备份
导出诊断包
打开输出目录
```

导出前检查：

```text
文件存在
路径在受控目录
不包含密钥
不包含完整请求头
不包含本机绝对路径
引用文件完整
```

真实 FFmpeg 通过必须证明：

```text
ffmpeg 存在
ffprobe 存在
输入片段存在
ffprobe 可读
concat 或转码成功
final.mp4 可播放
错误日志脱敏和截断
```
