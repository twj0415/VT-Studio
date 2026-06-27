import type { TaskStatus, TaskStepKind, TaskStepStatus } from '@/shared/enums/generated'

export interface CompositionTaskDto {
  taskId: string
  projectId: string
  segmentIds: string[]
  outputPath: string
  enhancements: Record<string, unknown>
  status: TaskStatus
  progress: number
  errorJson: Record<string, unknown> | null
  createdAt: string
  updatedAt: string
}

export interface StartCompositionRequest {
  projectId: string
  includeSubtitle?: boolean
  subtitlePath?: string | null
  includeBgm?: boolean
  bgmAssetId?: string | null
  bgmVolume?: number
  bgmLoop?: boolean
  bgmFadeInSeconds?: number
  bgmFadeOutSeconds?: number
  includeCoverMetadata?: boolean
  coverPath?: string | null
}

export interface TaskStepDto {
  stepId: string
  stepName: TaskStepKind
  status: TaskStepStatus
  outputJson?: Record<string, unknown>
}

export interface TaskDetailDto {
  taskId: string
  projectId: string
  taskStatus: TaskStatus
  currentStep?: TaskStepKind
  steps: TaskStepDto[]
  compositionTask?: CompositionTaskDto
}

export interface TaskSummaryDto {
  taskId: string
  projectId: string
  taskStatus: TaskStatus
  currentStep?: TaskStepKind
  summary: string
  createdAt: string
  updatedAt: string
}

export interface TaskProjectRequest {
  projectId: string
}

export interface RetryTaskStepRequest {
  projectId: string
  stepName: TaskStepKind
}

export interface ListTasksRequest {
  projectId?: string
}
