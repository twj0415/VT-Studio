<template>
  <header class="base-page-header">
    <div>
      <h1 class="base-page-title">{{ title }}</h1>
      <div v-if="subtitle" class="base-page-subtitle">{{ subtitle }}</div>
    </div>
    <div v-if="$slots.extra" class="base-page-extra">
      <slot name="extra" />
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>
