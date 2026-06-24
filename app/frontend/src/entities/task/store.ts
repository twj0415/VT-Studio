import { defineStore } from 'pinia'

import { approveTaskStep, getTaskDetail } from './api'
import type { TaskDetailDto } from './types'

export const useTaskStore = defineStore('task', {
  state: () => ({
    currentTask: null as TaskDetailDto | null,
  }),
  actions: {
    async loadTask(projectId: string) {
      this.currentTask = await getTaskDetail(projectId)
      return this.currentTask
    },
    async approveStep(projectId: string, stepName: string) {
      this.currentTask = await approveTaskStep(projectId, stepName)
      return this.currentTask
    },
  },
})
