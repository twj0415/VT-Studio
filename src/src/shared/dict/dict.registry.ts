import type { AppLocale, AspectRatio, ContentLanguage, ErrorKind, FileAccessPolicy, FileBucket, InputProcessMode, InputType, LayoutDensity, ModelCapability, ProjectLifecycle, ProviderAuthType, ProviderKind, ProviderStatus, ProviderVendor, SceneAssetStatus, TaskKind, TaskStatus, TaskStepKind, TaskStepStatus, ThemePreset, WorkflowType } from '@/shared/enums/generated'

export interface DictOption<TValue extends string = string> {
  label: string
  value: TValue
  colorToken?: string
  disabled?: boolean
}

export const inputTypeOptions: DictOption<InputType>[] = [
  { label: '一句话想法', value: 'topic' },
  { label: '文案粘贴', value: 'paste' },
  { label: '文章 / 笔记', value: 'article' },
  { label: '小说导入', value: 'novel', disabled: true },
  { label: '素材导入', value: 'material', disabled: true },
]

export const appLocaleOptions: DictOption<AppLocale>[] = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en-US' },
]

export const themePresetOptions: DictOption<ThemePreset>[] = [
  { label: '石墨深蓝', value: 'graphite' },
  { label: '极光墨青', value: 'aurora' },
  { label: '琥珀暖夜', value: 'ember' },
  { label: '瓷白晨雾', value: 'porcelain' },
  { label: '沙岩暖白', value: 'sandstone' },
]

export const layoutDensityOptions: DictOption<LayoutDensity>[] = [
  { label: '舒适', value: 'comfortable' },
  { label: '紧凑', value: 'compact' },
]

export const workflowTypeOptions: DictOption<WorkflowType>[] = [
  { label: '图生视频成片', value: 'image_to_video' },
  { label: '数字人口播', value: 'digital_human', disabled: true },
  { label: '素材剪辑成片', value: 'material_edit' },
  { label: '纯图文成片', value: 'image_slideshow' },
]

export const contentLanguageOptions: DictOption<ContentLanguage>[] = [
  { label: '中文', value: 'zh-CN' },
  { label: '英文', value: 'en-US' },
]

export const inputProcessModeOptions: DictOption<InputProcessMode>[] = [
  { label: 'AI 整理', value: 'generate' },
  { label: '保留原文', value: 'fixed' },
]

export const aspectRatioOptions: DictOption<AspectRatio>[] = [
  { label: '竖屏 9:16', value: '9:16' },
  { label: '横屏 16:9', value: '16:9' },
  { label: '方形 1:1', value: '1:1' },
]

export const taskStatusOptions: DictOption<TaskStatus>[] = [
  { label: '待处理', value: 'pending', colorToken: 'status.pending' },
  { label: '运行中', value: 'running', colorToken: 'status.running' },
  { label: '已完成', value: 'succeeded', colorToken: 'status.succeeded' },
  { label: '失败', value: 'failed', colorToken: 'status.failed' },
  { label: '已取消', value: 'cancelled', colorToken: 'status.cancelled' },
]

export const projectLifecycleOptions: DictOption<ProjectLifecycle>[] = [
  { label: '草稿', value: 'draft', colorToken: 'status.pending' },
  { label: '进行中', value: 'active', colorToken: 'status.running' },
  { label: '已归档', value: 'archived', colorToken: 'status.succeeded' },
  { label: '已删除', value: 'deleted', colorToken: 'status.cancelled' },
]

export const taskKindOptions: DictOption<TaskKind>[] = [
  { label: '图生视频作品', value: 'image_to_video' },
  { label: '数字人口播', value: 'digital_human' },
  { label: '素材剪辑成片', value: 'material_edit' },
  { label: '纯图文成片', value: 'image_slideshow' },
]

export const taskStepKindOptions: DictOption<TaskStepKind>[] = [
  { label: '作品初始化', value: 'project_init' },
  { label: '生成分镜', value: 'storyboard_generation' },
  { label: '确认分镜', value: 'storyboard_review' },
  { label: '生成生图提示词', value: 'image_prompt_generation' },
  { label: '生成图片', value: 'image_generation' },
  { label: '确认图片', value: 'image_review' },
  { label: '生成视频提示词', value: 'video_prompt_generation' },
  { label: '生成视频', value: 'video_generation' },
  { label: '确认视频', value: 'video_review' },
  { label: '最终合成', value: 'final_composition' },
  { label: '导出', value: 'export' },
  { label: '清理', value: 'cleanup' },
  { label: '确认口播文案', value: 'script_review' },
  { label: '确认数字人素材', value: 'digital_human_asset_review' },
  { label: '生成配音', value: 'tts_generation' },
  { label: '生成数字人视频', value: 'digital_human_generation' },
  { label: '生成字幕', value: 'subtitle_generation' },
  { label: '导入素材', value: 'material_import' },
  { label: '分析素材', value: 'material_analysis' },
  { label: '素材分组', value: 'material_grouping' },
  { label: '匹配素材', value: 'material_matching' },
  { label: '片段剪辑', value: 'segment_composition' },
  { label: '模板动效', value: 'template_motion' },
]

export const taskStepStatusOptions: DictOption<TaskStepStatus>[] = [
  { label: '待处理', value: 'pending', colorToken: 'status.pending' },
  { label: '运行中', value: 'running', colorToken: 'status.running' },
  { label: '重试中', value: 'retrying', colorToken: 'status.retrying' },
  { label: '等待确认', value: 'waiting_user', colorToken: 'status.waiting_user' },
  { label: '已完成', value: 'succeeded', colorToken: 'status.succeeded' },
  { label: '失败', value: 'failed', colorToken: 'status.failed' },
  { label: '已取消', value: 'cancelled', colorToken: 'status.cancelled' },
  { label: '已跳过', value: 'skipped', colorToken: 'status.skipped' },
]

