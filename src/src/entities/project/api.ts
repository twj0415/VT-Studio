import type { PageResult } from '@/shared/types/generated'
import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'
import { createMockId } from '@/shared/mock/ids'

import type { CreateProjectRequest, GenerateProjectCoverRequest, ListProjectsRequest, ProjectDetailDto, ProjectDto, ProjectSummaryDto, ReplaceProjectCoverImageRequest } from './types'

const INLINE_SOURCE_TEXT_LIMIT_BYTES = 20 * 1024
const LONG_SOURCE_TEXT_PATH = 'input/source.txt'
const MOCK_NOW = '2026-06-22 10:00'

const projects: ProjectSummaryDto[] = [
  {
    projectId: 'draft',
    title: '早睡改变清醒感',
    workflowType: 'image_to_video',
    inputType: 'topic',
    inputProcessMode: 'generate',
    inputOptions: {},
    sourceText: '为什么要早睡',
    aspectRatio: '9:16',
    targetSceneCount: 8,
    segmentDurationSeconds: 4,
    stylePrompt: '干净、真实、柔和自然光',
    activePackId: 'pack_knowledge_short',
    ruleRefs: {},
    executableRefs: {},
    contentLanguage: 'zh-CN',
    lifecycle: 'draft',
    createdAt: MOCK_NOW,
    updatedAt: MOCK_NOW,
    latestTask: {
      taskId: 'task_draft',
      taskStatus: 'running',
      summary: '等待确认分镜',
    },
  },
]

export async function listProjects(request: ListProjectsRequest): Promise<PageResult<ProjectSummaryDto>> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<PageResult<ProjectSummaryDto>>(tauriCommands.listProjects, { request })
  }

  const keyword = request.keyword?.trim()
  const filtered = keyword ? projects.filter((project) => project.title.includes(keyword)) : projects

  return {
    items: filtered,
    total: filtered.length,
    page: request.page,
    pageSize: request.pageSize,
  }
}

export async function createProject(request: CreateProjectRequest): Promise<ProjectDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProjectDetailDto>(tauriCommands.createProject, { request })
  }

  const projectId = createMockId('project')
  const inputProcessMode = request.inputType === 'paste' ? 'fixed' : request.inputProcessMode
  const { sourceText, sourceTextPath } = resolveSourceTextFields(request)
  const project: ProjectDto = {
    projectId,
    title: request.title || defaultTitle(request),
    workflowType: request.workflowType,
    inputType: request.inputType,
    inputProcessMode,
    inputOptions: request.inputOptions ?? {},
    sourceText,
    sourceTextPath,
    aspectRatio: request.aspectRatio,
    targetSceneCount: request.targetSceneCount,
    segmentDurationSeconds: request.segmentDurationSeconds,
    stylePrompt: request.stylePrompt,
    activePackId: request.activePackId,
    ruleRefs: request.ruleRefs ?? {},
    executableRefs: request.executableRefs ?? {},
    contentLanguage: request.contentLanguage,
    lifecycle: 'draft',
    createdAt: MOCK_NOW,
    updatedAt: MOCK_NOW,
    latestTask: {
      taskId: createMockId('task'),
      taskStatus: 'running',
      summary: '等待确认分镜',
    },
  }

  projects.unshift(project)
  return toProjectDetail(project)
}

export async function getProjectDetail(projectId: string): Promise<ProjectDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProjectDetailDto>(tauriCommands.getProjectDetail, { projectId })
  }

  const project = projects.find((item) => item.projectId === projectId) ?? projects[0]
  return toProjectDetail(project)
}

export async function generateProjectCover(request: GenerateProjectCoverRequest): Promise<ProjectDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProjectDetailDto, { request: GenerateProjectCoverRequest }>(tauriCommands.generateProjectCover, { request })
  }

  const project = projects.find((item) => item.projectId === request.projectId)
  if (!project) throw new Error(`Project not found: ${request.projectId}`)
  const coverTitle = normalizeCoverTitle(request.coverTitle || project.title)
  project.coverPath = `projects/${request.projectId}/cover/cover.png`
  project.coverTitle = coverTitle
  project.coverTemplateId = request.coverTemplateId || project.coverTemplateId || 'knowledge_bold'
  project.coverSourceItemId = request.coverSourceItemId || null
  project.updatedAt = MOCK_NOW
  return toProjectDetail(project)
}

export async function replaceProjectCoverImage(request: ReplaceProjectCoverImageRequest): Promise<ProjectDetailDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProjectDetailDto, { request: ReplaceProjectCoverImageRequest }>(tauriCommands.replaceProjectCoverImage, { request })
  }

  return generateProjectCover({
    projectId: request.projectId,
    coverTitle: request.coverTitle,
    coverTemplateId: request.coverTemplateId,
    sourceImagePath: `projects/${request.projectId}/cover/source.png`,
  })
}

function toProjectDetail(project: ProjectSummaryDto): ProjectDetailDto {
  return {
    project,
    projectBible: {
      projectId: project.projectId,
      summary: '第一阶段默认作品设定集',
    },
    styleBible: { id: 'style_default', styleId: 'style_default', name: '默认画风' },
    characterBibles: [],
    locationBibles: [],
  }
}

function normalizeCoverTitle(value: string) {
  const title = value.trim() || '未命名封面'
  return [...title].slice(0, 15).join('')
}

function defaultTitle(request: CreateProjectRequest) {
  if (request.inputType === 'topic' && request.topic) return request.topic.slice(0, 15)
  if (request.inputType === 'paste') return '固定文案作品'
  if (request.inputType === 'article') return '文章整理作品'
  return '新作品'
}

function resolveSourceTextFields(request: CreateProjectRequest): Pick<ProjectDto, 'sourceText' | 'sourceTextPath'> {
  const inlineText = request.sourceText ?? request.topic
  if (!inlineText) return { sourceText: undefined, sourceTextPath: request.sourceTextPath }

  if (request.sourceTextPath || getUtf8ByteLength(inlineText) > INLINE_SOURCE_TEXT_LIMIT_BYTES) {
    return {
      sourceText: undefined,
      sourceTextPath: request.sourceTextPath ?? LONG_SOURCE_TEXT_PATH,
    }
  }

  return {
    sourceText: inlineText,
    sourceTextPath: undefined,
  }
}

function getUtf8ByteLength(value: string) {
  return new TextEncoder().encode(value).length
}
