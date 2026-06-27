import { defineStore } from 'pinia'

import { approveStoryboard, clearHistoricalImageCandidates, clearHistoricalVideoSegments, createStoryboardDraftItem, generateSubtitles, getCompositionTask, getStoryboard, listConfirmedNarrations, listImageCandidates, listVideoSegments, probeStoryboardAudio, regenerateStoryboard, replaceStoryboardAudio, restorePreviousStoryboard, selectImageCandidate, selectVideoSegment, startComposition, startImageAssetGeneration, startImageGeneration, startTtsGeneration, startVideoGeneration, updateScene, updateStoryboardStructure, updateStoryboardSubtitles } from './api'
import { getTaskDetail } from '@/entities/task/api'
import type { CompositionTaskDto, StartCompositionRequest } from '@/entities/task/types'
import type { GeneratedImageAssetDto, GenerateSubtitlesRequest, GenerateSubtitlesResultDto, ImageCandidateDto, NarrationDto, ProbeStoryboardAudioRequest, RegenerateStoryboardRequest, ReplaceStoryboardAudioRequest, SceneDto, StartImageAssetGenerationRequest, StartTtsGenerationRequest, StoryboardDto, StoryboardItemDto, UpdateStoryboardSubtitlesRequest, VideoSegmentDto } from './types'
import { applyStoryboardDownstreamReset } from './reset'

