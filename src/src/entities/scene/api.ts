import { createMockId } from '@/shared/mock/ids'
import type { CompositionTaskDto, StartCompositionRequest } from '@/entities/task/types'
import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'
import { getProjectDetail } from '@/entities/project/api'
import type { ProjectDto } from '@/entities/project/types'

import type { ApplyScriptDraftRequest, BuildCharacterResourcePlanRequest, CharacterResourcePlanDto, GenerateSubtitlesRequest, GenerateSubtitlesResultDto, GeneratedImageAssetDto, ImageCandidateDto, NarrationDto, ProbeStoryboardAudioRequest, RegenerateStoryboardRequest, ReplaceStoryboardAudioRequest, SceneDto, ScriptDraftNarrationDto, StartImageAssetGenerationRequest, StartTtsGenerationRequest, StartVideoGenerationRequest, StoryboardDto, StoryboardItemDto, SubtitleChunkDto, SubtitlesFileDto, UpdateStoryboardSubtitlesRequest, VideoSegmentDto } from './types'
import { isStoryboardItemLockedForBulkImageGeneration, isStoryboardItemLockedForBulkVideoGeneration, isStoryboardItemLocked } from './reset'

const MOCK_NOW = '2026-06-22 10:00'
const MOCK_STORYBOARD_GOAL = '演示：基于作品内容生成的分镜占位，用于验证页面和数据流，不代表真实 AI 生成结果'
const DEFAULT_NEGATIVE_PROMPT = '低清晰度，畸形手指，扭曲面部，文字水印'

type TextSplitMode = 'paragraph' | 'line' | 'sentence'

interface StoryboardMockState {
  confirmedNarrations: NarrationDto[]
  items: StoryboardItemDto[]
  previousItems: StoryboardItemDto[] | null
  reviewStatus: StoryboardDto['reviewStatus']
}

const topicScenePlans = ['开场提出问题', '展示关键场景', '呈现核心变化', '补充生活化细节', '强化前后对比', '给出行动建议', '总结关键信息', '收束到结论']

const storyboardStateByProjectId = new Map<string, StoryboardMockState>()

let imageCandidates: ImageCandidateDto[] = []
let videoSegments: VideoSegmentDto[] = []
let compositionTasks: CompositionTaskDto[] = []
let generatedImageAssets: GeneratedImageAssetDto[] = []

export async function listConfirmedNarrations(projectId: string): Promise<NarrationDto[]> {
  const state = await ensureStoryboardState(projectId)
  return state.confirmedNarrations
}

export async function getStoryboard(projectId: string): Promise<StoryboardDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardDto>(tauriCommands.getStoryboard, { projectId })
  }

  const state = await ensureStoryboardState(projectId)

  return {
    storyboardId: `storyboard_${projectId}`,
    projectId,
    confirmedNarrations: state.confirmedNarrations,
    items: state.items.map((item) => withItemCandidates({ ...item, projectId })),
    reviewStatus: state.reviewStatus,
  }
}

export async function approveStoryboard(projectId: string): Promise<StoryboardDto> {
  const state = await ensureStoryboardState(projectId)
  state.reviewStatus = 'succeeded'
  return await getStoryboard(projectId)
}

export async function updateScene(scene: SceneDto): Promise<SceneDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<SceneDto>(tauriCommands.updateStoryboardItem, { item: scene })
  }

  const state = await ensureStoryboardState(scene.projectId)
  const nextScene = withItemRelations(scene)
  syncSelectedImageCandidates(nextScene)
  syncSelectedVideoSegments(nextScene)
  state.items = state.items.map((item) => (item.itemId === scene.itemId ? nextScene : item))
  return nextScene
}

export async function updateStoryboardStructure(projectId: string, items: StoryboardItemDto[]): Promise<StoryboardDto> {
  const state = await ensureStoryboardState(projectId)
  state.reviewStatus = 'waiting_user'
  state.items = normalizeStoryboardItems(projectId, items)
  state.confirmedNarrations = createNarrationsFromItems(state.items)
  return await getStoryboard(projectId)
}

export async function applyScriptDraft(request: ApplyScriptDraftRequest): Promise<StoryboardDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardDto, { request: ApplyScriptDraftRequest }>(tauriCommands.applyScriptDraft, { request })
  }

  const state = await ensureStoryboardState(request.projectId)
  const detail = await getProjectDetail(request.projectId)
  const project: ProjectDto = {
    ...detail.project,
    projectId: request.projectId,
  }
  const draftItems = parseScriptDraftNarrations(request.rawOutput, request.expectedCount)
  state.previousItems = cloneStoryboardItems(state.items)
  state.reviewStatus = 'waiting_user'
  state.items = draftItems.map((draft) => mergeScriptDraftItem(project, draft, state.items.find((item) => item.index === draft.index)))
  state.confirmedNarrations = createNarrationsFromItems(state.items)
  return await getStoryboard(request.projectId)
}

export async function regenerateStoryboard(projectId: string, request: RegenerateStoryboardRequest): Promise<StoryboardDto> {
  const state = await ensureStoryboardState(projectId)
  const detail = await getProjectDetail(projectId)
  const project: ProjectDto = {
    ...detail.project,
    projectId,
  }

  state.previousItems = cloneStoryboardItems(state.items)
  state.reviewStatus = 'waiting_user'
  const nextNarrations = createMockNarrations(project, request)
  state.items = nextNarrations.map((narration) => mergeScriptDraftItem(
    project,
    { index: narration.index, sourceText: narration.text, narrationText: narration.text },
    state.items.find((item) => item.index === narration.index),
  ))
  state.confirmedNarrations = createNarrationsFromItems(state.items)
  return await getStoryboard(projectId)
}

