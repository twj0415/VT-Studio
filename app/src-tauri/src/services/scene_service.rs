use crate::domain::scene::{
    GenerateImagePromptsRequest, ImageCandidateDto, SceneDto, SelectImageCandidateRequest,
    SelectVideoSegmentRequest, StartImageGenerationRequest, StartVideoGenerationRequest,
    StoryboardDto, VideoSegmentDto,
};
use serde_json::json;

pub fn get_storyboard(project_id: String) -> Result<StoryboardDto, String> {
    let narrations = vec![
        narration(1, "很多人以为早睡只是自律。"),
        narration(2, "其实它先改变的是你的清醒感。"),
        narration(3, "规律作息会让注意力和记忆力更稳定。"),
    ];

    Ok(StoryboardDto {
        storyboard_id: "storyboard_default".to_string(),
        project_id: project_id.clone(),
        confirmed_narrations: narrations.clone(),
        review_status: "waiting_user".to_string(),
        items: narrations
            .into_iter()
            .map(|item| storyboard_item(project_id.clone(), item.index, item.text))
            .collect(),
    })
}

pub fn update_storyboard_item(item: SceneDto) -> Result<SceneDto, String> {
    Ok(item)
}

pub fn batch_update_storyboard_items(items: Vec<SceneDto>) -> Result<Vec<SceneDto>, String> {
    Ok(items)
}

pub fn reorder_storyboard_items(items: Vec<SceneDto>) -> Result<Vec<SceneDto>, String> {
    Ok(items
        .into_iter()
        .enumerate()
        .map(|(index, mut item)| {
            item.index = (index + 1) as u32;
            item
        })
        .collect())
}

pub fn generate_image_prompts(
    request: GenerateImagePromptsRequest,
) -> Result<Vec<SceneDto>, String> {
    let items = get_storyboard(request.project_id)?.items;
    validate_items_for_image_generation(&items)?;
    if let Some(item_ids) = request.item_ids {
        return Ok(items
            .into_iter()
            .filter(|item| item_ids.contains(&item.item_id))
            .collect());
    }
    Ok(items)
}

pub fn start_image_generation(
    request: StartImageGenerationRequest,
) -> Result<Vec<ImageCandidateDto>, String> {
    validate_items_for_image_generation(&[storyboard_item(
        request.project_id.clone(),
        1,
        "很多人以为早睡只是自律。".to_string(),
    )])?;
    let count = request.count.unwrap_or(2).max(1);
    Ok((0..count)
        .map(|index| image_candidate(&request.project_id, &request.item_id, index + 1))
        .collect())
}

pub fn select_image_candidate(request: SelectImageCandidateRequest) -> Result<SceneDto, String> {
    let mut item = storyboard_item(
        "draft".to_string(),
        1,
        "很多人以为早睡只是自律。".to_string(),
    );
    item.item_id = request.item_id;
    item.selected_image_id = Some(request.image_id);
    Ok(item)
}

pub fn start_video_generation(
    request: StartVideoGenerationRequest,
) -> Result<Vec<VideoSegmentDto>, String> {
    let item = storyboard_item(
        request.project_id.clone(),
        1,
        "很多人以为早睡只是自律。".to_string(),
    );
    validate_items_for_video_generation(&[item])?;
    let count = request.count.unwrap_or(1).max(1);
    Ok((0..count)
        .map(|index| video_segment(&request.project_id, &request.item_id, index + 1))
        .collect())
}

pub fn select_video_segment(request: SelectVideoSegmentRequest) -> Result<SceneDto, String> {
    let mut item = storyboard_item(
        "draft".to_string(),
        1,
        "很多人以为早睡只是自律。".to_string(),
    );
    item.item_id = request.item_id;
    item.selected_video_segment_id = Some(request.segment_id);
    validate_items_for_composition(&[item.clone()])?;
    Ok(item)
}

fn narration(index: u32, text: &str) -> crate::domain::scene::NarrationDto {
    crate::domain::scene::NarrationDto {
        index,
        text: text.to_string(),
        locked: false,
    }
}

