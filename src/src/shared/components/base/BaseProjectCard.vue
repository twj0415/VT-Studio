<template>
  <article class="base-card project-card hoverable accent-line" @click="$emit('open')">
    <div class="project-card-head">
      <div>
        <div class="project-card-title">{{ title }}</div>
        <div class="project-card-meta">{{ meta }}</div>
      </div>
      <BaseStatusDot :status="status" />
    </div>

    <p class="project-card-summary">{{ summary }}</p>

    <footer class="project-card-footer">
      <span>{{ lifecycle }}</span>
      <span>{{ updatedAt }}</span>
    </footer>
  </article>
</template>

<script setup lang="ts">
import type { TaskStatus } from '@/shared/enums/generated'

import BaseStatusDot from './BaseStatusDot.vue'

defineEmits<{
  open: []
}>()

defineProps<{
  title: string
  meta: string
  summary: string
  status: TaskStatus
  lifecycle: string
  updatedAt: string
}>()
</script>
