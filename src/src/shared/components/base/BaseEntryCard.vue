<template>
  <article class="base-card entry-card hoverable accent-line" :class="{ selected, disabled }" @click="handleClick">
    <div class="entry-card-kicker">{{ value }}</div>
    <div class="entry-card-title">{{ title }}</div>
    <p class="entry-card-desc">{{ description }}</p>
    <footer class="entry-card-footer">
      <span>{{ mode }}</span>
      <span v-if="disabled">后续开放</span>
    </footer>
  </article>
</template>

<script setup lang="ts">
const emit = defineEmits<{
  select: []
}>()

const props = defineProps<{
  title: string
  value: string
  description: string
  mode: string
  selected?: boolean
  disabled?: boolean
}>()

function handleClick() {
  if (props.disabled) return
  emit('select')
}
</script>
