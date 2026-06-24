import { defineStore } from 'pinia'

import type { AppLocale, LayoutDensity, ThemePreset } from '@/shared/enums/generated'

export type { AppLocale, LayoutDensity, ThemePreset }

interface AppState {
  themePreset: ThemePreset
  layoutDensity: LayoutDensity
  appLocale: AppLocale
}

export const useAppStore = defineStore('app', {
  state: (): AppState => ({
    themePreset: 'graphite',
    layoutDensity: 'comfortable',
    appLocale: 'zh-CN',
  }),
  actions: {
    setThemePreset(themePreset: ThemePreset) {
      this.themePreset = themePreset
    },
    setLayoutDensity(layoutDensity: LayoutDensity) {
      this.layoutDensity = layoutDensity
    },
    setAppLocale(appLocale: AppLocale) {
      this.appLocale = appLocale
    },
  },
})
