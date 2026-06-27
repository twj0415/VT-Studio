import { createMockId } from '@/shared/mock/ids'
import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'

import type { VideoSegmentDto } from '@/entities/storyboard/types'
import type {
  ImageSlideshowItemStateDto,
  ImageSlideshowProjectRequest,
  ImageSlideshowProjectStateDto,
  RegisterTemplateMotionSegmentRequest,
} from './types'

const MOCK_NOW = '2026-06-22 10:00'
const states = new Map<string, ImageSlideshowProjectStateDto>()

export async function getImageSlideshowProjectState(projectId: string): Promise<ImageSlideshowProjectStateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ImageSlideshowProjectStateDto>(tauriCommands.getImageSlideshowProjectState, { projectId })
  }

  return ensureState(projectId)
}

export async function registerTemplateMotionSegment(
  request: RegisterTemplateMotionSegmentRequest,
): Promise<VideoSegmentDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoSegmentDto, { request: RegisterTemplateMotionSegmentRequest }>(
      tauriCommands.registerTemplateMotionSegment,
      { request },
    )
  }

  const segment: VideoSegmentDto = {
    segmentId: createMockId('slideshow_segment'),
    itemId: request.itemId,
    inputImageId: request.inputImageId,
    videoPath: request.videoPath,
    videoPrompt: `template_motion:${request.templateType}:${request.templateId}`,
    durationSeconds: request.durationSeconds,
    model: 'template_motion',
    providerModelId: 'local_template_motion',
    workflowPresetId: request.workflowPresetId ?? null,
    status: 'succeeded',
    selected: true,
    createdAt: MOCK_NOW,
    mediaProbe: null,
    generationContextSnapshot: {
      source: 'image_slideshow',
      renderKind: 'template_motion',
      templateId: request.templateId,
      templateType: request.templateType,
      externalNetwork: false,
      billable: false,
    },
  }
  const state = ensureState(request.projectId)
  const item = state.items.find((stateItem) => stateItem.itemId === request.itemId) ?? createItemState(request)
  const nextItem: ImageSlideshowItemStateDto = {
    ...item,
    selectedImageId: request.inputImageId,
    selectedVideoSegmentId: segment.segmentId,
    readyForComposition: true,
    templateSegments: [...item.templateSegments, segment],
  }
  upsertItem(request.projectId, nextItem)
  return segment
}

export async function validateImageSlideshowSegments(
  request: ImageSlideshowProjectRequest,
): Promise<ImageSlideshowProjectStateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ImageSlideshowProjectStateDto, { request: ImageSlideshowProjectRequest }>(
      tauriCommands.validateImageSlideshowSegments,
      { request },
    )
  }

  const state = ensureState(request.projectId)
  const missing = state.items.filter((item) => !item.readyForComposition)
  if (missing.length > 0) throw new Error(`image_slideshow.segments_missing:${missing.map((item) => item.itemId).join(',')}`)
  const next = { ...state, segmentCompositionStatus: 'succeeded' }
  states.set(request.projectId, next)
  return next
}

function ensureState(projectId: string): ImageSlideshowProjectStateDto {
  const state = states.get(projectId) ?? {
    projectId,
    templateMotionStatus: 'waiting_user',
    segmentCompositionStatus: 'pending',
    items: [],
  }
  states.set(projectId, state)
  return state
}

function createItemState(request: RegisterTemplateMotionSegmentRequest): ImageSlideshowItemStateDto {
  return {
    itemId: request.itemId,
    projectId: request.projectId,
    selectedImageId: request.inputImageId,
    selectedVideoSegmentId: null,
    templateSegments: [],
    readyForComposition: false,
  }
}

function upsertItem(projectId: string, item: ImageSlideshowItemStateDto) {
  const state = ensureState(projectId)
  const items = state.items.filter((stateItem) => stateItem.itemId !== item.itemId)
  states.set(projectId, {
    ...state,
    templateMotionStatus: 'waiting_user',
    items: [...items, item],
  })
}
