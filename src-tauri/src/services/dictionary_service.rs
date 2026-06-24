use crate::domain::dictionary::{DictionaryDto, DictionaryItemDto};

const DICTIONARY_CODES: &[&str] = &[
    "appLocale",
    "themePreset",
    "layoutDensity",
    "inputType",
    "inputProcessMode",
    "workflowType",
    "contentLanguage",
    "aspectRatio",
    "projectLifecycle",
    "taskKind",
    "taskStatus",
    "taskStepKind",
    "taskStepStatus",
    "sceneAssetStatus",
    "providerKind",
    "providerVendor",
    "providerStatus",
    "providerAuthType",
    "modelCapability",
    "fileBucket",
    "fileAccessPolicy",
    "errorKind",
];

pub fn list_dictionaries() -> Result<Vec<DictionaryDto>, String> {
    DICTIONARY_CODES
        .iter()
        .map(|code| get_dictionary((*code).to_string()))
        .collect()
}

pub fn get_dictionary(code: String) -> Result<DictionaryDto, String> {
    let items = match code.as_str() {
        "appLocale" => vec![item("简体中文", "zh-CN"), item("English", "en-US")],
        "themePreset" => vec![
            item("石墨深蓝", "graphite"),
            item("极光墨青", "aurora"),
            item("琥珀暖夜", "ember"),
            item("瓷白晨雾", "porcelain"),
            item("沙岩暖白", "sandstone"),
        ],
        "layoutDensity" => vec![item("舒适", "comfortable"), item("紧凑", "compact")],
        "inputType" => vec![
            item("主题输入", "topic"),
            item("文案粘贴", "paste"),
            item("文章 / 笔记", "article"),
            disabled_item("小说导入", "novel"),
            disabled_item("素材导入", "material"),
        ],
        "inputProcessMode" => vec![item("AI 生成", "generate"), item("固定原文", "fixed")],
        "workflowType" => vec![
            item("图生视频成片", "image_to_video"),
            disabled_item("数字人口播", "digital_human"),
            disabled_item("素材剪辑成片", "material_edit"),
            disabled_item("纯图文成片", "image_slideshow"),
        ],
        "contentLanguage" => vec![item("中文", "zh-CN"), item("英文", "en-US")],
        "aspectRatio" => vec![
            item("竖屏 9:16", "9:16"),
            item("横屏 16:9", "16:9"),
            item("方形 1:1", "1:1"),
        ],
        "projectLifecycle" => vec![
            colored_item("草稿", "draft", "status.pending"),
            colored_item("进行中", "active", "status.running"),
            colored_item("已归档", "archived", "status.succeeded"),
            colored_item("已删除", "deleted", "status.cancelled"),
        ],
        "taskKind" => vec![item("图生视频主线", "image_to_video")],
        "taskStatus" => vec![
            colored_item("待处理", "pending", "status.pending"),
            colored_item("运行中", "running", "status.running"),
            colored_item("重试中", "retrying", "status.retrying"),
            colored_item("等待确认", "waiting_user", "status.waiting_user"),
            colored_item("已完成", "succeeded", "status.succeeded"),
            colored_item("失败", "failed", "status.failed"),
            colored_item("已取消", "cancelled", "status.cancelled"),
            colored_item("已跳过", "skipped", "status.skipped"),
        ],
        "taskStepKind" => vec![
            item("项目初始化", "project_init"),
            item("生成分镜", "storyboard_generation"),
            item("确认分镜", "storyboard_review"),
            item("生成生图提示词", "image_prompt_generation"),
            item("生成图片", "image_generation"),
            item("确认图片", "image_review"),
            item("生成视频提示词", "video_prompt_generation"),
            item("生成视频", "video_generation"),
            item("确认视频", "video_review"),
            item("最终合成", "final_composition"),
            item("导出", "export"),
            item("清理", "cleanup"),
        ],
        "taskStepStatus" => vec![
            colored_item("待处理", "pending", "status.pending"),
            colored_item("运行中", "running", "status.running"),
            colored_item("重试中", "retrying", "status.retrying"),
            colored_item("等待确认", "waiting_user", "status.waiting_user"),
            colored_item("已完成", "succeeded", "status.succeeded"),
            colored_item("失败", "failed", "status.failed"),
            colored_item("已取消", "cancelled", "status.cancelled"),
            colored_item("已跳过", "skipped", "status.skipped"),
        ],
        "sceneAssetStatus" => vec![
            colored_item("等待中", "pending", "status.pending"),
            colored_item("生成中", "running", "status.running"),
            colored_item("已生成", "succeeded", "status.succeeded"),
            colored_item("失败", "failed", "status.failed"),
            colored_item("跳过", "skipped", "status.skipped"),
        ],
        "providerKind" => vec![
            item("LLM", "llm"),
            item("TTS", "tts"),
            item("生图", "image"),
            item("视频", "video"),
            item("视觉理解", "vlm"),
            item("Workflow", "workflow"),
        ],
        "providerVendor" => vec![
            item("OpenAI", "openai"),
            item("SiliconFlow", "siliconflow"),
            item("火山引擎", "volcengine"),
            item("ComfyUI", "comfyui"),
            item("RunningHub", "runninghub"),
            item("自定义", "custom"),
        ],
        "providerStatus" => vec![
            colored_item("禁用", "disabled", "status.skipped"),
            colored_item("可用", "available", "status.succeeded"),
            colored_item("异常", "error", "status.failed"),
        ],
        "providerAuthType" => vec![
            item("无认证", "none"),
            item("API Key", "api_key"),
            item("Bearer Token", "bearer_token"),
            item("Basic Auth", "basic"),
            item("自定义请求头", "custom_header"),
        ],
        "modelCapability" => vec![
            item("文本生成", "text_generation"),
            item("结构化输出", "structured_output"),
            item("文生图", "text_to_image"),
            item("图生视频", "image_to_video"),
            item("文本转语音", "text_to_speech"),
            item("视觉分析", "vision_analysis"),
            item("Workflow 执行", "workflow_execution"),
        ],
        "fileBucket" => vec![
            item("项目", "project"),
            item("资产", "asset"),
            item("输出", "output"),
            item("缓存", "cache"),
            item("临时", "temp"),
            item("日志", "log"),
            item("模板", "template"),
            item("Sidecar", "sidecar"),
        ],
        "fileAccessPolicy" => vec![
            item("只读", "read_only"),
            item("项目内写入", "write_project"),
            item("导出复制", "export_copy"),
            item("仅临时目录", "temp_only"),
        ],
        "errorKind" => vec![
            item("校验错误", "validation"),
            item("认证错误", "auth"),
            item("限流", "rate_limit"),
            item("网络错误", "network"),
            item("Provider 错误", "provider"),
            item("存储错误", "storage"),
            item("Workflow 错误", "workflow"),
            item("FFmpeg 错误", "ffmpeg"),
            item("数据库错误", "db"),
            item("安全拦截", "security"),
            item("未知错误", "unknown"),
        ],
        _ => return Err(format!("Unknown dictionary code: {code}")),
    };

    Ok(DictionaryDto { code, items })
}

fn item(label: &str, value: &str) -> DictionaryItemDto {
    dictionary_item(label, value, false, None)
}

fn disabled_item(label: &str, value: &str) -> DictionaryItemDto {
    dictionary_item(label, value, true, None)
}

fn colored_item(label: &str, value: &str, color_token: &str) -> DictionaryItemDto {
    dictionary_item(label, value, false, Some(color_token))
}

fn dictionary_item(
    label: &str,
    value: &str,
    disabled: bool,
    color_token: Option<&str>,
) -> DictionaryItemDto {
    DictionaryItemDto {
        label: label.to_string(),
        value: value.to_string(),
        disabled: Some(disabled),
        color_token: color_token.map(str::to_string),
    }
}
