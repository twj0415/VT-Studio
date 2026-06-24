<template>
  <n-tag :type="type" size="small" round>{{ label }}</n-tag>
</template>

<script setup lang="ts">
import { NTag } from 'naive-ui'

defineProps<{
  label: string
  type?: 'default' | 'success' | 'warning' | 'error' | 'info'
}>()
</script>
