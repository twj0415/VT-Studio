import type { AspectRatio, ContentLanguage, InputProcessMode, InputType, ProjectLifecycle, TaskStatus, WorkflowType } from '@/shared/enums/generated'
import type { PageRequest } from '@/shared/types/generated'

export interface ProjectLatestTaskDto {
  taskId: string
  taskStatus: TaskStatus
  summary: string
}

export interface ProjectDto {
  projectId: string
  title: string
  workflowType: WorkflowType
  inputType: InputType
  inputProcessMode: InputProcessMode
  inputOptions?: Record<string, unknown>
  sourceText?: string
  sourceTextPath?: string
  aspectRatio: AspectRatio
  targetSceneCount: number
  segmentDurationSeconds: number
  stylePrompt?: string
  tone?: string
  contentLanguage: ContentLanguage
  lifecycle: ProjectLifecycle
  createdAt: string
  updatedAt: string
  latestTask?: ProjectLatestTaskDto
}

export type ProjectSummaryDto = ProjectDto

export interface ProjectBibleDto {
  projectId: string
  summary: string
}

export interface ProjectDetailDto {
  project: ProjectDto
  projectBible: ProjectBibleDto
  styleBible?: { styleId: string; name: string }
  characterBibles: Array<{ characterId: string; name: string }>
  locationBibles: Array<{ locationId: string; name: string }>
}

export interface ListProjectsRequest extends PageRequest {
  keyword?: string
  lifecycle?: ProjectLifecycle
  sortBy?: 'updated_at' | 'created_at' | 'title'
  sortOrder?: 'asc' | 'desc'
}

export interface CreateProjectRequest {
  title: string
  workflowType: WorkflowType
  inputType: InputType
  topic?: string
  sourceText?: string
  sourceTextPath?: string
  contentLanguage: ContentLanguage
  tone?: string
  aspectRatio: AspectRatio
  targetSceneCount: number
  segmentDurationSeconds: number
  stylePrompt?: string
  inputProcessMode: InputProcessMode
  inputOptions?: Record<string, unknown>
}
