import { defineStore } from 'pinia'

import { approveTaskStep, cancelTask, createTask, getTaskDetail, listTasks, resumeTask, retryTaskStep, startTask } from './api'
import type { ListTasksRequest, RetryTaskStepRequest, TaskDetailDto, TaskProjectRequest, TaskSummaryDto } from './types'
import { listenTaskProgress } from '@/shared/api/events'
import type { TaskStepKind } from '@/shared/enums/generated'

export const useTaskStore = defineStore('task', {
  state: () => ({
    currentTask: null as TaskDetailDto | null,
    tasks: [] as TaskSummaryDto[],
    stopTaskProgressListener: null as null | (() => void),
  }),
  actions: {
    async createTask(projectId: string) {
      this.currentTask = await createTask(projectId)
      return this.currentTask
    },
    async loadTask(projectId: string) {
      this.currentTask = await getTaskDetail(projectId)
      return this.currentTask
    },
    async loadTasks(request: ListTasksRequest = {}) {
      this.tasks = await listTasks(request)
      return this.tasks
    },
    async startTask(request: TaskProjectRequest) {
      this.currentTask = await startTask(request)
      return this.currentTask
    },
    async cancelTask(request: TaskProjectRequest) {
      this.currentTask = await cancelTask(request)
      return this.currentTask
    },
    async resumeTask(request: TaskProjectRequest) {
      this.currentTask = await resumeTask(request)
      return this.currentTask
    },
    async retryTaskStep(request: RetryTaskStepRequest) {
      this.currentTask = await retryTaskStep(request)
      return this.currentTask
    },
    async approveStep(projectId: string, stepName: TaskStepKind) {
      this.currentTask = await approveTaskStep(projectId, stepName)
      return this.currentTask
    },
    async subscribeTaskProgress() {
      if (this.stopTaskProgressListener) return
      this.stopTaskProgressListener = await listenTaskProgress((event) => {
        void this.refreshFromProgressEvent(event.projectId, event.taskId)
      })
    },
    stopTaskProgress() {
      this.stopTaskProgressListener?.()
      this.stopTaskProgressListener = null
    },
    async refreshFromProgressEvent(projectId: string, taskId: string) {
      if (this.currentTask?.taskId !== taskId && this.currentTask?.projectId !== projectId) return
      this.currentTask = await getTaskDetail(projectId)
    },
  },
})
