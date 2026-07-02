import { VT_STATUS } from '@shared/constants/status';
import type {
  BusinessCanvasWheelMode,
  BusinessSettingsConfig,
  BusinessSettingsRestoreDefaultChapterRegResult,
  BusinessSettingsResult,
  BusinessSettingsSavePayload,
  BusinessSettingsSaveResult,
} from '@shared/types/business-settings';
import { getDatabase, withTransaction } from '../database';
import { createError } from '../result';

const DEFAULT_BUSINESS_SETTINGS: BusinessSettingsConfig = {
  chapterReg: '/第\\s*([0-9０-９零一二三四五六七八九十百千万]+)\\s*[章回节]\\s*([^\\n\\r]*)/g',
  requestTimeoutMs: 600000,
  canvasWheelMode: 'zoom',
  showInteractionState: true,
  assetsBatchGenerateSize: 5,
  scriptEpisodeLength: 5000,
};

const BUSINESS_SETTING_KEYS = Object.keys(DEFAULT_BUSINESS_SETTINGS) as Array<keyof BusinessSettingsConfig>;

function readSetting(key: keyof BusinessSettingsConfig): string | null {
  const row = getDatabase().prepare<[string], { value: string }>('SELECT value FROM app_settings WHERE key = ? LIMIT 1').get(key);
  return row?.value ?? null;
}

function upsertSetting(key: keyof BusinessSettingsConfig, value: string, now: number): void {
  getDatabase()
    .prepare<[string, string, number, number, number]>(
      `
      INSERT INTO app_settings (key, value, created_at, updated_at)
      VALUES (?, ?, ?, ?)
      ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = ?
      `,
    )
    .run(key, value, now, now, now);
}

function assertRegexString(value: string): string {
  const normalized = value.trim();
  if (!normalized) {
    throw createError(VT_STATUS.INVALID_PARAMS, '章节拆分正则不能为空');
  }

  const slashPattern = normalized.match(/^\/([\s\S]*)\/([a-z]*)$/i);
  try {
    if (slashPattern) {
      const [, pattern, flags] = slashPattern;
      new RegExp(pattern, flags);
      return normalized;
    }

    new RegExp(normalized);
    return normalized;
  } catch (error) {
    throw createError(VT_STATUS.INVALID_PARAMS, error instanceof Error ? `章节拆分正则无效：${error.message}` : '章节拆分正则无效', error);
  }
}

function normalizeRequestTimeoutMs(value: number): number {
  if (!Number.isFinite(value)) {
    throw createError(VT_STATUS.INVALID_PARAMS, '请求超时必须是数字');
  }

  const normalized = Math.floor(value);
  if (normalized < 10000 || normalized > 3600000) {
    throw createError(VT_STATUS.INVALID_PARAMS, '请求超时必须在 10-3600 秒之间');
  }

  return normalized;
}

function normalizeCanvasWheelMode(value: string): BusinessCanvasWheelMode {
  if (value === 'zoom' || value === 'scroll') {
    return value;
  }

  throw createError(VT_STATUS.INVALID_PARAMS, '画布滚轮方式无效');
}

function normalizeBoolean(value: boolean): boolean {
  if (typeof value !== 'boolean') {
    throw createError(VT_STATUS.INVALID_PARAMS, '交互态开关无效');
  }

  return value;
}

function normalizeAssetsBatchGenerateSize(value: number): number {
  if (!Number.isFinite(value)) {
    throw createError(VT_STATUS.INVALID_PARAMS, '资产生成并发数必须是数字');
  }

  const normalized = Math.floor(value);
  if (normalized < 1 || normalized > 8) {
    throw createError(VT_STATUS.INVALID_PARAMS, '资产生成并发数必须在 1-8 之间');
  }

  return normalized;
}

function normalizeScriptEpisodeLength(value: number): number {
  if (!Number.isFinite(value)) {
    throw createError(VT_STATUS.INVALID_PARAMS, '剧本单集长度限制必须是数字');
  }

  const normalized = Math.floor(value);
  if (normalized < 100 || normalized > 50000) {
    throw createError(VT_STATUS.INVALID_PARAMS, '剧本单集长度限制必须在 100-50000 之间');
  }

  return normalized;
}

function readNumberSetting(key: 'requestTimeoutMs' | 'assetsBatchGenerateSize' | 'scriptEpisodeLength'): number {
  const raw = readSetting(key);
  const value = Number(raw);
  if (!Number.isFinite(value)) {
    return DEFAULT_BUSINESS_SETTINGS[key];
  }

  if (key === 'requestTimeoutMs') {
    return Math.min(Math.max(Math.floor(value), 10000), 3600000);
  }

  if (key === 'assetsBatchGenerateSize') {
    return Math.min(Math.max(Math.floor(value), 1), 8);
  }

  return Math.min(Math.max(Math.floor(value), 100), 50000);
}

function readBooleanSetting(key: 'showInteractionState'): boolean {
  const raw = readSetting(key);
  if (raw === null) {
    return DEFAULT_BUSINESS_SETTINGS[key];
  }

  return raw === 'true';
}

function readCanvasWheelMode(): BusinessCanvasWheelMode {
  const raw = readSetting('canvasWheelMode');
  return raw === 'scroll' ? 'scroll' : DEFAULT_BUSINESS_SETTINGS.canvasWheelMode;
}

function readConfig(): BusinessSettingsConfig {
  return {
    chapterReg: readSetting('chapterReg') ?? DEFAULT_BUSINESS_SETTINGS.chapterReg,
    requestTimeoutMs: readNumberSetting('requestTimeoutMs'),
    canvasWheelMode: readCanvasWheelMode(),
    showInteractionState: readBooleanSetting('showInteractionState'),
    assetsBatchGenerateSize: readNumberSetting('assetsBatchGenerateSize'),
    scriptEpisodeLength: readNumberSetting('scriptEpisodeLength'),
  };
}

function saveConfig(config: BusinessSettingsConfig): void {
  const now = Date.now();
  withTransaction(() => {
    for (const key of BUSINESS_SETTING_KEYS) {
      upsertSetting(key, String(config[key]), now);
    }
  });
}

function normalizeConfig(payload: BusinessSettingsSavePayload): BusinessSettingsConfig {
  return {
    chapterReg: assertRegexString(payload.chapterReg),
    requestTimeoutMs: normalizeRequestTimeoutMs(payload.requestTimeoutMs),
    canvasWheelMode: normalizeCanvasWheelMode(payload.canvasWheelMode),
    showInteractionState: normalizeBoolean(payload.showInteractionState),
    assetsBatchGenerateSize: normalizeAssetsBatchGenerateSize(payload.assetsBatchGenerateSize),
    scriptEpisodeLength: normalizeScriptEpisodeLength(payload.scriptEpisodeLength),
  };
}

export function getBusinessSettings(): BusinessSettingsResult {
  return {
    config: readConfig(),
  };
}

export function saveBusinessSettings(payload: BusinessSettingsSavePayload): BusinessSettingsSaveResult {
  const config = normalizeConfig(payload);
  saveConfig(config);

  return {
    config: readConfig(),
  };
}

export function restoreDefaultBusinessChapterReg(): BusinessSettingsRestoreDefaultChapterRegResult {
  const config = {
    ...readConfig(),
    chapterReg: DEFAULT_BUSINESS_SETTINGS.chapterReg,
  };
  saveConfig(config);

  return {
    config: readConfig(),
  };
}