export async function restorePreviousStoryboard(projectId: string): Promise<StoryboardDto> {
  const state = await ensureStoryboardState(projectId)
  if (!state.previousItems) throw new Error('No previous storyboard snapshot')

  const currentItems = cloneStoryboardItems(state.items)
  state.items = normalizeStoryboardItems(projectId, state.previousItems)
  state.previousItems = currentItems
  state.confirmedNarrations = createNarrationsFromItems(state.items)
  state.reviewStatus = 'waiting_user'
  return await getStoryboard(projectId)
}

export async function createStoryboardDraftItem(projectId: string, index: number, sourceText = '', narrationText = ''): Promise<StoryboardItemDto> {
  const detail = await getProjectDetail(projectId)
  return createStoryboardItem(
    {
      ...detail.project,
      projectId,
    },
    {
      index,
      text: sourceText || narrationText,
      locked: false,
    },
    narrationText
  )
}

export async function listImageCandidates(projectId: string): Promise<ImageCandidateDto[]> {
  if (getApiAdapter() === 'tauri') {
    const storyboard = await getStoryboard(projectId)
    return storyboard.items.flatMap((item) => item.imageCandidates)
  }

  const state = await ensureStoryboardState(projectId)
  const itemIds = new Set(state.items.map((item) => item.itemId))
  return imageCandidates.filter((candidate) => itemIds.has(candidate.itemId))
}

export async function listVideoSegments(projectId: string): Promise<VideoSegmentDto[]> {
  if (getApiAdapter() === 'tauri') {
    const storyboard = await getStoryboard(projectId)
    return storyboard.items.flatMap((item) => item.videoSegments)
  }

  const state = await ensureStoryboardState(projectId)
  const itemIds = new Set(state.items.map((item) => item.itemId))
  return videoSegments.filter((segment) => itemIds.has(segment.itemId))
}

export async function getCompositionTask(taskId: string): Promise<CompositionTaskDto> {
  const task = compositionTasks.find((entry) => entry.taskId === taskId)
  if (!task) throw new Error(`Composition task not found: ${taskId}`)
  return task
}

export async function startImageGeneration(projectId: string, itemId: string, count = 4): Promise<ImageCandidateDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ImageCandidateDto[]>(tauriCommands.startImageGeneration, { request: { projectId, itemId, count, imageKind: 'storyboard_image', assetKind: 'generated_output' } })
  }

  const state = await ensureStoryboardState(projectId)
  const item = state.items.find((entry) => entry.itemId === itemId)
  if (!item) throw new Error(`Storyboard item not found: ${itemId}`)
  if (isStoryboardItemLockedForBulkImageGeneration(item)) throw new Error(`Storyboard item is locked for image generation: ${itemId}`)

  const revision = getNextImageRevision(itemId)
  const generated = Array.from({ length: count }, (_, index) => createImageCandidate(projectId, item, index + 1, count, revision))
  imageCandidates = [...imageCandidates, ...generated]
  state.items = state.items.map((entry) => (entry.itemId === itemId ? withItemCandidates({ ...entry, imageStatus: 'succeeded', status: 'succeeded' }) : entry))
  return generated
}

export async function buildCharacterResourcePlan(request: BuildCharacterResourcePlanRequest): Promise<CharacterResourcePlanDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CharacterResourcePlanDto, { request: BuildCharacterResourcePlanRequest }>(tauriCommands.buildCharacterResourcePlan, { request })
  }

  const state = await ensureStoryboardState(request.projectId)
  const item = state.items.find((entry) => entry.itemId === request.itemId)
  if (!item) throw new Error(`Storyboard item not found: ${request.itemId}`)
  const roles = ['character_front_view', 'character_side_view', 'character_back_view', 'character_full_body', 'character_face_closeup', 'character_expression_sheet', 'character_outfit', 'character_pose', 'character_mood']
  const items = item.characterIds.flatMap((characterId) =>
    roles.map((role) => ({
      characterId,
      characterName: characterId,
      role,
      requirement: role === 'character_front_view' ? 'optional' : 'unused',
      available: false,
      assetId: null,
      relativePath: null,
      missingReason: null,
      sourceOptions: ['upload', 'select_existing', 'generate'],
    }))
  )
  return {
    projectId: request.projectId,
    itemId: request.itemId,
    optionId: 'mock:image-default',
    sourceType: 'provider_model',
    sourceId: 'mock:image-default',
    providerModelId: 'mock:image-default',
    workflowPresetId: null,
    requiredCount: items.filter((entry) => entry.requirement === 'required').length,
    optionalCount: items.filter((entry) => entry.requirement === 'optional').length,
    unusedCount: items.filter((entry) => entry.requirement === 'unused').length,
    missingRequiredCount: items.filter((entry) => entry.requirement === 'required' && !entry.available).length,
    items,
  }
}

export async function startImageAssetGeneration(request: StartImageAssetGenerationRequest): Promise<GeneratedImageAssetDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<GeneratedImageAssetDto[], { request: StartImageAssetGenerationRequest }>(tauriCommands.startImageAssetGeneration, { request })
  }

  const count = Math.min(4, Math.max(1, request.count ?? 1))
  const assetKind = request.assetKind ?? defaultAssetKindForImageKind(request.imageKind)
  const generated = Array.from({ length: count }, (_, index) => {
    const assetId = createMockId('asset')
    const referenceId = createMockId('asset_ref')
    const relativePath = `assets/${request.projectId}/${request.imageKind}/${request.ownerId}/${assetId}_v${index + 1}.png`
    const snapshot = {
      mock: true,
      projectId: request.projectId,
      itemId: request.itemId,
      imageKind: request.imageKind,
      assetKind,
      ownerKind: request.ownerKind,
      ownerId: request.ownerId,
      referenceRole: request.referenceRole,
      variantIndex: index + 1,
      assetCount: count,
      prompt: request.prompt,
      negativePrompt: request.negativePrompt,
      providerOutputKind: 'local_mock_asset_placeholder',
    }
    return {
      asset: {
        assetId,
        kind: assetKind,
        relativePath,
        sourceKind: 'ai_generated',
        mimeType: 'image/png',
        isBuiltin: false,
        lifecycle: 'active',
        metadata: snapshot,
      },
      reference: {
        referenceId,
        assetId,
        ownerKind: request.ownerKind,
        ownerId: request.ownerId,
        usageKind: usageKindForImageKind(request.imageKind),
      },
      imageKind: request.imageKind,
      assetKind,
      ownerKind: request.ownerKind,
      ownerId: request.ownerId,
      referenceRole: request.referenceRole,
      usageKind: usageKindForImageKind(request.imageKind),
      relativePath,
      generationContextSnapshot: snapshot,
    } satisfies GeneratedImageAssetDto
  })
  generatedImageAssets = [...generatedImageAssets, ...generated]
  return generated
}

