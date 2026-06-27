import type { AspectRatio, ContentLanguage, InputProcessMode, InputType, ProjectLifecycle, TaskStatus, WorkflowType } from '@/shared/enums/generated'
import type { PageRequest } from '@/shared/types/generated'
import type { CreativeRuleRefDto } from '@/entities/config/types'

export interface NamedProjectAssetDto {
  id: string
  styleId?: string | null
  characterId?: string | null
  locationId?: string | null
  name: string
}

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
  activePackId?: string
  ruleRefs: Record<string, CreativeRuleRefDto>
  executableRefs: Record<string, unknown>
  coverPath?: string | null
  coverTitle?: string | null
  coverTemplateId?: string | null
  coverSourceItemId?: string | null
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
  styleBible?: NamedProjectAssetDto | null
  characterBibles: NamedProjectAssetDto[]
  locationBibles: NamedProjectAssetDto[]
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
  activePackId?: string
  ruleRefs?: Record<string, CreativeRuleRefDto>
  executableRefs?: Record<string, unknown>
  inputProcessMode: InputProcessMode
  inputOptions?: Record<string, unknown>
}

export interface UpdateProjectRequest {
  projectId: string
  patch: Partial<Pick<ProjectDto, 'title' | 'sourceText' | 'sourceTextPath' | 'inputOptions' | 'aspectRatio' | 'targetSceneCount' | 'segmentDurationSeconds' | 'stylePrompt' | 'activePackId' | 'ruleRefs' | 'executableRefs' | 'coverTitle' | 'tone' | 'contentLanguage'> & {
    topic: string
    contentCategory: string
  }>
}

export interface UpdateProjectLifecycleRequest {
  projectId: string
  lifecycle: ProjectLifecycle
}

export interface GenerateProjectCoverRequest {
  projectId: string
  coverTitle?: string
  coverTemplateId?: string
  coverSourceItemId?: string
  sourceImagePath?: string
}

export interface ReplaceProjectCoverImageRequest {
  projectId: string
  sourcePath: string
  coverTitle?: string
  coverTemplateId?: string
}
