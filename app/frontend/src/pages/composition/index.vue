<template>
  <section class="view h-full min-w-0 overflow-hidden bg-page text-primary">
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      <header class="flex h-12 flex-none items-center border-b border-border bg-panel px-vt-3">
        <div class="flex w-[320px] flex-none items-center gap-vt-3">
          <button type="button" class="grid size-8 flex-none place-items-center rounded-vt-sm border border-border text-sm text-muted transition hover:border-border-strong hover:bg-card hover:text-primary" @click="router.push(`/projects/${projectId}/workspace/video`)">←</button>
          <div class="flex min-w-0 items-center gap-vt-2">
            <div class="min-w-0 truncate text-base font-semibold">{{ projectTitle }}</div>
            <span class="flex-none rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-0.5 text-[11px] font-medium text-accent">{{ t('compositionGeneration.mockBadge') }}</span>
          </div>
        </div>
        <div class="flex min-w-0 flex-1 items-center justify-center">
          <WorkspaceStepBar class="hidden min-w-0 justify-center lg:flex" compact :project-id="projectId" current-step="composition" :access="workspaceAccess" @blocked="handleBlockedStep" />
        </div>
        <div class="flex w-[360px] flex-none items-center justify-end gap-vt-2">
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary" @click="handleOpenOutputDir">{{ t('compositionGeneration.openOutputDir') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm bg-accent px-vt-4 text-sm font-semibold text-accent-ink transition hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-50" :disabled="isComposing" @click="handleStartComposition">{{ t('compositionGeneration.startComposition') }}</button>
        </div>
      </header>

      <main class="flex min-h-0 flex-1 flex-col overflow-hidden bg-page">
        <section class="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_340px] gap-vt-3 overflow-hidden p-vt-3 max-xl:grid-cols-1">
          <div class="flex min-h-0 flex-col gap-vt-3 overflow-hidden">
            <div class="flex flex-none flex-wrap items-center gap-vt-2 rounded-vt-md border border-border bg-card px-vt-3 py-vt-2 text-xs">
              <span class="font-medium text-secondary">{{ t('compositionGeneration.checkTitle') }}</span>
              <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('compositionGeneration.segmentCount', { count: confirmedSegmentCount }) }}</span>
              <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('compositionGeneration.totalDuration', { seconds: totalDurationSeconds }) }}</span>
              <span class="ml-auto text-muted">{{ t('compositionGeneration.boundaryHint') }}</span>
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
                  <tr v-for="item in storyboardItems" :key="item.itemId" class="h-[96px] align-top transition hover:bg-card-hover/70">
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
                        </template>
                        <template v-else>
                          <div class="grid h-full place-items-center text-muted">{{ t('compositionGeneration.missingSegment') }}</div>
                        </template>
                      </div>
                    </td>
                    <td class="border-b border-border px-vt-2 py-vt-2 text-xs text-secondary">{{ selectedSegment(item)?.durationSeconds ?? '-' }}s</td>
                    <td class="border-b border-border px-vt-2 py-vt-2">
                      <span class="inline-flex rounded-vt-sm border px-vt-2 py-vt-1 text-xs" :class="selectedSegment(item) ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-muted'">{{ selectedSegment(item) ? t('compositionGeneration.ready') : t('compositionGeneration.missing') }}</span>
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
                <div class="text-muted">{{ t('compositionGeneration.includedSegments') }}</div>
                <div class="mt-vt-1 text-primary">{{ compositionTask?.segmentIds.length ?? 0 }}</div>
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
import WorkspaceStepBar from '@/features/workspace/WorkspaceStepBar.vue'
import { getRequiredWorkspaceStep, getWorkspaceStepAccess, getWorkspaceStepPath, type WorkspaceStepKey } from '@/features/workspace/steps'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const { t } = useI18n()
const projectStore = useProjectStore()
const storyboardStore = useStoryboardStore()
const projectId = String(route.params.projectId)
const isComposing = ref(false)

const storyboardItems = computed(() => storyboardStore.storyboard?.items ?? [])
const workspaceAccess = computed(() => getWorkspaceStepAccess(storyboardItems.value, storyboardStore.storyboard?.reviewStatus))
const projectTitle = computed(() => (projectStore.currentProject?.project.projectId === projectId ? projectStore.currentProject.project.title : projectId))
const compositionTask = computed(() => storyboardStore.currentCompositionTask)
const confirmedSegmentCount = computed(() => storyboardItems.value.filter((item) => selectedSegment(item)).length)
const totalDurationSeconds = computed(() => storyboardItems.value.reduce((total, item) => total + (selectedSegment(item)?.durationSeconds ?? 0), 0))
const taskStatusText = computed(() => (compositionTask.value ? t(`dict.taskStatus.${compositionTask.value.status}`) : t('compositionGeneration.notStarted')))

onMounted(async () => {
  await Promise.all([projectStore.loadProject(projectId), storyboardStore.loadStoryboard(projectId)])

  if (!workspaceAccess.value.canEnterComposition) {
    message.warning(t('workspaceStepBar.blocked.composition'))
    await router.replace(getWorkspaceStepPath(projectId, getRequiredWorkspaceStep('composition', workspaceAccess.value)))
  }
})

async function handleStartComposition() {
  if (!workspaceAccess.value.canEnterComposition) {
    message.warning(t('workspaceStepBar.blocked.composition'))
    return
  }

  isComposing.value = true
  try {
    await storyboardStore.startComposition(projectId)
    message.success(t('compositionGeneration.composeSuccess'))
  } finally {
    isComposing.value = false
  }
}

function handleOpenOutputDir() {
  message.info(t('compositionGeneration.openOutputDirPlaceholder'))
}

function selectedSegment(item: StoryboardItemDto) {
  return item.videoSegments.find((segment) => segment.segmentId === item.selectedVideoSegmentId || segment.selected) ?? null
}

function shortId(value?: string) {
  if (!value) return '-'
  return value.length > 12 ? value.slice(-12) : value
}

function handleBlockedStep(step: WorkspaceStepKey) {
  message.warning(t(`workspaceStepBar.blocked.${step}`))
}
</script>