export async function selectImageCandidate(itemId: string, imageId: string): Promise<StoryboardItemDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardItemDto>(tauriCommands.selectImageCandidate, { request: { itemId, imageId } })
  }

  const candidate = imageCandidates.find((entry) => entry.itemId === itemId && entry.imageId === imageId)
  if (!candidate) throw new Error(`Image candidate not found: ${imageId}`)

  const state = findStateByItemId(itemId)
  if (!state) throw new Error(`Storyboard item not found: ${itemId}`)

  imageCandidates = imageCandidates.map((entry) => (entry.itemId === itemId ? { ...entry, selected: entry.imageId === imageId } : entry))
  const updatedItem = state.items.find((entry) => entry.itemId === itemId)
  if (!updatedItem) throw new Error(`Storyboard item not found: ${itemId}`)
  if (isStoryboardItemLocked(updatedItem, 'selectedImage')) throw new Error(`Storyboard item selectedImage is locked: ${itemId}`)

  const nextItem = withItemCandidates({ ...updatedItem, selectedImageId: imageId })
  state.items = state.items.map((entry) => (entry.itemId === itemId ? nextItem : entry))
  return nextItem
}

export async function clearHistoricalImageCandidates(itemId: string): Promise<StoryboardItemDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardItemDto>(tauriCommands.clearHistoricalImageCandidates, { request: { itemId } })
  }

  const state = findStateByItemId(itemId)
  if (!state) throw new Error(`Storyboard item not found: ${itemId}`)

  const item = state.items.find((entry) => entry.itemId === itemId)
  if (!item) throw new Error(`Storyboard item not found: ${itemId}`)

  const latestRevision = getLatestImageRevision(itemId)
  imageCandidates = imageCandidates.filter((candidate) => candidate.itemId !== itemId || candidate.imageId === item.selectedImageId || getGenerationRevision(candidate) === latestRevision)

  const nextItem = withItemCandidates(item)
  state.items = state.items.map((entry) => (entry.itemId === itemId ? nextItem : entry))
  return nextItem
}

export async function startTtsGeneration(request: StartTtsGenerationRequest): Promise<StoryboardItemDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardItemDto, { request: StartTtsGenerationRequest }>(tauriCommands.startTtsGeneration, { request })
  }

  const state = await ensureStoryboardState(request.projectId)
  const item = state.items.find((entry) => entry.itemId === request.itemId)
  if (!item) throw new Error(`Storyboard item not found: ${request.itemId}`)
  const text = (item.narrationText || item.sourceText).trim()
  if (!text) throw new Error(`Storyboard item has no narration text: ${request.itemId}`)

  const updated = withItemRelations({
    ...item,
    audioPath: `mock-audio/${request.projectId}/${request.itemId}/voice.${request.format ?? 'mp3'}`,
    audioDurationSeconds: null,
    audioProbe: null,
    audioStatus: 'succeeded',
    audioLastErrorJson: null,
  })
  state.items = state.items.map((entry) => (entry.itemId === request.itemId ? updated : entry))
  return updated
}

export async function replaceStoryboardAudio(request: ReplaceStoryboardAudioRequest): Promise<StoryboardItemDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardItemDto, { request: ReplaceStoryboardAudioRequest }>(tauriCommands.replaceStoryboardAudio, { request })
  }

  const state = await ensureStoryboardState(request.projectId)
  const item = state.items.find((entry) => entry.itemId === request.itemId)
  if (!item) throw new Error(`Storyboard item not found: ${request.itemId}`)
  const updated = withItemRelations({
    ...item,
    audioPath: `mock-audio/${request.projectId}/${request.itemId}/uploaded.mp3`,
    audioDurationSeconds: null,
    audioProbe: null,
    audioStatus: 'succeeded',
    audioLastErrorJson: null,
  })
  state.items = state.items.map((entry) => (entry.itemId === request.itemId ? updated : entry))
  return updated
}

export async function probeStoryboardAudio(request: ProbeStoryboardAudioRequest): Promise<StoryboardItemDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardItemDto, { request: ProbeStoryboardAudioRequest }>(tauriCommands.probeStoryboardAudio, { request })
  }

  const state = await ensureStoryboardState(request.projectId)
  const item = state.items.find((entry) => entry.itemId === request.itemId)
  if (!item) throw new Error(`Storyboard item not found: ${request.itemId}`)
  if (!item.audioPath) throw new Error(`Storyboard item has no audio path: ${request.itemId}`)
  const updated = withItemRelations({
    ...item,
    audioDurationSeconds: item.audioDurationSeconds ?? 1,
    audioProbe: {
      path: item.audioPath,
      mediaKind: 'audio',
      durationSeconds: item.audioDurationSeconds ?? 1,
      audioCodec: 'mock',
      sampleRate: 24000,
      channels: 1,
      hasVideoStream: false,
      hasAudioStream: true,
    },
  })
  state.items = state.items.map((entry) => (entry.itemId === request.itemId ? updated : entry))
  return updated
}