export const useSceneStore = defineStore('scene', {
  state: () => ({
    narrations: [] as NarrationDto[],
    storyboard: null as StoryboardDto | null,
    imageCandidates: [] as ImageCandidateDto[],
    generatedImageAssets: [] as GeneratedImageAssetDto[],
    videoSegments: [] as VideoSegmentDto[],
    compositionTasks: [] as CompositionTaskDto[],
    currentCompositionTask: null as CompositionTaskDto | null,
    savedItemsById: {} as Record<string, StoryboardItemDto>,
    latestSubtitlesResult: null as GenerateSubtitlesResultDto | null,
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
      this.savedItemsById = createSavedItemsById(this.storyboard.items)
      await this.syncLatestCompositionTask(projectId)
      return this.storyboard
    },
    async saveScene(scene: SceneDto) {
      const previous = this.savedItemsById[scene.itemId] ?? this.storyboard?.items.find((item) => item.itemId === scene.itemId)
      const resetResult = previous ? applyStoryboardDownstreamReset(previous, scene) : null
      const updated = await updateScene(resetResult?.item ?? scene)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === updated.itemId ? updated : item))
        this.imageCandidates = this.storyboard.items.flatMap((item) => item.imageCandidates)
        this.videoSegments = this.storyboard.items.flatMap((item) => item.videoSegments)
        this.savedItemsById[updated.itemId] = cloneStoryboardItem(updated)
      }
      return updated
    },
    async approve(projectId: string) {
      this.storyboard = await approveStoryboard(projectId)
      this.savedItemsById = createSavedItemsById(this.storyboard.items)
      return this.storyboard
    },
    async saveStoryboardStructure(projectId: string, items: StoryboardItemDto[]) {
      this.storyboard = await updateStoryboardStructure(projectId, items)
      this.narrations = this.storyboard.confirmedNarrations
      this.savedItemsById = createSavedItemsById(this.storyboard.items)
      return this.storyboard
    },
    async regenerate(projectId: string, request: RegenerateStoryboardRequest) {
      this.storyboard = await regenerateStoryboard(projectId, request)
      this.narrations = this.storyboard.confirmedNarrations
      this.imageCandidates = await listImageCandidates(projectId)
      this.videoSegments = await listVideoSegments(projectId)
      this.savedItemsById = createSavedItemsById(this.storyboard.items)
      return this.storyboard
    },
    async restorePrevious(projectId: string) {
      this.storyboard = await restorePreviousStoryboard(projectId)
      this.narrations = this.storyboard.confirmedNarrations
      this.imageCandidates = await listImageCandidates(projectId)
      this.videoSegments = await listVideoSegments(projectId)
      this.savedItemsById = createSavedItemsById(this.storyboard.items)
      return this.storyboard
    },
    async createDraftItem(projectId: string, index: number, sourceText = '', narrationText = '') {
      return await createStoryboardDraftItem(projectId, index, sourceText, narrationText)
    },
    async generateImages(projectId: string, itemId: string, count = 4) {
      const candidates = await startImageGeneration(projectId, itemId, count)
      this.storyboard = await getStoryboard(projectId)
      this.narrations = this.storyboard.confirmedNarrations
      this.imageCandidates = this.storyboard.items.flatMap((item) => item.imageCandidates)
      this.videoSegments = this.storyboard.items.flatMap((item) => item.videoSegments)
      this.savedItemsById = createSavedItemsById(this.storyboard.items)
      return candidates
    },
    async generateImageAsset(request: StartImageAssetGenerationRequest) {
      const generated = await startImageAssetGeneration(request)
      this.generatedImageAssets = [...generated, ...this.generatedImageAssets]
      return generated
    },
    async selectImage(itemId: string, imageId: string) {
      const previous = this.savedItemsById[itemId] ?? this.storyboard?.items.find((item) => item.itemId === itemId)
      const selectedItem = await selectImageCandidate(itemId, imageId)
      const updatedItem = previous ? applyStoryboardDownstreamReset(previous, selectedItem).item : selectedItem
      if (updatedItem !== selectedItem) await updateScene(updatedItem)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === itemId ? updatedItem : item))
        this.imageCandidates = this.storyboard.items.flatMap((item) => item.imageCandidates)
        this.videoSegments = this.storyboard.items.flatMap((item) => item.videoSegments)
        this.savedItemsById[itemId] = cloneStoryboardItem(updatedItem)
      }
      return updatedItem
    },
    async clearOldImageCandidates(itemId: string) {
      const updatedItem = await clearHistoricalImageCandidates(itemId)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === itemId ? updatedItem : item))
        this.imageCandidates = this.storyboard.items.flatMap((item) => item.imageCandidates)
        this.savedItemsById[itemId] = cloneStoryboardItem(updatedItem)
      }
      return updatedItem
    },
    async generateTts(request: StartTtsGenerationRequest) {
      const updatedItem = await startTtsGeneration(request)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === updatedItem.itemId ? updatedItem : item))
        this.savedItemsById[updatedItem.itemId] = cloneStoryboardItem(updatedItem)
      }
      return updatedItem
    },
    async replaceAudio(request: ReplaceStoryboardAudioRequest) {
      const updatedItem = await replaceStoryboardAudio(request)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === updatedItem.itemId ? updatedItem : item))
        this.savedItemsById[updatedItem.itemId] = cloneStoryboardItem(updatedItem)
      }
      return updatedItem
    },
    async probeAudio(request: ProbeStoryboardAudioRequest) {
      const updatedItem = await probeStoryboardAudio(request)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === updatedItem.itemId ? updatedItem : item))
        this.savedItemsById[updatedItem.itemId] = cloneStoryboardItem(updatedItem)
      }
      return updatedItem
    },
    async generateSubtitles(request: GenerateSubtitlesRequest) {
      const result = await generateSubtitles(request)
      this.latestSubtitlesResult = result
      this.mergeSubtitleResult(result)
      return result
    },
    async updateSubtitles(request: UpdateStoryboardSubtitlesRequest) {
      const result = await updateStoryboardSubtitles(request)
      this.latestSubtitlesResult = result
      this.mergeSubtitleResult(result)
      return result
    },
    mergeSubtitleResult(result: GenerateSubtitlesResultDto) {
      if (!this.storyboard) return
      const updatedById = new Map(result.items.map((item) => [item.itemId, item]))
      this.storyboard.items = this.storyboard.items.map((item) => updatedById.get(item.itemId) ?? item)
      for (const item of result.items) {
        this.savedItemsById[item.itemId] = cloneStoryboardItem(item)
      }
    },
    async generateVideos(projectId: string, itemId: string) {
      const segments = await startVideoGeneration(projectId, itemId)
      this.storyboard = await getStoryboard(projectId)
      this.narrations = this.storyboard.confirmedNarrations
      this.imageCandidates = this.storyboard.items.flatMap((item) => item.imageCandidates)
      this.videoSegments = this.storyboard.items.flatMap((item) => item.videoSegments)
      this.savedItemsById = createSavedItemsById(this.storyboard.items)
      return segments
    },
    async selectVideo(itemId: string, segmentId: string) {
      const previous = this.savedItemsById[itemId] ?? this.storyboard?.items.find((item) => item.itemId === itemId)
      const selectedItem = await selectVideoSegment(itemId, segmentId)
      const updatedItem = previous ? applyStoryboardDownstreamReset(previous, selectedItem).item : selectedItem
      if (updatedItem !== selectedItem) await updateScene(updatedItem)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === itemId ? updatedItem : item))
        this.videoSegments = this.storyboard.items.flatMap((item) => item.videoSegments)
        this.savedItemsById[itemId] = cloneStoryboardItem(updatedItem)
      }
      return updatedItem
    },
    async clearOldVideoSegments(itemId: string) {
      const updatedItem = await clearHistoricalVideoSegments(itemId)
      if (this.storyboard) {
        this.storyboard.items = this.storyboard.items.map((item) => (item.itemId === itemId ? updatedItem : item))
        this.videoSegments = this.storyboard.items.flatMap((item) => item.videoSegments)
        this.savedItemsById[itemId] = cloneStoryboardItem(updatedItem)
      }
      return updatedItem
    },
    async startComposition(request: StartCompositionRequest) {
      this.currentCompositionTask = await startComposition(request)
      this.compositionTasks = [this.currentCompositionTask, ...this.compositionTasks.filter((task) => task.taskId !== this.currentCompositionTask?.taskId)]
      await this.loadStoryboard(request.projectId)
      return this.currentCompositionTask
    },
    async loadCompositionTask(taskId: string) {
      this.currentCompositionTask = await getCompositionTask(taskId)
      return this.currentCompositionTask
    },
    async syncLatestCompositionTask(projectId: string) {
      try {
        const task = await getTaskDetail(projectId)
        this.currentCompositionTask = task.compositionTask ?? null
        if (this.currentCompositionTask) {
          this.compositionTasks = [this.currentCompositionTask, ...this.compositionTasks.filter((entry) => entry.taskId !== this.currentCompositionTask?.taskId)]
        }
        return this.currentCompositionTask
      } catch {
        this.currentCompositionTask = null
        return null
      }
    },
  },
})

function createSavedItemsById(items: StoryboardItemDto[]): Record<string, StoryboardItemDto> {
  return Object.fromEntries(items.map((item) => [item.itemId, cloneStoryboardItem(item)]))
}

function cloneStoryboardItem(item: StoryboardItemDto): StoryboardItemDto {
  return {
    ...item,
    characters: [...item.characters],
    characterIds: [...item.characterIds],
    lockFlagsJson: { ...item.lockFlagsJson },
    imageLastErrorJson: item.imageLastErrorJson ? { ...item.imageLastErrorJson } : null,
    audioLastErrorJson: item.audioLastErrorJson ? { ...item.audioLastErrorJson } : null,
    imageCandidates: item.imageCandidates.map((candidate) => ({ ...candidate, generationContextSnapshot: { ...candidate.generationContextSnapshot } })),
    videoSegments: item.videoSegments.map((segment) => ({ ...segment, generationContextSnapshot: { ...segment.generationContextSnapshot } })),
    downstreamResetRecords: item.downstreamResetRecords?.map((record) => ({ ...record, affectedObjects: [...record.affectedObjects] })),
  }
}
