import type { PageResult } from '@/shared/types/generated'
import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type { CreateProjectRequest, GenerateProjectCoverRequest, ListProjectsRequest, ProjectDetailDto, ProjectSummaryDto, ReplaceProjectCoverImageRequest, UpdateProjectLifecycleRequest, UpdateProjectRequest } from './types'

export function listProjects(request: ListProjectsRequest): Promise<PageResult<ProjectSummaryDto>> {
  return callCommand<PageResult<ProjectSummaryDto>>(tauriCommands.listProjects, { request })
}

export function createProject(request: CreateProjectRequest): Promise<ProjectDetailDto> {
  return callCommand<ProjectDetailDto>(tauriCommands.createProject, { request })
}

export function getProjectDetail(projectId: string): Promise<ProjectDetailDto> {
  return callCommand<ProjectDetailDto>(tauriCommands.getProjectDetail, { projectId })
}

export function updateProject(request: UpdateProjectRequest): Promise<ProjectDetailDto> {
  return callCommand<ProjectDetailDto, { request: UpdateProjectRequest }>(tauriCommands.updateProject, { request })
}

export function updateProjectLifecycle(request: UpdateProjectLifecycleRequest): Promise<ProjectDetailDto> {
  return callCommand<ProjectDetailDto, { request: UpdateProjectLifecycleRequest }>(tauriCommands.updateProjectLifecycle, { request })
}

export function generateProjectCover(request: GenerateProjectCoverRequest): Promise<ProjectDetailDto> {
  return callCommand<ProjectDetailDto, { request: GenerateProjectCoverRequest }>(tauriCommands.generateProjectCover, { request })
}

export function replaceProjectCoverImage(request: ReplaceProjectCoverImageRequest): Promise<ProjectDetailDto> {
  return callCommand<ProjectDetailDto, { request: ReplaceProjectCoverImageRequest }>(tauriCommands.replaceProjectCoverImage, { request })
}
