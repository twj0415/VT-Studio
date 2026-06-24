import type { SceneAssetStatus } from '@/shared/enums/generated'

export interface NarrationDto {
  index: number
  text: string
  locked: boolean
}

export type StoryboardSplitMode = 'paragraph' | 'line_count' | 'sentence_count' | 'ai'

export interface RegenerateStoryboardRequest {
  mode: StoryboardSplitMode
  lineCount?: number
  sentenceCount?: number
  targetSceneCount?: number
}

export interface ImageCandidateDto {
  imageId: string
  itemId: string
  imagePath: string
  prompt: string
  negativePrompt: string
  model: string
  providerModelId: string
  workflowPresetId: string | null
  status: SceneAssetStatus
  selected: boolean
  createdAt: string
  derivedFromImageId: string | null
  generationContextSnapshot: Record<string, unknown>
}

export interface VideoSegmentDto {
  segmentId: string
  itemId: string
  inputImageId: string
  videoPath: string
  videoPrompt: string
  durationSeconds: number
  model: string
  providerModelId: string
  workflowPresetId: string | null
  status: SceneAssetStatus
  selected: boolean
  createdAt: string
  generationContextSnapshot: Record<string, unknown>
}

export type StoryboardItemLockFlags = Partial<Record<'sourceText' | 'narrationText' | 'visualDescription' | 'imagePrompt' | 'videoPrompt' | 'image' | 'video' | 'audio' | 'subtitle', boolean>>

export interface StoryboardItemDto {
  itemId: string
  projectId: string
  index: number
  sourceText: string
  narrationText: string
  visualGoal: string
  visualDescription: string
  characters: string[]
  characterIds: string[]
  locationId: string | null
  sceneDescription: string
  imagePrompt: string
  negativePrompt: string
  videoPrompt: string
  durationSeconds: number
  selectedImageId: string | null
  selectedVideoSegmentId: string | null
  status: SceneAssetStatus
  lockFlagsJson: StoryboardItemLockFlags
  shotSize?: string
  cameraMotion?: string
  composition?: string
  pace?: string
  transitionType?: string
  imageStatus: SceneAssetStatus
  audioStatus: SceneAssetStatus
  videoStatus: SceneAssetStatus
  subtitleStatus: SceneAssetStatus
  renderStatus: SceneAssetStatus
  segmentStatus: SceneAssetStatus
  imageCandidates: ImageCandidateDto[]
  videoSegments: VideoSegmentDto[]
}

// 公开业务入口使用 entities/storyboard；当前旧 Scene 命名只作为兼容别名。
export type SceneDto = StoryboardItemDto

export interface StoryboardDto {
  storyboardId: string
  projectId: string
  confirmedNarrations: NarrationDto[]
  items: StoryboardItemDto[]
  reviewStatus: 'waiting_user' | 'succeeded'
}
