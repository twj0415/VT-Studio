<script setup lang="ts">
import { reactive } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { LockOnIcon, UserIcon } from 'tdesign-icons-vue-next';
import { MessagePlugin } from 'tdesign-vue-next';
import { useAuthStore } from '@renderer/stores/auth';
import { useLanguageStore } from '@renderer/stores/language';

const router = useRouter();
const authStore = useAuthStore();
const languageStore = useLanguageStore();
const { t } = useI18n();

const form = reactive({
  username: 'admin',
  password: 'admin123',
});

function updateLocale(value: string): void {
  if (value === 'zh-CN' || value === 'en') {
    languageStore.setLocale(value);
  }
}

async function submitLogin(): Promise<void> {
  if (!form.username.trim() || !form.password.trim()) {
    MessagePlugin.warning(t('login.missingCredentials'));
    return;
  }

  const ok = await authStore.login(form.username, form.password);
  if (!ok) {
    MessagePlugin.error(authStore.error ?? t('login.failed'));
    return;
  }

  MessagePlugin.success(t('login.success'));
  await router.replace({ name: 'projects' });
}
</script>

<template>
  <main class="login-page">
    <section class="login-panel">
      <t-select class="login-language-select" :model-value="languageStore.locale" size="small" @update:model-value="updateLocale">
        <t-option v-for="option in languageStore.languageOptions" :key="option.value" :value="option.value" :label="t(option.labelKey)" />
      </t-select>
      <div class="login-brand">
        <div class="brand-mark">VT</div>
        <div>
          <p class="eyebrow">{{ t('login.localWorkspace') }}</p>
          <h1>VT Studio</h1>
        </div>
      </div>

      <t-form class="login-form" :data="form" layout="vertical">
        <t-form-item :label="t('login.username')">
          <t-input v-model="form.username" autocomplete="username" clearable :placeholder="t('login.usernamePlaceholder')">
            <template #prefix-icon>
              <UserIcon />
            </template>
          </t-input>
        </t-form-item>

        <t-form-item :label="t('login.password')">
          <t-input v-model="form.password" autocomplete="current-password" type="password" clearable :placeholder="t('login.passwordPlaceholder')" @enter="submitLogin">
            <template #prefix-icon>
              <LockOnIcon />
            </template>
          </t-input>
        </t-form-item>

        <t-button block theme="primary" size="large" :loading="authStore.loading" @click="submitLogin">{{ t('login.submit') }}</t-button>
      </t-form>

      <p class="login-note">{{ t('login.note') }}</p>
    </section>
  </main>
</template>
