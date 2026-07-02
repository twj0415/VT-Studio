import { defineStore } from 'pinia';
import {
  LANGUAGE_OPTIONS,
  type AppLocale,
  persistLocale,
  resolveInitialLocale,
  resolveTDesignGlobalConfig,
  setI18nLocale,
} from '@renderer/i18n';

interface LanguageState {
  locale: AppLocale;
  initialized: boolean;
}

export const useLanguageStore = defineStore('language', {
  state: (): LanguageState => ({
    locale: 'zh-CN',
    initialized: false,
  }),
  getters: {
    languageOptions: () => LANGUAGE_OPTIONS,
    tdesignGlobalConfig(state) {
      return resolveTDesignGlobalConfig(state.locale);
    },
  },
  actions: {
    init(): void {
      if (this.initialized) {
        return;
      }

      const locale = resolveInitialLocale();
      this.locale = locale;
      this.initialized = true;
      setI18nLocale(locale);
    },
    setLocale(locale: AppLocale): void {
      this.locale = locale;
      this.initialized = true;
      persistLocale(locale);
      setI18nLocale(locale);
    },
  },
});
