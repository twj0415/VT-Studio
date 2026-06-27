<template>
  <section class="view h-full min-w-0 overflow-hidden bg-page text-primary">
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      <WorkspaceHeader :project-id="projectId" :project-title="projectTitle" current-step="composition" :access="workspaceAccess" :back-to="`/projects/${projectId}/workspace/video`" :badge-label="t('compositionGeneration.mockBadge')" right-width-class="w-[620px]" :usage="resourceUsage" @blocked="handleBlockedStep">
        <template #actions>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="!latestExportRecord || isOpeningDirectory" @click="handleOpenOutputDir(latestExportRecord?.exportId)">{{ t('compositionGeneration.openOutputDir') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-accent-line bg-accent-soft px-vt-3 text-sm font-semibold text-accent transition hover:bg-accent-soft/80 disabled:cursor-not-allowed disabled:opacity-50" :disabled="!canExportFinalVideo || isExporting" @click="handleExportFinalVideo">{{ t('compositionGeneration.exportFinalVideo') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isExportingPackage" @click="handleExportProjectPackage">{{ t('compositionGeneration.exportProjectPackage') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isBackingUpWorkspace" @click="handleBackupWorkspace">{{ t('compositionGeneration.backupWorkspace') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm bg-accent px-vt-4 text-sm font-semibold text-accent-ink transition hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-50" :disabled="isComposing" @click="handleStartComposition">{{ t('compositionGeneration.startComposition') }}</button>
        </template>
      </WorkspaceHeader>

      <main class="flex min-h-0 flex-1 flex-col overflow-hidden bg-page">
        <section class="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_340px] gap-vt-3 overflow-hidden p-vt-3 max-xl:grid-cols-1">
          <div class="flex min-h-0 flex-col gap-vt-3 overflow-hidden">
            <div class="flex flex-none flex-wrap items-center gap-vt-2 rounded-vt-md border border-border bg-card px-vt-3 py-vt-2 text-xs">
              <span class="font-medium text-secondary">{{ t('compositionGeneration.checkTitle') }}</span>
              <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('compositionGeneration.segmentCount', { count: confirmedSegmentCount }) }}</span>
              <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('compositionGeneration.totalDuration', { seconds: totalDurationSeconds }) }}</span>
              <span v-if="compositionResetCount > 0" class="rounded-vt-sm border border-status-retrying/50 bg-status-retrying/10 px-vt-2 py-1 text-status-retrying">{{ t('compositionGeneration.downstreamReset.count', { count: compositionResetCount }) }}</span>
              <WorkspaceRowJump class="ml-auto" :count="storyboardItems.length" @jump="jumpToRow" />
              <span class="text-muted">{{ t('compositionGeneration.boundaryHint') }}</span>
            </div>

            <div class="min-h-0 flex-1 overflow-auto rounded-vt-md border border-border bg-card">
              <table class="w-full min-w-[980px] table-fixed border-separate border-spacing-0 text-left text-sm">
                <thead class="sticky top-0 z-10 bg-panel text-xs text-muted">
                  <tr>
                    <th class="w-[56px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('compositionGeneration.columns.index') }}</th>
                    <th class="w-[320px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('compositionGeneration.columns.source') }}</th>
                    <th class="w-[260px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('compositionGeneration.columns.segment') }}</th>
                    <th class="w-[120px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('compositionGeneration.columns.duration') }}</th>
                    <th class="w-[160px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('compositionGeneration.columns.status') }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="item in storyboardItems" :id="rowDomId(item.index)" :key="item.itemId" class="h-[96px] align-top transition hover:bg-card-hover/70">
                    <td class="border-b border-border px-vt-2 py-vt-2 align-middle text-center">
                      <div class="mx-auto grid size-8 place-items-center rounded-vt-sm border border-border bg-page text-xs text-muted">#{{ item.index.toString().padStart(2, '0') }}</div>
                    </td>
                    <td class="border-b border-border px-vt-2 py-vt-2">
                      <div class="h-20 overflow-y-auto rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">
                        <div class="font-medium text-primary">{{ item.sourceText || t('compositionGeneration.emptySource') }}</div>
                        <div v-if="item.narrationText && item.narrationText !== item.sourceText" class="mt-vt-1 text-muted">{{ item.narrationText }}</div>
                      </div>
                    </td>
                    <td class="border-b border-border px-vt-2 py-vt-2">
                      <div class="h-20 overflow-hidden rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">
                        <template v-if="selectedSegment(item)">
                          <div class="font-semibold text-primary">{{ shortId(selectedSegment(item)?.segmentId) }}</div>
                          <div class="truncate text-muted">{{ selectedSegment(item)?.videoPath }}</div>
                          <div class="mt-vt-1 truncate text-muted">{{ probeSummary(selectedSegment(item)) }}</div>
                        </template>
                        <template v-else>
                          <div class="grid h-full place-items-center text-muted">{{ t('compositionGeneration.missingSegment') }}</div>
                        </template>
                      </div>
                    </td>
                    <td class="border-b border-border px-vt-2 py-vt-2 text-xs text-secondary">{{ selectedSegment(item)?.durationSeconds ?? '-' }}s</td>
                    <td class="border-b border-border px-vt-2 py-vt-2">
                      <div class="grid gap-vt-1">
                        <span class="inline-flex rounded-vt-sm border px-vt-2 py-vt-1 text-xs" :class="selectedSegment(item) ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-muted'">{{ selectedSegment(item) ? t('compositionGeneration.ready') : t('compositionGeneration.missing') }}</span>
                        <span v-if="compositionResetRecord(item)" class="rounded-vt-sm border border-status-retrying/50 bg-status-retrying/10 px-vt-2 py-vt-1 text-xs text-status-retrying">{{ t('compositionGeneration.downstreamReset.row', { reason: resetReasonLabel(compositionResetRecord(item)?.triggerField) }) }}</span>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <aside class="flex min-h-0 flex-col gap-vt-3 overflow-y-auto rounded-vt-md border border-border bg-card p-vt-3">
            <div class="flex items-center justify-between gap-vt-2">
              <div class="text-sm font-semibold text-primary">{{ t('compositionGeneration.outputTitle') }}</div>
              <span class="rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-0.5 text-[11px] font-medium text-accent">{{ t('compositionGeneration.mockBadge') }}</span>
            </div>
            <div class="grid gap-vt-2 text-xs">
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="text-muted">{{ t('compositionGeneration.taskStatus') }}</div>
                <div class="mt-vt-1 text-sm font-semibold text-primary">{{ taskStatusText }}</div>
              </div>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="text-muted">{{ t('compositionGeneration.progress') }}</div>
                <div class="mt-vt-2 h-2 overflow-hidden rounded-full bg-card-hover">
                  <div class="h-full rounded-full bg-accent transition-all" :style="{ width: `${compositionTask?.progress ?? 0}%` }"></div>
                </div>
                <div class="mt-vt-1 text-muted">{{ compositionTask?.progress ?? 0 }}%</div>
              </div>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="text-muted">{{ t('compositionGeneration.outputPath') }}</div>
                <div class="mt-vt-1 break-all font-mono text-xs text-primary">{{ compositionTask?.outputPath ?? t('compositionGeneration.notGenerated') }}</div>
              </div>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="flex items-center justify-between gap-vt-2">
                  <div class="text-muted">{{ t('compositionGeneration.exportStatus.title') }}</div>
                  <span class="rounded-vt-sm border px-vt-2 py-0.5 text-[11px]" :class="latestExportRecord ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-card text-muted'">{{ latestExportRecord ? exportStatusLabel(latestExportRecord.status) : t('compositionGeneration.exportStatus.notExported') }}</span>
                </div>
                <div class="mt-vt-2 break-all font-mono text-xs text-primary">{{ latestExportRecord?.targetRelativePath ?? t('compositionGeneration.exportStatus.noTarget') }}</div>
                <div v-if="latestExportRecord?.finishedAt" class="mt-vt-2 text-muted">{{ t('compositionGeneration.exportStatus.finishedAt', { time: latestExportRecord.finishedAt }) }}</div>
              </div>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="text-muted">{{ t('compositionGeneration.includedSegments') }}</div>
                <div class="mt-vt-1 text-primary">{{ compositionTask?.segmentIds.length ?? 0 }}</div>
              </div>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="text-muted">{{ t('compositionGeneration.probeTitle') }}</div>
                <div class="mt-vt-1 text-primary">{{ t('compositionGeneration.probeCount', { count: probedSegmentCount, total: confirmedSegmentCount }) }}</div>
              </div>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="flex items-center justify-between gap-vt-2">
                  <div class="text-muted">{{ t('compositionGeneration.optional.title') }}</div>
                  <span class="text-[11px] text-muted">{{ t('compositionGeneration.optional.hint') }}</span>
                </div>
                <div class="mt-vt-2 grid gap-vt-2">
                  <label class="inline-flex items-center gap-vt-2 text-secondary">
                    <input v-model="includeSubtitle" type="checkbox" class="size-4 accent-[var(--accent)]" />
                    <span>{{ t('compositionGeneration.optional.subtitle') }}</span>
                  </label>
                  <div class="break-all rounded-vt-sm border border-border bg-card px-vt-2 py-vt-1 font-mono text-[11px] text-muted">{{ subtitlePath }}</div>
                  <label class="inline-flex items-center gap-vt-2 text-secondary">
                    <input v-model="includeCoverMetadata" type="checkbox" class="size-4 accent-[var(--accent)]" />
                    <span>{{ t('compositionGeneration.optional.cover') }}</span>
                  </label>
                  <div class="break-all rounded-vt-sm border border-border bg-card px-vt-2 py-vt-1 font-mono text-[11px] text-muted">{{ projectStore.currentProject?.project.coverPath || t('compositionGeneration.optional.noCover') }}</div>
                  <div v-if="enhancementStepSummary.length > 0" class="grid gap-vt-1">
                    <div v-for="step in enhancementStepSummary" :key="step.key" class="flex items-center justify-between gap-vt-2 text-[11px]">
                      <span class="text-muted">{{ step.label }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-secondary">{{ step.status }}</span>
                    </div>
                  </div>
                </div>
              </div>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="flex items-center justify-between gap-vt-2">
                  <div class="text-muted">{{ t('compositionGeneration.bgm.title') }}</div>
                  <label class="inline-flex items-center gap-vt-2 text-secondary">
                    <input v-model="includeBgm" type="checkbox" class="size-4 accent-[var(--accent)]" />
                    <span>{{ t('compositionGeneration.bgm.enable') }}</span>
                  </label>
                </div>
                <div class="mt-vt-2 grid gap-vt-2">
                  <select v-model="selectedBgmAssetId" class="h-9 rounded-vt-sm border border-border bg-card px-vt-2 text-xs text-primary outline-none focus:border-accent-line" :disabled="!includeBgm">
                    <option value="">{{ t('compositionGeneration.bgm.empty') }}</option>
                    <option v-for="asset in bgmAssets" :key="asset.assetId" :value="asset.assetId">{{ bgmAssetLabel(asset) }}</option>
                  </select>
                  <div class="grid gap-vt-1">
                    <div class="flex items-center justify-between gap-vt-2">
                      <span class="text-muted">{{ t('compositionGeneration.bgm.volume') }}</span>
                      <span class="font-mono text-primary">{{ Math.round(bgmVolume * 100) }}%</span>
                    </div>
                    <input v-model.number="bgmVolume" type="range" min="0" max="0.6" step="0.01" class="w-full accent-[var(--accent)]" :disabled="!includeBgm" />
                  </div>
                  <div class="grid grid-cols-2 gap-vt-2">
                    <label class="grid gap-vt-1">
                      <span class="text-muted">{{ t('compositionGeneration.bgm.fadeIn') }}</span>
                      <input v-model.number="bgmFadeInSeconds" type="number" min="0" max="30" step="0.5" class="h-8 rounded-vt-sm border border-border bg-card px-vt-2 text-xs text-primary outline-none focus:border-accent-line" :disabled="!includeBgm" />
                    </label>
                    <label class="grid gap-vt-1">
                      <span class="text-muted">{{ t('compositionGeneration.bgm.fadeOut') }}</span>
                      <input v-model.number="bgmFadeOutSeconds" type="number" min="0" max="30" step="0.5" class="h-8 rounded-vt-sm border border-border bg-card px-vt-2 text-xs text-primary outline-none focus:border-accent-line" :disabled="!includeBgm" />
                    </label>
                  </div>
                  <label class="inline-flex items-center gap-vt-2 text-secondary">
                    <input v-model="bgmLoop" type="checkbox" class="size-4 accent-[var(--accent)]" :disabled="!includeBgm" />
                    <span>{{ t('compositionGeneration.bgm.loop') }}</span>
                  </label>
                  <div class="flex gap-vt-2">
                    <input v-model.trim="bgmImportPath" type="text" class="min-w-0 flex-1 rounded-vt-sm border border-border bg-card px-vt-2 text-xs text-primary outline-none focus:border-accent-line" :placeholder="t('compositionGeneration.bgm.importPlaceholder')" />
                    <button type="button" class="rounded-vt-sm border border-border px-vt-2 py-vt-1 text-xs text-secondary transition hover:bg-card-hover hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="!bgmImportPath || isImportingBgm" @click="handleImportBgm">{{ t('compositionGeneration.bgm.import') }}</button>
                  </div>
                  <div class="text-[11px] leading-5 text-muted">{{ t('compositionGeneration.bgm.hint') }}</div>
                </div>
              </div>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="flex items-center justify-between gap-vt-2">
                  <div class="text-muted">{{ t('compositionGeneration.backupStatus.title') }}</div>
                  <span class="rounded-vt-sm border px-vt-2 py-0.5 text-[11px]" :class="latestBackup ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-card text-muted'">{{ latestBackup ? t('compositionGeneration.backupStatus.ready') : t('compositionGeneration.backupStatus.empty') }}</span>
                </div>
                <div class="mt-vt-2 break-all font-mono text-xs text-primary">{{ latestBackup?.targetRelativePath ?? t('compositionGeneration.backupStatus.noTarget') }}</div>
                <div v-if="latestBackup" class="mt-vt-2 text-muted">{{ t('compositionGeneration.backupStatus.summary', { projects: latestBackup.projectCount, assets: latestBackup.assetCount }) }}</div>
                <div v-if="latestBackup?.requiresSecretReentry" class="mt-vt-1 text-status-retrying">{{ t('compositionGeneration.backupStatus.secretHint') }}</div>
              </div>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="mb-vt-2 flex items-center justify-between gap-vt-2">
                  <div class="text-muted">{{ t('compositionGeneration.exportHistory.title') }}</div>
                  <span class="text-muted">{{ t('compositionGeneration.exportHistory.count', { count: exportRecords.length }) }}</span>
                </div>
                <div v-if="exportRecords.length === 0" class="rounded-vt-sm border border-dashed border-border px-vt-2 py-vt-3 text-center text-muted">{{ t('compositionGeneration.exportHistory.empty') }}</div>
                <div v-else class="grid max-h-56 gap-vt-2 overflow-y-auto">
                  <div v-for="record in exportRecords" :key="record.exportId" class="rounded-vt-sm border border-border bg-card px-vt-2 py-vt-2">
                    <div class="flex items-center justify-between gap-vt-2">
                      <span class="font-medium text-primary">{{ exportKindLabel(record.exportKind) }}</span>
                      <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-secondary">{{ exportStatusLabel(record.status) }}</span>
                    </div>
                    <div class="mt-vt-1 break-all font-mono text-[11px] text-muted">{{ record.targetRelativePath ?? t('compositionGeneration.exportStatus.noTarget') }}</div>
                    <div class="mt-vt-2 flex items-center justify-between gap-vt-2 text-[11px] text-muted">
                      <span>{{ record.finishedAt ?? record.startedAt }}</span>
                      <button type="button" class="rounded-vt-sm border border-border px-vt-2 py-0.5 text-secondary transition hover:bg-card-hover hover:text-primary" @click="handleOpenOutputDir(record.exportId)">{{ t('compositionGeneration.exportHistory.open') }}</button>
                    </div>
                    <div v-if="record.errorJson" class="mt-vt-1 break-all text-[11px] text-danger">{{ formatExportError(record.errorJson) }}</div>
                  </div>
                </div>
              </div>
            </div>
          </aside>
        </section>
      </main>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { useProjectStore } from '@/entities/project/store'
import { useStoryboardStore } from '@/entities/storyboard/store'
import type { StoryboardItemDto } from '@/entities/storyboard/types'
import { getLatestStoryboardResetRecord, isResetRelevantToComposition } from '@/entities/storyboard/reset'
import { getRequiredWorkspaceStep, getWorkspaceStepAccess, getWorkspaceStepPath, type WorkspaceStepKey } from '@/features/workspace/steps'
import WorkspaceHeader from '@/features/workspace/WorkspaceHeader.vue'
import WorkspaceRowJump from '@/features/workspace/WorkspaceRowJump.vue'
import { backupWorkspace, exportFinalVideo, exportProjectPackage, listExportRecords, openExportDirectory } from '@/entities/export/api'
import type { BackupWorkspaceDto, ExportKind, ExportRecordDto, ExportStatus } from '@/entities/export/types'
import { importAsset, listAssets } from '@/entities/config/api'
import type { AssetDto } from '@/entities/config/types'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const { t } = useI18n()
const projectStore = useProjectStore()
const storyboardStore = useStoryboardStore()
const projectId = String(route.params.projectId)
const isComposing = ref(false)
const isExporting = ref(false)
const isExportingPackage = ref(false)
const isBackingUpWorkspace = ref(false)
const isOpeningDirectory = ref(false)
const isImportingBgm = ref(false)
const exportRecords = ref<ExportRecordDto[]>([])
const latestBackup = ref<BackupWorkspaceDto | null>(null)
const bgmAssets = ref<AssetDto[]>([])
const includeBgm = ref(false)
const selectedBgmAssetId = ref('')
const bgmVolume = ref(0.18)
const bgmLoop = ref(true)
const bgmFadeInSeconds = ref(1)
const bgmFadeOutSeconds = ref(1)
const bgmImportPath = ref('')
const includeSubtitle = ref(false)
const includeCoverMetadata = ref(false)

const storyboardItems = computed(() => storyboardStore.storyboard?.items ?? [])
const workspaceAccess = computed(() => getWorkspaceStepAccess(storyboardItems.value, storyboardStore.storyboard?.reviewStatus))
const projectTitle = computed(() => (projectStore.currentProject?.project.projectId === projectId ? projectStore.currentProject.project.title : projectId))
const compositionTask = computed(() => storyboardStore.currentCompositionTask)
const confirmedSegmentCount = computed(() => storyboardItems.value.filter((item) => selectedSegment(item)).length)
const probedSegmentCount = computed(() => storyboardItems.value.filter((item) => selectedSegment(item)?.mediaProbe).length)
const totalDurationSeconds = computed(() => storyboardItems.value.reduce((total, item) => total + (selectedSegment(item)?.durationSeconds ?? 0), 0))
const taskStatusText = computed(() => (compositionTask.value ? t(`dict.taskStatus.${compositionTask.value.status}`) : t('compositionGeneration.notStarted')))
const compositionResetCount = computed(() => storyboardItems.value.filter((item) => compositionResetRecord(item)).length)
const canExportFinalVideo = computed(() => compositionTask.value?.status === 'succeeded' && Boolean(compositionTask.value.outputPath))
const latestExportRecord = computed(() => exportRecords.value[0] ?? null)
const subtitlePath = computed(() => `projects/${projectId}/subtitles/subtitles.json`)
const enhancementStepSummary = computed(() => {
  const steps = compositionTask.value?.enhancements.steps
  if (!Array.isArray(steps)) return []
  return steps
    .map((entry) => {
      if (!entry || typeof entry !== 'object') return null
      const record = entry as Record<string, unknown>
      const key = typeof record.step === 'string' ? record.step : ''
      const status = typeof record.status === 'string' ? record.status : ''
      if (!key || !status) return null
      return {
        key,
        label: t(`compositionGeneration.optional.steps.${key}`),
        status: t(`compositionGeneration.optional.stepStatus.${status}`),
      }
    })
    .filter((entry): entry is { key: string; label: string; status: string } => Boolean(entry))
})
const resourceUsage = computed(() => ({
  images: storyboardItems.value.reduce((total, item) => total + item.imageCandidates.length, 0),
  videos: storyboardItems.value.reduce((total, item) => total + item.videoSegments.length, 0),
  llm: null,
}))

onMounted(async () => {
  await Promise.all([projectStore.loadProject(projectId), storyboardStore.loadStoryboard(projectId), loadExportRecords(), loadBgmAssets()])

  if (!workspaceAccess.value.canEnterComposition) {
    message.warning(t('workspaceStepBar.blocked.composition'))
    await router.replace(getWorkspaceStepPath(projectId, getRequiredWorkspaceStep('composition', workspaceAccess.value)))
  }
  hydrateBgmFromTask()
})

async function handleStartComposition() {
  if (!workspaceAccess.value.canEnterComposition) {
    message.warning(t('workspaceStepBar.blocked.composition'))
    return
  }

  isComposing.value = true
  try {
    if (includeBgm.value && !selectedBgmAssetId.value) {
      message.warning(t('compositionGeneration.bgm.selectRequired'))
      return
    }
    await storyboardStore.startComposition({
      projectId,
      includeSubtitle: includeSubtitle.value,
      subtitlePath: includeSubtitle.value ? subtitlePath.value : null,
      includeBgm: includeBgm.value,
      bgmAssetId: includeBgm.value ? selectedBgmAssetId.value : null,
      bgmVolume: clampNumber(bgmVolume.value, 0, 0.6, 0.18),
      bgmLoop: bgmLoop.value,
      bgmFadeInSeconds: clampNumber(bgmFadeInSeconds.value, 0, 30, 0),
      bgmFadeOutSeconds: clampNumber(bgmFadeOutSeconds.value, 0, 30, 0),
      includeCoverMetadata: includeCoverMetadata.value,
      coverPath: includeCoverMetadata.value ? (projectStore.currentProject?.project.coverPath ?? null) : null,
    })
    await loadExportRecords()
    hydrateBgmFromTask()
    message.success(t('compositionGeneration.composeSuccess'))
  } finally {
    isComposing.value = false
  }
}

async function handleImportBgm() {
  if (!bgmImportPath.value) return
  isImportingBgm.value = true
  try {
    const asset = await importAsset({
      sourcePath: bgmImportPath.value,
      kind: 'bgm',
      displayName: bgmImportPath.value.split(/[\\/]/).pop() || 'bgm',
      metadata: { usage: 'composition_bgm' },
    })
    await loadBgmAssets()
    selectedBgmAssetId.value = asset.assetId
    includeBgm.value = true
    bgmImportPath.value = ''
    message.success(t('compositionGeneration.bgm.importSuccess'))
  } catch (error) {
    message.error(t('compositionGeneration.bgm.importFailed', { reason: errorMessage(error) }))
  } finally {
    isImportingBgm.value = false
  }
}

async function handleExportFinalVideo() {
  if (!canExportFinalVideo.value) {
    message.warning(t('compositionGeneration.exportStatus.needComposition'))
    return
  }

  isExporting.value = true
  try {
    const record = await exportFinalVideo({ projectId })
    exportRecords.value = [record, ...exportRecords.value.filter((entry) => entry.exportId !== record.exportId)]
    message.success(t('compositionGeneration.exportStatus.success', { path: record.targetRelativePath ?? '-' }))
  } catch (error) {
    message.error(t('compositionGeneration.exportStatus.failed', { reason: errorMessage(error) }))
  } finally {
    isExporting.value = false
  }
}

async function handleExportProjectPackage() {
  isExportingPackage.value = true
  try {
    const record = await exportProjectPackage({ projectId })
    exportRecords.value = [record, ...exportRecords.value.filter((entry) => entry.exportId !== record.exportId)]
    message.success(t('compositionGeneration.exportStatus.packageSuccess', { path: record.targetRelativePath ?? '-' }))
  } catch (error) {
    message.error(t('compositionGeneration.exportStatus.failed', { reason: errorMessage(error) }))
  } finally {
    isExportingPackage.value = false
  }
}

async function handleBackupWorkspace() {
  isBackingUpWorkspace.value = true
  try {
    const backup = await backupWorkspace()
    latestBackup.value = backup
    message.success(t('compositionGeneration.backupStatus.success', { path: backup.targetRelativePath }))
  } catch (error) {
    message.error(t('compositionGeneration.backupStatus.failed', { reason: errorMessage(error) }))
  } finally {
    isBackingUpWorkspace.value = false
  }
}

async function handleOpenOutputDir(exportId?: string) {
  if (!exportId) {
    message.warning(t('compositionGeneration.exportStatus.noExportRecord'))
    return
  }

  isOpeningDirectory.value = true
  try {
    const directory = await openExportDirectory({ exportId })
    message.success(t('compositionGeneration.exportStatus.directoryReady', { path: directory.directoryRelativePath }))
  } catch (error) {
    message.error(t('compositionGeneration.exportStatus.openFailed', { reason: errorMessage(error) }))
  } finally {
    isOpeningDirectory.value = false
  }
}

async function loadExportRecords() {
  exportRecords.value = await listExportRecords({ projectId })
}

async function loadBgmAssets() {
  bgmAssets.value = await listAssets({ kind: 'bgm' })
  if (!selectedBgmAssetId.value && bgmAssets.value.length > 0) {
    selectedBgmAssetId.value = bgmAssets.value[0].assetId
  }
}

function hydrateBgmFromTask() {
  const enhancements = compositionTask.value?.enhancements ?? {}
  includeSubtitle.value = enhancements.includeSubtitle === true
  includeCoverMetadata.value = enhancements.includeCoverMetadata === true
  const taskIncludeBgm = enhancements.includeBgm === true
  includeBgm.value = taskIncludeBgm
  selectedBgmAssetId.value = typeof enhancements.bgmAssetId === 'string' ? enhancements.bgmAssetId : selectedBgmAssetId.value
  bgmVolume.value = typeof enhancements.bgmVolume === 'number' ? enhancements.bgmVolume : bgmVolume.value
  bgmLoop.value = typeof enhancements.bgmLoop === 'boolean' ? enhancements.bgmLoop : bgmLoop.value
  bgmFadeInSeconds.value = typeof enhancements.bgmFadeInSeconds === 'number' ? enhancements.bgmFadeInSeconds : bgmFadeInSeconds.value
  bgmFadeOutSeconds.value = typeof enhancements.bgmFadeOutSeconds === 'number' ? enhancements.bgmFadeOutSeconds : bgmFadeOutSeconds.value
}

function bgmAssetLabel(asset: AssetDto) {
  const displayName = typeof asset.metadata.displayName === 'string' ? asset.metadata.displayName : asset.relativePath.split('/').pop()
  return displayName ? `${displayName} · ${asset.relativePath}` : asset.relativePath
}

function clampNumber(value: number, min: number, max: number, fallback: number) {
  return Number.isFinite(value) ? Math.min(max, Math.max(min, value)) : fallback
}

function selectedSegment(item: StoryboardItemDto) {
  return item.videoSegments.find((segment) => segment.segmentId === item.selectedVideoSegmentId || segment.selected) ?? null
}

function compositionResetRecord(item: StoryboardItemDto) {
  const record = getLatestStoryboardResetRecord(item)
  return isResetRelevantToComposition(record) ? record : null
}

function resetReasonLabel(triggerField?: string) {
  return triggerField ? t(`compositionGeneration.downstreamReset.reasons.${triggerField}`) : t('compositionGeneration.downstreamReset.reasons.unknown')
}

function shortId(value?: string) {
  if (!value) return '-'
  return value.length > 12 ? value.slice(-12) : value
}

function probeSummary(segment: ReturnType<typeof selectedSegment>) {
  if (!segment?.mediaProbe) return t('compositionGeneration.probePending')
  const probe = segment.mediaProbe
  const size = probe.width && probe.height ? `${probe.width}x${probe.height}` : t('compositionGeneration.probeUnknown')
  const fps = probe.fps ? `${Math.round(probe.fps * 100) / 100}fps` : t('compositionGeneration.probeUnknown')
  const codec = probe.videoCodec ?? t('compositionGeneration.probeUnknown')
  return t('compositionGeneration.probeSummary', { codec, size, fps })
}

function exportStatusLabel(status: ExportStatus) {
  return t(`compositionGeneration.exportStatus.status.${status}`)
}

function exportKindLabel(kind: ExportKind) {
  return t(`compositionGeneration.exportStatus.kind.${kind}`)
}

function formatExportError(errorJson: Record<string, unknown>) {
  const code = typeof errorJson.errorCode === 'string' ? errorJson.errorCode : typeof errorJson.code === 'string' ? errorJson.code : ''
  const message = typeof errorJson.message === 'string' ? errorJson.message : JSON.stringify(errorJson)
  return code ? `${code}: ${message}` : message
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}

function handleBlockedStep(step: WorkspaceStepKey) {
  message.warning(t(`workspaceStepBar.blocked.${step}`))
}

function rowDomId(index: number) {
  return `composition-row-${index}`
}

function jumpToRow(index: number) {
  document.getElementById(rowDomId(index))?.scrollIntoView({ block: 'center', behavior: 'smooth' })
}
</script>
