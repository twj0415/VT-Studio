<script setup lang="ts">
import { computed, ref, reactive, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';
import ModuleScaffold from '@renderer/features/shared/ModuleScaffold.vue';
import { useAuthStore } from '@renderer/stores/auth';
import AppearanceConfig from './components/AppearanceConfig.vue';
import LanguageConfig from './components/LanguageConfig.vue';
import ModelServiceConfig from './components/ModelServiceConfig.vue';
import DatabaseManagement from './components/DatabaseManagement.vue';
import FileManagement from './components/FileManagement.vue';
import MemoryConfig from './components/MemoryConfig.vue';
import ModelPromptConfig from './components/ModelPromptConfig.vue';
import PromptConfig from './components/PromptConfig.vue';
import SkillManagement from './components/SkillManagement.vue';
import VendorConfig from './components/VendorConfig.vue';

const router = useRouter();
const authStore = useAuthStore();
const { t } = useI18n();
const { user } = storeToRefs(authStore);
const developerVisible = ref(false);
const actions = computed(
  () =>
    [
      { title: t('settings.actions.appearance'), state: '基础入口' },
      { title: t('settings.actions.language'), state: '基础入口' },
      { title: t('settings.actions.modelService'), state: '基础入口' },
      { title: t('settings.actions.developer'), state: '基础入口' },
      { title: t('settings.actions.modelPrompt'), state: '基础入口' },
      { title: t('settings.actions.skill'), state: '基础入口' },
      { title: t('settings.actions.memory'), state: '基础入口' },
      { title: t('settings.actions.agent'), state: '待建任务' },
      { title: t('settings.actions.prompt'), state: '基础入口' },
      { title: t('settings.actions.database'), state: '基础入口' },
      { title: t('settings.actions.files'), state: '基础入口' },
    ] as const,
);

const userForm = reactive({
  name: user.value?.name ?? '',
  password: '',
});

watch(
  user,
  (nextUser) => {
    userForm.name = nextUser?.name ?? '';
    userForm.password = '';
  },
  { immediate: true },
);

async function saveLocalUser(): Promise<void> {
  if (!userForm.name.trim() || !userForm.password.trim()) {
    MessagePlugin.warning(t('settings.user.emptyError'));
    return;
  }

  const ok = await authStore.updateLocalUser(userForm.name, userForm.password);
  if (!ok) {
    MessagePlugin.error(authStore.error ?? t('settings.user.saveFailed'));
    return;
  }

  userForm.password = '';
  MessagePlugin.success(t('settings.user.saveSuccess'));
}

function confirmLogout(): void {
  const dialog = DialogPlugin.confirm({
    header: t('settings.logout.dialogTitle'),
    body: t('settings.logout.dialogBody'),
    confirmBtn: t('settings.logout.confirm'),
    cancelBtn: t('settings.logout.cancel'),
    theme: 'warning',
    async onConfirm() {
      await authStore.logout();
      MessagePlugin.success(t('settings.logout.success'));
      dialog.destroy();
      await router.replace({ name: 'login' });
    },
  });
}

</script>

<template>
  <div class="settings-page">
    <ModuleScaffold module-id="M-002" :title="t('settings.title')" :summary="t('settings.summary')" :actions="actions" />

    <AppearanceConfig />

    <LanguageConfig />

    <ModelServiceConfig />

    <PromptConfig />

    <FileManagement />

    <section class="settings-section developer-section">
      <div class="settings-section-head">
        <div>
          <p class="eyebrow">{{ t('settings.actions.developer') }}</p>
          <h3>{{ t('settings.developer.title') }}</h3>
        </div>
        <t-button variant="outline" @click="developerVisible = !developerVisible">
          {{ developerVisible ? t('settings.developer.collapse') : t('settings.developer.expand') }}
        </t-button>
      </div>
      <p class="settings-hint">{{ t('settings.developer.hint') }}</p>
      <template v-if="developerVisible">
        <MemoryConfig />
        <DatabaseManagement />
        <SkillManagement />
        <ModelPromptConfig />
        <VendorConfig />
      </template>
    </section>

    <section class="settings-section">
      <div>
        <p class="eyebrow">{{ t('settings.user.eyebrow') }}</p>
        <h3>{{ t('settings.user.title') }}</h3>
      </div>

      <t-form class="settings-form" :data="userForm" layout="vertical">
        <t-form-item :label="t('settings.user.username')">
          <t-input v-model="userForm.name" :placeholder="t('settings.user.usernamePlaceholder')" />
        </t-form-item>
        <t-form-item :label="t('settings.user.password')">
          <t-input v-model="userForm.password" type="password" :placeholder="t('settings.user.passwordPlaceholder')" @enter="saveLocalUser" />
        </t-form-item>
        <t-button theme="primary" :loading="authStore.loading" @click="saveLocalUser">{{ t('settings.user.save') }}</t-button>
      </t-form>
    </section>

    <section class="settings-section">
      <div>
        <p class="eyebrow">{{ t('settings.logout.eyebrow') }}</p>
        <h3>{{ t('settings.logout.title') }}</h3>
      </div>
      <div class="logout-row">
        <div>
          <strong>{{ user?.name ?? t('settings.logout.currentUserFallback') }}</strong>
          <p>{{ t('settings.logout.description') }}</p>
        </div>
        <t-button theme="danger" variant="outline" @click="confirmLogout">{{ t('settings.logout.button') }}</t-button>
      </div>
    </section>
  </div>
</template>
