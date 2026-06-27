<template>
  <div class="flex items-center gap-vt-2 text-xs text-secondary">
    <span>{{ t('workspaceRowJump.label') }}</span>
    <n-input-number class="w-20" size="small" :min="1" :max="Math.max(1, count)" :step="1" :value="target" @update:value="target = normalizeTarget($event)" />
    <button type="button" class="h-8 rounded-vt-sm border border-border-strong px-vt-3 text-xs font-medium text-secondary transition hover:bg-page hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="count === 0" @click="$emit('jump', target)">
      {{ t('workspaceRowJump.go') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { NInputNumber } from 'naive-ui'
import { useI18n } from 'vue-i18n'

const props = defineProps<{ count: number }>()
defineEmits<{ jump: [index: number] }>()

const { t } = useI18n()
const target = ref(1)

watch(() => props.count, (count) => {
  if (count <= 0) target.value = 1
  else if (target.value > count) target.value = count
})

function normalizeTarget(value: number | null) {
  return Math.min(Math.max(1, props.count || 1), Math.max(1, Math.round(value ?? 1)))
}
</script>
