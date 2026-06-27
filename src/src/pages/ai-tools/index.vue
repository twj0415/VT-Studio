<template>
  <section class="view h-full min-w-0 overflow-y-auto bg-page text-primary">
    <div class="mx-auto flex w-full max-w-6xl flex-col gap-vt-5 p-vt-6">
      <BasePageHeader :title="t('aiTools.title')" :subtitle="t('aiTools.desc')" />

      <div class="grid gap-vt-4 md:grid-cols-2 xl:grid-cols-3">
        <article v-for="tool in tools" :key="tool.key" class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
          <div class="flex items-start justify-between gap-vt-3">
            <div>
              <h2 class="text-base font-semibold text-primary">{{ t(`aiTools.tools.${tool.key}.title`) }}</h2>
              <p class="mt-vt-2 text-sm leading-6 text-secondary">{{ t(`aiTools.tools.${tool.key}.desc`) }}</p>
            </div>
            <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-xs text-muted">{{ t('aiTools.independent') }}</span>
          </div>
          <div class="mt-vt-4 rounded-vt-sm border border-dashed border-border bg-page p-vt-3 text-xs leading-5 text-muted">
            {{ t('aiTools.outputHint') }}
          </div>
        </article>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

import BasePageHeader from '@/shared/components/base/BasePageHeader.vue'

const { t } = useI18n()

const tools = [
  { key: 'hook' },
  { key: 'title' },
  { key: 'character' },
  { key: 'rewrite' },
  { key: 'visualPrompt' },
]
</script>
