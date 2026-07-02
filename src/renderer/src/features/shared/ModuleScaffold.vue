<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

interface ModuleAction {
  title: string;
  state: '待建任务' | '待确认' | '基础入口';
}

const props = defineProps<{
  moduleId: string;
  title: string;
  summary: string;
  actions: readonly ModuleAction[];
}>();

const { t } = useI18n();
const localizedActions = computed(() =>
  props.actions.map((action) => ({
    ...action,
    stateText: action.state === '待建任务' ? t('scaffold.state.pending') : action.state === '待确认' ? t('scaffold.state.confirm') : t('scaffold.state.ready'),
  })),
);
</script>

<template>
  <div class="module-page min-w-0">
    <section class="module-hero">
      <div>
        <p class="eyebrow">{{ moduleId }}</p>
        <h3>{{ title }}</h3>
        <p>{{ summary }}</p>
      </div>
      <div class="module-status">
        <span>{{ t('scaffold.stageLabel') }}</span>
        <strong>{{ t('scaffold.stageReady') }}</strong>
      </div>
    </section>

    <section class="module-grid">
      <article v-for="action in localizedActions" :key="action.title" class="module-card">
        <span>{{ action.stateText }}</span>
        <strong>{{ action.title }}</strong>
      </article>
    </section>
  </div>
</template>
