<template>
  <nav class="flex flex-none items-center overflow-x-auto" :class="compact ? 'min-h-0 border-0 bg-transparent px-0' : 'min-h-12 border-b border-border bg-page px-vt-6'" :aria-label="t('workspaceStepBar.label')">
    <template v-for="(step, index) in steps" :key="step.key">
      <div v-if="index > 0" class="h-px flex-none bg-border" :class="compact ? 'mx-vt-2 w-7' : 'mx-vt-3 w-9'"></div>
      <button type="button" class="group flex flex-none items-center gap-vt-2 whitespace-nowrap transition" :class="[compact ? 'text-xs' : 'text-sm', stepButtonClass(step)]" :aria-current="step.key === currentStep ? 'step' : undefined" :aria-disabled="step.locked" @click="handleStepClick(step)">
        <span class="grid place-items-center rounded-full border transition" :class="[compact ? 'size-5 text-[11px]' : 'size-6 text-xs', stepNumberClass(step)]">
          {{ step.done ? '✓' : step.locked ? '' : index + 1 }}
        </span>
        <span>{{ step.label }}</span>
      </button>
    </template>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { canEnterWorkspaceStep, getWorkspaceStepPath, workspaceStepKeys, type WorkspaceStepAccess, type WorkspaceStepKey } from './steps'

interface WorkspaceStepItem {
  key: WorkspaceStepKey
  label: string
  locked: boolean
  current: boolean
  done: boolean
}

const props = defineProps<{
  projectId: string
  currentStep: WorkspaceStepKey
  access: WorkspaceStepAccess
  compact?: boolean
}>()

const emit = defineEmits<{
  blocked: [step: WorkspaceStepKey]
}>()

const router = useRouter()
const { t } = useI18n()
const compact = computed(() => props.compact === true)

const currentStepIndex = computed(() => workspaceStepKeys.indexOf(props.currentStep))

const steps = computed<WorkspaceStepItem[]>(() =>
  workspaceStepKeys.map((key, index) => ({
    key,
    label: t(`storyboard.steps.${key}`),
    locked: !canEnterWorkspaceStep(key, props.access),
    current: key === props.currentStep,
    done: index < currentStepIndex.value,
  }))
)

function stepButtonClass(step: WorkspaceStepItem) {
  if (step.current) return 'cursor-default font-semibold text-primary'
  if (step.locked) return 'cursor-not-allowed text-muted'
  return step.done ? 'text-secondary hover:text-primary' : 'text-secondary hover:text-primary'
}

function stepNumberClass(step: WorkspaceStepItem) {
  if (step.current) return 'border-accent text-accent shadow-[0_0_0_4px_var(--accent-soft)]'
  if (step.done) return 'border-status-succeeded bg-status-succeeded text-accent-ink'
  if (step.locked) return 'border-border-strong text-muted'
  return 'border-border-strong text-secondary group-hover:border-accent-line group-hover:text-accent'
}

async function handleStepClick(step: WorkspaceStepItem) {
  if (step.current) return

  if (step.locked) {
    emit('blocked', step.key)
    return
  }

  await router.push(getWorkspaceStepPath(props.projectId, step.key))
}
</script>