export const sceneAssetStatusOptions: DictOption<SceneAssetStatus>[] = [
  { label: '等待中', value: 'pending', colorToken: 'status.pending' },
  { label: '生成中', value: 'running', colorToken: 'status.running' },
  { label: '已生成', value: 'succeeded', colorToken: 'status.succeeded' },
  { label: '失败', value: 'failed', colorToken: 'status.failed' },
  { label: '跳过', value: 'skipped', colorToken: 'status.skipped' },
]

export const providerKindOptions: DictOption<ProviderKind>[] = [
  { label: '文字模型', value: 'llm' },
  { label: '配音模型', value: 'tts' },
  { label: '生图模型', value: 'image' },
  { label: '视频模型', value: 'video' },
  { label: '视觉理解', value: 'vlm' },
  { label: '工作流', value: 'workflow' },
]

export const providerVendorOptions: DictOption<ProviderVendor>[] = [
  { label: 'OpenAI', value: 'openai' },
  { label: 'SiliconFlow', value: 'siliconflow' },
  { label: '火山引擎', value: 'volcengine' },
  { label: 'ComfyUI', value: 'comfyui' },
  { label: 'RunningHub', value: 'runninghub' },
  { label: '自定义', value: 'custom' },
]

export const providerStatusOptions: DictOption<ProviderStatus>[] = [
  { label: '禁用', value: 'disabled', colorToken: 'status.skipped' },
  { label: '可用', value: 'available', colorToken: 'status.succeeded' },
  { label: '异常', value: 'error', colorToken: 'status.failed' },
]

export const providerAuthTypeOptions: DictOption<ProviderAuthType>[] = [
  { label: '无认证', value: 'none' },
  { label: 'API Key', value: 'api_key' },
  { label: 'Bearer Token', value: 'bearer_token' },
  { label: 'Basic Auth', value: 'basic' },
  { label: '自定义请求头', value: 'custom_header' },
]

export const modelCapabilityOptions: DictOption<ModelCapability>[] = [
  { label: '文本生成', value: 'text_generation' },
  { label: '结构化输出', value: 'structured_output' },
  { label: '文生图', value: 'text_to_image' },
  { label: '图生图 / 改图', value: 'image_to_image' },
  { label: '文生视频', value: 'text_to_video' },
  { label: '图生视频', value: 'image_to_video' },
  { label: '首帧图生视频', value: 'first_frame_i2v' },
  { label: '首尾帧视频', value: 'start_end_frame_i2v' },
  { label: '参考图视频', value: 'reference_to_video' },
  { label: '视频续写', value: 'video_continuation' },
  { label: '视频编辑', value: 'video_editing' },
  { label: '动作迁移', value: 'action_transfer' },
  { label: '数字人', value: 'digital_human' },
  { label: '原生音频', value: 'native_audio' },
  { label: '声音参考', value: 'voice_reference' },
  { label: '多镜头', value: 'multi_shot' },
  { label: '文本转语音', value: 'text_to_speech' },
  { label: '视觉分析', value: 'vision_analysis' },
  { label: '工作流执行', value: 'workflow_execution' },
]

export const fileBucketOptions: DictOption<FileBucket>[] = [
  { label: '作品', value: 'project' },
  { label: '素材', value: 'asset' },
  { label: '输出', value: 'output' },
  { label: '缓存', value: 'cache' },
  { label: '临时', value: 'temp' },
  { label: '日志', value: 'log' },
  { label: '模板', value: 'template' },
  { label: '本地工具', value: 'sidecar' },
]

export const fileAccessPolicyOptions: DictOption<FileAccessPolicy>[] = [
  { label: '只读', value: 'read_only' },
  { label: '作品内写入', value: 'write_project' },
  { label: '导出复制', value: 'export_copy' },
  { label: '仅临时目录', value: 'temp_only' },
]

export const errorKindOptions: DictOption<ErrorKind>[] = [
  { label: '校验错误', value: 'validation' },
  { label: '认证错误', value: 'auth' },
  { label: '限流', value: 'rate_limit' },
  { label: '网络错误', value: 'network' },
  { label: '生成服务错误', value: 'provider' },
  { label: '存储错误', value: 'storage' },
  { label: '工作流错误', value: 'workflow' },
  { label: 'FFmpeg 错误', value: 'ffmpeg' },
  { label: '数据库错误', value: 'db' },
  { label: '安全拦截', value: 'security' },
  { label: '未知错误', value: 'unknown' },
]

export const dictRegistry = {
  appLocale: appLocaleOptions,
  themePreset: themePresetOptions,
  layoutDensity: layoutDensityOptions,
  inputType: inputTypeOptions,
  inputProcessMode: inputProcessModeOptions,
  workflowType: workflowTypeOptions,
  contentLanguage: contentLanguageOptions,
  aspectRatio: aspectRatioOptions,
  projectLifecycle: projectLifecycleOptions,
  taskKind: taskKindOptions,
  taskStatus: taskStatusOptions,
  taskStepKind: taskStepKindOptions,
  taskStepStatus: taskStepStatusOptions,
  sceneAssetStatus: sceneAssetStatusOptions,
  providerKind: providerKindOptions,
  providerVendor: providerVendorOptions,
  providerStatus: providerStatusOptions,
  providerAuthType: providerAuthTypeOptions,
  modelCapability: modelCapabilityOptions,
  fileBucket: fileBucketOptions,
  fileAccessPolicy: fileAccessPolicyOptions,
  errorKind: errorKindOptions,
} as const
