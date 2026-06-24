<template>
  <div class="app" :class="{ 'workspace-mode': isWorkspaceContext }">
    <aside v-if="!isWorkspaceContext" class="rail">
      <div class="logo" @click="router.push('/')">V</div>
      <div class="ri" :class="{ active: route.name === 'home' || isProjectContext }" :data-tip="t('nav.home')" @click="router.push('/')" v-html="icons.home"></div>
      <div class="ri disabled" :data-tip="t('nav.assets')" v-html="icons.assets"></div>
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
const isProjectContext = computed(() => route.name === 'create-project' || route.name === 'script-editor' || route.name === 'storyboard-editor' || route.name === 'project-workbench')
const isWorkspaceContext = computed(() => route.path.includes('/workspace/'))

const icons = {
  home: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>',
  assets: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/></svg>',
  settings: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="3.2"/><path d="M19.4 13a7.5 7.5 0 0 0 0-2l2-1.5-2-3.4-2.3 1a7 7 0 0 0-1.7-1l-.4-2.6H9l-.4 2.6a7 7 0 0 0-1.7 1l-2.3-1-2 3.4 2 1.5a7.5 7.5 0 0 0 0 2l-2 1.5 2 3.4 2.3-1a7 7 0 0 0 1.7 1l.4 2.6h4l.4-2.6a7 7 0 0 0 1.7-1l2.3 1 2-3.4z"/></svg>',
}
</script>
