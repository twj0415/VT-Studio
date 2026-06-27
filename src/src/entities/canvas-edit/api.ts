import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type { CanvasEditCandidateResultDto, CreateCanvasEditCandidateRequest } from './types'

export function createCanvasEditCandidate(request: CreateCanvasEditCandidateRequest): Promise<CanvasEditCandidateResultDto> {
  return callCommand<CanvasEditCandidateResultDto, { request: CreateCanvasEditCandidateRequest }>(tauriCommands.createCanvasEditCandidate, { request })
}
