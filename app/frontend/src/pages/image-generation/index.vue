<template>
  <section class="view h-full min-w-0 overflow-hidden bg-page text-primary">
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      <header class="flex h-12 flex-none items-center border-b border-border bg-panel px-vt-3">
        <div class="flex w-[320px] flex-none items-center gap-vt-3">
          <button type="button" class="grid size-8 flex-none place-items-center rounded-vt-sm border border-border text-sm text-muted transition hover:border-border-strong hover:bg-card hover:text-primary" @click="router.push(`/projects/${projectId}/workspace/storyboard`)">←</button>
          <div class="flex min-w-0 items-center gap-vt-2">
            <div class="min-w-0 truncate text-base font-semibold">{{ projectTitle }}</div>
            <span v-if="isMockImageFlow" class="flex-none rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-0.5 text-[11px] font-medium text-accent">{{ t('imageGeneration.mockBadge') }}</span>
          </div>
        </div>
        <div class="flex min-w-0 flex-1 items-center justify-center">
          <WorkspaceStepBar class="hidden min-w-0 justify-center lg:flex" compact :project-id="projectId" current-step="image" :access="workspaceAccess" @blocked="handleBlockedStep" />
        </div>
        <div class="flex w-[460px] flex-none items-center justify-end gap-vt-2">
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="dirtyItemIds.size === 0 || isSavingAll" @click="handleSaveAll">{{ t('imageGeneration.saveAll') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isBulkGenerating" @click="handleGenerateMissing">{{ t('imageGeneration.generateMissing') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isBulkGenerating" @click="handleGenerateAll">{{ t('imageGeneration.generateAll') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm bg-accent px-vt-4 text-sm font-semibold text-accent-ink transition hover:brightness-110" @click="handleEnterVideo">{{ t('imageGeneration.enterVideo') }}</button>
        </div>
      </header>

      <main class="flex min-h-0 flex-1 flex-col overflow-hidden bg-page">
        <section class="flex min-h-0 flex-1 flex-col gap-vt-3 overflow-hidden p-vt-3">
          <div class="flex flex-none flex-wrap items-center gap-vt-2 rounded-vt-md border border-border bg-card px-vt-3 py-vt-2 text-xs">
            <span class="font-medium text-secondary">{{ t('imageGeneration.toolbarTitle') }}</span>
            <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('imageGeneration.rowCount', { count: storyboardItems.length }) }}</span>
            <span v-if="dirtyItemIds.size > 0" class="rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-1 text-accent">{{ t('imageGeneration.dirtyCount', { count: dirtyItemIds.size }) }}</span>
            <span class="ml-auto text-muted">{{ t('imageGeneration.boundaryHint') }}</span>
          </div>

          <div class="min-h-0 flex-1 overflow-auto rounded-vt-md border border-border bg-card">
            <table class="w-full min-w-[2140px] table-fixed border-separate border-spacing-0 text-left text-sm">
              <thead class="sticky top-0 z-10 bg-panel text-xs text-muted">
                <tr>
                  <th class="w-[56px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.index') }}</th>
                  <th class="w-[240px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.source') }}</th>
                  <th class="w-[190px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.intent') }}</th>
                  <th class="w-[170px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.characters') }}</th>
                  <th class="w-[180px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.scene') }}</th>
                  <th class="w-[260px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.visual') }}</th>
                  <th class="w-[300px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.prompt') }}</th>
                  <th class="w-[220px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.negativePrompt') }}</th>
                  <th class="w-[260px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.candidates') }}</th>
                  <th class="w-[160px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.selected') }}</th>
                  <th class="w-[120px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.status') }}</th>
                  <th class="w-[170px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.actions') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in storyboardItems" :key="item.itemId" class="h-[132px] align-top transition hover:bg-card-hover/70" :class="dirtyItemIds.has(item.itemId) ? 'bg-accent-soft/40' : ''">
                  <td class="border-b border-border px-vt-2 py-vt-2 align-middle text-center">
                    <div class="mx-auto grid size-8 place-items-center rounded-vt-sm border border-border bg-page text-xs text-muted">#{{ item.index.toString().padStart(2, '0') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="h-[116px] overflow-y-auto rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">
                      <div class="font-medium text-primary">{{ item.sourceText || t('imageGeneration.emptySource') }}</div>
                      <div v-if="item.narrationText && item.narrationText !== item.sourceText" class="mt-vt-2 border-t border-border pt-vt-2 text-muted">{{ item.narrationText }}</div>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="h-[116px] overflow-y-auto rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">{{ item.visualGoal || t('imageGeneration.emptyIntent') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <n-input :value="formatCharacters(item.characters)" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :placeholder="t('imageGeneration.placeholders.characters')" @update:value="updateCharacters(item, $event)" />
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <n-input :value="item.sceneDescription" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :placeholder="t('imageGeneration.placeholders.scene')" @update:value="updateItem(item, { sceneDescription: $event })" />
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <n-input :value="item.visualDescription" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :placeholder="t('imageGeneration.placeholders.visual')" @update:value="updateItem(item, { visualDescription: $event })" />
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <n-input :value="item.imagePrompt" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :placeholder="t('imageGeneration.placeholders.prompt')" @update:value="updateItem(item, { imagePrompt: $event })" />
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <n-input :value="item.negativePrompt" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :placeholder="t('imageGeneration.placeholders.negativePrompt')" @update:value="updateItem(item, { negativePrompt: $event })" />
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div v-if="item.imageCandidates.length > 0" class="grid h-[116px] grid-cols-2 gap-vt-1 overflow-y-auto">
                      <button v-for="(candidate, candidateIndex) in item.imageCandidates" :key="candidate.imageId" type="button" class="grid min-h-[54px] grid-cols-[40px_1fr] items-center gap-vt-2 rounded-vt-sm border px-vt-2 py-vt-1 text-left text-[11px] leading-4 transition" :class="candidateButtonClass(item, candidate)" @click="handleSelectCandidate(item, candidate)">
                        <span class="grid h-9 w-10 place-items-center rounded-vt-sm border border-border text-[9px] font-semibold uppercase text-primary" :class="candidatePreviewClass(candidate, candidateIndex)">{{ t('imageGeneration.mockShort') }}</span>
                        <span class="min-w-0">
                          <span class="block truncate font-semibold">{{ t('imageGeneration.candidateVariant', { index: candidateIndex + 1 }) }}</span>
                          <span class="block truncate text-muted">{{ shortId(candidate.imageId) }}</span>
                          <span v-if="isCandidateSelected(item, candidate)" class="block text-accent">{{ t('imageGeneration.selectedMark') }}</span>
                        </span>
                      </button>
                    </div>
                    <div v-else class="grid h-[116px] place-items-center rounded-vt-sm border border-border bg-page px-vt-2 text-center text-xs text-muted">{{ t('imageGeneration.noCandidates') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="h-[116px] overflow-hidden rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">
                      <template v-if="selectedCandidate(item)">
                        <div class="font-semibold text-primary">{{ shortId(selectedCandidate(item)?.imageId) }}</div>
                        <div class="mt-vt-1 line-clamp-4 text-muted">{{ selectedCandidate(item)?.imagePath }}</div>
                      </template>
                      <template v-else>
                        <div class="grid h-full place-items-center text-center text-muted">{{ t('imageGeneration.notSelected') }}</div>
                      </template>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1 text-xs">
                      <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-vt-1 text-secondary">{{ t(`dict.sceneAssetStatus.${item.imageStatus}`) }}</span>
                      <span class="text-muted">{{ t('imageGeneration.candidateCount', { count: item.imageCandidates.length }) }}</span>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1">
                      <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="generatingItemIds.has(item.itemId)" @click="handleGenerateItem(item)">{{ item.imageCandidates.length > 0 ? t('imageGeneration.regenerateRow') : t('imageGeneration.generateRow') }}</n-button>
                      <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="savingItemId === item.itemId" :disabled="!dirtyItemIds.has(item.itemId)" @click="handleSaveItem(item)">{{ t('imageGeneration.saveRow') }}</n-button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>

            <div v-if="storyboardItems.length === 0" class="grid h-full min-h-80 place-items-center text-sm text-muted">{{ t('imageGeneration.empty') }}</div>
          </div>
        </section>
      </main>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { NButton, NInput, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { useProjectStore } from '@/entities/project/store'
import { useStoryboardStore } from '@/entities/storyboard/store'
import type { ImageCandidateDto, StoryboardItemDto } from '@/entities/storyboard/types'
import { validateStoryboardItemsForVideoGeneration, type StoryboardVideoEntryField } from '@/entities/storyboard/validation'
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
const isMockImageFlow = computed(() => storyboardItems.value.some((item) => item.imagePrompt.startsWith('MOCK') || item.imageCandidates.some((candidate) => candidate.providerModelId.startsWith('mock'))))

onMounted(async () => {
  await Promise.all([projectStore.loadProject(projectId), storyboardStore.loadStoryboard(projectId)])

  if (!workspaceAccess.value.canEnterImage) {
    message.warning(t('workspaceStepBar.blocked.image'))
    await router.replace(getWorkspaceStepPath(projectId, getRequiredWorkspaceStep('image', workspaceAccess.value)))
  }
})

function updateItem(item: StoryboardItemDto, patch: Partial<StoryboardItemDto>) {
  Object.assign(item, patch)
  markItemDirty(item.itemId)
}

function updateCharacters(item: StoryboardItemDto, value: string) {
  updateItem(item, { characters: parseListInput(value) })
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
    await storyboardStore.saveScene(normalizeImageItem(item))
    markItemClean(item.itemId)
    message.success(t('imageGeneration.saveSuccess'))
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
      if (item) await storyboardStore.saveScene(normalizeImageItem(item))
    }
    dirtyItemIds.value = new Set()
    message.success(t('imageGeneration.saveAllSuccess'))
  } finally {
    isSavingAll.value = false
  }
}

async function handleGenerateItem(item: StoryboardItemDto) {
  await saveDirtyItemIfNeeded(item)
  setGenerating(item.itemId, true)
  try {
    await storyboardStore.generateImages(projectId, item.itemId)
    message.success(t('imageGeneration.generateSuccess', { index: item.index }))
  } finally {
    setGenerating(item.itemId, false)
  }
}

async function handleGenerateAll() {
  await generateItems(storyboardItems.value)
}

async function handleGenerateMissing() {
  await generateItems(storyboardItems.value.filter((item) => item.imageCandidates.length === 0))
}

async function generateItems(items: StoryboardItemDto[]) {
  if (items.length === 0) {
    message.info(t('imageGeneration.noMissingRows'))
    return
  }

  isBulkGenerating.value = true
  try {
    await handleSaveAll()
    for (const item of items) {
      setGenerating(item.itemId, true)
      await storyboardStore.generateImages(projectId, item.itemId)
      setGenerating(item.itemId, false)
    }
    message.success(t('imageGeneration.bulkGenerateSuccess', { count: items.length }))
  } finally {
    generatingItemIds.value = new Set()
    isBulkGenerating.value = false
  }
}

async function handleSelectCandidate(item: StoryboardItemDto, candidate: ImageCandidateDto) {
  await storyboardStore.selectImage(item.itemId, candidate.imageId)
  message.success(t('imageGeneration.selectSuccess', { index: item.index }))
}

async function handleEnterVideo() {
  if (dirtyItemIds.value.size > 0) await handleSaveAll()

  const issues = validateStoryboardItemsForVideoGeneration(storyboardItems.value)
  if (issues.length > 0) {
    showFirstVideoIssue(issues[0])
    return
  }

  await router.push(`/projects/${projectId}/workspace/video`)
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

function selectedCandidate(item: StoryboardItemDto) {
  return item.imageCandidates.find((candidate) => candidate.imageId === item.selectedImageId || candidate.selected) ?? null
}

function isCandidateSelected(item: StoryboardItemDto, candidate: ImageCandidateDto) {
  return item.selectedImageId === candidate.imageId || candidate.selected
}

function candidateButtonClass(item: StoryboardItemDto, candidate: ImageCandidateDto) {
  return isCandidateSelected(item, candidate) ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary hover:border-border-strong hover:text-primary'
}

function candidatePreviewClass(candidate: ImageCandidateDto, fallbackIndex: number) {
  const toneClass = candidate.generationContextSnapshot.visualToneClass
  if (typeof toneClass === 'string' && /^scene-preview-tone-[0-4]$/.test(toneClass)) return toneClass
  return `scene-preview-tone-${fallbackIndex % 5}`
}

function normalizeImageItem(item: StoryboardItemDto): StoryboardItemDto {
  return {
    ...item,
    characters: normalizeStringList(item.characters),
    sceneDescription: item.sceneDescription.trim(),
    visualDescription: item.visualDescription.trim(),
    imagePrompt: item.imagePrompt.trim(),
    negativePrompt: item.negativePrompt.trim(),
  }
}

function formatCharacters(characters: string[]) {
  return characters.join('、')
}

function parseListInput(value: string) {
  return normalizeStringList(value.split(/[、,，\n]/))
}

function normalizeStringList(values: string[]) {
  return values.map((value) => value.trim()).filter(Boolean)
}

function shortId(value?: string) {
  if (!value) return '-'
  return value.length > 12 ? value.slice(-12) : value
}

function showFirstVideoIssue(issue: { index: number; fields: StoryboardVideoEntryField[] }) {
  const fields = issue.fields.map((field) => t(`imageGeneration.validation.fields.${field}`)).join('、')
  message.error(t('imageGeneration.validation.enterVideoBlocked', { index: issue.index, fields }))
}

function handleBlockedStep(step: WorkspaceStepKey) {
  message.warning(t(`workspaceStepBar.blocked.${step}`))
}
</script>
