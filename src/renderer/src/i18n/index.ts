import { createI18n } from 'vue-i18n';
import enUs from 'tdesign-vue-next/es/locale/en_US.mjs';
import zhCn from 'tdesign-vue-next/es/locale/zh_CN.mjs';
import { messages } from './messages';

export type AppLocale = 'zh-CN' | 'en';

export interface LanguageOption {
  value: AppLocale;
  labelKey: `language.options.${AppLocale}.label`;
  tipsKey: `language.options.${AppLocale}.tips`;
}

export const SUPPORTED_LOCALES: AppLocale[] = ['zh-CN', 'en'];
export const DEFAULT_LOCALE: AppLocale = 'zh-CN';
export const APP_LOCALE_KEY = 'vtStudio.locale';
export const APP_LOCALE_INITIALIZED_KEY = 'vtStudio.localeInitialized';

export const LANGUAGE_OPTIONS: LanguageOption[] = [
  { value: 'zh-CN', labelKey: 'language.options.zh-CN.label', tipsKey: 'language.options.zh-CN.tips' },
  { value: 'en', labelKey: 'language.options.en.label', tipsKey: 'language.options.en.tips' },
];

export const i18n = createI18n({
  legacy: false,
  locale: DEFAULT_LOCALE,
  fallbackLocale: DEFAULT_LOCALE,
  messages,
});

export function isSupportedLocale(value: string): value is AppLocale {
  return SUPPORTED_LOCALES.includes(value as AppLocale);
}

export function normalizeLocale(value: string | null | undefined): AppLocale {
  if (!value) {
    return DEFAULT_LOCALE;
  }

  if (isSupportedLocale(value)) {
    return value;
  }

  const normalized = value.toLowerCase();
  if (normalized.startsWith('en')) {
    return 'en';
  }

  return DEFAULT_LOCALE;
}

export function getStoredLocale(): AppLocale | null {
  const raw = window.localStorage.getItem(APP_LOCALE_KEY);
  return raw ? normalizeLocale(raw) : null;
}

export function persistLocale(locale: AppLocale): void {
  window.localStorage.setItem(APP_LOCALE_KEY, locale);
  window.localStorage.setItem(APP_LOCALE_INITIALIZED_KEY, '1');
}

export function hasLocaleInitialized(): boolean {
  return window.localStorage.getItem(APP_LOCALE_INITIALIZED_KEY) === '1';
}

export function detectSystemLocale(): AppLocale {
  return normalizeLocale(typeof navigator === 'undefined' ? null : navigator.language);
}

export function setI18nLocale(locale: AppLocale): void {
  const globalComposer = i18n.global as unknown as { locale: { value: AppLocale } };
  globalComposer.locale.value = locale;
  document.documentElement.lang = locale;
}

export function resolveInitialLocale(): AppLocale {
  if (hasLocaleInitialized()) {
    return getStoredLocale() ?? DEFAULT_LOCALE;
  }

  const detected = detectSystemLocale();
  persistLocale(detected);
  return detected;
}

export function resolveTDesignGlobalConfig(locale: AppLocale) {
  return locale === 'en' ? enUs : zhCn;
}