export async function generateSubtitles(request: GenerateSubtitlesRequest): Promise<GenerateSubtitlesResultDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<GenerateSubtitlesResultDto, { request: GenerateSubtitlesRequest }>(tauriCommands.generateSubtitles, { request })
  }

  const state = await ensureStoryboardState(request.projectId)
  const itemIdSet = request.itemIds ? new Set(request.itemIds) : null
  const targetItems = state.items.filter((item) => !itemIdSet || itemIdSet.has(item.itemId))
  if (targetItems.length === 0) throw new Error('No storyboard items available for subtitle generation')

  state.items = state.items.map((item) => {
    if (!targetItems.some((entry) => entry.itemId === item.itemId)) return item
    return withItemRelations({
      ...item,
      subtitleChunks: createSubtitleChunks(item, splitSubtitleText(item.narrationText || item.sourceText)),
      subtitleStatus: 'succeeded',
      renderStatus: 'pending',
    })
  })

  const subtitles = createSubtitlesFile(request.projectId, state.items)
  return {
    projectId: request.projectId,
    subtitlePath: `projects/${request.projectId}/subtitles/subtitles.json`,
    items: state.items.filter((item) => !itemIdSet || itemIdSet.has(item.itemId)),
    subtitles,
  }
}

export async function updateStoryboardSubtitles(request: UpdateStoryboardSubtitlesRequest): Promise<GenerateSubtitlesResultDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<GenerateSubtitlesResultDto, { request: UpdateStoryboardSubtitlesRequest }>(tauriCommands.updateStoryboardSubtitles, { request })
  }

  const state = await ensureStoryboardState(request.projectId)
  const item = state.items.find((entry) => entry.itemId === request.itemId)
  if (!item) throw new Error(`Storyboard item not found: ${request.itemId}`)
  const lines = request.subtitleChunks.map((chunk) => normalizeText(chunk.text)).filter(Boolean)
  if (lines.length === 0) throw new Error('subtitleChunks must contain text')
  const updated = withItemRelations({
    ...item,
    subtitleChunks: createSubtitleChunks(item, lines),
    subtitleStatus: 'succeeded',
    renderStatus: 'pending',
  })
  state.items = state.items.map((entry) => (entry.itemId === request.itemId ? updated : entry))
  const subtitles = createSubtitlesFile(request.projectId, state.items)
  return {
    projectId: request.projectId,
    subtitlePath: `projects/${request.projectId}/subtitles/subtitles.json`,
    items: [updated],
    subtitles,
  }
}

export async function startVideoGeneration(projectId: string, itemId: string, count = 1): Promise<VideoSegmentDto[]> {
  const request: StartVideoGenerationRequest = { projectId, itemId, count }
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoSegmentDto[], { request: StartVideoGenerationRequest }>(tauriCommands.startVideoGeneration, { request })
  }

  const state = await ensureStoryboardState(projectId)
  const item = state.items.find((entry) => entry.itemId === itemId)
  if (!item) throw new Error(`Storyboard item not found: ${itemId}`)
  if (isStoryboardItemLockedForBulkVideoGeneration(item)) throw new Error(`Storyboard item is locked for video generation: ${itemId}`)
  if (!item.selectedImageId) throw new Error(`Storyboard item has no selected image: ${itemId}`)

  const selectedImage = imageCandidates.find((candidate) => candidate.itemId === itemId && candidate.imageId === item.selectedImageId)
  if (!selectedImage) throw new Error(`Selected image candidate not found: ${item.selectedImageId}`)

  const revision = getNextVideoRevision(itemId)
  const generated = Array.from({ length: count }, (_, index) => createVideoSegment(projectId, item, selectedImage, index + 1, count, revision))
  videoSegments = [...videoSegments, ...generated]
  state.items = state.items.map((entry) => (entry.itemId === itemId ? withItemRelations({ ...entry, videoStatus: 'succeeded', segmentStatus: 'succeeded' }) : entry))
  return generated
}

export async function selectVideoSegment(itemId: string, segmentId: string): Promise<StoryboardItemDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardItemDto>(tauriCommands.selectVideoSegment, { request: { itemId, segmentId } })
  }

  const segment = videoSegments.find((entry) => entry.itemId === itemId && entry.segmentId === segmentId)
  if (!segment) throw new Error(`Video segment not found: ${segmentId}`)

  const state = findStateByItemId(itemId)
  if (!state) throw new Error(`Storyboard item not found: ${itemId}`)

  videoSegments = videoSegments.map((entry) => (entry.itemId === itemId ? { ...entry, selected: entry.segmentId === segmentId } : entry))
  const updatedItem = state.items.find((entry) => entry.itemId === itemId)
  if (!updatedItem) throw new Error(`Storyboard item not found: ${itemId}`)
  if (isStoryboardItemLocked(updatedItem, 'selectedVideoSegment')) throw new Error(`Storyboard item selectedVideoSegment is locked: ${itemId}`)

  const nextItem = withItemRelations({ ...updatedItem, selectedVideoSegmentId: segmentId })
  state.items = state.items.map((entry) => (entry.itemId === itemId ? nextItem : entry))
  return nextItem
}

