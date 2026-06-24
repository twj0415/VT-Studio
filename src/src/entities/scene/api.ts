import { createMockId } from '@/shared/mock/ids'
import type { CompositionTaskDto } from '@/entities/task/types'
import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'
import { getProjectDetail } from '@/entities/project/api'
import type { ProjectDto } from '@/entities/project/types'

import type { ImageCandidateDto, NarrationDto, RegenerateStoryboardRequest, SceneDto, StoryboardDto, StoryboardItemDto, VideoSegmentDto } from './types'

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

export async function regenerateStoryboard(projectId: string, request: RegenerateStoryboardRequest): Promise<StoryboardDto> {
  const state = await ensureStoryboardState(projectId)
  const detail = await getProjectDetail(projectId)
  const project: ProjectDto = {
    ...detail.project,
    projectId,
  }

  state.previousItems = cloneStoryboardItems(state.items)
  state.reviewStatus = 'waiting_user'
  state.confirmedNarrations = createMockNarrations(project, request)
  state.items = state.confirmedNarrations.map((narration) => createStoryboardItem(project, narration))
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
  const state = await ensureStoryboardState(projectId)
  const itemIds = new Set(state.items.map((item) => item.itemId))
  return imageCandidates.filter((candidate) => itemIds.has(candidate.itemId))
}

export async function listVideoSegments(projectId: string): Promise<VideoSegmentDto[]> {
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
    return callCommand<ImageCandidateDto[]>(tauriCommands.startImageGeneration, { request: { projectId, itemId, count } })
  }

  const state = await ensureStoryboardState(projectId)
  const item = state.items.find((entry) => entry.itemId === itemId)
  if (!item) throw new Error(`Storyboard item not found: ${itemId}`)

  const generated = Array.from({ length: count }, (_, index) => createImageCandidate(projectId, item, index + 1, count))
  imageCandidates = [...imageCandidates.filter((candidate) => candidate.itemId !== itemId), ...generated]
  state.items = state.items.map((entry) => (entry.itemId === itemId ? withItemCandidates({ ...entry, imageStatus: 'succeeded', status: 'succeeded', selectedImageId: null }) : entry))
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

  const nextItem = withItemCandidates({ ...updatedItem, selectedImageId: imageId })
  state.items = state.items.map((entry) => (entry.itemId === itemId ? nextItem : entry))
  return nextItem
}

export async function startVideoGeneration(projectId: string, itemId: string, count = 1): Promise<VideoSegmentDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoSegmentDto[]>(tauriCommands.startVideoGeneration, { request: { projectId, itemId, count } })
  }

  const state = await ensureStoryboardState(projectId)
  const item = state.items.find((entry) => entry.itemId === itemId)
  if (!item) throw new Error(`Storyboard item not found: ${itemId}`)
  if (!item.selectedImageId) throw new Error(`Storyboard item has no selected image: ${itemId}`)

  const selectedImage = imageCandidates.find((candidate) => candidate.itemId === itemId && candidate.imageId === item.selectedImageId)
  if (!selectedImage) throw new Error(`Selected image candidate not found: ${item.selectedImageId}`)

  const generated = Array.from({ length: count }, (_, index) => createVideoSegment(projectId, item, selectedImage, index + 1, count))
  videoSegments = [...videoSegments.filter((segment) => segment.itemId !== itemId), ...generated]
  state.items = state.items.map((entry) => (entry.itemId === itemId ? withItemRelations({ ...entry, videoStatus: 'succeeded', segmentStatus: 'succeeded', selectedVideoSegmentId: null }) : entry))
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

  const nextItem = withItemRelations({ ...updatedItem, selectedVideoSegmentId: segmentId })
  state.items = state.items.map((entry) => (entry.itemId === itemId ? nextItem : entry))
  return nextItem
}

export async function startComposition(projectId: string): Promise<CompositionTaskDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CompositionTaskDto>(tauriCommands.startComposition, { request: { projectId } })
  }

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
    audioStatus: 'pending',
    videoStatus: 'pending',
    subtitleStatus: 'pending',
    renderStatus: 'pending',
    segmentStatus: 'pending',
    imageCandidates: [],
    videoSegments: [],
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
    locked: false,
  }))
}

function cloneStoryboardItems(items: StoryboardItemDto[]): StoryboardItemDto[] {
  return items.map((item) => ({
    ...item,
    characters: [...item.characters],
    characterIds: [...item.characterIds],
    lockFlagsJson: { ...item.lockFlagsJson },
    imageCandidates: [...item.imageCandidates],
    videoSegments: [...item.videoSegments],
  }))
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

function createImageCandidate(projectId: string, item: StoryboardItemDto, variantIndex: number, totalCount: number): ImageCandidateDto {
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
      candidateCount: totalCount,
      variantIndex,
      visualToneClass: `scene-preview-tone-${(item.index + variantIndex) % 5}`,
      imagePrompt: item.imagePrompt,
      negativePrompt: item.negativePrompt,
      providerOutputKind: 'local_mock_placeholder',
    },
  }
}

function createVideoSegment(projectId: string, item: StoryboardItemDto, selectedImage: ImageCandidateDto, variantIndex: number, totalCount: number): VideoSegmentDto {
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
    generationContextSnapshot: {
      mock: true,
      projectId,
      itemId: item.itemId,
      index: item.index,
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
