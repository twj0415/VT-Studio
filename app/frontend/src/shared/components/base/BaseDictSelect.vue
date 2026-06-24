<template>
  <n-select v-bind="$attrs" :options="options" />
</template>

<script setup lang="ts">
import { NSelect, type SelectOption } from 'naive-ui'

defineProps<{
  options: SelectOption[]
}>()
</script>