export async function clearHistoricalVideoSegments(itemId: string): Promise<StoryboardItemDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardItemDto>(tauriCommands.clearHistoricalVideoSegments, { request: { itemId } })
  }

  const state = findStateByItemId(itemId)
  if (!state) throw new Error(`Storyboard item not found: ${itemId}`)

  const item = state.items.find((entry) => entry.itemId === itemId)
  if (!item) throw new Error(`Storyboard item not found: ${itemId}`)

  const latestRevision = getLatestVideoRevision(itemId)
  videoSegments = videoSegments.filter((segment) => segment.itemId !== itemId || segment.segmentId === item.selectedVideoSegmentId || getGenerationRevision(segment) === latestRevision)

  const nextItem = withItemRelations(item)
  state.items = state.items.map((entry) => (entry.itemId === itemId ? nextItem : entry))
  return nextItem
}

export async function startComposition(request: StartCompositionRequest): Promise<CompositionTaskDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CompositionTaskDto, { request: StartCompositionRequest }>(tauriCommands.startComposition, { request })
  }

  const projectId = request.projectId
  const state = await ensureStoryboardState(projectId)
  const orderedItems = [...state.items].sort((left, right) => left.index - right.index)
  const missingItem = orderedItems.find((item) => !item.selectedVideoSegmentId)
  if (missingItem) throw new Error(`Storyboard item has no selected video segment: ${missingItem.itemId}`)

  const segmentIds = orderedItems.map((item) => item.selectedVideoSegmentId as string)
  const missingSegmentId = segmentIds.find((segmentId) => !videoSegments.some((segment) => segment.segmentId === segmentId))
  if (missingSegmentId) throw new Error(`Selected video segment not found: ${missingSegmentId}`)

  const task: CompositionTaskDto = {
    taskId: createMockId('composition_task'),
    projectId,
    segmentIds,
    outputPath: `mock-outputs/${projectId}/final.mp4`,
    enhancements: {
      includeSubtitle: request.includeSubtitle === true,
      subtitlePath: request.includeSubtitle ? (request.subtitlePath ?? `projects/${projectId}/subtitles/subtitles.json`) : null,
      includeBgm: request.includeBgm === true,
      bgmAssetId: request.bgmAssetId ?? null,
      bgmVolume: request.bgmVolume ?? 0.18,
      bgmLoop: request.bgmLoop ?? true,
      bgmFadeInSeconds: request.bgmFadeInSeconds ?? 0,
      bgmFadeOutSeconds: request.bgmFadeOutSeconds ?? 0,
      includeCoverMetadata: request.includeCoverMetadata === true,
      coverPath: request.coverPath ?? null,
      steps: [
        { step: 'concat', status: 'succeeded' },
        { step: 'subtitle', status: request.includeSubtitle ? 'succeeded' : 'skipped' },
        { step: 'bgm', status: request.includeBgm ? 'succeeded' : 'skipped' },
        { step: 'cover_metadata', status: request.includeCoverMetadata ? 'succeeded' : 'skipped' },
      ],
    },
    status: 'succeeded',
    progress: 100,
    errorJson: {
      mock: true,
      providerOutputKind: 'local_mock_composition_placeholder',
      workflowType: 'image_to_video',
    },
    createdAt: MOCK_NOW,
    updatedAt: MOCK_NOW,
  }
  compositionTasks = [task, ...compositionTasks]
  return task
}

async function ensureStoryboardState(projectId: string): Promise<StoryboardMockState> {
  const existing = storyboardStateByProjectId.get(projectId)
  if (existing) return existing

  const detail = await getProjectDetail(projectId)
  const project: ProjectDto = {
    ...detail.project,
    projectId,
  }
  const confirmedNarrations = createMockNarrations(project)
  const state: StoryboardMockState = {
    confirmedNarrations,
    items: confirmedNarrations.map((narration) => createStoryboardItem(project, narration)),
    previousItems: null,
    reviewStatus: 'waiting_user',
  }

  storyboardStateByProjectId.set(projectId, state)
  return state
}

function createMockNarrations(project: ProjectDto, request?: RegenerateStoryboardRequest): NarrationDto[] {
  const canGenerateSource = project.inputProcessMode !== 'fixed'
  const sourceSegments = canGenerateSource && (project.inputType === 'topic' || request?.mode === 'ai') ? createTopicSegments(project, request) : createOriginalTextSegments(project, request)

  return sourceSegments.map((text, index) => ({
    index: index + 1,
    text,
    locked: false,
  }))
}

function createTopicSegments(project: ProjectDto, request?: RegenerateStoryboardRequest): string[] {
  const topic = normalizeText(project.sourceText || project.title || '未命名主题')
  const count = clampSceneCount(request?.targetSceneCount ?? project.targetSceneCount)
  const prefix = request?.mode === 'ai' ? '演示 AI 分镜' : '演示分镜'

  return Array.from({ length: count }, (_, index) => {
    const plan = topicScenePlans[index % topicScenePlans.length]
    return `${prefix} ${index + 1}：${topic} - ${plan}`
  })
}

function createOriginalTextSegments(project: ProjectDto, request?: RegenerateStoryboardRequest): string[] {
  const text = project.sourceText || project.title || '未提供原文'
  if (request?.mode === 'line_count') {
    return splitByFixedGroup(splitText(text, 'line'), clampGroupCount(request.lineCount), '\n')
  }

  if (request?.mode === 'sentence_count') {
    return splitByFixedGroup(splitText(text, 'sentence'), clampGroupCount(request.sentenceCount), '')
  }

  const splitMode = resolveSplitMode(project.inputOptions)
  const segments = splitText(text, splitMode)
  return limitSegments(segments.length > 0 ? segments : [text.trim()], clampSceneCount(request?.targetSceneCount ?? project.targetSceneCount), splitMode === 'sentence' ? '' : '\n')
}

