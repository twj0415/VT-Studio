import { defineStore } from 'pinia'

import { approveStoryboard, createStoryboardDraftItem, getCompositionTask, getStoryboard, listConfirmedNarrations, listImageCandidates, listVideoSegments, regenerateStoryboard, restorePreviousStoryboard, selectImageCandidate, selectVideoSegment, startComposition, startImageGeneration, startVideoGeneration, updateScene, updateStoryboardStructure } from './api'
import type { CompositionTaskDto } from '@/entities/task/types'
import type { ImageCandidateDto, NarrationDto, RegenerateStoryboardRequest, SceneDto, StoryboardDto, StoryboardItemDto, VideoSegmentDto } from './types'

export const useSceneStore = defineStore('scene', {
  state: () => ({
    narrations: [] as NarrationDto[],
    storyboard: null as StoryboardDto | null,
    imageCandidates: [] as ImageCandidateDto[],
    videoSegments: [] as VideoSegmentDto[],
    compositionTasks: [] as CompositionTaskDto[],
    currentCompositionTask: null as CompositionTaskDto | null,
  }),
  actions: {
    async loadNarrations(projectId: string) {
      this.narrations = await listConfirmedNarrations(projectId)
    },
    async loadStoryboard(projectId: string) {
      this.storyboard = await getStoryboard(projectId)
      this.narrations = this.storyboard.confirmedNarrations
      this.imageCandidates = await listImageCandidates(projectId)
      this.videoSegments = await listVideoSegments(projectId)
      return this.storyboard
    },
    async saveScene(scene: SceneDto) {
      const updated = await updateScene(scene)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === updated.itemId ? updated : item))
      }
      return updated
    },
    async approve(projectId: string) {
      this.storyboard = await approveStoryboard(projectId)
      return this.storyboard
    },
    async saveStoryboardStructure(projectId: string, items: StoryboardItemDto[]) {
      this.storyboard = await updateStoryboardStructure(projectId, items)
      this.narrations = this.storyboard.confirmedNarrations
      return this.storyboard
    },
    async regenerate(projectId: string, request: RegenerateStoryboardRequest) {
      this.storyboard = await regenerateStoryboard(projectId, request)
      this.narrations = this.storyboard.confirmedNarrations
      this.imageCandidates = await listImageCandidates(projectId)
      this.videoSegments = await listVideoSegments(projectId)
      return this.storyboard
    },
    async restorePrevious(projectId: string) {
      this.storyboard = await restorePreviousStoryboard(projectId)
      this.narrations = this.storyboard.confirmedNarrations
      this.imageCandidates = await listImageCandidates(projectId)
      this.videoSegments = await listVideoSegments(projectId)
      return this.storyboard
    },
    async createDraftItem(projectId: string, index: number, sourceText = '', narrationText = '') {
      return await createStoryboardDraftItem(projectId, index, sourceText, narrationText)
    },
    async generateImages(projectId: string, itemId: string, count = 4) {
      const candidates = await startImageGeneration(projectId, itemId, count)
      this.imageCandidates = await listImageCandidates(projectId)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === itemId ? { ...item, imageCandidates: candidates } : item))
      }
      return candidates
    },
    async selectImage(itemId: string, imageId: string) {
      const updatedItem = await selectImageCandidate(itemId, imageId)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === itemId ? updatedItem : item))
        this.imageCandidates = this.storyboard.items.flatMap((item) => item.imageCandidates)
      }
      return updatedItem
    },
    async generateVideos(projectId: string, itemId: string) {
      const segments = await startVideoGeneration(projectId, itemId)
      this.videoSegments = await listVideoSegments(projectId)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === itemId ? { ...item, videoSegments: segments } : item))
      }
      return segments
    },
    async selectVideo(itemId: string, segmentId: string) {
      const updatedItem = await selectVideoSegment(itemId, segmentId)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === itemId ? updatedItem : item))
        this.videoSegments = this.storyboard.items.flatMap((item) => item.videoSegments)
      }
      return updatedItem
    },
    async startComposition(projectId: string) {
      this.currentCompositionTask = await startComposition(projectId)
      this.compositionTasks = [this.currentCompositionTask, ...this.compositionTasks.filter((task) => task.taskId !== this.currentCompositionTask?.taskId)]
      return this.currentCompositionTask
    },
    async loadCompositionTask(taskId: string) {
      this.currentCompositionTask = await getCompositionTask(taskId)
      return this.currentCompositionTask
    },
  },
})
