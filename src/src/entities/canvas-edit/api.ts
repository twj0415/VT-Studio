import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'

import type { CanvasEditCandidateResultDto, CreateCanvasEditCandidateRequest } from './types'

export async function createCanvasEditCandidate(request: CreateCanvasEditCandidateRequest): Promise<CanvasEditCandidateResultDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CanvasEditCandidateResultDto, { request: CreateCanvasEditCandidateRequest }>(tauriCommands.createCanvasEditCandidate, { request })
  }

  throw new Error('Canvas edit requires a real workspace file and is only available in the Tauri runtime.')
}
