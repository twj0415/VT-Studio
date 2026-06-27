import type { CompositionTaskDto, StartCompositionRequest } from '@/entities/task/types'
import { getTaskDetail } from '@/entities/task/api'
import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type { ApplyScriptDraftRequest, BuildCharacterResourcePlanRequest, CharacterResourcePlanDto, GenerateSubtitlesRequest, GenerateSubtitlesResultDto, GeneratedImageAssetDto, ImageCandidateDto, NarrationDto, ProbeStoryboardAudioRequest, RegenerateStoryboardRequest, ReplaceStoryboardAudioRequest, SceneDto, StartImageAssetGenerationRequest, StartTtsGenerationRequest, StartVideoGenerationRequest, StoryboardDto, StoryboardItemDto, UpdateStoryboardSubtitlesRequest, VideoSegmentDto } from './types'

export async function listConfirmedNarrations(projectId: string): Promise<NarrationDto[]> {
  const storyboard = await getStoryboard(projectId)
  return storyboard.confirmedNarrations
}

export function getStoryboard(projectId: string): Promise<StoryboardDto> {
  return callCommand<StoryboardDto>(tauriCommands.getStoryboard, { projectId })
}

export async function approveStoryboard(projectId: string): Promise<StoryboardDto> {
  await callCommand(tauriCommands.approveTaskStep, { projectId, stepName: 'storyboard_review' })
  return getStoryboard(projectId)
}

export function updateScene(scene: SceneDto): Promise<SceneDto> {
  return callCommand<SceneDto>(tauriCommands.updateStoryboardItem, { item: scene })
}

export async function updateStoryboardStructure(projectId: string, items: StoryboardItemDto[]): Promise<StoryboardDto> {
  await callCommand<SceneDto[], { items: StoryboardItemDto[] }>(tauriCommands.batchUpdateStoryboardItems, { items })
  return getStoryboard(projectId)
}

export function applyScriptDraft(request: ApplyScriptDraftRequest): Promise<StoryboardDto> {
  return callCommand<StoryboardDto, { request: ApplyScriptDraftRequest }>(tauriCommands.applyScriptDraft, { request })
}

export function regenerateStoryboard(_projectId: string, _request: RegenerateStoryboardRequest): Promise<StoryboardDto> {
  return Promise.reject(new Error('storyboard.regenerate_unavailable: Regenerate requires a real script draft from the LLM pipeline.'))
}

export function restorePreviousStoryboard(_projectId: string): Promise<StoryboardDto> {
  return Promise.reject(new Error('storyboard.restore_unavailable: No persisted previous storyboard snapshot is available.'))
}

export function createStoryboardDraftItem(projectId: string, index: number, sourceText = '', narrationText = ''): Promise<StoryboardItemDto> {
  return Promise.resolve({
    itemId: `${projectId}_draft_${Date.now()}_${index}`,
    projectId,
    index,
    sourceText,
    narrationText,
    visualGoal: '',
    visualDescription: '',
    characters: [],
    characterIds: [],
    locationId: null,
    sceneDescription: '',
    imagePrompt: '',
    negativePrompt: '',
    videoPrompt: '',
    durationSeconds: 4,
    subtitleChunks: [],
    audioPath: null,
    audioDurationSeconds: null,
    audioProbe: null,
    selectedImageId: null,
    selectedVideoSegmentId: null,
    status: 'pending',
    lockFlagsJson: {},
    shotSize: undefined,
    cameraMotion: undefined,
    composition: undefined,
    pace: undefined,
    transitionType: undefined,
    imageStatus: 'pending',
    imageLastErrorJson: null,
    imageRetryCount: 0,
    audioStatus: 'pending',
    audioLastErrorJson: null,
    audioRetryCount: 0,
    videoStatus: 'pending',
    subtitleStatus: 'pending',
    renderStatus: 'pending',
    segmentStatus: 'pending',
    imageCandidates: [],
    videoSegments: [],
    downstreamResetRecords: [],
  })
}

export async function listImageCandidates(projectId: string): Promise<ImageCandidateDto[]> {
  const storyboard = await getStoryboard(projectId)
  return storyboard.items.flatMap((item) => item.imageCandidates)
}

export async function listVideoSegments(projectId: string): Promise<VideoSegmentDto[]> {
  const storyboard = await getStoryboard(projectId)
  return storyboard.items.flatMap((item) => item.videoSegments)
}

