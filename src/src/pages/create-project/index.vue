<template>
  <section class="view h-full w-full min-w-0 overflow-hidden bg-page text-primary">
    <header class="flex h-16 w-full flex-none items-center gap-vt-4 border-b border-border bg-panel px-vt-6">
      <button type="button" class="flex items-center gap-vt-2 text-sm text-secondary transition hover:text-primary" @click="router.push('/')">
        <span aria-hidden="true">←</span>
        <span>{{ t('common.back') }}</span>
      </button>
      <div class="min-w-0">
        <h2 class="truncate text-base font-semibold">{{ t('createProject.title') }}</h2>
        <div class="text-xs text-muted">{{ t('createProject.stepInfo', { current: 1, total: 2 }) }}</div>
      </div>
      <div class="ml-auto flex rounded-vt-sm border border-border bg-page p-1">
        <button v-for="mode in createModes" :key="mode.value" type="button" :disabled="mode.disabled" class="rounded-vt-sm px-vt-4 py-vt-2 text-sm transition" :class="modePillClass(mode.value === selectedMode, mode.disabled)" @click="selectMode(mode)">
          {{ mode.label }}
        </button>
      </div>
    </header>

    <main class="min-h-0 w-full flex-1 overflow-hidden">
      <div class="flex h-full w-full min-w-0 flex-col gap-vt-5 p-vt-5 lg:p-vt-6 xl:flex-row">
        <section class="flex min-h-0 w-full min-w-0 flex-1 flex-col rounded-vt-md border border-border bg-card shadow-vt-md">
          <div class="flex flex-none items-start justify-between gap-vt-4 border-b border-border p-vt-5">
            <div>
              <h3 class="text-base font-semibold">{{ t('createProject.textImportTitle') }}</h3>
              <p class="mt-vt-1 text-sm text-muted">{{ t('createProject.textImportHint') }}</p>
            </div>
            <span class="rounded-vt-sm border border-accent-line bg-accent-soft px-vt-3 py-vt-1 text-xs text-accent">{{ t('createProject.currentMainline') }}</span>
          </div>

          <div class="flex min-h-0 flex-1 flex-col gap-vt-4 p-vt-5">
            <div class="grid flex-none gap-vt-2 md:grid-cols-3">
              <button v-for="entry in textEntries" :key="entry.value" type="button" class="min-h-20 rounded-vt-md border p-vt-4 text-left transition" :class="entryCardClass(entry.value === form.inputType)" @click="selectEntry(entry.value)">
                <span class="block text-sm font-semibold text-primary">{{ entry.label }}</span>
                <span class="mt-vt-1 block text-xs text-muted">{{ t(`createProject.entrySub.${entry.value}`) }}</span>
              </button>
            </div>

            <label class="flex min-h-0 flex-1 flex-col">
              <span class="mb-vt-2 block text-xs text-secondary">{{ inputLabel }}</span>
              <n-input v-model:value="form.sourceText" class="inp min-h-0 flex-1" type="textarea" :autosize="{ minRows: form.inputType === 'topic' ? 12 : 20, maxRows: 32 }" :placeholder="inputPlaceholder" />
              <span class="mt-vt-2 block text-xs text-muted">{{ t('createProject.textLength', { count: sourceTextLength }) }}</span>
            </label>

            <div v-if="form.inputType === 'paste'" class="flex flex-none flex-wrap items-center gap-vt-2 rounded-vt-sm border border-accent-line bg-accent-soft p-vt-3 text-xs text-secondary">
              <span class="font-medium text-accent">{{ t('createProject.fixedOriginalTitle') }}</span>
              <span>{{ t('createProject.fixedOriginalDesc') }}</span>
              <span class="rounded-vt-sm border border-accent-line bg-page px-vt-2 py-vt-1 text-accent">{{ splitModeLabel }}</span>
              <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-vt-1 text-muted">{{ t('createProject.noAiRewrite') }}</span>
            </div>

            <div v-if="showSplitPreview" class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-vt-sm border border-border bg-page">
              <div class="flex flex-none flex-wrap items-center gap-vt-2 border-b border-border px-vt-3 py-vt-2 text-xs">
                <span class="font-semibold text-primary">{{ t('createProject.splitPreview.title') }}</span>
                <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-vt-1 text-muted">{{ t('createProject.splitPreview.count', { count: segmentDrafts.length }) }}</span>
                <div class="ml-auto flex flex-wrap items-center gap-vt-1">
                  <button v-for="mode in splitModeOptions" :key="mode.value" type="button" class="rounded-vt-sm border px-vt-2 py-vt-1 transition" :class="splitMode === mode.value ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-card text-secondary hover:border-border-strong hover:text-primary'" @click="setSplitMode(mode.value)">
                    {{ mode.label }}
                  </button>
                </div>
              </div>
              <div class="min-h-0 flex-1 overflow-y-auto p-vt-3">
                <div v-if="longTextWarning" class="mb-vt-3 rounded-vt-sm border border-status-retrying/40 bg-status-retrying/10 px-vt-3 py-vt-2 text-xs leading-5 text-status-retrying">
                  {{ t('createProject.splitPreview.longTextWarning', { count: sourceTextLength }) }}
                </div>
                <div v-if="segmentDrafts.length === 0" class="rounded-vt-sm border border-dashed border-border p-vt-5 text-center text-xs text-muted">
                  {{ t('createProject.splitPreview.empty') }}
                </div>
                <div v-else class="grid gap-vt-2">
                  <div v-for="(segment, index) in segmentDrafts" :key="segment.id" class="rounded-vt-sm border border-border bg-card p-vt-3">
                    <div class="mb-vt-2 flex items-center gap-vt-2 text-xs">
                      <span class="grid size-6 place-items-center rounded-vt-sm border border-border bg-page text-muted">#{{ index + 1 }}</span>
                      <span class="text-muted">{{ t('createProject.splitPreview.segmentLength', { count: segment.text.length }) }}</span>
                      <button type="button" class="ml-auto rounded-vt-sm border border-border px-vt-2 py-vt-1 text-secondary transition hover:bg-card-hover hover:text-primary" @click="splitSegment(index)">{{ t('createProject.splitPreview.split') }}</button>
                      <button type="button" class="rounded-vt-sm border border-border px-vt-2 py-vt-1 text-secondary transition hover:bg-card-hover hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="index === 0" @click="mergeSegment(index)">{{ t('createProject.splitPreview.mergePrev') }}</button>
                      <button type="button" class="rounded-vt-sm border border-border px-vt-2 py-vt-1 text-secondary transition hover:bg-card-hover hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="segmentDrafts.length <= 1" @click="removeSegment(index)">{{ t('createProject.splitPreview.remove') }}</button>
                    </div>
                    <n-input v-model:value="segment.text" class="inp" type="textarea" :autosize="{ minRows: 2, maxRows: 5 }" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <aside class="flex min-h-0 w-full flex-none flex-col gap-vt-5 overflow-y-auto xl:w-96">
          <section class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
            <div class="flex items-center gap-vt-3">
              <h3 class="text-sm font-semibold">{{ t('createProject.videoPackTitle') }}</h3>
              <n-button class="ml-auto" size="tiny" :loading="isLoadingPacks" @click="loadVideoPacks">{{ t('creativeResources.packs.refresh') }}</n-button>
            </div>
            <p class="mt-vt-2 text-xs leading-5 text-muted">{{ t('createProject.videoPackHint') }}</p>
            <div class="mt-vt-4 grid gap-vt-2">
              <button v-for="pack in selectablePacks" :key="pack.packId" type="button" class="rounded-vt-sm border p-vt-3 text-left transition" :class="entryCardClass(form.activePackId === pack.packId)" @click="selectVideoPack(pack)">
                <div class="flex min-w-0 items-center gap-vt-2">
                  <span class="min-w-0 truncate text-sm font-semibold text-primary">{{ pack.name }}</span>
                  <span class="ml-auto rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted">{{ pack.sourceType }}</span>
                </div>
                <p class="mt-vt-1 line-clamp-2 text-xs leading-5 text-muted">{{ pack.description }}</p>
                <div class="mt-vt-2 flex flex-wrap gap-vt-1 text-[11px] text-secondary">
                  <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-0.5">{{ pack.defaultAspectRatio }}</span>
                  <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-0.5">{{ t('createProject.packSceneCount', { count: pack.defaultSceneCount }) }}</span>
                  <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-0.5">{{ t('createProject.packDuration', { seconds: pack.defaultDurationSeconds }) }}</span>
                </div>
              </button>
              <div v-if="selectablePacks.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs text-muted">{{ t('createProject.noVideoPacks') }}</div>
            </div>
          </section>

          <section class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
            <h3 class="text-sm font-semibold">{{ t('createProject.contentSettingsTitle') }}</h3>
            <div class="mt-vt-4 space-y-vt-5">
              <label class="block">
                <span class="mb-vt-2 block text-xs text-secondary">{{ t('createProject.aspectRatio') }}</span>
                <div class="flex flex-wrap gap-vt-2">
                  <button v-for="option in aspectRatioOptions" :key="option.value" type="button" class="rounded-vt-sm px-vt-3 py-vt-2 text-sm transition" :class="pillClass(form.aspectRatio === option.value)" @click="form.aspectRatio = option.value">
                    {{ option.label }}
                  </button>
                </div>
              </label>

              <label class="block">
                <span class="mb-vt-2 block text-xs text-secondary">{{ t('createProject.contentLanguage') }}</span>
                <div class="flex flex-wrap gap-vt-2">
                  <button v-for="option in contentLanguageOptions" :key="option.value" type="button" class="rounded-vt-sm px-vt-3 py-vt-2 text-sm transition" :class="pillClass(form.contentLanguage === option.value)" @click="form.contentLanguage = option.value">
                    {{ option.label }}
                  </button>
                </div>
              </label>

              <label class="block">
                <span class="mb-vt-2 block text-xs text-secondary">{{ t('createProject.targetSceneCount') }}</span>
                <n-input-number class="inp w-full" :value="form.targetSceneCount" :min="1" :max="60" :step="1" @update:value="setTargetSceneCount" />
              </label>

              <label class="block">
                <span class="mb-vt-2 block text-xs text-secondary">{{ t('createProject.segmentDuration') }}</span>
                <div class="flex flex-wrap gap-vt-2">
                  <button v-for="duration in durationOptions" :key="duration" type="button" class="rounded-vt-sm px-vt-3 py-vt-2 text-sm transition" :class="pillClass(form.segmentDurationSeconds === duration)" @click="form.segmentDurationSeconds = duration">
                    {{ duration }}s
                  </button>
                </div>
              </label>

              <label class="block">
                <span class="mb-vt-2 block text-xs text-secondary">{{ t('createProject.stylePrompt') }}</span>
                <n-input v-model:value="form.stylePrompt" class="inp" :placeholder="t('createProject.stylePromptPlaceholder')" />
              </label>
            </div>
          </section>

          <section class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
            <h3 class="text-sm font-semibold">{{ t('createProject.afterImportTitle') }}</h3>
            <div class="mt-vt-4 grid gap-vt-2">
              <div v-for="(step, index) in mainlineSteps" :key="step" class="flex items-center gap-vt-3 rounded-vt-sm border px-vt-3 py-vt-3" :class="index === 0 ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary'">
                <span class="grid size-6 place-items-center rounded-full border text-xs" :class="index === 0 ? 'border-accent-line' : 'border-border-strong'">{{ index + 1 }}</span>
                <span class="text-sm">{{ step }}</span>
              </div>
            </div>
            <p class="mt-vt-4 text-xs leading-5 text-muted">{{ t('createProject.afterImportHint') }}</p>
          </section>
        </aside>
      </div>
    </main>

    <footer class="flex w-full flex-none items-center justify-end gap-vt-3 border-t border-border bg-panel px-vt-6 py-vt-4">
      <n-button class="btn btn-ghost" :disabled="isCreating" @click="router.push('/')">{{ t('createProject.cancel') }}</n-button>
      <n-button class="btn btn-primary" :loading="isCreating" :disabled="!canCreate" @click="handleCreate">{{ t('createProject.importAndEnterWorkbench') }} →</n-button>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { NButton, NInput, NInputNumber, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { listVideoPacks } from '@/entities/config/api'
import type { VideoPackDto } from '@/entities/config/types'
import { useProjectStore } from '@/entities/project/store'
import type { CreateProjectRequest } from '@/entities/project/types'
import { useDictOptions } from '@/shared/dict/useDictOptions'
import type { AspectRatio, ContentLanguage, InputProcessMode, InputType } from '@/shared/enums/generated'

type CreateMode = 'text' | 'image' | 'video'
type SplitMode = 'paragraph' | 'line' | 'sentence'

interface CreateProjectForm {
  inputType: InputType
  contentLanguage: ContentLanguage
  aspectRatio: AspectRatio
  targetSceneCount: number
  segmentDurationSeconds: number
  stylePrompt: string
  sourceText: string
  activePackId?: string
}

interface CreateModeOption {
  label: string
  value: CreateMode
  disabled: boolean
}

interface SegmentDraft {
  id: string
  text: string
}

const router = useRouter()
const projectStore = useProjectStore()
const message = useMessage()
const { t } = useI18n()

const selectedMode = ref<CreateMode>('text')
const isCreating = ref(false)
const isLoadingPacks = ref(false)
const videoPacks = ref<VideoPackDto[]>([])
const durationOptions = [4, 6, 8] as const
const textInputTypes: readonly InputType[] = ['topic', 'paste', 'article']
const splitMode = ref<SplitMode>('paragraph')
const segmentDrafts = ref<SegmentDraft[]>([])
let segmentDraftCounter = 0

const inputOptions = useDictOptions('inputType')
const aspectRatioOptions = useDictOptions('aspectRatio')
const contentLanguageOptions = useDictOptions('contentLanguage')

const form = reactive<CreateProjectForm>({
  inputType: 'topic',
  contentLanguage: 'zh-CN',
  aspectRatio: '9:16',
  targetSceneCount: 8,
  segmentDurationSeconds: 4,
  stylePrompt: '',
  sourceText: '',
  activePackId: undefined,
})

const createModes = computed<CreateModeOption[]>(() => [
  { label: t('createProject.createModes.text'), value: 'text', disabled: false },
  { label: t('createProject.createModes.image'), value: 'image', disabled: true },
  { label: t('createProject.createModes.video'), value: 'video', disabled: true },
])

const textEntries = computed(() => inputOptions.value.filter((option) => textInputTypes.includes(option.value)))
const selectablePacks = computed(() => videoPacks.value.filter((pack) => pack.isEnabled && pack.applicableInputTypes.includes(form.inputType)))
const mainlineSteps = computed(() => [t('storyboard.steps.storyboard'), t('storyboard.steps.image'), t('storyboard.steps.video'), t('storyboard.steps.composition')])
const sourceText = computed(() => form.sourceText.trim())
const sourceTextLength = computed(() => sourceText.value.length)
const inputProcessMode = computed<InputProcessMode>(() => (form.inputType === 'paste' ? 'fixed' : 'generate'))
const showSplitPreview = computed(() => form.inputType === 'paste' || form.inputType === 'article')
const confirmedSegmentTexts = computed(() => segmentDrafts.value.map((segment) => segment.text.trim()).filter(Boolean))
const confirmedSourceText = computed(() => (showSplitPreview.value ? confirmedSegmentTexts.value.join('\n\n') : sourceText.value))
const canCreate = computed(() => selectedMode.value === 'text' && confirmedSourceText.value.length > 0 && !isCreating.value)
const longTextWarning = computed(() => showSplitPreview.value && sourceTextLength.value >= 1200)
const splitModeOptions = computed(() => [
  { value: 'paragraph' as const, label: t('createProject.splitParagraph') },
  { value: 'line' as const, label: t('createProject.splitLine') },
  { value: 'sentence' as const, label: t('createProject.splitSentence') },
])
const splitModeLabel = computed(() => splitModeOptions.value.find((option) => option.value === splitMode.value)?.label ?? t('createProject.splitParagraph'))

const inputLabel = computed(() => {
  if (form.inputType === 'paste') return t('createProject.inputLabels.paste')
  if (form.inputType === 'article') return t('createProject.inputLabels.article')
  return t('createProject.inputLabels.topic')
})

const inputPlaceholder = computed(() => (form.inputType === 'topic' ? t('createProject.inputPlaceholderTopic') : t('createProject.inputPlaceholderContent')))

onMounted(loadVideoPacks)

watch(
  () => [form.sourceText, form.inputType, splitMode.value] as const,
  () => {
    if (!showSplitPreview.value) {
      segmentDrafts.value = []
      return
    }
    rebuildSegmentsFromSource()
  },
  { immediate: true },
)

function modePillClass(selected: boolean, disabled: boolean) {
  if (disabled) return 'cursor-not-allowed text-muted opacity-50'
  return selected ? 'bg-accent text-accent-ink' : 'text-secondary hover:bg-card hover:text-primary'
}

function entryCardClass(selected: boolean) {
  return selected ? 'border-accent-line bg-accent-soft' : 'border-border bg-page hover:border-border-strong hover:bg-card-hover'
}

function pillClass(selected: boolean) {
  return selected ? 'border border-accent-line bg-accent-soft text-accent' : 'border border-border bg-page text-secondary hover:border-border-strong hover:text-primary'
}

function selectMode(mode: CreateModeOption) {
  if (mode.disabled) return
  selectedMode.value = mode.value
}

function selectEntry(inputType: InputType) {
  form.inputType = inputType
  if (form.activePackId && !selectablePacks.value.some((pack) => pack.packId === form.activePackId)) {
    form.activePackId = undefined
  }
}

function setSplitMode(mode: SplitMode) {
  splitMode.value = mode
}

function setTargetSceneCount(value: number | null) {
  form.targetSceneCount = Math.min(60, Math.max(1, Math.round(value ?? 8)))
}

async function loadVideoPacks() {
  isLoadingPacks.value = true
  try {
    videoPacks.value = await listVideoPacks({ includeDisabled: false })
    if (!form.activePackId && selectablePacks.value[0]) {
      selectVideoPack(selectablePacks.value[0])
    }
  } catch (error) {
    message.error(error instanceof Error ? error.message : String(error))
  } finally {
    isLoadingPacks.value = false
  }
}

function selectVideoPack(pack: VideoPackDto) {
  form.activePackId = pack.packId
  form.aspectRatio = pack.defaultAspectRatio as AspectRatio
  form.targetSceneCount = pack.defaultSceneCount
  form.segmentDurationSeconds = Math.max(1, Math.round(pack.defaultDurationSeconds / Math.max(1, pack.defaultSceneCount)))
  if (!form.stylePrompt.trim() && pack.defaultTone) {
    form.stylePrompt = pack.defaultTone
  }
}

function toCreateProjectRequest(): CreateProjectRequest {
  return {
    title: '',
    workflowType: 'image_to_video',
    inputType: form.inputType,
    topic: form.inputType === 'topic' ? confirmedSourceText.value : undefined,
    sourceText: form.inputType !== 'topic' ? confirmedSourceText.value : undefined,
    contentLanguage: form.contentLanguage,
    aspectRatio: form.aspectRatio,
    targetSceneCount: form.targetSceneCount,
    segmentDurationSeconds: form.segmentDurationSeconds,
    stylePrompt: form.stylePrompt.trim() || undefined,
    activePackId: form.activePackId,
    inputProcessMode: inputProcessMode.value,
    inputOptions: showSplitPreview.value
      ? {
          splitMode: splitMode.value,
          confirmedSegments: confirmedSegmentTexts.value.map((text, index) => ({
            index: index + 1,
            sourceText: text,
            narrationText: text,
          })),
        }
      : undefined,
  }
}

async function handleCreate() {
  if (!canCreate.value) {
    message.warning(t('createProject.validation.fillRequired'))
    return
  }

  isCreating.value = true

  try {
    const detail = await projectStore.createDraftProject(toCreateProjectRequest())
    await router.push(`/projects/${detail.project.projectId}`)
  } catch (error) {
    message.error(error instanceof Error && error.message ? error.message : t('createProject.validation.createFailed'))
  } finally {
    isCreating.value = false
  }
}

function rebuildSegmentsFromSource() {
  segmentDrafts.value = splitSourceText(sourceText.value, splitMode.value).map((text) => createSegmentDraft(text))
}

function splitSourceText(value: string, mode: SplitMode) {
  const text = value.trim()
  if (!text) return []
  if (mode === 'line') return text.split(/\r?\n/).map((part) => part.trim()).filter(Boolean)
  if (mode === 'sentence') {
    const matches = text.match(/[^。！？!?；;\n]+[。！？!?；;]?/g) ?? [text]
    return matches.map((part) => part.trim()).filter(Boolean)
  }
  return text.split(/\n\s*\n+/).map((part) => part.trim()).filter(Boolean)
}

function createSegmentDraft(text: string): SegmentDraft {
  segmentDraftCounter += 1
  return {
    id: `segment_${segmentDraftCounter}`,
    text,
  }
}

function splitSegment(index: number) {
  const segment = segmentDrafts.value[index]
  if (!segment) return
  const text = segment.text.trim()
  const middle = Math.floor(text.length / 2)
  const splitAt = findSplitIndex(text, middle)
  if (splitAt <= 0 || splitAt >= text.length - 1) return
  segmentDrafts.value.splice(index, 1, createSegmentDraft(text.slice(0, splitAt).trim()), createSegmentDraft(text.slice(splitAt).trim()))
}

function mergeSegment(index: number) {
  if (index <= 0) return
  const previous = segmentDrafts.value[index - 1]
  const current = segmentDrafts.value[index]
  if (!previous || !current) return
  segmentDrafts.value.splice(index - 1, 2, createSegmentDraft(`${previous.text.trim()}\n${current.text.trim()}`.trim()))
}

function removeSegment(index: number) {
  if (segmentDrafts.value.length <= 1) return
  segmentDrafts.value.splice(index, 1)
}

function findSplitIndex(text: string, fallback: number) {
  const separators = ['。', '！', '？', '；', ';', '!', '?', '\n', '，', ',']
  for (const separator of separators) {
    const right = text.indexOf(separator, fallback)
    if (right > 0) return right + separator.length
    const left = text.lastIndexOf(separator, fallback)
    if (left > 0) return left + separator.length
  }
  return fallback
}
</script>
