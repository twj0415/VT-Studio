import type { AppLocale, LayoutDensity, ThemePreset } from '@/shared/stores/appStore'

export interface AppConfigDto {
  appLocale: AppLocale
  themePreset: ThemePreset
  layoutDensity: LayoutDensity
}
