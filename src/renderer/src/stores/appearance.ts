import { defineStore } from 'pinia';
import type {
  AppearanceFontSize,
  AppearanceMode,
  AppearancePresetId,
  AppearanceSettings,
} from '@renderer/features/settings/appearance/theme';
import {
  DEFAULT_APPEARANCE_SETTINGS,
  applyAppearanceSettings,
  initAppearance,
  normalizeAppearanceSettings,
  persistAppearanceSettings,
  resolveAppearanceMode,
} from '@renderer/features/settings/appearance/theme';

interface AppearanceState extends AppearanceSettings {
  initialized: boolean;
}

export const useAppearanceStore = defineStore('appearance', {
  state: (): AppearanceState => ({
    ...DEFAULT_APPEARANCE_SETTINGS,
    initialized: false,
  }),
  getters: {
    resolvedMode(state): 'light' | 'dark' {
      return resolveAppearanceMode(state.mode);
    },
  },
  actions: {
    init(): void {
      if (this.initialized) {
        return;
      }

      Object.assign(this, initAppearance(), { initialized: true });
    },
    applySettings(nextSettings: Partial<AppearanceSettings>): void {
      const settings = normalizeAppearanceSettings({
        mode: nextSettings.mode ?? this.mode,
        themePresetId: nextSettings.themePresetId ?? this.themePresetId,
        fontSize: nextSettings.fontSize ?? this.fontSize,
      });

      this.mode = settings.mode;
      this.themePresetId = settings.themePresetId;
      this.fontSize = settings.fontSize;
      persistAppearanceSettings(settings);
      applyAppearanceSettings(settings);
    },
    setMode(mode: AppearanceMode): void {
      this.applySettings({ mode });
    },
    setThemePresetId(themePresetId: AppearancePresetId): void {
      this.applySettings({ themePresetId });
    },
    setFontSize(fontSize: AppearanceFontSize): void {
      this.applySettings({ fontSize });
    },
    restoreDefault(): void {
      this.applySettings(DEFAULT_APPEARANCE_SETTINGS);
    },
  },
});
