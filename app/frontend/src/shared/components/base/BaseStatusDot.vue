<template>
  <span class="base-status-dot" :class="status" :title="status">
    <span class="base-status-dot-core"></span>
  </span>
</template>

<script setup lang="ts">
import type { TaskStatus } from '@/shared/enums/generated'

defineProps<{
  status: TaskStatus
}>()
</script>
