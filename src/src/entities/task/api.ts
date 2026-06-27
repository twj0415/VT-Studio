import type { ListTasksRequest, RetryTaskStepRequest, TaskDetailDto, TaskProjectRequest, TaskSummaryDto } from './types'
import type { TaskStepKind } from '@/shared/enums/generated'
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
    taskStatus: 'running',
    currentStep: 'storyboard_review',
    steps: createMockPipelineSteps(),
  }
}

export async function createTask(projectId: string): Promise<TaskDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TaskDetailDto>(tauriCommands.createTask, { projectId })
  }

  return getTaskDetail(projectId)
}

export async function startTask(request: TaskProjectRequest): Promise<TaskDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TaskDetailDto, { request: TaskProjectRequest }>(tauriCommands.startTask, { request })
  }

  return { ...(await getTaskDetail(request.projectId)), taskStatus: 'running' }
}

export async function cancelTask(request: TaskProjectRequest): Promise<TaskDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TaskDetailDto, { request: TaskProjectRequest }>(tauriCommands.cancelTask, { request })
  }

  const detail = await getTaskDetail(request.projectId)
  return {
    ...detail,
    taskStatus: 'cancelled',
    currentStep: undefined,
    steps: detail.steps.map((step) => (step.status === 'succeeded' ? step : { ...step, status: 'cancelled' })),
  }
}

export async function resumeTask(request: TaskProjectRequest): Promise<TaskDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TaskDetailDto, { request: TaskProjectRequest }>(tauriCommands.resumeTask, { request })
  }

  return getTaskDetail(request.projectId)
}

export async function retryTaskStep(request: RetryTaskStepRequest): Promise<TaskDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TaskDetailDto, { request: RetryTaskStepRequest }>(tauriCommands.retryTaskStep, { request })
  }

  const detail = await getTaskDetail(request.projectId)
  return {
    ...detail,
    taskStatus: 'running',
    currentStep: request.stepName,
    steps: detail.steps.map((step) => (step.stepName === request.stepName ? { ...step, status: 'pending' } : step)),
  }
}

export async function listTasks(request: ListTasksRequest = {}): Promise<TaskSummaryDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TaskSummaryDto[], { request: ListTasksRequest }>(tauriCommands.listTasks, { request })
  }

  const projectId = request.projectId ?? 'draft'
  return [
    {
      taskId: 'task_draft',
      projectId,
      taskStatus: 'running',
      currentStep: 'storyboard_review',
      summary: '等待确认分镜',
      createdAt: '2026-06-22 10:00',
      updatedAt: '2026-06-22 10:00',
    },
  ]
}

export async function approveTaskStep(projectId: string, stepName: TaskStepKind): Promise<TaskDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TaskDetailDto>(tauriCommands.approveTaskStep, { projectId, stepName })
  }

  return {
    taskId: 'task_draft',
    projectId,
    taskStatus: stepName === 'cleanup' ? 'succeeded' : 'running',
    currentStep: nextStepName(stepName),
    steps: createMockPipelineSteps(stepName),
  }
}

function nextStepName(stepName: TaskStepKind): TaskStepKind | undefined {
  const index = imageToVideoPipelineSteps.indexOf(stepName)
  return index >= 0 ? imageToVideoPipelineSteps[index + 1] : undefined
}

const imageToVideoPipelineSteps: TaskStepKind[] = [
  'project_init',
  'storyboard_generation',
  'storyboard_review',
  'image_prompt_generation',
  'image_generation',
  'image_review',
  'video_prompt_generation',
  'video_generation',
  'video_review',
  'final_composition',
  'export',
  'cleanup',
] as const

function createMockPipelineSteps(approvedThrough?: TaskStepKind): TaskDetailDto['steps'] {
  const approvedIndex = approvedThrough ? imageToVideoPipelineSteps.indexOf(approvedThrough) : 1
  const currentStep = approvedThrough ? nextStepName(approvedThrough) : 'storyboard_review'

  return imageToVideoPipelineSteps.map((stepName, index) => {
    let status: TaskDetailDto['steps'][number]['status'] = 'pending'
    if (index <= approvedIndex) status = 'succeeded'
    if (stepName === currentStep && (stepName === 'storyboard_review' || stepName === 'image_review' || stepName === 'video_review')) status = 'waiting_user'
    if (!approvedThrough && stepName === 'storyboard_review') status = 'waiting_user'
    return {
      stepId: `step_${stepName}`,
      stepName,
      status,
    }
  })
}
