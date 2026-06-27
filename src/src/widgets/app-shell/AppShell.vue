<template>
  <div class="app" :class="{ 'workspace-mode': isWorkspaceContext }">
    <aside v-if="!isWorkspaceContext" class="rail">
      <div class="logo" @click="router.push('/')">V</div>
      <div class="ri" :class="{ active: route.name === 'home' || isProjectContext }" :data-tip="t('nav.home')" @click="router.push('/')" v-html="icons.home"></div>
      <div class="ri" :class="{ active: route.name === 'ai-tools' }" :data-tip="t('nav.aiTools')" @click="router.push('/ai-tools')" v-html="icons.ai"></div>
      <div class="ri" :class="{ active: route.name === 'creative-resources' }" :data-tip="t('nav.assets')" @click="router.push('/creative-resources')" v-html="icons.assets"></div>
      <div class="ri" :class="{ active: route.name === 'model-workflow' }" :data-tip="t('nav.modelWorkflow')" @click="router.push('/model-workflow')" v-html="icons.workflow"></div>
      <div class="sp"></div>
      <div class="ri" :class="{ active: route.name === 'settings' }" :data-tip="t('nav.settings')" @click="router.push('/settings')" v-html="icons.settings"></div>
    </aside>

    <main class="stage-root h-full w-full min-w-0">
      <slot />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const isProjectContext = computed(() => route.name === 'create-project' || route.name === 'storyboard-editor' || route.name === 'project-workbench')
const isWorkspaceContext = computed(() => route.path.includes('/workspace/'))

const icons = {
  home: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>',
  ai: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 3v3"/><path d="M12 18v3"/><path d="M4.2 7.5l2.6 1.5"/><path d="M17.2 15l2.6 1.5"/><path d="M4.2 16.5l2.6-1.5"/><path d="M17.2 9l2.6-1.5"/><circle cx="12" cy="12" r="5"/><path d="M9.5 13.5 11 10l1.5 3.5"/><path d="M10 12.4h2"/><path d="M14.5 10v3.5"/></svg>',
  assets: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/></svg>',
  workflow: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 6h5"/><path d="M15 6h5"/><path d="M9 6a3 3 0 1 0 6 0 3 3 0 0 0-6 0Z"/><path d="M4 18h5"/><path d="M15 18h5"/><path d="M9 18a3 3 0 1 0 6 0 3 3 0 0 0-6 0Z"/></svg>',
  settings: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="3.2"/><path d="M19.4 13a7.5 7.5 0 0 0 0-2l2-1.5-2-3.4-2.3 1a7 7 0 0 0-1.7-1l-.4-2.6H9l-.4 2.6a7 7 0 0 0-1.7 1l-2.3-1-2 3.4 2 1.5a7.5 7.5 0 0 0 0 2l-2 1.5 2 3.4 2.3-1a7 7 0 0 0 1.7 1l.4 2.6h4l.4-2.6a7 7 0 0 0 1.7-1l2.3 1 2-3.4z"/></svg>',
}
</script>
