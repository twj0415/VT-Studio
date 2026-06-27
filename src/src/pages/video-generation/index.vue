<template>
  <section class="view h-full min-w-0 overflow-hidden bg-page text-primary">
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      <WorkspaceHeader :project-id="projectId" :project-title="projectTitle" current-step="video" :access="workspaceAccess" :back-to="`/projects/${projectId}/workspace/image`" :badge-label="isMockVideoFlow ? t('videoGeneration.mockBadge') : ''" right-width-class="w-[500px]" :usage="resourceUsage" @blocked="handleBlockedStep">
        <template #actions>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="dirtyItemIds.size === 0 || isSavingAll" @click="handleSaveAll">{{ t('videoGeneration.saveAll') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isBulkGenerating || isBulkGeneratingTts" @click="handleGenerateMissingTts">{{ t('videoGeneration.generateMissingTts') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isBulkGenerating" @click="handleGenerateMissing">{{ t('videoGeneration.generateMissing') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isBulkGenerating" @click="handleGenerateAll">{{ t('videoGeneration.generateAll') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm bg-accent px-vt-4 text-sm font-semibold text-accent-ink transition hover:brightness-110" @click="handleEnterComposition">{{ t('videoGeneration.enterComposition') }}</button>
        </template>
      </WorkspaceHeader>

      <main class="flex min-h-0 flex-1 flex-col overflow-hidden bg-page">
        <section class="flex min-h-0 flex-1 flex-col gap-vt-3 overflow-hidden p-vt-3">
          <div class="flex flex-none flex-wrap items-center gap-vt-2 rounded-vt-md border border-border bg-card px-vt-3 py-vt-2 text-xs">
            <span class="font-medium text-secondary">{{ t('videoGeneration.toolbarTitle') }}</span>
            <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('videoGeneration.rowCount', { count: storyboardItems.length }) }}</span>
            <span v-if="dirtyItemIds.size > 0" class="rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-1 text-accent">{{ t('videoGeneration.dirtyCount', { count: dirtyItemIds.size }) }}</span>
            <span v-if="videoResetCount > 0" class="rounded-vt-sm border border-status-retrying/50 bg-status-retrying/10 px-vt-2 py-1 text-status-retrying">{{ t('videoGeneration.downstreamReset.count', { count: videoResetCount }) }}</span>
            <span v-if="bulkVideoLockedCount > 0" class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('videoGeneration.lockedSkipCount', { count: bulkVideoLockedCount }) }}</span>
            <button type="button" class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-secondary hover:border-border-strong hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isGeneratingSubtitles" @click="handleGenerateAllSubtitles">{{ t('videoGeneration.subtitle.generateAll') }}</button>
            <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('videoGeneration.subtitle.enabledOptional') }}</span>
            <WorkspaceRowJump class="ml-auto" :count="storyboardItems.length" @jump="jumpToRow" />
            <span class="text-muted">{{ t('videoGeneration.boundaryHint') }}</span>
          </div>

          <div class="min-h-0 flex-1 overflow-auto rounded-vt-md border border-border bg-card">
            <table class="w-full min-w-[2060px] table-fixed border-separate border-spacing-0 text-left text-sm">
              <thead class="sticky top-0 z-10 bg-panel text-xs text-muted">
                <tr>
                  <th class="w-[56px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.index') }}</th>
                  <th class="w-[190px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.selectedImage') }}</th>
                  <th class="w-[240px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.source') }}</th>
                  <th class="w-[340px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.videoPrompt') }}</th>
                  <th class="w-[120px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.duration') }}</th>
                  <th class="w-[200px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.audio') }}</th>
                  <th class="w-[220px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.subtitle') }}</th>
                  <th class="w-[300px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.segments') }}</th>
                  <th class="w-[180px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.selectedSegment') }}</th>
                  <th class="w-[120px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.status') }}</th>
                  <th class="w-[170px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('videoGeneration.columns.actions') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in storyboardItems" :id="rowDomId(item.index)" :key="item.itemId" class="h-[132px] align-top transition hover:bg-card-hover/70" :class="dirtyItemIds.has(item.itemId) ? 'bg-accent-soft/40' : ''">
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
                    <div class="grid gap-vt-1">
                      <button type="button" class="justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'videoPrompt')">{{ lockButtonLabel(item, 'videoPrompt') }}</button>
                      <n-input :value="item.videoPrompt" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :disabled="isItemLocked(item, 'videoPrompt')" :placeholder="t('videoGeneration.placeholders.videoPrompt')" @update:value="updateItem(item, { videoPrompt: $event }, 'videoPrompt')" />
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <n-input-number class="inp compact-inp" size="small" :min="1" :max="30" :step="1" :value="item.durationSeconds" @update:value="updateItem(item, { durationSeconds: normalizeDuration($event) })" />
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid h-[116px] gap-vt-1 rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5">
                      <div class="flex items-center justify-between gap-vt-2">
                        <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-secondary">{{ t(`dict.sceneAssetStatus.${item.audioStatus}`) }}</span>
                        <span v-if="item.audioRetryCount > 0" class="text-muted">{{ t('videoGeneration.audio.retryCount', { count: item.audioRetryCount }) }}</span>
                      </div>
                      <div class="truncate text-muted">{{ item.audioPath || t('videoGeneration.audio.noAudio') }}</div>
                      <div class="text-muted">{{ item.audioDurationSeconds ? t('videoGeneration.audio.duration', { seconds: item.audioDurationSeconds }) : t('videoGeneration.audio.noDuration') }}</div>
                      <div v-if="item.audioLastErrorJson" class="line-clamp-1 text-status-failed">{{ t('videoGeneration.audio.lastError', { reason: audioErrorReason(item) }) }}</div>
                      <div class="grid grid-cols-2 gap-vt-1">
                        <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="generatingTtsItemIds.has(item.itemId)" @click="handleGenerateTtsItem(item)">{{ item.audioPath ? t('videoGeneration.audio.regenerate') : t('videoGeneration.audio.generate') }}</n-button>
                        <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="probingAudioItemIds.has(item.itemId)" :disabled="!item.audioPath" @click="handleProbeAudioItem(item)">{{ t('videoGeneration.audio.probe') }}</n-button>
                      </div>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid h-[116px] grid-rows-[auto_1fr_auto] gap-vt-1 rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5">
                      <div class="flex items-center justify-between gap-vt-2">
                        <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-secondary">{{ t(`dict.sceneAssetStatus.${item.subtitleStatus}`) }}</span>
                        <span class="text-muted">{{ t('videoGeneration.subtitle.count', { count: item.subtitleChunks.length }) }}</span>
                      </div>
                      <div v-if="item.subtitleChunks.length > 0" class="truncate text-[11px] text-muted">{{ t('videoGeneration.subtitle.estimatedKaraoke') }}</div>
                      <div class="min-h-0 overflow-y-auto text-muted">
                        <div v-if="item.subtitleChunks.length > 0" class="grid gap-0.5">
                          <div v-for="chunk in item.subtitleChunks.slice(0, 3)" :key="chunk.chunkId" class="truncate">{{ chunk.text }}</div>
                        </div>
                        <div v-else class="grid h-full place-items-center text-center">{{ t('videoGeneration.subtitle.empty') }}</div>
                      </div>
                      <div class="grid grid-cols-2 gap-vt-1">
                        <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="generatingSubtitleItemIds.has(item.itemId)" @click="handleGenerateSubtitleItem(item)">{{ t('videoGeneration.subtitle.generate') }}</n-button>
                        <n-button class="btn btn-ghost btn-block compact-action" size="small" :disabled="item.subtitleChunks.length === 0" @click="openSubtitleEditor(item)">{{ t('videoGeneration.subtitle.edit') }}</n-button>
                      </div>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div v-if="item.videoSegments.length > 0" class="grid h-[116px] grid-cols-2 gap-vt-1 overflow-y-auto">
                      <div v-for="segment in sortedVideoSegments(item)" :key="segment.segmentId" class="grid min-h-[54px] grid-cols-[44px_1fr] items-center gap-vt-2 rounded-vt-sm border px-vt-2 py-vt-1 text-left text-[11px] leading-4 transition" :class="segmentButtonClass(item, segment)">
                        <span class="grid h-9 w-11 place-items-center rounded-vt-sm border border-border bg-card-hover text-[9px] font-semibold uppercase text-primary">{{ t('videoGeneration.segmentMockShort') }}</span>
                        <span class="min-w-0">
                          <span class="block truncate font-semibold">{{ shortId(segment.segmentId) }}</span>
                          <span class="block truncate text-muted">{{ t('videoGeneration.revisionLabel', { revision: segmentRevision(segment) }) }} · {{ segmentFreshnessLabel(item, segment) }}</span>
                          <span class="block truncate text-muted">{{ t('videoGeneration.segmentDuration', { seconds: segment.durationSeconds }) }}</span>
                          <span v-if="isSegmentSelected(item, segment)" class="block text-accent">{{ t('videoGeneration.confirmedMark') }}</span>
                          <span class="mt-1 flex gap-vt-1">
                            <button type="button" class="rounded-vt-sm border border-border bg-card px-vt-1.5 py-0.5 text-[10px] text-secondary hover:border-border-strong hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isItemLocked(item, 'selectedVideoSegment')" @click.stop="handleSelectSegment(item, segment)">{{ t('videoGeneration.selectSegment') }}</button>
                            <button type="button" class="rounded-vt-sm border border-border bg-card px-vt-1.5 py-0.5 text-[10px] text-secondary hover:border-border-strong hover:text-primary" @click.stop="handleOpenSnapshot(segment)">{{ t('videoGeneration.snapshot.view') }}</button>
                          </span>
                        </span>
                      </div>
                    </div>
                    <div v-else class="grid h-[116px] place-items-center rounded-vt-sm border border-border bg-page px-vt-2 text-center text-xs text-muted">{{ t('videoGeneration.noSegments') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="h-[116px] overflow-hidden rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">
                      <template v-if="selectedSegment(item)">
                        <button type="button" class="mb-vt-1 rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'selectedVideoSegment')">{{ lockButtonLabel(item, 'selectedVideoSegment') }}</button>
                        <div class="font-semibold text-primary">{{ shortId(selectedSegment(item)?.segmentId) }}</div>
                        <div class="mt-vt-1 line-clamp-4 text-muted">{{ selectedSegment(item)?.videoPath }}</div>
                      </template>
                      <template v-else>
                        <button type="button" class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'selectedVideoSegment')">{{ lockButtonLabel(item, 'selectedVideoSegment') }}</button>
                        <div class="grid h-full place-items-center text-center text-muted">{{ t('videoGeneration.notConfirmed') }}</div>
                      </template>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1 text-xs">
                      <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-vt-1 text-secondary">{{ t(`dict.sceneAssetStatus.${item.videoStatus}`) }}</span>
                      <span class="text-muted">{{ t('videoGeneration.segmentCount', { count: item.videoSegments.length }) }}</span>
                      <span v-if="videoResetRecord(item)" class="rounded-vt-sm border border-status-retrying/50 bg-status-retrying/10 px-vt-2 py-vt-1 text-status-retrying">{{ t('videoGeneration.downstreamReset.row', { reason: resetReasonLabel(videoResetRecord(item)?.triggerField) }) }}</span>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1">
                      <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="generatingItemIds.has(item.itemId)" @click="handleGenerateItem(item)">{{ item.videoSegments.length > 0 ? t('videoGeneration.regenerateRow') : t('videoGeneration.generateRow') }}</n-button>
                      <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="savingItemId === item.itemId" :disabled="!dirtyItemIds.has(item.itemId)" @click="handleSaveItem(item)">{{ t('videoGeneration.saveRow') }}</n-button>
                      <n-popconfirm :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleClearHistoricalSegments(item)">
                        <template #trigger>
                          <n-button class="btn btn-ghost btn-block compact-action" size="small" :disabled="!hasClearableVideoSegments(item)">{{ t('videoGeneration.clearHistorical') }}</n-button>
                        </template>
                        {{ t('videoGeneration.clearHistoricalConfirm') }}
                      </n-popconfirm>
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
    <n-modal v-model:show="isSnapshotOpen" preset="card" :title="t('videoGeneration.snapshot.title')" class="max-w-4xl">
      <div v-if="snapshotSegment" class="grid gap-vt-4 text-sm">
        <section class="grid grid-cols-2 gap-vt-2 text-xs md:grid-cols-4">
          <div class="rounded-vt-sm border border-border bg-page p-vt-3">
            <div class="text-muted">{{ t('videoGeneration.snapshot.segmentId') }}</div>
            <div class="mt-vt-1 truncate font-semibold text-primary">{{ shortId(snapshotSegment.segmentId) }}</div>
          </div>
          <div class="rounded-vt-sm border border-border bg-page p-vt-3">
            <div class="text-muted">{{ t('videoGeneration.snapshot.revision') }}</div>
            <div class="mt-vt-1 font-semibold text-primary">{{ segmentRevision(snapshotSegment) }}</div>
          </div>
          <div class="rounded-vt-sm border border-border bg-page p-vt-3">
            <div class="text-muted">{{ t('videoGeneration.snapshot.model') }}</div>
            <div class="mt-vt-1 truncate font-semibold text-primary">{{ snapshotText(snapshotSegment.generationContextSnapshot.modelSnapshot, 'label') || snapshotSegment.model }}</div>
          </div>
          <div class="rounded-vt-sm border border-border bg-page p-vt-3">
            <div class="text-muted">{{ t('videoGeneration.snapshot.inputImage') }}</div>
            <div class="mt-vt-1 truncate font-semibold text-primary">{{ shortId(String(snapshotSegment.generationContextSnapshot.inputImageId ?? snapshotSegment.inputImageId)) }}</div>
          </div>
        </section>
        <section class="grid gap-vt-2">
          <div class="text-xs font-semibold text-muted">{{ t('videoGeneration.snapshot.videoPrompt') }}</div>
          <pre class="max-h-32 overflow-auto whitespace-pre-wrap rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-secondary">{{ formatSnapshotJson(snapshotSegment.generationContextSnapshot.videoPromptSnapshot) }}</pre>
        </section>
        <section class="grid gap-vt-2">
          <div class="text-xs font-semibold text-muted">{{ t('videoGeneration.snapshot.inputImageSnapshot') }}</div>
          <pre class="max-h-48 overflow-auto whitespace-pre-wrap rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-secondary">{{ formatSnapshotJson(snapshotSegment.generationContextSnapshot.inputImageSnapshot) }}</pre>
        </section>
        <section class="grid gap-vt-2">
          <div class="text-xs font-semibold text-muted">{{ t('videoGeneration.snapshot.rawJson') }}</div>
          <pre class="max-h-80 overflow-auto whitespace-pre-wrap rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-secondary">{{ formatSnapshotJson(snapshotSegment.generationContextSnapshot) }}</pre>
        </section>
      </div>
    </n-modal>
    <n-modal v-model:show="isSubtitleEditorOpen" preset="card" :title="t('videoGeneration.subtitle.editorTitle')" class="max-w-3xl">
      <div v-if="subtitleEditingItem" class="grid gap-vt-4 text-sm">
        <section class="grid gap-vt-2 rounded-vt-sm border border-border bg-page p-vt-3 text-xs">
          <div class="flex flex-wrap items-center gap-vt-2">
            <span class="font-semibold text-primary">#{{ subtitleEditingItem.index.toString().padStart(2, '0') }}</span>
            <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ t('videoGeneration.subtitle.stylePreset') }}</span>
            <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ t('videoGeneration.subtitle.safeArea') }}</span>
            <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ t('videoGeneration.subtitle.karaokeMode') }}</span>
          </div>
          <div class="text-muted">{{ t('videoGeneration.subtitle.editorHint') }}</div>
        </section>
        <n-input v-model:value="subtitleEditorText" class="inp compact-inp" type="textarea" :autosize="{ minRows: 8, maxRows: 14 }" :placeholder="t('videoGeneration.subtitle.placeholder')" />
        <section class="grid gap-vt-2 rounded-vt-sm border border-border bg-page p-vt-3 text-xs">
          <div class="font-semibold text-secondary">{{ t('videoGeneration.subtitle.previewTitle') }}</div>
          <div class="aspect-[9/16] max-h-[260px] w-[146px] rounded-vt-sm border border-border bg-panel p-vt-2">
            <div class="relative h-full rounded-vt-sm border border-border bg-card-hover">
              <div class="absolute inset-x-vt-2 bottom-[40px] grid gap-1 text-center text-[11px] font-semibold text-white [text-shadow:0_1px_3px_rgba(0,0,0,.8)]">
                <span v-for="line in subtitleEditorPreviewLines" :key="line" class="rounded-vt-sm bg-black/30 px-vt-1 py-0.5">{{ line }}</span>
              </div>
            </div>
          </div>
        </section>
        <div class="flex justify-end gap-vt-2">
          <n-button class="btn btn-ghost" @click="isSubtitleEditorOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button class="btn btn-primary" :loading="isSavingSubtitles" @click="handleSaveSubtitles">{{ t('common.save') }}</n-button>
        </div>
      </div>
    </n-modal>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { NButton, NInput, NInputNumber, NModal, NPopconfirm, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { useProjectStore } from '@/entities/project/store'
import { useStoryboardStore } from '@/entities/storyboard/store'
import type { ImageCandidateDto, StoryboardItemDto, StoryboardItemLockField, SubtitleChunkDto, VideoSegmentDto } from '@/entities/storyboard/types'
import { validateStoryboardItemsForComposition, type StoryboardCompositionEntryField } from '@/entities/storyboard/validation'
import { getLatestStoryboardResetRecord, isResetRelevantToVideo, isStoryboardItemLocked, isStoryboardItemLockedForBulkVideoGeneration, lockedFieldsForVideoGeneration, setStoryboardItemLock } from '@/entities/storyboard/reset'
import { getRequiredWorkspaceStep, getWorkspaceStepAccess, getWorkspaceStepPath, type WorkspaceStepKey } from '@/features/workspace/steps'
import WorkspaceHeader from '@/features/workspace/WorkspaceHeader.vue'
import WorkspaceRowJump from '@/features/workspace/WorkspaceRowJump.vue'

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
const generatingTtsItemIds = ref<Set<string>>(new Set())
const probingAudioItemIds = ref<Set<string>>(new Set())
const generatingSubtitleItemIds = ref<Set<string>>(new Set())
const isSavingAll = ref(false)
const isBulkGenerating = ref(false)
const isBulkGeneratingTts = ref(false)
const isGeneratingSubtitles = ref(false)
const isSavingSubtitles = ref(false)
const isSnapshotOpen = ref(false)
const snapshotSegment = ref<VideoSegmentDto | null>(null)
const isSubtitleEditorOpen = ref(false)
const subtitleEditingItem = ref<StoryboardItemDto | null>(null)
const subtitleEditorText = ref('')

const storyboardItems = computed(() => storyboardStore.storyboard?.items ?? [])
const workspaceAccess = computed(() => getWorkspaceStepAccess(storyboardItems.value, storyboardStore.storyboard?.reviewStatus))
const projectTitle = computed(() => (projectStore.currentProject?.project.projectId === projectId ? projectStore.currentProject.project.title : projectId))
const isMockVideoFlow = computed(() => storyboardItems.value.some((item) => item.videoPrompt.startsWith('MOCK') || item.videoSegments.some((segment) => segment.providerModelId.startsWith('mock'))))
const videoResetCount = computed(() => storyboardItems.value.filter((item) => videoResetRecord(item)).length)
const bulkVideoLockedCount = computed(() => storyboardItems.value.filter(isStoryboardItemLockedForBulkVideoGeneration).length)
const subtitleEditorPreviewLines = computed(() => subtitleEditorText.value.split('\n').map((line) => line.trim()).filter(Boolean).slice(0, 2))
const resourceUsage = computed(() => ({
  images: storyboardItems.value.reduce((total, item) => total + item.imageCandidates.length, 0),
  videos: storyboardItems.value.reduce((total, item) => total + item.videoSegments.length, 0),
  llm: null,
}))

onMounted(async () => {
  await Promise.all([projectStore.loadProject(projectId), storyboardStore.loadStoryboard(projectId)])

  if (!workspaceAccess.value.canEnterVideo) {
    message.warning(t('workspaceStepBar.blocked.video'))
    await router.replace(getWorkspaceStepPath(projectId, getRequiredWorkspaceStep('video', workspaceAccess.value)))
  }
})

function updateItem(item: StoryboardItemDto, patch: Partial<StoryboardItemDto>, lockedField?: StoryboardItemLockField) {
  if (lockedField && isItemLocked(item, lockedField)) {
    showLockedWarning(lockedField)
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

function showLockedWarning(field: StoryboardItemLockField) {
  message.warning(t('videoGeneration.lockedActionBlocked', { field: lockFieldLabel(field) }))
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
  const lockedFields = lockedFieldsForVideoGeneration(item)
  if (lockedFields.length > 0) {
    showLockedWarning(lockedFields[0])
    return
  }
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

async function handleGenerateTtsItem(item: StoryboardItemDto) {
  await saveDirtyItemIfNeeded(item)
  setGeneratingTts(item.itemId, true)
  try {
    await storyboardStore.generateTts({ projectId, itemId: item.itemId })
    message.success(t('videoGeneration.audio.generateSuccess', { index: item.index }))
  } finally {
    setGeneratingTts(item.itemId, false)
  }
}

async function handleProbeAudioItem(item: StoryboardItemDto) {
  if (!item.audioPath) {
    message.warning(t('videoGeneration.audio.noAudio'))
    return
  }
  setProbingAudio(item.itemId, true)
  try {
    await storyboardStore.probeAudio({ projectId, itemId: item.itemId })
    message.success(t('videoGeneration.audio.probeSuccess', { index: item.index }))
  } finally {
    setProbingAudio(item.itemId, false)
  }
}

async function handleGenerateSubtitleItem(item: StoryboardItemDto) {
  await saveDirtyItemIfNeeded(item)
  setGeneratingSubtitle(item.itemId, true)
  try {
    const result = await storyboardStore.generateSubtitles({ projectId, itemIds: [item.itemId] })
    message.success(t('videoGeneration.subtitle.generateSuccess', { index: item.index, path: result.subtitlePath }))
  } finally {
    setGeneratingSubtitle(item.itemId, false)
  }
}

async function handleGenerateAllSubtitles() {
  if (storyboardItems.value.length === 0) {
    message.info(t('videoGeneration.empty'))
    return
  }
  isGeneratingSubtitles.value = true
  try {
    await handleSaveAll()
    const result = await storyboardStore.generateSubtitles({ projectId })
    message.success(t('videoGeneration.subtitle.generateAllSuccess', { count: result.items.length, path: result.subtitlePath }))
  } finally {
    isGeneratingSubtitles.value = false
  }
}

function openSubtitleEditor(item: StoryboardItemDto) {
  subtitleEditingItem.value = item
  subtitleEditorText.value = item.subtitleChunks.map((chunk) => chunk.text).join('\n')
  isSubtitleEditorOpen.value = true
}

async function handleSaveSubtitles() {
  const item = subtitleEditingItem.value
  if (!item) return
  const lines = subtitleEditorText.value.split('\n').map((line) => line.trim()).filter(Boolean)
  if (lines.length === 0) {
    message.warning(t('videoGeneration.subtitle.emptyEdit'))
    return
  }
  isSavingSubtitles.value = true
  try {
    await storyboardStore.updateSubtitles({
      projectId,
      itemId: item.itemId,
      subtitleChunks: lines.map((text, index) => ({
        chunkId: `edited_${item.itemId}_${index + 1}`,
        text,
        startSeconds: null,
        endSeconds: null,
        estimated: true,
      } satisfies SubtitleChunkDto)),
    })
    isSubtitleEditorOpen.value = false
    message.success(t('videoGeneration.subtitle.saveSuccess', { index: item.index }))
  } finally {
    isSavingSubtitles.value = false
  }
}

async function handleGenerateMissingTts() {
  const items = storyboardItems.value.filter((item) => !item.audioPath)
  if (items.length === 0) {
    message.info(t('videoGeneration.audio.noMissingRows'))
    return
  }

  isBulkGeneratingTts.value = true
  let successCount = 0
  let failedCount = 0
  try {
    await handleSaveAll()
    for (const item of items) {
      setGeneratingTts(item.itemId, true)
      try {
        await storyboardStore.generateTts({ projectId, itemId: item.itemId })
        successCount += 1
      } catch {
        failedCount += 1
      } finally {
        setGeneratingTts(item.itemId, false)
      }
    }
    if (failedCount > 0) message.warning(t('videoGeneration.audio.bulkGeneratePartial', { success: successCount, failed: failedCount }))
    else message.success(t('videoGeneration.audio.bulkGenerateSuccess', { count: successCount }))
  } finally {
    generatingTtsItemIds.value = new Set()
    isBulkGeneratingTts.value = false
  }
}

async function handleGenerateAll() {
  await generateItems(storyboardItems.value.filter((item) => !isStoryboardItemLockedForBulkVideoGeneration(item)))
}

async function handleGenerateMissing() {
  await generateItems(storyboardItems.value.filter((item) => item.videoSegments.length === 0 && !isStoryboardItemLockedForBulkVideoGeneration(item)))
}

async function generateItems(items: StoryboardItemDto[]) {
  if (items.length === 0) {
    message.info(t(bulkVideoLockedCount.value > 0 ? 'videoGeneration.noRowsAfterLockSkip' : 'videoGeneration.noMissingRows'))
    return
  }

  isBulkGenerating.value = true
  let successCount = 0
  let failedCount = 0
  try {
    await handleSaveAll()
    for (const item of items) {
      if (!item.selectedImageId) continue
      setGenerating(item.itemId, true)
      try {
        await storyboardStore.generateVideos(projectId, item.itemId)
        successCount += 1
      } catch (error) {
        failedCount += 1
      } finally {
        setGenerating(item.itemId, false)
      }
    }
    if (failedCount > 0) message.warning(t('videoGeneration.bulkGeneratePartial', { success: successCount, failed: failedCount }))
    else message.success(t('videoGeneration.bulkGenerateSuccess', { count: successCount }))
  } finally {
    generatingItemIds.value = new Set()
    isBulkGenerating.value = false
  }
}

async function handleSelectSegment(item: StoryboardItemDto, segment: VideoSegmentDto) {
  if (isItemLocked(item, 'selectedVideoSegment')) {
    showLockedWarning('selectedVideoSegment')
    return
  }
  await storyboardStore.selectVideo(item.itemId, segment.segmentId)
  message.success(t('videoGeneration.selectSuccess', { index: item.index }))
}

function handleOpenSnapshot(segment: VideoSegmentDto) {
  snapshotSegment.value = segment
  isSnapshotOpen.value = true
}

async function handleClearHistoricalSegments(item: StoryboardItemDto) {
  await storyboardStore.clearOldVideoSegments(item.itemId)
  message.success(t('videoGeneration.clearHistoricalSuccess', { index: item.index }))
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

function setGeneratingTts(itemId: string, loading: boolean) {
  const next = new Set(generatingTtsItemIds.value)
  if (loading) next.add(itemId)
  else next.delete(itemId)
  generatingTtsItemIds.value = next
}

function setProbingAudio(itemId: string, loading: boolean) {
  const next = new Set(probingAudioItemIds.value)
  if (loading) next.add(itemId)
  else next.delete(itemId)
  probingAudioItemIds.value = next
}

function setGeneratingSubtitle(itemId: string, loading: boolean) {
  const next = new Set(generatingSubtitleItemIds.value)
  if (loading) next.add(itemId)
  else next.delete(itemId)
  generatingSubtitleItemIds.value = next
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

function sortedVideoSegments(item: StoryboardItemDto) {
  return [...item.videoSegments].sort((left, right) => {
    const revisionDelta = segmentRevision(right) - segmentRevision(left)
    if (revisionDelta !== 0) return revisionDelta
    return segmentVariantIndex(left) - segmentVariantIndex(right)
  })
}

function isSegmentSelected(item: StoryboardItemDto, segment: VideoSegmentDto) {
  return item.selectedVideoSegmentId === segment.segmentId || segment.selected
}

function hasClearableVideoSegments(item: StoryboardItemDto) {
  const latestRevision = latestSegmentRevision(item)
  return item.videoSegments.some((segment) => segmentRevision(segment) < latestRevision && !isSegmentSelected(item, segment))
}

function segmentFreshnessLabel(item: StoryboardItemDto, segment: VideoSegmentDto) {
  return segmentRevision(segment) === latestSegmentRevision(item) ? t('videoGeneration.latestSegment') : t('videoGeneration.historicalSegment')
}

function latestSegmentRevision(item: StoryboardItemDto) {
  return Math.max(0, ...item.videoSegments.map(segmentRevision))
}

function segmentRevision(segment: VideoSegmentDto) {
  const revision = segment.generationContextSnapshot.revision
  return typeof revision === 'number' && Number.isFinite(revision) ? revision : 1
}

function segmentVariantIndex(segment: VideoSegmentDto) {
  const variantIndex = segment.generationContextSnapshot.variantIndex
  return typeof variantIndex === 'number' && Number.isFinite(variantIndex) ? variantIndex : 1
}

function segmentButtonClass(item: StoryboardItemDto, segment: VideoSegmentDto) {
  return isSegmentSelected(item, segment) ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary hover:border-border-strong hover:text-primary'
}

function videoResetRecord(item: StoryboardItemDto) {
  const record = getLatestStoryboardResetRecord(item)
  return isResetRelevantToVideo(record) ? record : null
}

function resetReasonLabel(triggerField?: string) {
  return triggerField ? t(`videoGeneration.downstreamReset.reasons.${triggerField}`) : t('videoGeneration.downstreamReset.reasons.unknown')
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

function snapshotText(source: unknown, key: string) {
  if (!source || typeof source !== 'object') return ''
  const value = (source as Record<string, unknown>)[key]
  return typeof value === 'string' ? value : ''
}

function formatSnapshotJson(value: unknown) {
  return JSON.stringify(value ?? null, null, 2)
}

function audioErrorReason(item: StoryboardItemDto) {
  return snapshotText(item.audioLastErrorJson, 'message') || snapshotText(item.audioLastErrorJson, 'errorCode') || t('videoGeneration.audio.unknownError')
}

function showFirstCompositionIssue(issue: { index: number; fields: StoryboardCompositionEntryField[] }) {
  const fields = issue.fields.map((field) => t(`videoGeneration.validation.fields.${field}`)).join('、')
  message.error(t('videoGeneration.validation.enterCompositionBlocked', { index: issue.index, fields }))
}

function handleBlockedStep(step: WorkspaceStepKey) {
  message.warning(t(`workspaceStepBar.blocked.${step}`))
}

function rowDomId(index: number) {
  return `video-row-${index}`
}

function jumpToRow(index: number) {
  document.getElementById(rowDomId(index))?.scrollIntoView({ block: 'center', behavior: 'smooth' })
}
</script>
