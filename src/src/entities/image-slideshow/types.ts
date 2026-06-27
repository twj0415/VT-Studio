import type { VideoSegmentDto } from '@/entities/storyboard/types'

export interface ImageSlideshowProjectStateDto {
  projectId: string
  templateMotionStatus: string
  segmentCompositionStatus: string
  items: ImageSlideshowItemStateDto[]
}

export interface ImageSlideshowItemStateDto {
  itemId: string
  projectId: string
  selectedImageId: string | null
  selectedVideoSegmentId: string | null
  templateSegments: VideoSegmentDto[]
  readyForComposition: boolean
}

export interface RegisterTemplateMotionSegmentRequest {
  projectId: string
  itemId: string
  inputImageId: string
  videoPath: string
  durationSeconds: number
  templateId: string
  templateType: string
  workflowPresetId?: string | null
}

export interface ImageSlideshowProjectRequest {
  projectId: string
}