function parseScriptDraftNarrations(rawOutput: string, expectedCount?: number): ScriptDraftNarrationDto[] {
  const parsed = JSON.parse(extractJsonPayload(rawOutput)) as { narrations?: unknown }
  if (!parsed || !Array.isArray(parsed.narrations)) throw new Error('script draft schema invalid: narrations is required')
  if (expectedCount !== undefined && parsed.narrations.length !== expectedCount) throw new Error(`script draft schema invalid: expected ${expectedCount} narrations`)
  return parsed.narrations.map((item, index) => {
    if (!item || typeof item !== 'object') throw new Error(`script draft schema invalid: narrations[${index}] must be an object`)
    const entry = item as Record<string, unknown>
    if (!Number.isInteger(entry.index)) throw new Error(`script draft schema invalid: narrations[${index}].index is required`)
    if (typeof entry.sourceText !== 'string') throw new Error(`script draft schema invalid: narrations[${index}].sourceText is required`)
    if (entry.narrationText !== undefined && typeof entry.narrationText !== 'string') throw new Error(`script draft schema invalid: narrations[${index}].narrationText must be a string`)
    return {
      index: entry.index as number,
      sourceText: entry.sourceText,
      narrationText: entry.narrationText,
    }
  })
}

function extractJsonPayload(rawOutput: string): string {
  const trimmed = rawOutput.trim()
  if (!trimmed) throw new Error('script draft schema invalid: output is empty')
  const fenced = trimmed.match(/^```(?:json|JSON)?\s*([\s\S]*?)\s*```$/)
  return fenced?.[1]?.trim() || trimmed
}

function mergeScriptDraftItem(project: ProjectDto, draft: ScriptDraftNarrationDto, existing?: StoryboardItemDto): StoryboardItemDto {
  const sourceText = draft.sourceText.trim()
  const narrationText = draft.narrationText?.trim() || sourceText
  if (!existing) {
    return createStoryboardItem(project, {
      index: draft.index,
      text: sourceText,
      locked: false,
    }, narrationText)
  }

  const next = cloneStoryboardItem(existing)
  if (!isStoryboardItemLocked(next, 'sourceText')) {
    next.sourceText = sourceText
  }
  if (!isStoryboardItemLocked(next, 'narrationText')) {
    next.narrationText = narrationText
  }
  return next
}

function createStoryboardItem(project: ProjectDto, narration: NarrationDto, narrationText = narration.text): StoryboardItemDto {
  const sourceText = narration.text
  const visualSeed = toShortText(sourceText, 48)
  const stylePrompt = project.stylePrompt?.trim()
  const styleSuffix = stylePrompt ? `，整体画风：${stylePrompt}` : ''

  return {
    itemId: createMockId('item'),
    projectId: project.projectId,
    index: narration.index,
    sourceText,
    narrationText,
    visualGoal: `讲清：${visualSeed}`,
    visualDescription: `${MOCK_STORYBOARD_GOAL}：围绕“${visualSeed}”设计一个清晰、可生图的生活化镜头${styleSuffix}。`,
    characters: ['主角'],
    characterIds: [],
    locationId: null,
    sceneDescription: '演示场景占位',
    imagePrompt: `演示生图提示词：${visualSeed}，主角，清晰主体，真实短视频画面，${project.aspectRatio} 构图${styleSuffix}`,
    negativePrompt: DEFAULT_NEGATIVE_PROMPT,
    videoPrompt: `演示图生视频动作：基于第 ${narration.index} 条分镜做轻微镜头推进，保持主体稳定。`,
    durationSeconds: project.segmentDurationSeconds,
    subtitleChunks: [],
    audioPath: null,
    audioDurationSeconds: null,
    audioProbe: null,
    selectedImageId: null,
    selectedVideoSegmentId: null,
    status: 'pending',
    lockFlagsJson: {},
    shotSize: 'medium',
    cameraMotion: 'static',
    composition: 'center',
    pace: 'normal',
    transitionType: 'cut',
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
  }
}

function resolveSplitMode(inputOptions?: Record<string, unknown>): TextSplitMode {
  const splitMode = inputOptions?.splitMode
  if (splitMode === 'line' || splitMode === 'sentence') return splitMode
  return 'paragraph'
}

function splitText(text: string, splitMode: TextSplitMode): string[] {
  const normalized = text.replace(/\r\n/g, '\n').trim()
  if (!normalized) return []

  if (splitMode === 'line') {
    return normalized
      .split('\n')
      .map(trimSegment)
      .filter(Boolean)
  }

  if (splitMode === 'sentence') {
    return (normalized.match(/[^。！？.!?]+[。！？.!?]?/g) ?? [normalized]).map(trimSegment).filter(Boolean)
  }

  return normalized
    .split(/\n\s*\n|\n/)
    .map(trimSegment)
    .filter(Boolean)
}

function splitSubtitleText(text: string): string[] {
  const sentences = (normalizeText(text).match(/[^。！？；!?;.]+[。！？；!?;.]?/g) ?? [normalizeText(text)]).map(trimSegment).filter(Boolean)
  const chunks: string[] = []
  let current = ''
  for (const sentence of sentences) {
    for (const part of splitLongSubtitleSentence(sentence)) {
      if (!current) current = part
      else if (subtitleTextWeight(current) + subtitleTextWeight(part) <= 18) current += part
      else {
        chunks.push(current)
        current = part
      }
    }
  }
  if (current) chunks.push(current)
  return chunks.length > 0 ? chunks : [normalizeText(text)]
}

function splitLongSubtitleSentence(sentence: string): string[] {
  if (subtitleTextWeight(sentence) <= 18) return [sentence]
  const parts: string[] = []
  let current = ''
  for (const char of sentence) {
    current += char
    const shouldBreak = subtitleTextWeight(current) >= 12 && (/[，、, ：:]/.test(char) || subtitleTextWeight(current) >= 18)
    if (shouldBreak) {
      parts.push(normalizeText(current))
      current = ''
    }
  }
  if (current) parts.push(normalizeText(current))
  return parts.filter(Boolean)
}

