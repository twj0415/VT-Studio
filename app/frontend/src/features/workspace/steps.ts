import {
  validateStoryboardItemsForComposition,
  validateStoryboardItemsForImageGeneration,
  validateStoryboardItemsForVideoGeneration,
} from '@/entities/storyboard/validation'
import type { StoryboardDto, StoryboardItemDto } from '@/entities/storyboard/types'

export type WorkspaceStepKey = 'storyboard' | 'image' | 'video' | 'composition'

export interface WorkspaceStepAccess {
  canEnterImage: boolean
  canEnterVideo: boolean
  canEnterComposition: boolean
}

export const workspaceStepKeys: WorkspaceStepKey[] = ['storyboard', 'image', 'video', 'composition']

export function getWorkspaceStepPath(projectId: string, step: WorkspaceStepKey) {
  const routeSegment: Record<WorkspaceStepKey, string> = {
    storyboard: 'storyboard',
    image: 'image',
    video: 'video',
    composition: 'compose',
  }

  return `/projects/${projectId}/workspace/${routeSegment[step]}`
}

export function getWorkspaceStepAccess(items: StoryboardItemDto[], reviewStatus?: StoryboardDto['reviewStatus']): WorkspaceStepAccess {
  const canEnterImage = reviewStatus === 'succeeded' && items.length > 0 && validateStoryboardItemsForImageGeneration(items).length === 0
  const canEnterVideo = canEnterImage && validateStoryboardItemsForVideoGeneration(items).length === 0
  const canEnterComposition = canEnterVideo && validateStoryboardItemsForComposition(items).length === 0

  return {
    canEnterImage,
    canEnterVideo,
    canEnterComposition,
  }
}

export function canEnterWorkspaceStep(step: WorkspaceStepKey, access: WorkspaceStepAccess) {
  if (step === 'storyboard') return true
  if (step === 'image') return access.canEnterImage
  if (step === 'video') return access.canEnterVideo
  return access.canEnterComposition
}

export function getRequiredWorkspaceStep(step: WorkspaceStepKey, access: WorkspaceStepAccess): WorkspaceStepKey {
  if (step === 'storyboard' || canEnterWorkspaceStep(step, access)) return step
  if (step === 'image') return 'storyboard'
  if (step === 'video') return access.canEnterImage ? 'image' : 'storyboard'
  if (!access.canEnterImage) return 'storyboard'
  if (!access.canEnterVideo) return 'image'
  return 'video'
}
