<script setup lang="ts">
import { computed } from 'vue';
import { CheckCircleFilledIcon } from 'tdesign-icons-vue-next';
import { MessagePlugin } from 'tdesign-vue-next';
import { useI18n } from 'vue-i18n';
import { useLanguageStore } from '@renderer/stores/language';
import type { AppLocale } from '@renderer/i18n';

const languageStore = useLanguageStore();
const { t } = useI18n();

const options = computed(() =>
  languageStore.languageOptions.map((option) => ({
    ...option,
    label: t(option.labelKey),
    tips: t(option.tipsKey),
  })),
);

function switchLocale(locale: AppLocale): void {
  if (languageStore.locale === locale) {
    return;
  }

  languageStore.setLocale(locale);
  MessagePlugin.success(t('language.saved'));
}
</script>

<template>
  <section class="language-section">
    <div class="language-section-head">
      <div>
        <strong>{{ t('language.title') }}</strong>
        <p>{{ t('language.hint') }}</p>
      </div>
      <t-tag variant="light">{{ t('language.status') }}</t-tag>
    </div>

    <div class="language-card-grid">
      <button
        v-for="option in options"
        :key="option.value"
        class="language-card"
        :class="{ 'is-active': languageStore.locale === option.value }"
        type="button"
        @click="switchLocale(option.value)"
      >
        <div>
          <strong>{{ option.label }}</strong>
          <small>{{ option.tips }}</small>
        </div>
        <CheckCircleFilledIcon v-if="languageStore.locale === option.value" />
      </button>
    </div>
  </section>
</template>
