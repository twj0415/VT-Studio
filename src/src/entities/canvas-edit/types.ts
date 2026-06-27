import type { ImageCandidateDto } from '@/entities/scene/types'

export interface CreateCanvasEditCandidateRequest {
  projectId: string
  sourceImageId: string
  editedImagePath: string
  editFlowPath: string
  prompt?: string
  negativePrompt?: string
  providerModelId?: string
  workflowPresetId?: string
  editKind?: string
  flowSnapshot?: Record<string, unknown> | unknown[]
  selectAfterCreate?: boolean
}

export interface CanvasEditCandidateResultDto {
  candidate: ImageCandidateDto
  selectedItemId?: string | null
}