export async function getCompositionTask(taskId: string): Promise<CompositionTaskDto> {
  const projectId = taskId.split('_composition_')[0]
  if (!projectId || projectId === taskId) throw new Error(`composition.task_lookup_failed: project id cannot be derived from ${taskId}`)
  const task = await getTaskDetail(projectId)
  if (!task.compositionTask || task.compositionTask.taskId !== taskId) throw new Error(`composition.task_not_found: ${taskId}`)
  return task.compositionTask
}

export function startImageGeneration(projectId: string, itemId: string, count = 4): Promise<ImageCandidateDto[]> {
  return callCommand<ImageCandidateDto[]>(tauriCommands.startImageGeneration, { request: { projectId, itemId, count, imageKind: 'storyboard_image', assetKind: 'generated_output' } })
}

export function buildCharacterResourcePlan(request: BuildCharacterResourcePlanRequest): Promise<CharacterResourcePlanDto> {
  return callCommand<CharacterResourcePlanDto, { request: BuildCharacterResourcePlanRequest }>(tauriCommands.buildCharacterResourcePlan, { request })
}

export function startImageAssetGeneration(request: StartImageAssetGenerationRequest): Promise<GeneratedImageAssetDto[]> {
  return callCommand<GeneratedImageAssetDto[], { request: StartImageAssetGenerationRequest }>(tauriCommands.startImageAssetGeneration, { request })
}

export function selectImageCandidate(itemId: string, imageId: string): Promise<StoryboardItemDto> {
  return callCommand<StoryboardItemDto>(tauriCommands.selectImageCandidate, { request: { itemId, imageId } })
}

export function clearHistoricalImageCandidates(itemId: string): Promise<StoryboardItemDto> {
  return callCommand<StoryboardItemDto>(tauriCommands.clearHistoricalImageCandidates, { request: { itemId } })
}

export function startTtsGeneration(request: StartTtsGenerationRequest): Promise<StoryboardItemDto> {
  return callCommand<StoryboardItemDto, { request: StartTtsGenerationRequest }>(tauriCommands.startTtsGeneration, { request })
}

export function replaceStoryboardAudio(request: ReplaceStoryboardAudioRequest): Promise<StoryboardItemDto> {
  return callCommand<StoryboardItemDto, { request: ReplaceStoryboardAudioRequest }>(tauriCommands.replaceStoryboardAudio, { request })
}

export function probeStoryboardAudio(request: ProbeStoryboardAudioRequest): Promise<StoryboardItemDto> {
  return callCommand<StoryboardItemDto, { request: ProbeStoryboardAudioRequest }>(tauriCommands.probeStoryboardAudio, { request })
}

export function generateSubtitles(request: GenerateSubtitlesRequest): Promise<GenerateSubtitlesResultDto> {
  return callCommand<GenerateSubtitlesResultDto, { request: GenerateSubtitlesRequest }>(tauriCommands.generateSubtitles, { request })
}

export function updateStoryboardSubtitles(request: UpdateStoryboardSubtitlesRequest): Promise<GenerateSubtitlesResultDto> {
  return callCommand<GenerateSubtitlesResultDto, { request: UpdateStoryboardSubtitlesRequest }>(tauriCommands.updateStoryboardSubtitles, { request })
}

export function startVideoGeneration(projectId: string, itemId: string, count = 1): Promise<VideoSegmentDto[]> {
  const request: StartVideoGenerationRequest = { projectId, itemId, count }
  return callCommand<VideoSegmentDto[], { request: StartVideoGenerationRequest }>(tauriCommands.startVideoGeneration, { request })
}

export function selectVideoSegment(itemId: string, segmentId: string): Promise<StoryboardItemDto> {
  return callCommand<StoryboardItemDto>(tauriCommands.selectVideoSegment, { request: { itemId, segmentId } })
}

export function clearHistoricalVideoSegments(itemId: string): Promise<StoryboardItemDto> {
  return callCommand<StoryboardItemDto>(tauriCommands.clearHistoricalVideoSegments, { request: { itemId } })
}

export function startComposition(request: StartCompositionRequest): Promise<CompositionTaskDto> {
  return callCommand<CompositionTaskDto, { request: StartCompositionRequest }>(tauriCommands.startComposition, { request })
}
