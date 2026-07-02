<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import ModuleScaffold from '@renderer/features/shared/ModuleScaffold.vue';

const { t, tm } = useI18n();
const actions = computed(() => (tm('home.tasks.actions') as string[]).map((title) => ({ title, state: '待建任务' as const })));
</script>

<template>
  <ModuleScaffold module-id="M-009" :title="t('home.tasks.title')" :summary="t('home.tasks.summary')" :actions="actions" />
</template>
