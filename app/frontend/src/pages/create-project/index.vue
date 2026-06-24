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
              <span class="rounded-vt-sm border border-accent-line bg-page px-vt-2 py-vt-1 text-accent">{{ t('createProject.splitParagraph') }}</span>
              <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-vt-1 text-muted">{{ t('createProject.noAiRewrite') }}</span>
            </div>
          </div>
        </section>

        <aside class="flex min-h-0 w-full flex-none flex-col gap-vt-5 overflow-y-auto xl:w-96">
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
      <n-button class="btn btn-primary" :loading="isCreating" :disabled="!canCreate" @click="handleCreate">{{ t('createProject.importAndEnterStoryboard') }} →</n-button>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { NButton, NInput, NInputNumber, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { useProjectStore } from '@/entities/project/store'
import type { CreateProjectRequest } from '@/entities/project/types'
import { useDictOptions } from '@/shared/dict/useDictOptions'
import type { AspectRatio, ContentLanguage, InputProcessMode, InputType } from '@/shared/enums/generated'

type CreateMode = 'text' | 'image' | 'video'

interface CreateProjectForm {
  inputType: InputType
  contentLanguage: ContentLanguage
  aspectRatio: AspectRatio
  targetSceneCount: number
  segmentDurationSeconds: number
  stylePrompt: string
  sourceText: string
}

interface CreateModeOption {
  label: string
  value: CreateMode
  disabled: boolean
}

const router = useRouter()
const projectStore = useProjectStore()
const message = useMessage()
const { t } = useI18n()

const selectedMode = ref<CreateMode>('text')
const isCreating = ref(false)
const durationOptions = [4, 6, 8] as const
const textInputTypes: readonly InputType[] = ['topic', 'paste', 'article']

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
})

const createModes = computed<CreateModeOption[]>(() => [
  { label: t('createProject.createModes.text'), value: 'text', disabled: false },
  { label: t('createProject.createModes.image'), value: 'image', disabled: true },
  { label: t('createProject.createModes.video'), value: 'video', disabled: true },
])

const textEntries = computed(() => inputOptions.value.filter((option) => textInputTypes.includes(option.value)))
const mainlineSteps = computed(() => [t('storyboard.steps.storyboard'), t('storyboard.steps.image'), t('storyboard.steps.video'), t('storyboard.steps.composition')])
const sourceText = computed(() => form.sourceText.trim())
const sourceTextLength = computed(() => sourceText.value.length)
const inputProcessMode = computed<InputProcessMode>(() => (form.inputType === 'paste' ? 'fixed' : 'generate'))
const canCreate = computed(() => selectedMode.value === 'text' && sourceTextLength.value > 0 && !isCreating.value)

const inputLabel = computed(() => {
  if (form.inputType === 'paste') return t('createProject.inputLabels.paste')
  if (form.inputType === 'article') return t('createProject.inputLabels.article')
  return t('createProject.inputLabels.topic')
})

const inputPlaceholder = computed(() => (form.inputType === 'topic' ? t('createProject.inputPlaceholderTopic') : t('createProject.inputPlaceholderContent')))

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
}

function setTargetSceneCount(value: number | null) {
  form.targetSceneCount = Math.min(60, Math.max(1, Math.round(value ?? 8)))
}

function toCreateProjectRequest(): CreateProjectRequest {
  return {
    title: '',
    workflowType: 'image_to_video',
    inputType: form.inputType,
    topic: form.inputType === 'topic' ? sourceText.value : undefined,
    sourceText: form.inputType !== 'topic' ? sourceText.value : undefined,
    contentLanguage: form.contentLanguage,
    aspectRatio: form.aspectRatio,
    targetSceneCount: form.targetSceneCount,
    segmentDurationSeconds: form.segmentDurationSeconds,
    stylePrompt: form.stylePrompt.trim() || undefined,
    inputProcessMode: inputProcessMode.value,
    inputOptions: form.inputType === 'paste' ? { splitMode: 'paragraph' } : undefined,
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
    await router.push(`/projects/${detail.project.projectId}/workspace/storyboard`)
  } catch (error) {
    message.error(error instanceof Error && error.message ? error.message : t('createProject.validation.createFailed'))
  } finally {
    isCreating.value = false
  }
}
</script>
