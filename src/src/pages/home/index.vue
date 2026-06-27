<template>
  <section class="view">
    <div class="home-scroll">
      <div class="wrap">
        <div class="phead">
          <div>
            <h1>{{ t('home.title') }}</h1>
            <div class="desc">{{ t('home.desc') }}</div>
          </div>
          <n-button class="btn btn-primary" @click="router.push('/create-project')">＋ {{ t('home.newProject') }}</n-button>
        </div>

        <div class="toolbar">
          <div class="search">
            <span class="ic">⌕</span>
            <n-input v-model:value="keyword" :placeholder="t('home.searchPlaceholder')" :bordered="false" />
          </div>
          <div class="filters">
            <button v-for="filter in lifecycleFilters" :key="filter.value" type="button" class="f" :class="{ active: activeFilter === filter.value }" @click="activeFilter = filter.value">{{ filter.label }}</button>
          </div>
        </div>

        <div class="project-grid">
          <div v-if="isLoading" class="pcard ghost">
            <div class="plus">…</div>
            <div>{{ t('home.loading') }}</div>
          </div>

          <div v-else-if="loadError" class="pcard ghost">
            <div class="plus">!</div>
            <div>{{ t('home.loadFailed') }}</div>
            <n-button size="small" @click.stop="loadProjects">{{ t('home.retry') }}</n-button>
          </div>

          <div v-for="project in filteredProjects" v-else :key="project.projectId" class="pcard" @click="router.push(`/projects/${project.projectId}`)">
            <div class="ptop">
              <h3>{{ project.title }}</h3>
              <span class="src">{{ workflowTypeLabel(project.workflowType) }}</span>
            </div>
            <span class="style-tag">{{ contentLanguageText(project.contentLanguage) }} · {{ project.aspectRatio }}</span>
            <p>{{ project.latestTask?.summary ?? t('home.emptySummary') }}</p>
            <div class="pfoot">
              <div class="meta-line">
                <span class="lifecycle" :class="lifecycleClass(project.lifecycle)"><span class="d"></span>{{ lifecycleText(project.lifecycle) }}</span>
                <span class="date">{{ project.updatedAt }}</span>
              </div>
              <div class="acts">
                <button type="button" class="a" :title="t('home.actions.continue')" @click.stop="router.push(`/projects/${project.projectId}`)">✎</button>
                <button
                  v-if="project.lifecycle === 'archived'"
                  type="button"
                  class="a"
                  :title="t('home.actions.restore')"
                  @click.stop="confirmLifecycle(project.projectId, 'active')"
                >
                  ↥
                </button>
                <button
                  v-else
                  type="button"
                  class="a"
                  :title="t('home.actions.archive')"
                  @click.stop="confirmLifecycle(project.projectId, 'archived')"
                >
                  ⊙
                </button>
                <button type="button" class="a danger" :title="t('home.actions.delete')" @click.stop="confirmLifecycle(project.projectId, 'deleted')">×</button>
              </div>
            </div>
          </div>

          <div v-if="!isLoading && !loadError && filteredProjects.length === 0" class="pcard ghost" @click="router.push('/create-project')">
            <div class="plus">＋</div>
            <div>{{ t('home.emptyTitle') }}</div>
            <p>{{ t('home.emptyHint') }}</p>
          </div>

          <div v-if="!isLoading && !loadError" class="pcard ghost" @click="router.push('/create-project')">
            <div class="plus">＋</div>
            <div>{{ t('home.newProject') }}</div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { NButton, NInput, useDialog, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { useProjectStore } from '@/entities/project/store'
import { useDictOptions } from '@/shared/dict/useDictOptions'
import type { ContentLanguage, ProjectLifecycle, WorkflowType } from '@/shared/enums/generated'
import { getStatusToneClass } from '@/shared/theme'

const router = useRouter()
const projectStore = useProjectStore()
const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()
const keyword = ref('')
const activeFilter = ref<'all' | ProjectLifecycle>('all')
const isLoading = ref(false)
const loadError = ref('')
const workflowOptions = useDictOptions('workflowType')
const projectLifecycleOptions = useDictOptions('projectLifecycle')
const contentLanguageOptions = useDictOptions('contentLanguage')

const lifecycleFilters = computed(() => [
  { value: 'all' as const, label: t('home.filters.all') },
  { value: 'active' as const, label: t('home.filters.active') },
  { value: 'draft' as const, label: t('home.filters.draft') },
  { value: 'archived' as const, label: t('home.filters.archived') },
])

const filteredProjects = computed(() => {
  const q = keyword.value.trim()
  return projectStore.projects.filter((project) => {
    if (activeFilter.value !== 'all' && project.lifecycle !== activeFilter.value) return false
    if (!q) return true
    return project.title.includes(q)
  })
})

function lifecycleClass(lifecycle: ProjectLifecycle) {
  return getStatusToneClass(projectLifecycleOptions.value.find((option) => option.value === lifecycle)?.colorToken)
}

function workflowTypeLabel(workflowType: WorkflowType) {
  return workflowOptions.value.find((option) => option.value === workflowType)?.label ?? workflowType
}

function lifecycleText(lifecycle: ProjectLifecycle) {
  return projectLifecycleOptions.value.find((option) => option.value === lifecycle)?.label ?? lifecycle
}

function contentLanguageText(contentLanguage: ContentLanguage) {
  return contentLanguageOptions.value.find((option) => option.value === contentLanguage)?.label ?? contentLanguage
}

onMounted(loadProjects)

async function loadProjects() {
  isLoading.value = true
  loadError.value = ''
  try {
    await projectStore.loadProjects()
  } catch (error) {
    loadError.value = error instanceof Error ? error.message : String(error)
  } finally {
    isLoading.value = false
  }
}

function confirmLifecycle(projectId: string, lifecycle: Extract<ProjectLifecycle, 'active' | 'archived' | 'deleted'>) {
  const key = lifecycle === 'deleted' ? 'delete' : lifecycle === 'archived' ? 'archive' : 'restore'
  dialog.warning({
    title: t(`home.actions.${key}Title`),
    content: t(`home.actions.${key}Confirm`),
    positiveText: t('common.confirm'),
    negativeText: t('common.cancel'),
    onPositiveClick: async () => {
      await projectStore.updateLifecycle({ projectId, lifecycle })
      message.success(t(`home.actions.${key}Success`))
    },
  })
}
</script>
