use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::scene::{
    GenerateImagePromptsRequest, ImageCandidateDto, SceneDto, SelectImageCandidateRequest,
    SelectVideoSegmentRequest, StartImageGenerationRequest, StartVideoGenerationRequest,
    StoryboardDto, VideoSegmentDto,
};
use crate::services::scene_service;

#[tauri::command]
pub fn get_storyboard(project_id: String) -> AppResult<StoryboardDto> {
    scene_service::get_storyboard(project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn update_storyboard_item(item: SceneDto) -> AppResult<SceneDto> {
    scene_service::update_storyboard_item(item).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn batch_update_storyboard_items(items: Vec<SceneDto>) -> AppResult<Vec<SceneDto>> {
    scene_service::batch_update_storyboard_items(items).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn reorder_storyboard_items(items: Vec<SceneDto>) -> AppResult<Vec<SceneDto>> {
    scene_service::reorder_storyboard_items(items).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn generate_image_prompts(request: GenerateImagePromptsRequest) -> AppResult<Vec<SceneDto>> {
    scene_service::generate_image_prompts(request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn start_image_generation(
    request: StartImageGenerationRequest,
) -> AppResult<Vec<ImageCandidateDto>> {
    scene_service::start_image_generation(request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn select_image_candidate(request: SelectImageCandidateRequest) -> AppResult<SceneDto> {
    scene_service::select_image_candidate(request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn start_video_generation(
    request: StartVideoGenerationRequest,
) -> AppResult<Vec<VideoSegmentDto>> {
    scene_service::start_video_generation(request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn select_video_segment(request: SelectVideoSegmentRequest) -> AppResult<SceneDto> {
    scene_service::select_video_segment(request).map_err(AppErrorDto::from)
}
