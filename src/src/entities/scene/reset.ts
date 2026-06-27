import type { StoryboardDownstreamResetRecord, StoryboardItemDto, StoryboardItemLockField, StoryboardResetAffectedObject, StoryboardResetTriggerField } from './types'

export interface StoryboardDownstreamResetResult {
  item: StoryboardItemDto
  records: StoryboardDownstreamResetRecord[]
  affectedObjects: StoryboardResetAffectedObject[]
}

export const STORYBOARD_DEPENDENCY_RESET_MAP = {
  imagePrompt: ['imageCandidates', 'selectedImageId', 'videoSegments', 'selectedVideoSegmentId', 'composition'],
  selectedImageId: ['videoSegments', 'selectedVideoSegmentId', 'composition'],
  videoPrompt: ['videoSegments', 'selectedVideoSegmentId', 'composition'],
  selectedVideoSegmentId: ['composition'],
} as const satisfies Record<StoryboardResetTriggerField, readonly StoryboardResetAffectedObject[]>

const RESET_REASON_BY_TRIGGER_FIELD: Record<StoryboardResetTriggerField, string> = {
  imagePrompt: 'image_prompt_changed',
  selectedImageId: 'selected_image_changed',
  videoPrompt: 'video_prompt_changed',
  selectedVideoSegmentId: 'selected_video_segment_changed',
}

const RESET_TRIGGER_FIELDS: StoryboardResetTriggerField[] = ['imagePrompt', 'selectedImageId', 'videoPrompt', 'selectedVideoSegmentId']
const MAX_RESET_RECORDS_PER_ITEM = 20

export function applyStoryboardDownstreamReset(previous: StoryboardItemDto, next: StoryboardItemDto, createdAt = new Date().toISOString()): StoryboardDownstreamResetResult {
  const triggerFields = getChangedResetTriggerFields(previous, next)
  if (triggerFields.length === 0) {
    return {
      item: next,
      records: [],
      affectedObjects: [],
    }
  }

  const affectedObjects = uniqueAffectedObjects(triggerFields)
  const records = triggerFields.map((triggerField) => createResetRecord(next.itemId, triggerField, STORYBOARD_DEPENDENCY_RESET_MAP[triggerField], createdAt))
  const item = applyAffectedObjects(next, affectedObjects, records)

  return {
    item,
    records,
    affectedObjects,
  }
}

export function getLatestStoryboardResetRecord(item: StoryboardItemDto): StoryboardDownstreamResetRecord | null {
  const records = item.downstreamResetRecords ?? []
  return records.length > 0 ? records[records.length - 1] : null
}

export function isResetRelevantToImage(record: StoryboardDownstreamResetRecord | null): boolean {
  return Boolean(record?.affectedObjects.some((object) => object === 'imageCandidates' || object === 'selectedImageId'))
}

export function isResetRelevantToVideo(record: StoryboardDownstreamResetRecord | null): boolean {
  return Boolean(record?.affectedObjects.some((object) => object === 'videoSegments' || object === 'selectedVideoSegmentId'))
}

export function isResetRelevantToComposition(record: StoryboardDownstreamResetRecord | null): boolean {
  return Boolean(record?.affectedObjects.includes('composition'))
}

export function isStoryboardItemLockedForBulkImageGeneration(item: StoryboardItemDto): boolean {
  return isStoryboardItemLocked(item, 'imagePrompt') || isStoryboardItemLocked(item, 'negativePrompt') || isStoryboardItemLocked(item, 'selectedImage') || Boolean(item.lockFlagsJson.image)
}

export function isStoryboardItemLockedForBulkVideoGeneration(item: StoryboardItemDto): boolean {
  return isStoryboardItemLocked(item, 'videoPrompt') || isStoryboardItemLocked(item, 'selectedVideoSegment') || Boolean(item.lockFlagsJson.video)
}

export function isStoryboardItemLocked(item: StoryboardItemDto, field: StoryboardItemLockField): boolean {
  return Boolean(item.lockFlagsJson[field])
}

export function setStoryboardItemLock(item: StoryboardItemDto, field: StoryboardItemLockField, locked: boolean): StoryboardItemDto {
  const lockFlagsJson = { ...item.lockFlagsJson }
  if (locked) lockFlagsJson[field] = true
  else delete lockFlagsJson[field]
  return {
    ...item,
    lockFlagsJson,
  }
}

export function lockedFieldsForImageGeneration(item: StoryboardItemDto): StoryboardItemLockField[] {
  return (['imagePrompt', 'negativePrompt', 'selectedImage'] as StoryboardItemLockField[]).filter((field) => isStoryboardItemLocked(item, field))
}

