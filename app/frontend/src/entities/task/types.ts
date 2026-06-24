import type { TaskStatus } from '@/shared/enums/generated'

export interface CompositionTaskDto {
  taskId: string
  projectId: string
  segmentIds: string[]
  outputPath: string
  status: TaskStatus
  progress: number
  errorJson: Record<string, unknown> | null
  createdAt: string
  updatedAt: string
}

export interface TaskStepDto {
  stepId: string
  stepName: string
  status: TaskStatus
  outputJson?: Record<string, unknown>
}

export interface TaskDetailDto {
  taskId: string
  projectId: string
  taskStatus: TaskStatus
  currentStep?: string
  steps: TaskStepDto[]
  compositionTask?: CompositionTaskDto
}
