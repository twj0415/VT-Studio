export type AppearanceMode = 'auto' | 'light' | 'dark';
export type AppearancePresetId = 'studio' | 'warm' | 'work';
export type AppearanceFontSize = 12 | 13 | 14 | 16 | 18 | 20 | 22;
export type ResolvedAppearanceMode = 'light' | 'dark';

export interface AppearanceSettings {
  mode: AppearanceMode;
  themePresetId: AppearancePresetId;
  fontSize: AppearanceFontSize;
}

export interface AppearancePresetMeta {
  id: AppearancePresetId;
  name: string;
  description: string;
  preview: [string, string, string];
}

const APPEARANCE_STORAGE_KEY = 'vtStudio.appearance';
const FONT_SIZES: AppearanceFontSize[] = [12, 13, 14, 16, 18, 20, 22];

export const DEFAULT_APPEARANCE_SETTINGS: AppearanceSettings = {
  mode: 'auto',
  themePresetId: 'studio',
  fontSize: 14,
};

export const APPEARANCE_PRESETS: AppearancePresetMeta[] = [
  {
    id: 'studio',
    name: 'Studio',
    description: '当前产品默认风格，平衡创作感和工具感。',
    preview: ['#2f6f63', '#f3efe7', '#9a641f'],
  },
  {
    id: 'warm',
    name: '暖色',
    description: '偏柔和和内容创作场景，更适合长时间阅读。',
    preview: ['#b36a3c', '#f6eee4', '#7b8b53'],
  },
  {
    id: 'work',
    name: '工作风',
    description: '偏冷静和结构化，适合高密度操作。',
    preview: ['#2563eb', '#eef2f6', '#0f172a'],
  },
];

let currentSettings: AppearanceSettings = { ...DEFAULT_APPEARANCE_SETTINGS };
let mediaQueryList: MediaQueryList | null = null;
let mediaQueryHandlerBound = false;

function isAppearanceMode(value: string): value is AppearanceMode {
  return value === 'auto' || value === 'light' || value === 'dark';
}

function isAppearancePresetId(value: string): value is AppearancePresetId {
  return value === 'studio' || value === 'warm' || value === 'work';
}

function isAppearanceFontSize(value: number): value is AppearanceFontSize {
  return FONT_SIZES.includes(value as AppearanceFontSize);
}

export function normalizeAppearanceSettings(value: Partial<AppearanceSettings> | null | undefined): AppearanceSettings {
  const mode = typeof value?.mode === 'string' && isAppearanceMode(value.mode) ? value.mode : DEFAULT_APPEARANCE_SETTINGS.mode;
  const themePresetId = typeof value?.themePresetId === 'string' && isAppearancePresetId(value.themePresetId) ? value.themePresetId : DEFAULT_APPEARANCE_SETTINGS.themePresetId;
  const fontSizeNumber = Number(value?.fontSize);
  const fontSize = isAppearanceFontSize(fontSizeNumber) ? fontSizeNumber : DEFAULT_APPEARANCE_SETTINGS.fontSize;

  return {
    mode,
    themePresetId,
    fontSize,
  };
}

export function readStoredAppearanceSettings(): AppearanceSettings {
  const raw = window.localStorage.getItem(APPEARANCE_STORAGE_KEY);
  if (!raw) {
    return { ...DEFAULT_APPEARANCE_SETTINGS };
  }

  try {
    return normalizeAppearanceSettings(JSON.parse(raw) as Partial<AppearanceSettings>);
  } catch {
    return { ...DEFAULT_APPEARANCE_SETTINGS };
  }
}

export function persistAppearanceSettings(settings: AppearanceSettings): void {
  window.localStorage.setItem(APPEARANCE_STORAGE_KEY, JSON.stringify(settings));
}

export function getSystemAppearanceMode(): ResolvedAppearanceMode {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return 'light';
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function resolveAppearanceMode(mode: AppearanceMode): ResolvedAppearanceMode {
  return mode === 'auto' ? getSystemAppearanceMode() : mode;
}

export function applyAppearanceSettings(settings: AppearanceSettings): void {
  currentSettings = normalizeAppearanceSettings(settings);
  const root = document.documentElement;
  const resolvedMode = resolveAppearanceMode(currentSettings.mode);

  root.setAttribute('theme-mode', currentSettings.mode);
  root.setAttribute('data-theme-preset', currentSettings.themePresetId);
  root.setAttribute('data-color-mode', resolvedMode);
  root.classList.toggle('dark', resolvedMode === 'dark');
  root.style.fontSize = `${currentSettings.fontSize}px`;
}

function handleSystemAppearanceChange(): void {
  if (currentSettings.mode !== 'auto') {
    return;
  }

  applyAppearanceSettings(currentSettings);
}

export function watchSystemAppearance(): void {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return;
  }

  if (!mediaQueryList) {
    mediaQueryList = window.matchMedia('(prefers-color-scheme: dark)');
  }

  if (mediaQueryHandlerBound) {
    return;
  }

  if (typeof mediaQueryList.addEventListener === 'function') {
    mediaQueryList.addEventListener('change', handleSystemAppearanceChange);
  } else {
    mediaQueryList.addListener(handleSystemAppearanceChange);
  }
  mediaQueryHandlerBound = true;
}

export function initAppearance(): AppearanceSettings {
  const settings = readStoredAppearanceSettings();
  applyAppearanceSettings(settings);
  watchSystemAppearance();
  return settings;
}
