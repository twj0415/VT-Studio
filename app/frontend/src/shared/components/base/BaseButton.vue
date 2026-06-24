<template>
  <n-button v-bind="$attrs">
    <slot />
  </n-button>
</template>

<script setup lang="ts">
import { NButton } from 'naive-ui'
</script>
