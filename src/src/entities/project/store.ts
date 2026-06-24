import { defineStore } from 'pinia'

import { createProject, getProjectDetail, listProjects } from './api'
import type { CreateProjectRequest, ProjectDetailDto, ProjectSummaryDto } from './types'

export const useProjectStore = defineStore('project', {
  state: () => ({
    projects: [] as ProjectSummaryDto[],
    currentProject: null as ProjectDetailDto | null,
  }),
  actions: {
    async loadProjects() {
      const result = await listProjects({ page: 1, pageSize: 20, sortBy: 'updated_at', sortOrder: 'desc' })
      this.projects = result.items
    },
    async createDraftProject(request: CreateProjectRequest) {
      this.currentProject = await createProject(request)
      await this.loadProjects()
      return this.currentProject
    },
    async loadProject(projectId: string) {
      this.currentProject = await getProjectDetail(projectId)
      return this.currentProject
    },
  },
})
