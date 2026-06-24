<template>
  <n-card v-bind="$attrs">
    <slot />
  </n-card>
</template>

<script setup lang="ts">
import { NCard } from 'naive-ui'
</script>
