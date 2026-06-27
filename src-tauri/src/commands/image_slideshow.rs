use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::image_slideshow::{
    ImageSlideshowProjectRequest, ImageSlideshowProjectStateDto,
    RegisterTemplateMotionSegmentRequest,
};
use crate::domain::scene::VideoSegmentDto;
use crate::services::image_slideshow_service;
use tauri::State;

#[tauri::command]
pub fn get_image_slideshow_project_state(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<ImageSlideshowProjectStateDto> {
    image_slideshow_service::get_image_slideshow_project_state(state.database(), project_id)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn register_template_motion_segment(
    state: State<'_, AppState>,
    request: RegisterTemplateMotionSegmentRequest,
) -> AppResult<VideoSegmentDto> {
    image_slideshow_service::register_template_motion_segment(
        state.database(),
        state.workspace_root(),
        request,
    )
    .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn validate_image_slideshow_segments(
    state: State<'_, AppState>,
    request: ImageSlideshowProjectRequest,
) -> AppResult<ImageSlideshowProjectStateDto> {
    image_slideshow_service::validate_image_slideshow_segments(state.database(), request)
        .map_err(AppErrorDto::from)
}