fn storyboard_item(project_id: String, index: u32, text: String) -> SceneDto {
    let (visual_description, scene_description, image_prompt, video_prompt) = match index {
        1 => (
            "清晨卧室，主角醒来。",
            "清晨卧室",
            "清晨卧室，主角醒来，柔和自然光，竖屏构图",
            "晨光缓慢进入房间，主角睁眼坐起。",
        ),
        2 => (
            "对比熬夜和早睡状态。",
            "对比场景",
            "熬夜疲惫和早睡清醒状态对比，真实生活方式短视频，竖屏构图",
            "画面在疲惫和清醒状态之间平滑对比。",
        ),
        _ => (
            "办公室里保持清醒工作。",
            "办公室",
            "办公室里保持清醒工作，干净真实光线，竖屏构图",
            "主角专注工作，镜头轻微推进。",
        ),
    };

    SceneDto {
        item_id: format!("item_{index}"),
        project_id,
        index,
        source_text: text.clone(),
        narration_text: text,
        visual_goal: "表达旁白核心信息".to_string(),
        visual_description: visual_description.to_string(),
        characters: vec!["主角".to_string()],
        character_ids: vec![],
        location_id: None,
        scene_description: scene_description.to_string(),
        image_prompt: image_prompt.to_string(),
        negative_prompt: "低清晰度，畸形手指，扭曲面部，文字水印".to_string(),
        video_prompt: video_prompt.to_string(),
        duration_seconds: 4.0,
        selected_image_id: None,
        selected_video_segment_id: None,
        status: "pending".to_string(),
        lock_flags_json: json!({}),
        shot_size: Some("medium".to_string()),
        camera_motion: Some("static".to_string()),
        composition: Some("center".to_string()),
        pace: Some("normal".to_string()),
        transition_type: Some("cut".to_string()),
        image_status: "pending".to_string(),
        audio_status: "pending".to_string(),
        video_status: "pending".to_string(),
        subtitle_status: "pending".to_string(),
        render_status: "pending".to_string(),
        segment_status: "pending".to_string(),
        image_candidates: vec![],
        video_segments: vec![],
    }
}

fn image_candidate(project_id: &str, item_id: &str, index: u32) -> ImageCandidateDto {
    let image_id = format!("img_{index}");
    ImageCandidateDto {
        image_id: image_id.clone(),
        item_id: item_id.to_string(),
        image_path: format!("images/{item_id}/{image_id}.png"),
        prompt: "mock image prompt".to_string(),
        negative_prompt: "low quality, watermark".to_string(),
        model: "mock-image-model".to_string(),
        provider_model_id: "mock/image-to-video-still".to_string(),
        workflow_preset_id: Some("mock-still-v1".to_string()),
        status: "succeeded".to_string(),
        selected: false,
        created_at: "2026-06-22 10:00".to_string(),
        derived_from_image_id: None,
        generation_context_snapshot: json!({
            "projectId": project_id,
            "itemId": item_id,
            "providerOutputKind": "remote_url_converted_to_local_path"
        }),
    }
}

fn video_segment(project_id: &str, item_id: &str, index: u32) -> VideoSegmentDto {
    let segment_id = format!("seg_{index}");
    VideoSegmentDto {
        segment_id: segment_id.clone(),
        item_id: item_id.to_string(),
        input_image_id: "img_selected".to_string(),
        video_path: format!("videos/{item_id}/{segment_id}.mp4"),
        video_prompt: "mock image-to-video motion prompt".to_string(),
        duration_seconds: 4.0,
        model: "mock-video-model".to_string(),
        provider_model_id: "mock/image-to-video".to_string(),
        workflow_preset_id: Some("mock-i2v-v1".to_string()),
        status: "succeeded".to_string(),
        selected: false,
        created_at: "2026-06-22 10:00".to_string(),
        generation_context_snapshot: json!({
            "projectId": project_id,
            "itemId": item_id,
            "inputImageId": "img_selected",
            "providerOutputKind": "remote_url_converted_to_local_path",
            "workflowType": "image_to_video"
        }),
    }
}

fn validate_items_for_image_generation(items: &[SceneDto]) -> Result<(), String> {
    if items.is_empty() {
        return Err(
            "Storyboard must contain at least one item before image generation.".to_string(),
        );
    }

    for item in items {
        let mut missing = vec![];

        if item.source_text.trim().is_empty() && item.narration_text.trim().is_empty() {
            missing.push("source_text_or_narration_text");
        }
        if item.visual_description.trim().is_empty() {
            missing.push("visual_description");
        }
        if item.image_prompt.trim().is_empty() {
            missing.push("image_prompt");
        }
        if item.duration_seconds <= 0.0 {
            missing.push("duration_seconds");
        }

        if !missing.is_empty() {
            return Err(format!(
                "Storyboard item {} is missing required fields: {}.",
                item.index,
                missing.join(", ")
            ));
        }
    }

    Ok(())
}

fn validate_items_for_video_generation(items: &[SceneDto]) -> Result<(), String> {
    for item in items {
        if item.selected_image_id.is_none() {
            return Err(format!(
                "Storyboard item {} has no selected_image_id.",
                item.index
            ));
        }
    }

    Ok(())
}

fn validate_items_for_composition(items: &[SceneDto]) -> Result<(), String> {
    for item in items {
        if item.selected_video_segment_id.is_none() {
            return Err(format!(
                "Storyboard item {} has no selected_video_segment_id.",
                item.index
            ));
        }
    }

    Ok(())
}
