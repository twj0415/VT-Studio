import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'

import type { DigitalHumanProjectStateDto, StartDigitalHumanVideoRequest } from './types'

const states = new Map<string, DigitalHumanProjectStateDto>()

export async function getDigitalHumanProjectState(projectId: string): Promise<DigitalHumanProjectStateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<DigitalHumanProjectStateDto>(tauriCommands.getDigitalHumanProjectState, { projectId })
  }

  return ensureState(projectId)
}

export async function markDigitalHumanTtsSucceeded(projectId: string, referenceAudioPath: string): Promise<DigitalHumanProjectStateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<DigitalHumanProjectStateDto>(tauriCommands.markDigitalHumanTtsSucceeded, { projectId, referenceAudioPath })
  }

  const state = ensureState(projectId)
  const next = { ...state, ttsStatus: 'succeeded', referenceAudioPath }
  states.set(projectId, next)
  return next
}

export async function markDigitalHumanTtsFailed(projectId: string): Promise<DigitalHumanProjectStateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<DigitalHumanProjectStateDto>(tauriCommands.markDigitalHumanTtsFailed, { projectId, errorReason: 'TTS failed' })
  }

  const state = ensureState(projectId)
  const next = { ...state, ttsStatus: 'failed' }
  states.set(projectId, next)
  return next
}

export async function startDigitalHumanVideo(request: StartDigitalHumanVideoRequest): Promise<DigitalHumanProjectStateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<DigitalHumanProjectStateDto, { request: StartDigitalHumanVideoRequest }>(tauriCommands.startDigitalHumanVideo, { request })
  }

  const state = ensureState(request.projectId)
  if (state.ttsStatus !== 'succeeded') throw new Error('TTS must succeed before digital human video generation.')
  const next = {
    ...state,
    videoStatus: 'succeeded',
    referenceImagePath: request.referenceImagePath ?? null,
    outputVideoPath: `projects/${request.projectId}/digital-human/output.mp4`,
  }
  states.set(request.projectId, next)
  return next
}

function ensureState(projectId: string): DigitalHumanProjectStateDto {
  const state = states.get(projectId) ?? {
    projectId,
    ttsStatus: 'pending',
    videoStatus: 'pending',
    referenceImagePath: null,
    referenceAudioPath: null,
    outputVideoPath: null,
  }
  states.set(projectId, state)
  return state
}
