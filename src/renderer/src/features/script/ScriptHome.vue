<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import ModuleScaffold from '@renderer/features/shared/ModuleScaffold.vue';

const { t, tm } = useI18n();
const actions = computed(() => (tm('home.script.actions') as string[]).map((title) => ({ title, state: '待建任务' as const })));
</script>

<template>
  <ModuleScaffold module-id="M-005" :title="t('home.script.title')" :summary="t('home.script.summary')" :actions="actions" />
</template>