function createSubtitleChunks(item: StoryboardItemDto, lines: string[]): SubtitleChunkDto[] {
  const duration = subtitleItemDuration(item)
  const totalWeight = Math.max(lines.length, lines.reduce((total, line) => total + subtitleTextWeight(line), 0))
  let cursor = 0
  return lines.map((text, index) => {
    const startSeconds = cursor
    const endSeconds = index === lines.length - 1 ? duration : Math.min(duration, cursor + duration * (Math.max(1, subtitleTextWeight(text)) / totalWeight))
    cursor = endSeconds
    return {
      chunkId: `sub_${item.itemId}_${index + 1}`,
      text,
      startSeconds: roundSeconds(startSeconds),
      endSeconds: roundSeconds(Math.max(startSeconds, endSeconds)),
      estimated: true,
    }
  })
}

function createSubtitlesFile(projectId: string, items: StoryboardItemDto[]): SubtitlesFileDto {
  let cursor = 0
  const chunks = [...items]
    .sort((left, right) => left.index - right.index)
    .flatMap((item) => {
      const itemStart = cursor
      const duration = subtitleItemDuration(item)
      cursor += duration
      return item.subtitleChunks.map((chunk) => ({
        itemId: item.itemId,
        itemIndex: item.index,
        chunkId: chunk.chunkId,
        text: chunk.text,
        startSeconds: roundSeconds(itemStart + Math.max(0, chunk.startSeconds ?? 0)),
        endSeconds: roundSeconds(itemStart + Math.min(duration, chunk.endSeconds ?? duration)),
        estimated: chunk.estimated,
        wordTimings: createEstimatedWordTimings(chunk.text, itemStart + Math.max(0, chunk.startSeconds ?? 0), itemStart + Math.min(duration, chunk.endSeconds ?? duration)),
      }))
    })
  return {
    schemaVersion: 1,
    projectId,
    generatedAt: MOCK_NOW,
    style: {
      presetId: 'vertical_cn_default',
      position: 'bottom',
      fontSize: 42,
      color: '#FFFFFF',
      outlineColor: '#111111',
      outlineWidth: 3,
      highlightColor: '#FFD54A',
      mode: 'karaoke_estimated',
      safeTop: 96,
      safeBottom: 160,
      safeLeft: 48,
      safeRight: 48,
      maxCharsPerLine: 18,
    },
    chunks,
  }
}

function createEstimatedWordTimings(text: string, startSeconds: number, endSeconds: number) {
  const tokens = subtitleTimingTokens(text)
  if (tokens.length === 0) return []
  const duration = Math.max(0.001, endSeconds - startSeconds)
  const totalWeight = Math.max(tokens.length, tokens.reduce((total, token) => total + subtitleTextWeight(token), 0))
  let cursor = startSeconds
  return tokens.map((token, index) => {
    const tokenStart = cursor
    const tokenEnd = index === tokens.length - 1 ? endSeconds : Math.min(endSeconds, cursor + duration * (Math.max(1, subtitleTextWeight(token)) / totalWeight))
    cursor = tokenEnd
    return {
      token,
      startSeconds: roundSeconds(tokenStart),
      endSeconds: roundSeconds(Math.max(tokenStart, tokenEnd)),
      estimated: true,
    }
  })
}

function subtitleTimingTokens(text: string): string[] {
  const normalized = normalizeText(text)
  if (normalized.includes(' ')) return normalized.split(' ').map((token) => token.trim()).filter(Boolean)
  return [...normalized].filter((char) => !/\s/.test(char))
}

function subtitleItemDuration(item: StoryboardItemDto): number {
  return item.audioDurationSeconds && item.audioDurationSeconds > 0 ? item.audioDurationSeconds : Math.max(1, item.durationSeconds || 1)
}

function subtitleTextWeight(text: string): number {
  return [...text].filter((char) => !/\s/.test(char)).length
}

function roundSeconds(value: number): number {
  return Math.round(value * 1000) / 1000
}

function splitByFixedGroup(segments: string[], groupCount: number, joiner: string): string[] {
  if (segments.length === 0) return []

  const grouped: string[] = []
  for (let index = 0; index < segments.length; index += groupCount) {
    grouped.push(segments.slice(index, index + groupCount).join(joiner))
  }
  return grouped
}

function limitSegments(segments: string[], targetCount: number, joiner: string): string[] {
  if (segments.length <= targetCount) return segments
  if (targetCount <= 1) return [segments.join(joiner)]

  return [...segments.slice(0, targetCount - 1), segments.slice(targetCount - 1).join(joiner)]
}

function clampSceneCount(value: number): number {
  return Math.min(60, Math.max(1, Math.round(value || 1)))
}

function clampGroupCount(value?: number): number {
  return Math.min(20, Math.max(1, Math.round(value || 1)))
}

function normalizeStoryboardItems(projectId: string, items: StoryboardItemDto[]): StoryboardItemDto[] {
  return items.map((item, index) =>
    withItemRelations({
      ...item,
      projectId,
      index: index + 1,
    })
  )
}

function createNarrationsFromItems(items: StoryboardItemDto[]): NarrationDto[] {
  return items.map((item) => ({
    index: item.index,
    text: item.narrationText || item.sourceText,
    locked: isScriptTextLocked(item),
  }))
}

function cloneStoryboardItems(items: StoryboardItemDto[]): StoryboardItemDto[] {
  return items.map((item) => cloneStoryboardItem(item))
}

function cloneStoryboardItem(item: StoryboardItemDto): StoryboardItemDto {
  return {
    ...item,
    characters: [...item.characters],
    characterIds: [...item.characterIds],
    lockFlagsJson: { ...item.lockFlagsJson },
    imageCandidates: [...item.imageCandidates],
    videoSegments: [...item.videoSegments],
    downstreamResetRecords: [...(item.downstreamResetRecords ?? [])],
  }
}

