<template>
  <section class="view h-full min-w-0 overflow-hidden bg-page text-primary">
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      <WorkspaceHeader :project-id="projectId" :project-title="projectTitle" current-step="storyboard" :access="workspaceAccess" back-to="/" :badge-label="isMockStoryboard ? t('storyboard.mockBadge') : ''" right-width-class="w-[380px]" :usage="resourceUsage" @blocked="handleBlockedStep">
        <template #actions>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" @click="handleSaveAll" :disabled="dirtyItemIds.size === 0 || isSavingAll">{{ t('storyboard.saveAll') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" @click="handleApprove" :disabled="isSavingAll">{{ t('storyboard.approveStoryboard') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm bg-accent px-vt-4 text-sm font-semibold text-accent-ink transition hover:brightness-110" @click="handleEnterImage">{{ t('storyboard.enterImage') }}</button>
        </template>
      </WorkspaceHeader>

      <main class="flex min-h-0 flex-1 flex-col overflow-hidden bg-page">
        <section class="flex min-h-0 flex-1 flex-col gap-vt-3 overflow-hidden p-vt-3">
          <div class="flex flex-none flex-wrap items-center gap-vt-2 rounded-vt-md border border-border bg-card px-vt-3 py-vt-2">
            <span class="mr-vt-1 text-xs font-medium text-muted">{{ t('storyboard.splitMode.label') }}</span>
            <button v-for="mode in splitModes" :key="mode.value" type="button" class="h-8 rounded-vt-sm border px-vt-3 text-xs font-medium transition" :class="splitMode === mode.value ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary hover:border-border-strong hover:text-primary'" @click="splitMode = mode.value">
              {{ mode.label }}
            </button>

            <label v-if="splitMode === 'line_count'" class="ml-vt-2 flex items-center gap-vt-2 text-xs text-secondary">
              <span>{{ t('storyboard.splitMode.lineCount') }}</span>
              <n-input-number class="inp w-20" size="small" :min="1" :max="20" :step="1" :value="lineCount" @update:value="lineCount = normalizeCount($event, 2)" />
            </label>

            <label v-if="splitMode === 'sentence_count'" class="ml-vt-2 flex items-center gap-vt-2 text-xs text-secondary">
              <span>{{ t('storyboard.splitMode.sentenceCount') }}</span>
              <n-input-number class="inp w-20" size="small" :min="1" :max="20" :step="1" :value="sentenceCount" @update:value="sentenceCount = normalizeCount($event, 1)" />
            </label>

            <label v-if="splitMode === 'ai'" class="ml-vt-2 flex items-center gap-vt-2 text-xs text-secondary">
              <span>{{ t('storyboard.splitMode.targetCount') }}</span>
              <n-input-number class="inp w-20" size="small" :min="1" :max="60" :step="1" :value="aiTargetCount" @update:value="aiTargetCount = normalizeCount($event, 8, 60)" />
            </label>

            <div class="ml-auto flex flex-wrap items-center gap-vt-2">
              <WorkspaceRowJump :count="storyboardItems.length" @jump="jumpToRow" />
              <span v-if="dirtyItemIds.size > 0" class="text-xs text-accent">{{ t('storyboard.dirtyCount', { count: dirtyItemIds.size }) }}</span>
              <button type="button" class="inline-flex h-8 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-xs font-medium text-secondary transition hover:bg-page hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isRegenerating" @click="handleRegenerate">{{ t('storyboard.regenerateStoryboard') }}</button>
              <button type="button" class="inline-flex h-8 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-xs font-medium text-secondary transition hover:bg-page hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isRestoring" @click="handleRestore">{{ t('storyboard.restorePrevious') }}</button>
              <button type="button" class="inline-flex h-8 items-center justify-center rounded-vt-sm bg-accent px-vt-3 text-xs font-semibold text-accent-ink transition hover:brightness-110" @click="handleAddItem">{{ t('storyboard.addStoryboardItem') }}</button>
            </div>
          </div>

          <div v-if="pendingMergeItem" class="flex flex-none flex-wrap items-center justify-between gap-vt-2 rounded-vt-md border border-accent-line bg-accent-soft px-vt-3 py-vt-2 text-xs">
            <div class="flex min-w-0 flex-col gap-0.5">
              <span class="font-semibold text-accent">{{ t('storyboard.mergeMode.title', { index: pendingMergeItem.index }) }}</span>
              <span class="text-secondary">{{ t('storyboard.mergeMode.hint') }}</span>
            </div>
            <button type="button" class="inline-flex h-8 items-center justify-center rounded-vt-sm border border-accent-line bg-page px-vt-3 font-medium text-accent transition hover:bg-card" @click="cancelMerge">{{ t('storyboard.mergeMode.cancel') }}</button>
          </div>

          <div class="min-h-0 flex-1 overflow-auto rounded-vt-md border border-border bg-card">
            <table class="w-full min-w-[1260px] table-fixed border-separate border-spacing-0 text-left text-sm">
              <thead class="sticky top-0 z-10 bg-panel text-xs text-muted">
                <tr>
                  <th class="w-[56px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('storyboard.columns.index') }}</th>
                  <th class="w-[320px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('storyboard.columns.source') }}</th>
                  <th class="w-[320px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('storyboard.columns.narration') }}</th>
                  <th class="w-[260px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('storyboard.columns.intent') }}</th>
                  <th class="w-[240px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('storyboard.columns.characters') }}</th>
                  <th class="w-[220px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('storyboard.columns.scene') }}</th>
                  <th class="w-[190px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('storyboard.columns.actions') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in storyboardItems" :id="rowDomId(item.index)" :key="item.itemId" class="h-[112px] align-top transition hover:bg-card-hover/70" :class="[dirtyItemIds.has(item.itemId) ? 'bg-accent-soft/50' : '', isMergeSource(item) ? 'bg-accent-soft ring-1 ring-inset ring-accent-line' : '', canMergeWithPending(item) ? 'bg-card-hover ring-1 ring-inset ring-accent-line' : '']">
                  <td class="border-b border-border px-vt-2 py-vt-2 align-middle text-center">
                    <div class="mx-auto grid size-8 place-items-center rounded-vt-sm border border-border bg-page text-xs text-muted">#{{ item.index.toString().padStart(2, '0') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1">
                      <button type="button" class="justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'sourceText')">{{ lockButtonLabel(item, 'sourceText') }}</button>
                      <n-input :value="item.sourceText" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :disabled="isItemLocked(item, 'sourceText')" :placeholder="t('storyboard.sourceText')" @update:value="updateItem(item, { sourceText: $event }, 'sourceText')" />
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1">
                      <button type="button" class="justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'narrationText')">{{ lockButtonLabel(item, 'narrationText') }}</button>
                      <n-input :value="item.narrationText" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :disabled="isItemLocked(item, 'narrationText')" :placeholder="t('storyboard.narration')" @update:value="updateItem(item, { narrationText: $event }, 'narrationText')" />
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <n-input :value="item.visualGoal" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :placeholder="t('storyboard.intentPlaceholder')" @update:value="updateItem(item, { visualGoal: $event })" />
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <button type="button" class="mb-vt-1 justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'characters')">{{ lockButtonLabel(item, 'characters') }}</button>
                    <n-select :value="item.characterIds" multiple size="small" clearable :disabled="isItemLocked(item, 'characters')" :options="characterOptions" :placeholder="t('storyboard.characterIdsPlaceholder')" @update:value="updateItem(item, { characterIds: normalizeSelectedCharacterIds($event) }, 'characters')" />
                    <div class="mt-vt-2 truncate text-[11px] text-muted">{{ formatItemCharacters(item) }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-2">
                      <button type="button" class="justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'location')">{{ lockButtonLabel(item, 'location') }}</button>
                      <n-select :value="item.locationId" size="small" clearable :disabled="isItemLocked(item, 'location')" :options="locationOptions" :placeholder="t('storyboard.locationIdPlaceholder')" @update:value="updateLocationId(item, $event)" />
                      <n-input v-if="!item.locationId" :value="item.sceneDescription" class="inp compact-inp" size="small" :disabled="isItemLocked(item, 'location')" :placeholder="t('storyboard.sceneDescriptionPlaceholder')" @update:value="updateItem(item, { sceneDescription: $event }, 'location')" />
                      <div v-else class="truncate text-[11px] text-muted">{{ formatItemLocation(item) }}</div>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div v-if="pendingMergeItemId" class="grid gap-vt-1">
                      <template v-if="isMergeSource(item)">
                        <div class="rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-vt-1 text-[11px] leading-4 text-accent">{{ t('storyboard.mergeMode.selected', { index: item.index }) }}</div>
                        <button type="button" class="h-7 rounded-vt-sm border border-border bg-page px-vt-1 text-xs text-secondary transition hover:border-border-strong hover:text-primary" @click="cancelMerge">{{ t('storyboard.mergeMode.cancel') }}</button>
                      </template>
                      <template v-else>
                        <button type="button" class="h-7 rounded-vt-sm px-vt-1 text-xs font-semibold transition disabled:cursor-not-allowed disabled:opacity-40" :class="canMergeWithPending(item) ? 'bg-accent text-accent-ink hover:brightness-110' : 'border border-border bg-page text-muted'" :disabled="!canMergeWithPending(item)" @click="confirmMergeWith(item)">{{ t('storyboard.rowActions.mergeWithSelected') }}</button>
                        <div class="rounded-vt-sm border px-vt-2 py-vt-1 text-[11px] leading-4" :class="canMergeWithPending(item) ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-muted'">
                          {{ canMergeWithPending(item) ? t('storyboard.mergeMode.ready', { selected: pendingMergeItem?.index, target: item.index }) : t('storyboard.mergeMode.adjacentOnly') }}
                        </div>
                      </template>
                    </div>
                    <div v-else class="grid grid-cols-2 gap-vt-1">
                      <button type="button" class="h-7 rounded-vt-sm border border-border bg-page px-vt-1 text-xs text-secondary transition hover:border-border-strong hover:text-primary disabled:cursor-not-allowed disabled:opacity-40" :disabled="item.index === 1" @click="handleMove(item, -1)">{{ t('storyboard.rowActions.up') }}</button>
                      <button type="button" class="h-7 rounded-vt-sm border border-border bg-page px-vt-1 text-xs text-secondary transition hover:border-border-strong hover:text-primary disabled:cursor-not-allowed disabled:opacity-40" :disabled="item.index === storyboardItems.length" @click="handleMove(item, 1)">{{ t('storyboard.rowActions.down') }}</button>
                      <button type="button" class="h-7 rounded-vt-sm border border-border bg-page px-vt-1 text-xs text-secondary transition hover:border-border-strong hover:text-primary" @click="handleSplitItem(item)">{{ t('storyboard.rowActions.split') }}</button>
                      <button type="button" class="h-7 rounded-vt-sm border border-border bg-page px-vt-1 text-xs text-secondary transition hover:border-border-strong hover:text-primary disabled:cursor-not-allowed disabled:opacity-40" :disabled="storyboardItems.length <= 1" @click="startMergeSelection(item)">{{ t('storyboard.rowActions.startMerge') }}</button>
                      <button type="button" class="h-7 rounded-vt-sm border border-status-failed/40 bg-page px-vt-1 text-xs text-status-failed transition hover:border-status-failed disabled:cursor-not-allowed disabled:opacity-40" :disabled="storyboardItems.length <= 1" @click="handleDeleteItem(item)">{{ t('storyboard.rowActions.delete') }}</button>
                      <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="savingItemId === item.itemId" :disabled="!dirtyItemIds.has(item.itemId)" @click="handleSaveItem(item)">{{ t('storyboard.saveRow') }}</n-button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>

            <div v-if="storyboardItems.length === 0" class="grid h-full min-h-80 place-items-center text-sm text-muted">{{ t('storyboard.emptyStoryboard') }}</div>
          </div>
        </section>
      </main>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { NButton, NInput, NInputNumber, NSelect, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { useProjectStore } from '@/entities/project/store'
import { listProjectCharacterBibles, listProjectLocationBibles } from '@/entities/config/api'
import type { CharacterBibleDto, LocationBibleDto } from '@/entities/config/types'
import { useStoryboardStore } from '@/entities/storyboard/store'
import { validateStoryboardItemsForImageGeneration, type StoryboardImageEntryField } from '@/entities/storyboard/validation'
import type { RegenerateStoryboardRequest, StoryboardItemDto, StoryboardSplitMode } from '@/entities/storyboard/types'
import { useTaskStore } from '@/entities/task/store'
import { getWorkspaceStepAccess, type WorkspaceStepKey } from '@/features/workspace/steps'
import WorkspaceHeader from '@/features/workspace/WorkspaceHeader.vue'
import WorkspaceRowJump from '@/features/workspace/WorkspaceRowJump.vue'
import { isStoryboardItemLocked, setStoryboardItemLock } from '@/entities/storyboard/reset'
import type { StoryboardItemLockField } from '@/entities/storyboard/types'

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const storyboardStore = useStoryboardStore()
const taskStore = useTaskStore()
const { t } = useI18n()
const message = useMessage()

const projectId = String(route.params.projectId)
const dirtyItemIds = ref<Set<string>>(new Set())
const savingItemId = ref<string | null>(null)
const isSavingAll = ref(false)
const isRegenerating = ref(false)
const isRestoring = ref(false)
const pendingMergeItemId = ref<string | null>(null)
const characterBibles = ref<CharacterBibleDto[]>([])
const locationBibles = ref<LocationBibleDto[]>([])
const splitMode = ref<StoryboardSplitMode>('paragraph')
const lineCount = ref(2)
const sentenceCount = ref(1)
const aiTargetCount = ref(8)

const splitModes = computed<Array<{ label: string; value: StoryboardSplitMode }>>(() => [
  { label: t('storyboard.splitMode.paragraph'), value: 'paragraph' },
  { label: t('storyboard.splitMode.lineCountMode'), value: 'line_count' },
  { label: t('storyboard.splitMode.sentenceCountMode'), value: 'sentence_count' },
  { label: t('storyboard.splitMode.ai'), value: 'ai' },
])

const projectTitle = computed(() => (projectStore.currentProject?.project.projectId === projectId ? projectStore.currentProject.project.title : projectId))
const storyboardItems = computed(() => storyboardStore.storyboard?.items ?? [])
const pendingMergeItem = computed(() => storyboardItems.value.find((item) => item.itemId === pendingMergeItemId.value) ?? null)
const isMockStoryboard = computed(() => storyboardItems.value.some((item) => item.visualDescription.startsWith('MOCK') || item.imagePrompt.startsWith('MOCK')))
const workspaceAccess = computed(() => getWorkspaceStepAccess(storyboardItems.value, storyboardStore.storyboard?.reviewStatus))
const characterOptions = computed(() => characterBibles.value.map((character) => ({ label: `${character.name} · ${character.characterId}`, value: character.characterId })))
const locationOptions = computed(() => locationBibles.value.map((location) => ({ label: `${location.name} · ${location.locationId}`, value: location.locationId })))
const resourceUsage = computed(() => ({
  images: storyboardItems.value.reduce((total, item) => total + item.imageCandidates.length, 0),
  videos: storyboardItems.value.reduce((total, item) => total + item.videoSegments.length, 0),
  llm: null,
}))

onMounted(async () => {
  const [projectDetail] = await Promise.all([projectStore.loadProject(projectId), storyboardStore.loadStoryboard(projectId)])
  const [characters, locations] = await Promise.all([listProjectCharacterBibles(projectId), listProjectLocationBibles(projectId)])
  characterBibles.value = characters
  locationBibles.value = locations
  aiTargetCount.value = projectDetail.project.targetSceneCount
})

function updateItem(item: StoryboardItemDto, patch: Partial<StoryboardItemDto>, lockedField?: StoryboardItemLockField) {
  if (lockedField && isItemLocked(item, lockedField)) {
    message.warning(t('storyboard.fieldLocked', { field: lockFieldLabel(lockedField) }))
    return
  }
  Object.assign(item, patch)
  markItemDirty(item.itemId)
}

function isItemLocked(item: StoryboardItemDto, field: StoryboardItemLockField) {
  return isStoryboardItemLocked(item, field)
}

function toggleItemLock(item: StoryboardItemDto, field: StoryboardItemLockField) {
  Object.assign(item, setStoryboardItemLock(item, field, !isItemLocked(item, field)))
  markItemDirty(item.itemId)
}

function lockButtonLabel(item: StoryboardItemDto, field: StoryboardItemLockField) {
  return isItemLocked(item, field) ? t('common.unlock') : t('common.lock')
}

function lockFieldLabel(field: StoryboardItemLockField) {
  return t(`storyboard.lockFields.${field}`)
}

function normalizeSelectedCharacterIds(value: Array<string | number>) {
  return value.map(String).filter(Boolean)
}

function updateLocationId(item: StoryboardItemDto, value: string | null) {
  if (isItemLocked(item, 'location')) {
    message.warning(t('storyboard.fieldLocked', { field: lockFieldLabel('location') }))
    return
  }
  const locationId = value?.trim() || null
  const location = locationId ? locationBibles.value.find((entry) => entry.locationId === locationId) : null
  updateItem(item, {
    locationId,
    sceneDescription: location ? location.name : item.sceneDescription,
  })
}

function formatItemCharacters(item: StoryboardItemDto) {
  if (item.characterIds.length === 0) return t('storyboard.noCharacterIds')
  return item.characterIds
    .map((characterId) => characterBibles.value.find((character) => character.characterId === characterId)?.name ?? characterId)
    .join('、')
}

function formatItemLocation(item: StoryboardItemDto) {
  if (!item.locationId) return item.sceneDescription || t('storyboard.noLocationId')
  const location = locationBibles.value.find((entry) => entry.locationId === item.locationId)
  return location ? `${location.name} · ${location.locationId}` : item.locationId
}

function markItemDirty(itemId: string) {
  dirtyItemIds.value = new Set(dirtyItemIds.value).add(itemId)
  if (storyboardStore.storyboard) storyboardStore.storyboard.reviewStatus = 'waiting_user'
}

function markStructureDirty(items: StoryboardItemDto[]) {
  dirtyItemIds.value = new Set(items.map((item) => item.itemId))
  if (storyboardStore.storyboard) storyboardStore.storyboard.reviewStatus = 'waiting_user'
}

function applyLocalItems(items: StoryboardItemDto[]) {
  if (!storyboardStore.storyboard) return
  const normalized = items.map((item, index) => ({ ...item, index: index + 1 }))
  storyboardStore.storyboard.items = normalized
  pendingMergeItemId.value = null
  markStructureDirty(normalized)
}

async function handleSaveItem(item: StoryboardItemDto) {
  savingItemId.value = item.itemId
  try {
    await saveStructure('storyboard.saveSuccess')
  } finally {
    savingItemId.value = null
  }
}

async function handleSaveAll() {
  if (dirtyItemIds.value.size === 0) return
  await saveStructure('storyboard.saveAllSuccess')
}

async function saveStructure(successKey: string) {
  if (!storyboardStore.storyboard) return

  isSavingAll.value = true
  try {
    await storyboardStore.saveStoryboardStructure(projectId, storyboardItems.value)
    dirtyItemIds.value = new Set()
    message.success(t(successKey))
  } finally {
    isSavingAll.value = false
  }
}

async function handleApprove() {
  const issues = validateStoryboardItemsForImageGeneration(storyboardItems.value)
  if (issues.length > 0) {
    showFirstIssue(issues[0])
    return
  }

  if (dirtyItemIds.value.size > 0) await saveStructure('storyboard.saveAllSuccess')
  await storyboardStore.approve(projectId)
  await taskStore.approveStep(projectId, 'storyboard_review')
  message.success(t('storyboard.approveSuccess'))
}

async function handleRegenerate() {
  isRegenerating.value = true
  try {
    await storyboardStore.regenerate(projectId, createRegenerateRequest())
    dirtyItemIds.value = new Set()
    message.success(t('storyboard.regenerateSuccess'))
  } finally {
    isRegenerating.value = false
  }
}

async function handleRestore() {
  isRestoring.value = true
  try {
    await storyboardStore.restorePrevious(projectId)
    dirtyItemIds.value = new Set()
    message.success(t('storyboard.restoreSuccess'))
  } catch {
    message.warning(t('storyboard.restoreUnavailable'))
  } finally {
    isRestoring.value = false
  }
}

async function handleAddItem() {
  const item = await storyboardStore.createDraftItem(projectId, storyboardItems.value.length + 1)
  applyLocalItems([...storyboardItems.value, item])
}

function handleMove(item: StoryboardItemDto, offset: -1 | 1) {
  pendingMergeItemId.value = null
  const currentIndex = storyboardItems.value.findIndex((entry) => entry.itemId === item.itemId)
  const targetIndex = currentIndex + offset
  if (currentIndex < 0 || targetIndex < 0 || targetIndex >= storyboardItems.value.length) return

  const nextItems = [...storyboardItems.value]
  const [movedItem] = nextItems.splice(currentIndex, 1)
  nextItems.splice(targetIndex, 0, movedItem)
  applyLocalItems(nextItems)
}

async function handleSplitItem(item: StoryboardItemDto) {
  pendingMergeItemId.value = null
  const sourceParts = splitIntoTwo(item.sourceText || item.narrationText)
  if (!sourceParts) {
    message.warning(t('storyboard.splitUnavailable'))
    return
  }

  const narrationParts = item.narrationText && item.narrationText !== item.sourceText ? splitIntoTwo(item.narrationText) ?? [item.narrationText, ''] : sourceParts
  const currentIndex = storyboardItems.value.findIndex((entry) => entry.itemId === item.itemId)
  if (currentIndex < 0) return

  const nextItem = await storyboardStore.createDraftItem(projectId, item.index + 1, sourceParts[1], narrationParts[1])
  const updatedItem: StoryboardItemDto = {
    ...item,
    sourceText: sourceParts[0],
    narrationText: narrationParts[0],
  }
  const nextItems = [...storyboardItems.value]
  nextItems.splice(currentIndex, 1, updatedItem, nextItem)
  applyLocalItems(nextItems)
}

function startMergeSelection(item: StoryboardItemDto) {
  pendingMergeItemId.value = item.itemId
}

function confirmMergeWith(targetItem: StoryboardItemDto) {
  const sourceIndex = storyboardItems.value.findIndex((entry) => entry.itemId === pendingMergeItemId.value)
  const targetIndex = storyboardItems.value.findIndex((entry) => entry.itemId === targetItem.itemId)
  if (sourceIndex < 0 || targetIndex < 0 || Math.abs(sourceIndex - targetIndex) !== 1) {
    message.warning(t('storyboard.mergeMode.adjacentOnly'))
    return
  }

  const firstIndex = Math.min(sourceIndex, targetIndex)
  const firstItem = storyboardItems.value[firstIndex]
  const secondItem = storyboardItems.value[firstIndex + 1]
  const mergedItem: StoryboardItemDto = {
    ...firstItem,
    sourceText: mergeText(firstItem.sourceText, secondItem.sourceText),
    narrationText: mergeText(firstItem.narrationText, secondItem.narrationText),
    visualGoal: mergeText(firstItem.visualGoal, secondItem.visualGoal),
  }
  const nextItems = [...storyboardItems.value]
  nextItems.splice(firstIndex, 2, mergedItem)
  applyLocalItems(nextItems)
}

function handleDeleteItem(item: StoryboardItemDto) {
  pendingMergeItemId.value = null
  if (storyboardItems.value.length <= 1) return
  applyLocalItems(storyboardItems.value.filter((entry) => entry.itemId !== item.itemId))
}

function isMergeSource(item: StoryboardItemDto) {
  return pendingMergeItemId.value === item.itemId
}

function canMergeWithPending(item: StoryboardItemDto) {
  if (!pendingMergeItemId.value || pendingMergeItemId.value === item.itemId) return false
  const sourceIndex = storyboardItems.value.findIndex((entry) => entry.itemId === pendingMergeItemId.value)
  const targetIndex = storyboardItems.value.findIndex((entry) => entry.itemId === item.itemId)
  return sourceIndex >= 0 && targetIndex >= 0 && Math.abs(sourceIndex - targetIndex) === 1
}

function cancelMerge() {
  pendingMergeItemId.value = null
}

function handleBlockedStep(step: WorkspaceStepKey) {
  message.warning(t(`workspaceStepBar.blocked.${step}`))
}

function rowDomId(index: number) {
  return `storyboard-row-${index}`
}

function jumpToRow(index: number) {
  document.getElementById(rowDomId(index))?.scrollIntoView({ block: 'center', behavior: 'smooth' })
}

async function handleEnterImage() {
  const issues = validateStoryboardItemsForImageGeneration(storyboardItems.value)
  if (issues.length > 0) {
    showFirstIssue(issues[0])
    return
  }

  if (storyboardStore.storyboard?.reviewStatus !== 'succeeded') {
    message.error(t('storyboard.validation.storyboardNotApproved'))
    return
  }

  await router.push(`/projects/${projectId}/workspace/image`)
}

function showFirstIssue(issue: { index: number; fields: StoryboardImageEntryField[] }) {
  const fields = issue.fields.map((field) => t(`storyboard.validation.fields.${field}`)).join('、')
  message.error(t('storyboard.validation.enterImageBlocked', { index: issue.index, fields }))
}

function createRegenerateRequest(): RegenerateStoryboardRequest {
  if (splitMode.value === 'line_count') return { mode: 'line_count', lineCount: lineCount.value }
  if (splitMode.value === 'sentence_count') return { mode: 'sentence_count', sentenceCount: sentenceCount.value }
  if (splitMode.value === 'ai') return { mode: 'ai', targetSceneCount: aiTargetCount.value }
  return { mode: 'paragraph' }
}

function normalizeCount(value: number | null, fallback: number, max = 20) {
  return Math.min(max, Math.max(1, Math.round(value ?? fallback)))
}

function splitIntoTwo(text: string): [string, string] | null {
  const trimmed = text.trim()
  if (!trimmed) return null

  const lines = trimmed.split('\n').filter(Boolean)
  if (lines.length > 1) {
    const midpoint = Math.ceil(lines.length / 2)
    return [lines.slice(0, midpoint).join('\n'), lines.slice(midpoint).join('\n')]
  }

  const sentences = trimmed.match(/[^。！？.!?]+[。！？.!?]?/g)?.map((entry) => entry.trim()).filter(Boolean) ?? []
  if (sentences.length > 1) {
    const midpoint = Math.ceil(sentences.length / 2)
    return [sentences.slice(0, midpoint).join(''), sentences.slice(midpoint).join('')]
  }

  if (trimmed.length < 8) return null
  const midpoint = Math.ceil(trimmed.length / 2)
  return [trimmed.slice(0, midpoint), trimmed.slice(midpoint)]
}

function mergeText(left: string, right: string) {
  return [left.trim(), right.trim()].filter(Boolean).join('\n')
}
</script>
