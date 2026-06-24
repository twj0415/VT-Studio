<template>
  <n-modal v-bind="$attrs">
    <slot />
  </n-modal>
</template>

<script setup lang="ts">
import { NModal } from 'naive-ui'
</script>