function isScriptTextLocked(item: StoryboardItemDto): boolean {
  return isStoryboardItemLocked(item, 'sourceText') || isStoryboardItemLocked(item, 'narrationText')
}

function normalizeText(text: string): string {
  return text.replace(/[ \t]+/g, ' ').trim()
}

function trimSegment(text: string): string {
  return text.trim()
}

function toShortText(text: string, maxLength: number): string {
  const normalized = normalizeText(text)
  if (normalized.length <= maxLength) return normalized
  return `${normalized.slice(0, maxLength)}...`
}

function findStateByItemId(itemId: string): StoryboardMockState | undefined {
  return [...storyboardStateByProjectId.values()].find((state) => state.items.some((item) => item.itemId === itemId))
}

function withItemCandidates(item: StoryboardItemDto): StoryboardItemDto {
  return withItemRelations(item)
}

function withItemRelations(item: StoryboardItemDto): StoryboardItemDto {
  return {
    ...item,
    imageCandidates: imageCandidates.filter((candidate) => candidate.itemId === item.itemId),
    videoSegments: videoSegments.filter((segment) => segment.itemId === item.itemId),
  }
}

function syncSelectedImageCandidates(item: StoryboardItemDto) {
  imageCandidates = imageCandidates.map((candidate) => (candidate.itemId === item.itemId ? { ...candidate, selected: Boolean(item.selectedImageId && candidate.imageId === item.selectedImageId) } : candidate))
}

function syncSelectedVideoSegments(item: StoryboardItemDto) {
  videoSegments = videoSegments.map((segment) => (segment.itemId === item.itemId ? { ...segment, selected: Boolean(item.selectedVideoSegmentId && segment.segmentId === item.selectedVideoSegmentId) } : segment))
}

function getNextImageRevision(itemId: string): number {
  return getLatestImageRevision(itemId) + 1
}

function getLatestImageRevision(itemId: string): number {
  return Math.max(0, ...imageCandidates.filter((candidate) => candidate.itemId === itemId).map(getGenerationRevision))
}

function getNextVideoRevision(itemId: string): number {
  return getLatestVideoRevision(itemId) + 1
}

function getLatestVideoRevision(itemId: string): number {
  return Math.max(0, ...videoSegments.filter((segment) => segment.itemId === itemId).map(getGenerationRevision))
}

function getGenerationRevision(entry: { generationContextSnapshot: Record<string, unknown> }): number {
  const revision = entry.generationContextSnapshot.revision
  return typeof revision === 'number' && Number.isFinite(revision) ? revision : 1
}

function createImageCandidate(projectId: string, item: StoryboardItemDto, variantIndex: number, totalCount: number, revision: number): ImageCandidateDto {
  const imageId = createMockId('img')
  return {
    imageId,
    itemId: item.itemId,
    imagePath: `mock-images/${projectId}/${item.itemId}/${imageId}.png`,
    prompt: item.imagePrompt,
    negativePrompt: item.negativePrompt,
    model: 'mock-image-model',
    providerModelId: 'mock/image-to-video-still',
    workflowPresetId: 'mock-still-v1',
    status: 'succeeded',
    selected: false,
    createdAt: MOCK_NOW,
    derivedFromImageId: item.selectedImageId,
    generationContextSnapshot: {
      mock: true,
      projectId,
      itemId: item.itemId,
      index: item.index,
      revision,
      candidateCount: totalCount,
      variantIndex,
      visualToneClass: `scene-preview-tone-${(item.index + variantIndex) % 5}`,
      imagePrompt: item.imagePrompt,
      negativePrompt: item.negativePrompt,
      providerOutputKind: 'local_mock_placeholder',
    },
  }
}

function defaultAssetKindForImageKind(imageKind: StartImageAssetGenerationRequest['imageKind']) {
  if (imageKind === 'character_reference') return 'character_reference_image'
  if (imageKind === 'scene_reference') return 'scene_reference_image'
  if (imageKind === 'style_reference') return 'style_reference_image'
  if (imageKind === 'cover_image') return 'cover_source'
  return 'generated_output'
}

function usageKindForImageKind(imageKind: StartImageAssetGenerationRequest['imageKind']) {
  if (imageKind === 'character_reference') return 'character_reference'
  if (imageKind === 'scene_reference') return 'location_reference'
  if (imageKind === 'style_reference') return 'style_reference'
  if (imageKind === 'cover_image') return 'cover'
  return 'reference_image'
}

function createVideoSegment(projectId: string, item: StoryboardItemDto, selectedImage: ImageCandidateDto, variantIndex: number, totalCount: number, revision: number): VideoSegmentDto {
  const segmentId = createMockId('seg')
  return {
    segmentId,
    itemId: item.itemId,
    inputImageId: selectedImage.imageId,
    videoPath: `mock-videos/${projectId}/${item.itemId}/${segmentId}.mp4`,
    videoPrompt: item.videoPrompt,
    durationSeconds: item.durationSeconds,
    model: 'mock-video-model',
    providerModelId: 'mock/image-to-video',
    workflowPresetId: 'mock-i2v-v1',
    status: 'succeeded',
    selected: false,
    createdAt: MOCK_NOW,
    mediaProbe: null,
    generationContextSnapshot: {
      mock: true,
      projectId,
      itemId: item.itemId,
      index: item.index,
      revision,
      segmentCount: totalCount,
      variantIndex,
      inputImageId: selectedImage.imageId,
      inputImagePath: selectedImage.imagePath,
      videoPrompt: item.videoPrompt,
      providerOutputKind: 'local_mock_video_placeholder',
      statusFlow: ['pending', 'running', 'succeeded', 'selected_by_user'],
      workflowType: 'image_to_video',
    },
  }
}
