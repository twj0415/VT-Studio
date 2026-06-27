import { createMockId } from '@/shared/mock/ids'
import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'

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

const MOCK_NOW = '2026-06-22 10:00'
const states = new Map<string, MaterialEditProjectStateDto>()

export async function getMaterialEditProjectState(projectId: string): Promise<MaterialEditProjectStateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<MaterialEditProjectStateDto>(tauriCommands.getMaterialEditProjectState, { projectId })
  }

  return ensureState(projectId)
}

export async function saveMaterialAnalysisSuggestion(
  request: SaveMaterialAnalysisSuggestionRequest,
): Promise<MaterialAnalysisSuggestionDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<MaterialAnalysisSuggestionDto, { request: SaveMaterialAnalysisSuggestionRequest }>(
      tauriCommands.saveMaterialAnalysisSuggestion,
      { request },
    )
  }

  const state = ensureState(request.projectId)
  const suggestion: MaterialAnalysisSuggestionDto = {
    suggestionId: createMockId('material_suggestion'),
    projectId: request.projectId,
    assetId: request.assetId,
    providerId: request.providerId ?? null,
    modelId: request.modelId ?? null,
    suggestion: JSON.parse(request.rawOutput) as Record<string, unknown>,
    status: 'waiting_user',
    createdAt: MOCK_NOW,
    updatedAt: MOCK_NOW,
  }
  states.set(request.projectId, {
    ...state,
    analysisStatus: 'waiting_user',
    suggestions: [...state.suggestions, suggestion],
  })
  return suggestion
}

export async function approveMaterialAnalysisSuggestion(
  request: MaterialAnalysisSuggestionIdRequest,
): Promise<MaterialAnalysisSuggestionDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<MaterialAnalysisSuggestionDto, { request: MaterialAnalysisSuggestionIdRequest }>(
      tauriCommands.approveMaterialAnalysisSuggestion,
      { request },
    )
  }

  return updateMockSuggestion(request.suggestionId, 'approved')
}

export async function rejectMaterialAnalysisSuggestion(
  request: MaterialAnalysisSuggestionIdRequest,
): Promise<MaterialAnalysisSuggestionDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<MaterialAnalysisSuggestionDto, { request: MaterialAnalysisSuggestionIdRequest }>(
      tauriCommands.rejectMaterialAnalysisSuggestion,
      { request },
    )
  }

  return updateMockSuggestion(request.suggestionId, 'rejected')
}

export async function bindStoryboardMaterial(request: BindStoryboardMaterialRequest): Promise<StoryboardMaterialCoverageDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardMaterialCoverageDto, { request: BindStoryboardMaterialRequest }>(
      tauriCommands.bindStoryboardMaterial,
      { request },
    )
  }

  const state = ensureState(request.projectId)
  const coverage = state.coverage.find((item) => item.itemId === request.itemId) ?? createCoverage(request.projectId, request.itemId)
  const nextCoverage: StoryboardMaterialCoverageDto = {
    ...coverage,
    satisfied: true,
    requirement: {
      itemId: request.itemId,
      projectId: request.projectId,
      requirementStatus: 'needs_material',
      noMaterialReason: null,
      confirmedByUser: true,
      createdAt: MOCK_NOW,
      updatedAt: MOCK_NOW,
    },
    boundAssets: [
      ...coverage.boundAssets,
      {
        referenceId: createMockId('asset_ref'),
        assetId: request.assetId,
        ownerKind: 'storyboard_item',
        ownerId: request.itemId,
        usageKind: 'source_material',
        createdAt: MOCK_NOW,
      },
    ],
  }
  upsertCoverage(request.projectId, nextCoverage)
  return nextCoverage
}

export async function markStoryboardNoMaterialNeeded(
  request: MarkStoryboardNoMaterialRequest,
): Promise<StoryboardMaterialCoverageDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StoryboardMaterialCoverageDto, { request: MarkStoryboardNoMaterialRequest }>(
      tauriCommands.markStoryboardNoMaterialNeeded,
      { request },
    )
  }

  const coverage = createCoverage(request.projectId, request.itemId)
  const nextCoverage: StoryboardMaterialCoverageDto = {
    ...coverage,
    satisfied: true,
    requirement: {
      itemId: request.itemId,
      projectId: request.projectId,
      requirementStatus: 'no_material_needed',
      noMaterialReason: request.reason,
      confirmedByUser: true,
      createdAt: MOCK_NOW,
      updatedAt: MOCK_NOW,
    },
  }
  upsertCoverage(request.projectId, nextCoverage)
  return nextCoverage
}

export async function validateMaterialStoryboardCoverage(
  request: MaterialEditProjectRequest,
): Promise<MaterialEditProjectStateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<MaterialEditProjectStateDto, { request: MaterialEditProjectRequest }>(
      tauriCommands.validateMaterialStoryboardCoverage,
      { request },
    )
  }

  const state = ensureState(request.projectId)
  const missing = state.coverage.filter((coverage) => !coverage.satisfied)
  if (missing.length > 0) throw new Error(`material.coverage_missing:${missing.map((item) => item.itemId).join(',')}`)
  const next = { ...state, matchingStatus: 'succeeded' }
  states.set(request.projectId, next)
  return next
}

function ensureState(projectId: string): MaterialEditProjectStateDto {
  const state = states.get(projectId) ?? {
    projectId,
    importStatus: 'waiting_user',
    analysisStatus: 'waiting_user',
    matchingStatus: 'waiting_user',
    suggestions: [],
    coverage: [],
  }
  states.set(projectId, state)
  return state
}

function updateMockSuggestion(suggestionId: string, status: MaterialAnalysisSuggestionDto['status']): MaterialAnalysisSuggestionDto {
  for (const [projectId, state] of states.entries()) {
    const suggestionIndex = state.suggestions.findIndex((suggestion) => suggestion.suggestionId === suggestionId)
    if (suggestionIndex >= 0) {
      const suggestion = { ...state.suggestions[suggestionIndex], status, updatedAt: MOCK_NOW }
      const suggestions = [...state.suggestions]
      suggestions[suggestionIndex] = suggestion
      states.set(projectId, { ...state, suggestions })
      return suggestion
    }
  }
  throw new Error(`Material analysis suggestion not found: ${suggestionId}`)
}

function createCoverage(projectId: string, itemId: string): StoryboardMaterialCoverageDto {
  return {
    itemId,
    projectId,
    boundAssets: [],
    requirement: null,
    satisfied: false,
  }
}

function upsertCoverage(projectId: string, coverage: StoryboardMaterialCoverageDto) {
  const state = ensureState(projectId)
  const coverages = state.coverage.filter((item) => item.itemId !== coverage.itemId)
  states.set(projectId, { ...state, coverage: [...coverages, coverage] })
}
