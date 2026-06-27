import type { SceneAssetStatus } from '@/shared/enums/generated'
import type { MediaProbeDto } from '@/entities/config/types'
import type { AssetDto, AssetReferenceDto } from '@/entities/config/types'

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

export interface ScriptDraftNarrationDto {
  index: number
  sourceText: string
  narrationText?: string
}

export interface ApplyScriptDraftRequest {
  projectId: string
  rawOutput: string
  expectedCount?: number
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

export type GeneratedImageKind = 'character_reference' | 'scene_reference' | 'style_reference' | 'prop_reference' | 'end_frame' | 'control_image' | 'cover_image'

export interface StartImageAssetGenerationRequest {
  projectId: string
  imageKind: GeneratedImageKind
  assetKind?: string
  ownerKind: string
  ownerId: string
  referenceRole: string
  itemId?: string
  prompt: string
  negativePrompt?: string
  count?: number
  providerModelId?: string
  workflowPresetId?: string
  workflowParams?: Record<string, unknown>
  aspectRatio?: string
  width?: number
  height?: number
  seed?: number
}

export interface BuildCharacterResourcePlanRequest {
  projectId: string
  itemId: string
  providerModelId?: string
  workflowPresetId?: string
}

export interface CharacterResourceRequirementDto {
  characterId: string
  characterName: string
  role: string
  requirement: 'required' | 'optional' | 'unused' | string
  available: boolean
  assetId?: string | null
  relativePath?: string | null
  missingReason?: string | null
  sourceOptions: string[]
}

export interface CharacterResourcePlanDto {
  projectId: string
  itemId: string
  optionId: string
  sourceType: string
  sourceId: string
  providerModelId?: string | null
  workflowPresetId?: string | null
  requiredCount: number
  optionalCount: number
  unusedCount: number
  missingRequiredCount: number
  items: CharacterResourceRequirementDto[]
}

export interface GeneratedImageAssetDto {
  asset: AssetDto
  reference: AssetReferenceDto
  imageKind: GeneratedImageKind
  assetKind: string
  ownerKind: string
  ownerId: string
  referenceRole: string
  usageKind: string
  relativePath: string
  generationContextSnapshot: Record<string, unknown>
}

export interface StartVideoGenerationRequest {
  projectId: string
  itemId: string
  count?: number
  providerModelId?: string
  workflowPresetId?: string
  workflowParams?: Record<string, unknown>
  aspectRatio?: string
  resolution?: string
  fps?: number
  seed?: number
}

export interface StartTtsGenerationRequest {
  projectId: string
  itemId: string
  providerModelId?: string
  voiceId?: string
  speed?: number
  pitch?: number
  volume?: number
  format?: 'mp3' | 'wav' | string
  sampleRate?: number
}

export interface ReplaceStoryboardAudioRequest {
  projectId: string
  itemId: string
  sourcePath: string
}

export interface ProbeStoryboardAudioRequest {
  projectId: string
  itemId: string
}

export interface GenerateSubtitlesRequest {
  projectId: string
  itemIds?: string[]
}

export interface UpdateStoryboardSubtitlesRequest {
  projectId: string
  itemId: string
  subtitleChunks: SubtitleChunkDto[]
}

export interface SubtitleStyleDto {
  presetId: string
  position: string
  fontSize: number
  color: string
  outlineColor: string
  outlineWidth: number
  highlightColor: string
  mode: string
  safeTop: number
  safeBottom: number
  safeLeft: number
  safeRight: number
  maxCharsPerLine: number
}

export interface SubtitleWordTimingDto {
  token: string
  startSeconds: number
  endSeconds: number
  estimated: boolean
}

export interface SubtitleTimelineChunkDto {
  itemId: string
  itemIndex: number
  chunkId: string
  text: string
  startSeconds: number
  endSeconds: number
  estimated: boolean
  wordTimings: SubtitleWordTimingDto[]
}

export interface SubtitlesFileDto {
  schemaVersion: number
  projectId: string
  generatedAt: string
  style: SubtitleStyleDto
  chunks: SubtitleTimelineChunkDto[]
}

export interface GenerateSubtitlesResultDto {
  projectId: string
  subtitlePath: string
  items: StoryboardItemDto[]
  subtitles: SubtitlesFileDto
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
  mediaProbe?: MediaProbeDto | null
  generationContextSnapshot: Record<string, unknown>
}

export type StoryboardItemLockField =
  | 'sourceText'
  | 'narrationText'
  | 'visualDescription'
  | 'characters'
  | 'location'
  | 'imagePrompt'
  | 'negativePrompt'
  | 'videoPrompt'
  | 'selectedImage'
  | 'selectedVideoSegment'

export type StoryboardLegacyLockField = 'image' | 'video' | 'audio' | 'subtitle'

export type StoryboardItemLockFlags = Partial<Record<StoryboardItemLockField | StoryboardLegacyLockField, boolean>>

export type StoryboardResetTriggerField = 'imagePrompt' | 'selectedImageId' | 'videoPrompt' | 'selectedVideoSegmentId'

export type StoryboardResetAffectedObject = 'imageCandidates' | 'selectedImageId' | 'videoSegments' | 'selectedVideoSegmentId' | 'composition'

export interface StoryboardDownstreamResetRecord {
  resetId: string
  itemId: string
  triggerField: StoryboardResetTriggerField
  affectedObjects: StoryboardResetAffectedObject[]
  reason: string
  createdAt: string
}

export interface SubtitleChunkDto {
  chunkId: string
  text: string
  startSeconds?: number | null
  endSeconds?: number | null
  estimated: boolean
}

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
  subtitleChunks: SubtitleChunkDto[]
  audioPath?: string | null
  audioDurationSeconds?: number | null
  audioProbe?: MediaProbeDto | null
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
  imageLastErrorJson: Record<string, unknown> | null
  imageRetryCount: number
  audioStatus: SceneAssetStatus
  audioLastErrorJson: Record<string, unknown> | null
  audioRetryCount: number
  videoStatus: SceneAssetStatus
  subtitleStatus: SceneAssetStatus
  renderStatus: SceneAssetStatus
  segmentStatus: SceneAssetStatus
  imageCandidates: ImageCandidateDto[]
  videoSegments: VideoSegmentDto[]
  downstreamResetRecords?: StoryboardDownstreamResetRecord[]
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
