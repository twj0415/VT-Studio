import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type {
  BindStoryboardMaterialRequest,
  MarkStoryboardNoMaterialRequest,
  MaterialAnalysisSuggestionDto,
  MaterialAnalysisSuggestionIdRequest,
  MaterialEditProjectRequest,
  MaterialEditProjectStateDto,
  SaveMaterialAnalysisSuggestionRequest,
  StoryboardMaterialCoverageDto,
} from './types'

export function getMaterialEditProjectState(projectId: string): Promise<MaterialEditProjectStateDto> {
  return callCommand<MaterialEditProjectStateDto>(tauriCommands.getMaterialEditProjectState, { projectId })
}

export function saveMaterialAnalysisSuggestion(
  request: SaveMaterialAnalysisSuggestionRequest,
): Promise<MaterialAnalysisSuggestionDto> {
  return callCommand<MaterialAnalysisSuggestionDto, { request: SaveMaterialAnalysisSuggestionRequest }>(
    tauriCommands.saveMaterialAnalysisSuggestion,
    { request },
  )
}

export function approveMaterialAnalysisSuggestion(
  request: MaterialAnalysisSuggestionIdRequest,
): Promise<MaterialAnalysisSuggestionDto> {
  return callCommand<MaterialAnalysisSuggestionDto, { request: MaterialAnalysisSuggestionIdRequest }>(
    tauriCommands.approveMaterialAnalysisSuggestion,
    { request },
  )
}

export function rejectMaterialAnalysisSuggestion(
  request: MaterialAnalysisSuggestionIdRequest,
): Promise<MaterialAnalysisSuggestionDto> {
  return callCommand<MaterialAnalysisSuggestionDto, { request: MaterialAnalysisSuggestionIdRequest }>(
    tauriCommands.rejectMaterialAnalysisSuggestion,
    { request },
  )
}

export function bindStoryboardMaterial(request: BindStoryboardMaterialRequest): Promise<StoryboardMaterialCoverageDto> {
  return callCommand<StoryboardMaterialCoverageDto, { request: BindStoryboardMaterialRequest }>(
    tauriCommands.bindStoryboardMaterial,
    { request },
  )
}

export function markStoryboardNoMaterialNeeded(
  request: MarkStoryboardNoMaterialRequest,
): Promise<StoryboardMaterialCoverageDto> {
  return callCommand<StoryboardMaterialCoverageDto, { request: MarkStoryboardNoMaterialRequest }>(
    tauriCommands.markStoryboardNoMaterialNeeded,
    { request },
  )
}

export function validateMaterialStoryboardCoverage(
  request: MaterialEditProjectRequest,
): Promise<MaterialEditProjectStateDto> {
  return callCommand<MaterialEditProjectStateDto, { request: MaterialEditProjectRequest }>(
    tauriCommands.validateMaterialStoryboardCoverage,
    { request },
  )
}
