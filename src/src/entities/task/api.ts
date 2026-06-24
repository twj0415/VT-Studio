import type { TaskDetailDto } from './types'
import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'

export async function getTaskDetail(projectId: string): Promise<TaskDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TaskDetailDto>(tauriCommands.getTaskDetail, { projectId })
  }

  return {
    taskId: 'task_draft',
    projectId,
    taskStatus: 'waiting_user',
    currentStep: 'storyboard_review',
    steps: [
      { stepId: 'step_storyboard_review', stepName: 'storyboard_review', status: 'waiting_user' },
      { stepId: 'step_image_review', stepName: 'image_review', status: 'pending' },
      { stepId: 'step_video_review', stepName: 'video_review', status: 'pending' },
      { stepId: 'step_final_composition', stepName: 'final_composition', status: 'pending' },
    ],
  }
}

export async function approveTaskStep(projectId: string, stepName: string): Promise<TaskDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TaskDetailDto>(tauriCommands.approveTaskStep, { projectId, stepName })
  }

  return {
    taskId: 'task_draft',
    projectId,
    taskStatus: stepName === 'final_composition' ? 'succeeded' : 'waiting_user',
    currentStep: nextStepName(stepName),
    steps: [
      { stepId: 'step_storyboard_review', stepName: 'storyboard_review', status: stepName === 'storyboard_review' ? 'succeeded' : 'waiting_user' },
      { stepId: 'step_image_review', stepName: 'image_review', status: stepName === 'image_review' ? 'succeeded' : stepName === 'storyboard_review' ? 'waiting_user' : 'pending' },
      { stepId: 'step_video_review', stepName: 'video_review', status: stepName === 'video_review' ? 'succeeded' : stepName === 'image_review' ? 'waiting_user' : 'pending' },
      { stepId: 'step_final_composition', stepName: 'final_composition', status: stepName === 'final_composition' ? 'succeeded' : stepName === 'video_review' ? 'waiting_user' : 'pending' },
    ],
  }
}

function nextStepName(stepName: string) {
  const next: Record<string, string | undefined> = {
    storyboard_review: 'image_review',
    image_review: 'video_review',
    video_review: 'final_composition',
    final_composition: undefined,
  }
  return next[stepName]
}
