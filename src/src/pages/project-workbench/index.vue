<template>
  <section class="page-stack">
    <BasePageHeader :title="t('workbench.title')" :subtitle="t('workbench.desc')" />

    <div class="project-grid">
      <article class="base-card project-card hoverable accent-line" @click="router.push(`/projects/${projectId}/workspace/storyboard`)" >
        <div>
          <div class="project-card-title">{{ t('workbench.storyboardTitle') }}</div>
          <div class="project-card-meta">Step 01</div>
        </div>
        <p class="project-card-summary">{{ t('workbench.storyboardSummary') }}</p>
        <footer class="project-card-footer">
          <span>{{ t('commonStatus.editable') }}</span>
          <span>{{ t('commonStatus.enter') }}</span>
        </footer>
      </article>

      <article class="base-card project-card hoverable accent-line" @click="router.push(`/projects/${projectId}/workspace/image`)" >
        <div>
          <div class="project-card-title">{{ t('workbench.imageTitle') }}</div>
          <div class="project-card-meta">Step 02</div>
        </div>
        <p class="project-card-summary">{{ t('workbench.imageSummary') }}</p>
        <footer class="project-card-footer">
          <span>{{ t('commonStatus.mainlineEntry') }}</span>
          <span>{{ t('commonStatus.enter') }}</span>
        </footer>
      </article>

      <article class="base-card project-card hoverable accent-line" @click="router.push(`/projects/${projectId}/workspace/video`)" >
        <div>
          <div class="project-card-title">{{ t('workbench.videoTitle') }}</div>
          <div class="project-card-meta">Step 03</div>
        </div>
        <p class="project-card-summary">{{ t('workbench.videoSummary') }}</p>
        <footer class="project-card-footer">
          <span>{{ t('commonStatus.mainlineEntry') }}</span>
          <span>{{ t('commonStatus.enter') }}</span>
        </footer>
      </article>

      <article class="base-card project-card hoverable accent-line" @click="router.push(`/projects/${projectId}/workspace/compose`)" >
        <div>
          <div class="project-card-title">{{ t('workbench.compositionTitle') }}</div>
          <div class="project-card-meta">Step 04</div>
        </div>
        <p class="project-card-summary">{{ t('workbench.compositionSummary') }}</p>
        <footer class="project-card-footer">
          <span>{{ t('commonStatus.mainlineEntry') }}</span>
          <span>{{ t('commonStatus.enter') }}</span>
        </footer>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import BasePageHeader from '@/shared/components/base/BasePageHeader.vue'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const projectId = computed(() => String(route.params.projectId))
</script>
