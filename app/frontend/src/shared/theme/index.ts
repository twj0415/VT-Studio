import type { ThemePreset } from '@/shared/stores/appStore'

import { applyThemeVars } from './cssVars'
import graphite from './presets/graphite'
import aurora from './presets/aurora'
import ember from './presets/ember'
import porcelain from './presets/porcelain'
import sandstone from './presets/sandstone'

export interface AppThemePresetConfig {
  name: string
  mode: 'dark' | 'light'
  cssVars: Record<string, string>
  naiveThemeOverrides: Record<string, unknown>
}

export const themeMap = {
  graphite,
  aurora,
  ember,
  porcelain,
  sandstone,
} satisfies Record<ThemePreset, AppThemePresetConfig>

export function getThemePreviewStyle(theme: ThemePreset) {
  return {
    background: themeMap[theme].cssVars['--bg-page'],
  }
}

export function getThemePreviewCardStyle(theme: ThemePreset) {
  return {
    background: themeMap[theme].cssVars['--bg-card'],
  }
}

export { applyThemeVars }
export { getStatusToneClass } from './status'
