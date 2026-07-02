<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import ModuleScaffold from '@renderer/features/shared/ModuleScaffold.vue';

const { t, tm } = useI18n();
const actions = computed(() => (tm('home.scriptAgent.actions') as string[]).map((title) => ({ title, state: '待建任务' as const })));
</script>

<template>
  <ModuleScaffold module-id="M-011" :title="t('home.scriptAgent.title')" :summary="t('home.scriptAgent.summary')" :actions="actions" />
</template>
