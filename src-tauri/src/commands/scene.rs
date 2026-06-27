use crate::core::app_state::AppState;
use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::scene::{
    ApplyScriptDraftRequest, BuildCharacterResourcePlanRequest, CharacterResourcePlanDto,
    ClearHistoricalImageCandidatesRequest, ClearHistoricalVideoSegmentsRequest,
    GenerateImagePromptsRequest, GenerateSubtitlesRequest, GenerateSubtitlesResultDto,
    GeneratedImageAssetDto, ImageCandidateDto, ProbeStoryboardAudioRequest,
    ReplaceStoryboardAudioRequest, SceneDto, SelectImageCandidateRequest,
    SelectVideoSegmentRequest, StartImageAssetGenerationRequest, StartImageGenerationRequest,
    StartTtsGenerationRequest, StartVideoGenerationRequest, StoryboardDto,
    UpdateStoryboardSubtitlesRequest, VideoSegmentDto,
};
use crate::services::scene_service;
use tauri::State;

#[tauri::command]
pub fn get_storyboard(state: State<'_, AppState>, project_id: String) -> AppResult<StoryboardDto> {
    scene_service::get_storyboard(state.database(), project_id).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn update_storyboard_item(state: State<'_, AppState>, item: SceneDto) -> AppResult<SceneDto> {
    scene_service::update_storyboard_item(state.database(), item).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn batch_update_storyboard_items(
    state: State<'_, AppState>,
    items: Vec<SceneDto>,
) -> AppResult<Vec<SceneDto>> {
    scene_service::batch_update_storyboard_items(state.database(), items).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn apply_script_draft(
    state: State<'_, AppState>,
    request: ApplyScriptDraftRequest,
) -> AppResult<StoryboardDto> {
    scene_service::apply_script_draft(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn reorder_storyboard_items(
    state: State<'_, AppState>,
    items: Vec<SceneDto>,
) -> AppResult<Vec<SceneDto>> {
    scene_service::reorder_storyboard_items(state.database(), items).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn generate_image_prompts(
    state: State<'_, AppState>,
    request: GenerateImagePromptsRequest,
) -> AppResult<Vec<SceneDto>> {
    scene_service::generate_image_prompts(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn start_image_generation(
    state: State<'_, AppState>,
    request: StartImageGenerationRequest,
) -> AppResult<Vec<ImageCandidateDto>> {
    scene_service::start_image_generation(
        state.database(),
        state.workspace_root(),
        state.keyring_service(),
        request,
    )
    .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn build_character_resource_plan(
    state: State<'_, AppState>,
    request: BuildCharacterResourcePlanRequest,
) -> AppResult<CharacterResourcePlanDto> {
    scene_service::build_character_resource_plan(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn start_image_asset_generation(
    state: State<'_, AppState>,
    request: StartImageAssetGenerationRequest,
) -> AppResult<Vec<GeneratedImageAssetDto>> {
    scene_service::start_image_asset_generation(
        state.database(),
        state.workspace_root(),
        state.keyring_service(),
        request,
    )
    .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn select_image_candidate(
    state: State<'_, AppState>,
    request: SelectImageCandidateRequest,
) -> AppResult<SceneDto> {
    scene_service::select_image_candidate(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn clear_historical_image_candidates(
    state: State<'_, AppState>,
    request: ClearHistoricalImageCandidatesRequest,
) -> AppResult<SceneDto> {
    scene_service::clear_historical_image_candidates(state.database(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn start_tts_generation(
    state: State<'_, AppState>,
    request: StartTtsGenerationRequest,
) -> AppResult<SceneDto> {
    scene_service::start_tts_generation(
        state.database(),
        state.workspace_root(),
        state.keyring_service(),
        request,
    )
    .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn replace_storyboard_audio(
    state: State<'_, AppState>,
    request: ReplaceStoryboardAudioRequest,
) -> AppResult<SceneDto> {
    scene_service::replace_storyboard_audio(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn probe_storyboard_audio(
    state: State<'_, AppState>,
    request: ProbeStoryboardAudioRequest,
) -> AppResult<SceneDto> {
    scene_service::probe_storyboard_audio(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn generate_subtitles(
    state: State<'_, AppState>,
    request: GenerateSubtitlesRequest,
) -> AppResult<GenerateSubtitlesResultDto> {
    scene_service::generate_subtitles(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn update_storyboard_subtitles(
    state: State<'_, AppState>,
    request: UpdateStoryboardSubtitlesRequest,
) -> AppResult<GenerateSubtitlesResultDto> {
    scene_service::update_storyboard_subtitles(state.database(), state.workspace_root(), request)
        .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn start_video_generation(
    state: State<'_, AppState>,
    request: StartVideoGenerationRequest,
) -> AppResult<Vec<VideoSegmentDto>> {
    scene_service::start_video_generation(
        state.database(),
        state.workspace_root(),
        state.keyring_service(),
        request,
    )
    .map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn select_video_segment(
    state: State<'_, AppState>,
    request: SelectVideoSegmentRequest,
) -> AppResult<SceneDto> {
    scene_service::select_video_segment(state.database(), request).map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn clear_historical_video_segments(
    state: State<'_, AppState>,
    request: ClearHistoricalVideoSegmentsRequest,
) -> AppResult<SceneDto> {
    scene_service::clear_historical_video_segments(state.database(), request)
        .map_err(AppErrorDto::from)
}
