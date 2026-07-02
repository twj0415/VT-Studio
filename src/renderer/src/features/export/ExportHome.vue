<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import ModuleScaffold from '@renderer/features/shared/ModuleScaffold.vue';

const { t, tm } = useI18n();
const actions = computed(() => (tm('home.export.actions') as string[]).map((title) => ({ title, state: '待确认' as const })));
</script>

<template>
  <ModuleScaffold module-id="M-010" :title="t('home.export.title')" :summary="t('home.export.summary')" :actions="actions" />
</template>
