import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import type { TaskStepKind } from '@/shared/enums/generated'

import type { ListTasksRequest, RetryTaskStepRequest, TaskDetailDto, TaskProjectRequest, TaskSummaryDto } from './types'

export function getTaskDetail(projectId: string): Promise<TaskDetailDto> {
  return callCommand<TaskDetailDto>(tauriCommands.getTaskDetail, { projectId })
}

export function createTask(projectId: string): Promise<TaskDetailDto> {
  return callCommand<TaskDetailDto>(tauriCommands.createTask, { projectId })
}

export function startTask(request: TaskProjectRequest): Promise<TaskDetailDto> {
  return callCommand<TaskDetailDto, { request: TaskProjectRequest }>(tauriCommands.startTask, { request })
}

export function cancelTask(request: TaskProjectRequest): Promise<TaskDetailDto> {
  return callCommand<TaskDetailDto, { request: TaskProjectRequest }>(tauriCommands.cancelTask, { request })
}

export function resumeTask(request: TaskProjectRequest): Promise<TaskDetailDto> {
  return callCommand<TaskDetailDto, { request: TaskProjectRequest }>(tauriCommands.resumeTask, { request })
}

export function retryTaskStep(request: RetryTaskStepRequest): Promise<TaskDetailDto> {
  return callCommand<TaskDetailDto, { request: RetryTaskStepRequest }>(tauriCommands.retryTaskStep, { request })
}

export function listTasks(request: ListTasksRequest = {}): Promise<TaskSummaryDto[]> {
  return callCommand<TaskSummaryDto[], { request: ListTasksRequest }>(tauriCommands.listTasks, { request })
}

export function approveTaskStep(projectId: string, stepName: TaskStepKind): Promise<TaskDetailDto> {
  return callCommand<TaskDetailDto>(tauriCommands.approveTaskStep, { projectId, stepName })
}
