import type { AssetReferenceDto } from '@/entities/config/types'

export interface MaterialAnalysisSuggestionDto {
  suggestionId: string
  projectId: string
  assetId: string
  providerId: string | null
  modelId: string | null
  suggestion: Record<string, unknown>
  status: 'waiting_user' | 'approved' | 'rejected' | string
  createdAt: string
  updatedAt: string
}

export interface StoryboardMaterialRequirementDto {
  itemId: string
  projectId: string
  requirementStatus: 'needs_material' | 'no_material_needed' | string
  noMaterialReason: string | null
  confirmedByUser: boolean
  createdAt: string
  updatedAt: string
}

export interface StoryboardMaterialCoverageDto {
  itemId: string
  projectId: string
  boundAssets: AssetReferenceDto[]
  requirement: StoryboardMaterialRequirementDto | null
  satisfied: boolean
}

export interface MaterialEditProjectStateDto {
  projectId: string
  importStatus: string
  analysisStatus: string
  matchingStatus: string
  suggestions: MaterialAnalysisSuggestionDto[]
  coverage: StoryboardMaterialCoverageDto[]
}

export interface SaveMaterialAnalysisSuggestionRequest {
  projectId: string
  assetId: string
  providerId?: string | null
  modelId?: string | null
  rawOutput: string
}

export interface MaterialAnalysisSuggestionIdRequest {
  suggestionId: string
}

export interface BindStoryboardMaterialRequest {
  projectId: string
  itemId: string
  assetId: string
}

export interface MarkStoryboardNoMaterialRequest {
  projectId: string
  itemId: string
  reason: string
}

export interface MaterialEditProjectRequest {
  projectId: string
}
