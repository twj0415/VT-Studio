import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type { VideoSegmentDto } from '@/entities/storyboard/types'
import type {
  ImageSlideshowProjectRequest,
  ImageSlideshowProjectStateDto,
  RegisterTemplateMotionSegmentRequest,
} from './types'

export function getImageSlideshowProjectState(projectId: string): Promise<ImageSlideshowProjectStateDto> {
  return callCommand<ImageSlideshowProjectStateDto>(tauriCommands.getImageSlideshowProjectState, { projectId })
}

export function registerTemplateMotionSegment(
  request: RegisterTemplateMotionSegmentRequest,
): Promise<VideoSegmentDto> {
  return callCommand<VideoSegmentDto, { request: RegisterTemplateMotionSegmentRequest }>(
    tauriCommands.registerTemplateMotionSegment,
    { request },
  )
}

export function validateImageSlideshowSegments(
  request: ImageSlideshowProjectRequest,
): Promise<ImageSlideshowProjectStateDto> {
  return callCommand<ImageSlideshowProjectStateDto, { request: ImageSlideshowProjectRequest }>(
    tauriCommands.validateImageSlideshowSegments,
    { request },
  )
}
