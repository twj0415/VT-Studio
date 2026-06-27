<template>
  <header class="flex h-12 flex-none items-center border-b border-border bg-panel px-vt-3">
    <div class="relative flex w-[360px] flex-none items-center gap-vt-3">
      <button type="button" class="grid size-8 flex-none place-items-center rounded-vt-sm border border-border text-sm text-muted transition hover:border-border-strong hover:bg-card hover:text-primary" :aria-label="t('workspaceHeader.menu')" @click="isMenuOpen = !isMenuOpen">☰</button>
      <button type="button" class="grid size-8 flex-none place-items-center rounded-vt-sm border border-border text-sm text-muted transition hover:border-border-strong hover:bg-card hover:text-primary" @click="router.push(backTo)">←</button>
      <div class="flex min-w-0 items-center gap-vt-2">
        <div class="min-w-0 truncate text-base font-semibold">{{ projectTitle }}</div>
        <span v-if="badgeLabel" class="flex-none rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-0.5 text-[11px] font-medium text-accent">{{ badgeLabel }}</span>
      </div>

      <div v-if="isMenuOpen" class="absolute left-0 top-11 z-30 w-60 rounded-vt-md border border-border bg-card p-vt-2 shadow-vt-lg">
        <button v-for="entry in navEntries" :key="entry.path" type="button" class="flex w-full items-center gap-vt-2 rounded-vt-sm px-vt-3 py-vt-2 text-left text-sm text-secondary transition hover:bg-page hover:text-primary" @click="go(entry.path)">
          <span class="w-5 text-center text-xs text-muted">{{ entry.icon }}</span>
          <span>{{ t(entry.labelKey) }}</span>
        </button>
        <div class="my-vt-2 border-t border-border"></div>
        <button type="button" class="flex w-full items-center gap-vt-2 rounded-vt-sm px-vt-3 py-vt-2 text-left text-sm text-secondary transition hover:bg-page hover:text-primary" @click="go(`/projects/${projectId}`)">
          <span class="w-5 text-center text-xs text-muted">▣</span>
          <span>{{ t('workspaceHeader.projectOverview') }}</span>
        </button>
      </div>
    </div>

    <div class="flex min-w-0 flex-1 items-center justify-center">
      <WorkspaceStepBar class="hidden min-w-0 justify-center lg:flex" compact :project-id="projectId" :current-step="currentStep" :access="access" @blocked="$emit('blocked', $event)" />
    </div>

    <div class="flex flex-none items-center justify-end gap-vt-2" :class="rightClass">
      <div class="hidden max-w-[300px] flex-none items-center gap-vt-2 rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-[11px] text-muted 2xl:flex">
        <span>{{ t('workspaceHeader.usage.images', { count: usage.images }) }}</span>
        <span>·</span>
        <span>{{ t('workspaceHeader.usage.videos', { count: usage.videos }) }}</span>
        <span>·</span>
        <span>{{ usage.llm === null ? t('workspaceHeader.usage.llmUnknown') : t('workspaceHeader.usage.llm', { count: usage.llm }) }}</span>
      </div>
      <slot name="actions" />
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import WorkspaceStepBar from '@/features/workspace/WorkspaceStepBar.vue'
import type { WorkspaceStepAccess, WorkspaceStepKey } from '@/features/workspace/steps'
import { showAiToolsEntry } from '@/shared/runtime/productMode'

const props = withDefaults(defineProps<{
  projectId: string
  projectTitle: string
  currentStep: WorkspaceStepKey
  access: WorkspaceStepAccess
  backTo: string
  badgeLabel?: string
  rightWidthClass?: string
  usage?: { images: number; videos: number; llm: number | null }
}>(), {
  badgeLabel: '',
  rightWidthClass: 'w-[420px]',
  usage: () => ({ images: 0, videos: 0, llm: null }),
})

defineEmits<{
  blocked: [step: WorkspaceStepKey]
}>()

const router = useRouter()
const { t } = useI18n()
const isMenuOpen = ref(false)
const rightClass = computed(() => props.rightWidthClass)

const navEntries = [
  { path: '/', labelKey: 'nav.home', icon: '⌂' },
  ...(showAiToolsEntry ? [{ path: '/ai-tools', labelKey: 'nav.aiTools', icon: 'AI' }] : []),
  { path: '/creative-resources', labelKey: 'nav.assets', icon: '▦' },
  { path: '/model-workflow', labelKey: 'nav.modelWorkflow', icon: '⌘' },
  { path: '/settings', labelKey: 'nav.settings', icon: '⚙' },
]

async function go(path: string) {
  isMenuOpen.value = false
  await router.push(path)
}
</script>
