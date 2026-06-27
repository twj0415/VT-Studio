import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type { DigitalHumanProjectStateDto, StartDigitalHumanVideoRequest } from './types'

export function getDigitalHumanProjectState(projectId: string): Promise<DigitalHumanProjectStateDto> {
  return callCommand<DigitalHumanProjectStateDto>(tauriCommands.getDigitalHumanProjectState, { projectId })
}

export function markDigitalHumanTtsSucceeded(projectId: string, referenceAudioPath: string): Promise<DigitalHumanProjectStateDto> {
  return callCommand<DigitalHumanProjectStateDto>(tauriCommands.markDigitalHumanTtsSucceeded, { projectId, referenceAudioPath })
}

export function markDigitalHumanTtsFailed(projectId: string): Promise<DigitalHumanProjectStateDto> {
  return callCommand<DigitalHumanProjectStateDto>(tauriCommands.markDigitalHumanTtsFailed, { projectId, errorReason: 'TTS failed' })
}

export function startDigitalHumanVideo(request: StartDigitalHumanVideoRequest): Promise<DigitalHumanProjectStateDto> {
  return callCommand<DigitalHumanProjectStateDto, { request: StartDigitalHumanVideoRequest }>(tauriCommands.startDigitalHumanVideo, { request })
}