export function lockedFieldsForVideoGeneration(item: StoryboardItemDto): StoryboardItemLockField[] {
  return (['videoPrompt', 'selectedVideoSegment'] as StoryboardItemLockField[]).filter((field) => isStoryboardItemLocked(item, field))
}

export function hasValidSelectedImage(item: StoryboardItemDto): boolean {
  if (!item.selectedImageId) return false
  const selected = item.imageCandidates.find((candidate) => candidate.imageId === item.selectedImageId)
  if (!selected) return false

  return normalizeText(selected.prompt) === normalizeText(item.imagePrompt)
}

export function hasValidSelectedVideoSegment(item: StoryboardItemDto): boolean {
  if (!item.selectedVideoSegmentId) return false
  const selected = item.videoSegments.find((segment) => segment.segmentId === item.selectedVideoSegmentId)
  if (!selected) return false
  if (item.selectedImageId && selected.inputImageId !== item.selectedImageId) return false

  return normalizeText(selected.videoPrompt) === normalizeText(item.videoPrompt)
}

function getChangedResetTriggerFields(previous: StoryboardItemDto, next: StoryboardItemDto): StoryboardResetTriggerField[] {
  return RESET_TRIGGER_FIELDS.filter((field) => normalizeComparableValue(previous[field]) !== normalizeComparableValue(next[field]))
}

function uniqueAffectedObjects(triggerFields: StoryboardResetTriggerField[]): StoryboardResetAffectedObject[] {
  const affectedObjects = new Set<StoryboardResetAffectedObject>()
  for (const triggerField of triggerFields) {
    for (const affectedObject of STORYBOARD_DEPENDENCY_RESET_MAP[triggerField]) {
      affectedObjects.add(affectedObject)
    }
  }
  return [...affectedObjects]
}

function createResetRecord(itemId: string, triggerField: StoryboardResetTriggerField, affectedObjects: readonly StoryboardResetAffectedObject[], createdAt: string): StoryboardDownstreamResetRecord {
  return {
    resetId: `reset_${itemId}_${triggerField}_${createdAt.replace(/[^0-9A-Za-z]/g, '')}`,
    itemId,
    triggerField,
    affectedObjects: [...affectedObjects],
    reason: RESET_REASON_BY_TRIGGER_FIELD[triggerField],
    createdAt,
  }
}

function applyAffectedObjects(item: StoryboardItemDto, affectedObjects: StoryboardResetAffectedObject[], records: StoryboardDownstreamResetRecord[]): StoryboardItemDto {
  const affected = new Set(affectedObjects)
  const shouldClearImageSelection = affected.has('selectedImageId')
  const shouldClearVideoSelection = affected.has('selectedVideoSegmentId')
  const shouldResetImage = affected.has('imageCandidates') || affected.has('selectedImageId')

  return {
    ...item,
    selectedImageId: shouldClearImageSelection ? null : item.selectedImageId,
    selectedVideoSegmentId: shouldClearVideoSelection ? null : item.selectedVideoSegmentId,
    imageStatus: shouldResetImage ? 'pending' : item.imageStatus,
    imageLastErrorJson: shouldResetImage ? null : item.imageLastErrorJson,
    imageRetryCount: shouldResetImage ? 0 : item.imageRetryCount,
    videoStatus: affected.has('videoSegments') || affected.has('selectedVideoSegmentId') ? 'pending' : item.videoStatus,
    segmentStatus: affected.has('videoSegments') || affected.has('selectedVideoSegmentId') ? 'pending' : item.segmentStatus,
    renderStatus: affected.has('composition') ? 'pending' : item.renderStatus,
    status: affectedObjects.length > 0 ? 'pending' : item.status,
    imageCandidates: item.imageCandidates.map((candidate) => ({
      ...candidate,
      selected: shouldClearImageSelection ? false : candidate.imageId === item.selectedImageId || candidate.selected,
    })),
    videoSegments: item.videoSegments.map((segment) => ({
      ...segment,
      selected: shouldClearVideoSelection ? false : segment.segmentId === item.selectedVideoSegmentId || segment.selected,
    })),
    downstreamResetRecords: [...(item.downstreamResetRecords ?? []), ...records].slice(-MAX_RESET_RECORDS_PER_ITEM),
  }
}

function normalizeComparableValue(value: string | null | undefined): string {
  return normalizeText(value ?? '')
}

function normalizeText(value: string): string {
  return value.replace(/\s+/g, ' ').trim()
}
