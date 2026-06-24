<template>
  <section class="view h-full min-w-0 overflow-hidden bg-page text-primary">
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      <header class="flex h-12 flex-none items-center border-b border-border bg-panel px-vt-3">
        <div class="flex w-[320px] flex-none items-center gap-vt-3">
          <button type="button" class="grid size-8 flex-none place-items-center rounded-vt-sm border border-border text-sm text-muted transition hover:border-border-strong hover:bg-card hover:text-primary" @click="router.push(`/projects/${projectId}/workspace/image`)">←</button>
          <div class="flex min-w-0 items-center gap-vt-2">
            <div class="min-w-0 truncate text-base font-semibold">{{ projectTitle }}</div>
            <span v-if="isMockVideoFlow" class="flex-none rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-0.5 text-[11px] font-medium text-accent">{{ t('videoGeneration.mockBadge') }}</span>
          </div>
        </div>
        <div class="flex min-w-0 flex-1 items-center justify-center">
          <WorkspaceStepBar class="hidden min-w-0 justify-center lg:flex" compact :project-id="projectId" current-step="video" :access="workspaceAccess" @blocked="handleBlockedStep" />
        </div>
        <div class="flex w-[460px] flex-none items-center justify-end gap-vt-2">
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="dirtyItemIds.size === 0 || isSavingAll" @click="handleSaveAll">{{ t('videoGeneration.saveAll') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isBulkGenerating" @click="handleGenerateMissing">{{ t('videoGeneration.generateMissing') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isBulkGenerating" @click="handleGenerateAll">{{ t('videoGeneration.generateAll') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm bg-accent px-vt-4 text-sm font-semibold text-accent-ink transition hover:brightness-110" @click="handleEnterComposition">{{ t('videoGeneration.enterComposition') }}</button>
        </div>
      </header>

      <main class="flex min-h-0 flex-1 flex-col overflow-hidden bg-page">
        <section class="flex min-h-0 flex-1 flex-col gap-vt-3 overflow-hidden p-vt-3">
          <div class="flex flex-none flex-wrap items-center gap-vt-2 rounded-vt-md border border-border bg-card px-vt-3 py-vt-2 text-xs">
            <span class="font-medium text-secondary">{{ t('videoGeneration.toolbarTitle') }}</span>
            <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('videoGeneration.rowCount', { count: storyboardItems.length }) }}</span>
            <span v-if="dirtyItemIds.size > 0" class="rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-1 text-accent">{{ t('videoGeneration.dirtyCount', { count: dirtyItemIds.size }) }}</span>
            <span class="ml-auto text-muted">{{ t('videoGeneration.boundaryHint') }}</span>
          </div>

          <div class="min-h-0 flex-1 overflow-auto rounded-vt-md border border-border bg-card">
            <table class="w-full min-w-[1640px] table-fixed border-separate border-spacing-0 text-left text-sm">
              <thead class="sticky top-0 z-10 bg-panel text-xs text-muted">
                <tr>
                  <th class="w-[56px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.index') }}</th>
                  <th class="w-[190px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.selectedImage') }}</th>
                  <th class="w-[240px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.source') }}</th>
                  <th class="w-[340px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.videoPrompt') }}</th>
                  <th class="w-[120px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.duration') }}</th>
                  <th class="w-[300px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.segments') }}</th>
                  <th class="w-[180px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.selectedSegment') }}</th>
                  <th class="w-[120px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.status') }}</th>
                  <th class="w-[170px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.actions') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in storyboardItems" :key="item.itemId" class="h-[132px] align-top transition hover:bg-card-hover/70" :class="dirtyItemIds.has(item.itemId) ? 'bg-accent-soft/40' : ''">
                  <td class="border-b border-border px-vt-2 py-vt-2 align-middle text-center">
                    <div class="mx-auto grid size-8 place-items-center rounded-vt-sm border border-border bg-page text-xs text-muted">#{{ item.index.toString().padStart(2, '0') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid h-[116px] grid-rows-[1fr_auto] gap-vt-1 rounded-vt-sm border border-border bg-page p-vt-2 text-xs">
                      <div class="grid place-items-center rounded-vt-sm border border-border text-[10px] font-semibold uppercase text-primary" :class="selectedImagePreviewClass(item)">{{ t('videoGeneration.imageMockShort') }}</div>
                      <div class="truncate text-muted">{{ selectedImage(item)?.imagePath ?? t('videoGeneration.noSelectedImage') }}</div>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="h-[116px] overflow-y-auto rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">
                      <div class="font-medium text-primary">{{ item.sourceText || t('videoGeneration.emptySource') }}</div>
                      <div v-if="item.narrationText && item.narrationText !== item.sourceText" class="mt-vt-2 border-t border-border pt-vt-2 text-muted">{{ item.narrationText }}</div>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <n-input :value="item.videoPrompt" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :placeholder="t('videoGeneration.placeholders.videoPrompt')" @update:value="updateItem(item, { videoPrompt: $event })" />
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <n-input-number class="inp compact-inp" size="small" :min="1" :max="30" :step="1" :value="item.durationSeconds" @update:value="updateItem(item, { durationSeconds: normalizeDuration($event) })" />
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div v-if="item.videoSegments.length > 0" class="grid h-[116px] grid-cols-2 gap-vt-1 overflow-y-auto">
                      <button v-for="segment in item.videoSegments" :key="segment.segmentId" type="button" class="grid min-h-[54px] grid-cols-[44px_1fr] items-center gap-vt-2 rounded-vt-sm border px-vt-2 py-vt-1 text-left text-[11px] leading-4 transition" :class="segmentButtonClass(item, segment)" @click="handleSelectSegment(item, segment)">
                        <span class="grid h-9 w-11 place-items-center rounded-vt-sm border border-border bg-card-hover text-[9px] font-semibold uppercase text-primary">{{ t('videoGeneration.segmentMockShort') }}</span>
                        <span class="min-w-0">
                          <span class="block truncate font-semibold">{{ shortId(segment.segmentId) }}</span>
                          <span class="block truncate text-muted">{{ t('videoGeneration.segmentDuration', { seconds: segment.durationSeconds }) }}</span>
                          <span v-if="isSegmentSelected(item, segment)" class="block text-accent">{{ t('videoGeneration.confirmedMark') }}</span>
                        </span>
                      </button>
                    </div>
                    <div v-else class="grid h-[116px] place-items-center rounded-vt-sm border border-border bg-page px-vt-2 text-center text-xs text-muted">{{ t('videoGeneration.noSegments') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="h-[116px] overflow-hidden rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">
                      <template v-if="selectedSegment(item)">
                        <div class="font-semibold text-primary">{{ shortId(selectedSegment(item)?.segmentId) }}</div>
                        <div class="mt-vt-1 line-clamp-4 text-muted">{{ selectedSegment(item)?.videoPath }}</div>
                      </template>
                      <template v-else>
                        <div class="grid h-full place-items-center text-center text-muted">{{ t('videoGeneration.notConfirmed') }}</div>
                      </template>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1 text-xs">
                      <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-vt-1 text-secondary">{{ t(`dict.sceneAssetStatus.${item.videoStatus}`) }}</span>
                      <span class="text-muted">{{ t('videoGeneration.segmentCount', { count: item.videoSegments.length }) }}</span>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1">
                      <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="generatingItemIds.has(item.itemId)" @click="handleGenerateItem(item)">{{ item.videoSegments.length > 0 ? t('videoGeneration.regenerateRow') : t('videoGeneration.generateRow') }}</n-button>
                      <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="savingItemId === item.itemId" :disabled="!dirtyItemIds.has(item.itemId)" @click="handleSaveItem(item)">{{ t('videoGeneration.saveRow') }}</n-button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>

            <div v-if="storyboardItems.length === 0" class="grid h-full min-h-80 place-items-center text-sm text-muted">{{ t('videoGeneration.empty') }}</div>
          </div>
        </section>
      </main>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { NButton, NInput, NInputNumber, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { useProjectStore } from '@/entities/project/store'
import { useStoryboardStore } from '@/entities/storyboard/store'
import type { ImageCandidateDto, StoryboardItemDto, VideoSegmentDto } from '@/entities/storyboard/types'
import { validateStoryboardItemsForComposition, type StoryboardCompositionEntryField } from '@/entities/storyboard/validation'
import WorkspaceStepBar from '@/features/workspace/WorkspaceStepBar.vue'
import { getRequiredWorkspaceStep, getWorkspaceStepAccess, getWorkspaceStepPath, type WorkspaceStepKey } from '@/features/workspace/steps'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const { t } = useI18n()
const projectStore = useProjectStore()
const storyboardStore = useStoryboardStore()
const projectId = String(route.params.projectId)

const dirtyItemIds = ref<Set<string>>(new Set())
const savingItemId = ref<string | null>(null)
const generatingItemIds = ref<Set<string>>(new Set())
const isSavingAll = ref(false)
const isBulkGenerating = ref(false)

const storyboardItems = computed(() => storyboardStore.storyboard?.items ?? [])
const workspaceAccess = computed(() => getWorkspaceStepAccess(storyboardItems.value, storyboardStore.storyboard?.reviewStatus))
const projectTitle = computed(() => (projectStore.currentProject?.project.projectId === projectId ? projectStore.currentProject.project.title : projectId))
const isMockVideoFlow = computed(() => storyboardItems.value.some((item) => item.videoPrompt.startsWith('MOCK') || item.videoSegments.some((segment) => segment.providerModelId.startsWith('mock'))))

onMounted(async () => {
  await Promise.all([projectStore.loadProject(projectId), storyboardStore.loadStoryboard(projectId)])

  if (!workspaceAccess.value.canEnterVideo) {
    message.warning(t('workspaceStepBar.blocked.video'))
    await router.replace(getWorkspaceStepPath(projectId, getRequiredWorkspaceStep('video', workspaceAccess.value)))
  }
})

function updateItem(item: StoryboardItemDto, patch: Partial<StoryboardItemDto>) {
  Object.assign(item, patch)
  markItemDirty(item.itemId)
}

function markItemDirty(itemId: string) {
  dirtyItemIds.value = new Set(dirtyItemIds.value).add(itemId)
}

function markItemClean(itemId: string) {
  const next = new Set(dirtyItemIds.value)
  next.delete(itemId)
  dirtyItemIds.value = next
}

async function handleSaveItem(item: StoryboardItemDto) {
  savingItemId.value = item.itemId
  try {
    await storyboardStore.saveScene(normalizeVideoItem(item))
    markItemClean(item.itemId)
    message.success(t('videoGeneration.saveSuccess'))
  } finally {
    savingItemId.value = null
  }
}

async function handleSaveAll() {
  if (dirtyItemIds.value.size === 0) return

  isSavingAll.value = true
  try {
    const dirtyIds = [...dirtyItemIds.value]
    for (const itemId of dirtyIds) {
      const item = storyboardItems.value.find((entry) => entry.itemId === itemId)
      if (item) await storyboardStore.saveScene(normalizeVideoItem(item))
    }
    dirtyItemIds.value = new Set()
    message.success(t('videoGeneration.saveAllSuccess'))
  } finally {
    isSavingAll.value = false
  }
}

async function handleGenerateItem(item: StoryboardItemDto) {
  if (!item.selectedImageId) {
    message.warning(t('videoGeneration.noSelectedImage'))
    return
  }

  await saveDirtyItemIfNeeded(item)
  setGenerating(item.itemId, true)
  try {
    await storyboardStore.generateVideos(projectId, item.itemId)
    message.success(t('videoGeneration.generateSuccess', { index: item.index }))
  } finally {
    setGenerating(item.itemId, false)
  }
}

async function handleGenerateAll() {
  await generateItems(storyboardItems.value)
}

async function handleGenerateMissing() {
  await generateItems(storyboardItems.value.filter((item) => item.videoSegments.length === 0))
}

async function generateItems(items: StoryboardItemDto[]) {
  if (items.length === 0) {
    message.info(t('videoGeneration.noMissingRows'))
    return
  }

  isBulkGenerating.value = true
  try {
    await handleSaveAll()
    for (const item of items) {
      if (!item.selectedImageId) continue
      setGenerating(item.itemId, true)
      await storyboardStore.generateVideos(projectId, item.itemId)
      setGenerating(item.itemId, false)
    }
    message.success(t('videoGeneration.bulkGenerateSuccess', { count: items.length }))
  } finally {
    generatingItemIds.value = new Set()
    isBulkGenerating.value = false
  }
}

async function handleSelectSegment(item: StoryboardItemDto, segment: VideoSegmentDto) {
  await storyboardStore.selectVideo(item.itemId, segment.segmentId)
  message.success(t('videoGeneration.selectSuccess', { index: item.index }))
}

async function handleEnterComposition() {
  if (dirtyItemIds.value.size > 0) await handleSaveAll()

  const issues = validateStoryboardItemsForComposition(storyboardItems.value)
  if (issues.length > 0) {
    showFirstCompositionIssue(issues[0])
    return
  }

  await router.push(`/projects/${projectId}/workspace/compose`)
}

async function saveDirtyItemIfNeeded(item: StoryboardItemDto) {
  if (dirtyItemIds.value.has(item.itemId)) await handleSaveItem(item)
}

function setGenerating(itemId: string, loading: boolean) {
  const next = new Set(generatingItemIds.value)
  if (loading) next.add(itemId)
  else next.delete(itemId)
  generatingItemIds.value = next
}

function selectedImage(item: StoryboardItemDto): ImageCandidateDto | null {
  return item.imageCandidates.find((candidate) => candidate.imageId === item.selectedImageId || candidate.selected) ?? null
}

function selectedImagePreviewClass(item: StoryboardItemDto) {
  const toneClass = selectedImage(item)?.generationContextSnapshot.visualToneClass
  if (typeof toneClass === 'string' && /^scene-preview-tone-[0-4]$/.test(toneClass)) return toneClass
  return 'scene-preview-tone-0'
}

function selectedSegment(item: StoryboardItemDto) {
  return item.videoSegments.find((segment) => segment.segmentId === item.selectedVideoSegmentId || segment.selected) ?? null
}

function isSegmentSelected(item: StoryboardItemDto, segment: VideoSegmentDto) {
  return item.selectedVideoSegmentId === segment.segmentId || segment.selected
}

function segmentButtonClass(item: StoryboardItemDto, segment: VideoSegmentDto) {
  return isSegmentSelected(item, segment) ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary hover:border-border-strong hover:text-primary'
}

function normalizeVideoItem(item: StoryboardItemDto): StoryboardItemDto {
  return {
    ...item,
    videoPrompt: item.videoPrompt.trim(),
    durationSeconds: normalizeDuration(item.durationSeconds),
  }
}

function normalizeDuration(value: number | null) {
  return Math.min(30, Math.max(1, Math.round(value ?? 3)))
}

function shortId(value?: string) {
  if (!value) return '-'
  return value.length > 12 ? value.slice(-12) : value
}

function showFirstCompositionIssue(issue: { index: number; fields: StoryboardCompositionEntryField[] }) {
  const fields = issue.fields.map((field) => t(`videoGeneration.validation.fields.${field}`)).join('、')
  message.error(t('videoGeneration.validation.enterCompositionBlocked', { index: issue.index, fields }))
}

function handleBlockedStep(step: WorkspaceStepKey) {
  message.warning(t(`workspaceStepBar.blocked.${step}`))
}
</script>
