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
            <div class="f active">{{ t('home.filters.all') }}</div>
            <div class="f">{{ t('home.filters.active') }}</div>
            <div class="f">{{ t('home.filters.draft') }}</div>
            <div class="f">{{ t('home.filters.archived') }}</div>
          </div>
        </div>

        <div class="project-grid">
          <div v-for="project in filteredProjects" :key="project.projectId" class="pcard" @click="router.push(`/projects/${project.projectId}/workspace/storyboard`)" >
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
              <div class="acts"><div class="a">✎</div><div class="a">🗑</div></div>
            </div>
          </div>

          <div class="pcard ghost" @click="router.push('/create-project')">
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
import { NButton, NInput } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { useProjectStore } from '@/entities/project/store'
import { useDictOptions } from '@/shared/dict/useDictOptions'
import type { ContentLanguage, ProjectLifecycle, WorkflowType } from '@/shared/enums/generated'
import { getStatusToneClass } from '@/shared/theme'

const router = useRouter()
const projectStore = useProjectStore()
const { t } = useI18n()
const keyword = ref('')
const workflowOptions = useDictOptions('workflowType')
const projectLifecycleOptions = useDictOptions('projectLifecycle')
const contentLanguageOptions = useDictOptions('contentLanguage')

const filteredProjects = computed(() => {
  const q = keyword.value.trim()
  if (!q) return projectStore.projects
  return projectStore.projects.filter((project) => project.title.includes(q))
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

onMounted(() => {
  void projectStore.loadProjects()
})
</script>
