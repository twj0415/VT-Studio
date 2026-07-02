import { existsSync } from 'node:fs';
import { VT_STATUS } from '@shared/constants/status';
import type { MemoryClearType } from '@shared/types/memory';
import type {
  MemoryModelDtype,
  MemorySettingsClearPayload,
  MemorySettingsClearResult,
  MemorySettingsConfig,
  MemorySettingsResult,
  MemorySettingsRestoreDefaultResult,
  MemorySettingsSavePayload,
  MemorySettingsSaveResult,
  MemorySettingsValidateModelPathPayload,
  MemorySettingsValidateModelPathResult,
  MemoryStatsItem,
  MemoryStatsResult,
} from '@shared/types/memory-settings';
import { clearMemory } from '../memory';
import { disposeEmbedding, getEmbeddingModelStatus } from '../embedding';
import { getDatabase, withTransaction } from '../database';
import { getRuntimeDirectories, safeJoin } from '../file-system';
import { createError } from '../result';

const DEFAULT_MEMORY_CONFIG: MemorySettingsConfig = {
  modelOnnxFile: ['all-MiniLM-L6-v2', 'onnx', 'model_fp16.onnx'],
  modelDtype: 'fp16',
  messagesPerSummary: 10,
  shortTermLimit: 5,
  summaryMaxLength: 500,
  summaryLimit: 10,
  ragLimit: 3,
  deepRetrieveSummaryLimit: 5,
};

const NUMBER_RANGES: Record<Exclude<keyof MemorySettingsConfig, 'modelOnnxFile' | 'modelDtype'>, { min: number; max: number }> = {
  messagesPerSummary: { min: 1, max: 200 },
  shortTermLimit: { min: 1, max: 100 },
  summaryMaxLength: { min: 0, max: 1000 },
  summaryLimit: { min: 0, max: 100 },
  ragLimit: { min: 0, max: 50 },
  deepRetrieveSummaryLimit: { min: 0, max: 100 },
};

const MEMORY_SETTING_KEYS = Object.keys(DEFAULT_MEMORY_CONFIG) as Array<keyof MemorySettingsConfig>;

function readSetting(key: keyof MemorySettingsConfig): string | null {
  const row = getDatabase().prepare<[string], { value: string }>('SELECT value FROM app_settings WHERE key = ? LIMIT 1').get(key);
  return row?.value ?? null;
}

function upsertSetting(key: keyof MemorySettingsConfig, value: string, now: number): void {
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

function assertDtype(value: string): MemoryModelDtype {
  if (value === 'fp32' || value === 'fp16' || value === 'q8') {
    return value;
  }

  throw createError(VT_STATUS.INVALID_PARAMS, '模型 dtype 无效');
}

function assertModelOnnxFile(value: unknown): string[] {
  if (!Array.isArray(value) || value.length < 2) {
    throw createError(VT_STATUS.INVALID_PARAMS, 'ONNX 模型路径必须至少包含模型目录和文件名');
  }

  const parts = value.map((item) => String(item ?? '').trim());
  if (parts.some((part) => !part || part.includes('/') || part.includes('\\') || part.includes('\0'))) {
    throw createError(VT_STATUS.INVALID_PARAMS, 'ONNX 模型路径片段无效');
  }

  safeJoin(getRuntimeDirectories().models, parts.join('/'));
  return parts;
}

function parseModelOnnxFile(raw: string | null): string[] {
  if (!raw) {
    return [...DEFAULT_MEMORY_CONFIG.modelOnnxFile];
  }

  try {
    return assertModelOnnxFile(JSON.parse(raw));
  } catch {
    return [...DEFAULT_MEMORY_CONFIG.modelOnnxFile];
  }
}

function readNumberConfig(key: keyof typeof NUMBER_RANGES): number {
  const raw = readSetting(key);
  const value = Number(raw);
  const fallback = DEFAULT_MEMORY_CONFIG[key];
  const range = NUMBER_RANGES[key];

  if (!Number.isFinite(value)) {
    return fallback;
  }

  return Math.min(Math.max(Math.floor(value), range.min), range.max);
}

function readConfig(): MemorySettingsConfig {
  const dtype = readSetting('modelDtype') ?? DEFAULT_MEMORY_CONFIG.modelDtype;

  return {
    modelOnnxFile: parseModelOnnxFile(readSetting('modelOnnxFile')),
    modelDtype: dtype === 'fp32' || dtype === 'fp16' || dtype === 'q8' ? dtype : DEFAULT_MEMORY_CONFIG.modelDtype,
    messagesPerSummary: readNumberConfig('messagesPerSummary'),
    shortTermLimit: readNumberConfig('shortTermLimit'),
    summaryMaxLength: readNumberConfig('summaryMaxLength'),
    summaryLimit: readNumberConfig('summaryLimit'),
    ragLimit: readNumberConfig('ragLimit'),
    deepRetrieveSummaryLimit: readNumberConfig('deepRetrieveSummaryLimit'),
  };
}

function assertNumberRange(key: keyof typeof NUMBER_RANGES, value: number): number {
  const range = NUMBER_RANGES[key];
  if (!Number.isFinite(value) || value < range.min || value > range.max) {
    throw createError(VT_STATUS.INVALID_PARAMS, `${key} 必须在 ${range.min}-${range.max} 之间`);
  }

  return Math.floor(value);
}

function normalizeConfig(payload: MemorySettingsSavePayload): MemorySettingsConfig {
  return {
    modelOnnxFile: assertModelOnnxFile(payload.modelOnnxFile),
    modelDtype: assertDtype(payload.modelDtype),
    messagesPerSummary: assertNumberRange('messagesPerSummary', payload.messagesPerSummary),
    shortTermLimit: assertNumberRange('shortTermLimit', payload.shortTermLimit),
    summaryMaxLength: assertNumberRange('summaryMaxLength', payload.summaryMaxLength),
    summaryLimit: assertNumberRange('summaryLimit', payload.summaryLimit),
    ragLimit: assertNumberRange('ragLimit', payload.ragLimit),
    deepRetrieveSummaryLimit: assertNumberRange('deepRetrieveSummaryLimit', payload.deepRetrieveSummaryLimit),
  };
}

function saveConfig(config: MemorySettingsConfig): void {
  const now = Date.now();
  withTransaction(() => {
    for (const key of MEMORY_SETTING_KEYS) {
      const value = key === 'modelOnnxFile' ? JSON.stringify(config.modelOnnxFile) : String(config[key]);
      upsertSetting(key, value, now);
    }
  });
}

function getModelRelativePath(modelOnnxFile: string[]): string {
  safeJoin(getRuntimeDirectories().models, modelOnnxFile.join('/'));
  return modelOnnxFile.join('/');
}

function getModelStatus(config: MemorySettingsConfig) {
  const relativePath = getModelRelativePath(config.modelOnnxFile);
  const modelPath = safeJoin(getRuntimeDirectories().models, relativePath);
  const runtimeStatus = getEmbeddingModelStatus();

  return {
    available: existsSync(modelPath),
    relativePath,
    modelFolder: config.modelOnnxFile[0],
    modelDtype: runtimeStatus.modelDtype,
  };
}

export function getMemoryStats(): MemoryStatsResult {
  const total = getDatabase().prepare<[], { count: number }>('SELECT COUNT(*) as count FROM memories').get()?.count ?? 0;
  const messages = getDatabase().prepare<[], { count: number }>("SELECT COUNT(*) as count FROM memories WHERE type = 'message'").get()?.count ?? 0;
  const summaries = getDatabase().prepare<[], { count: number }>("SELECT COUNT(*) as count FROM memories WHERE type = 'summary'").get()?.count ?? 0;
  const rows = getDatabase()
    .prepare<[], MemoryStatsItem>(
      `
      SELECT
        isolation_key as isolationKey,
        COUNT(*) as total,
        SUM(CASE WHEN type = 'message' THEN 1 ELSE 0 END) as messages,
        SUM(CASE WHEN type = 'summary' THEN 1 ELSE 0 END) as summaries
      FROM memories
      GROUP BY isolation_key
      ORDER BY total DESC, isolation_key ASC
      `,
    )
    .all();

  return {
    total,
    messages,
    summaries,
    isolations: rows.map((row) => ({
      isolationKey: row.isolationKey,
      total: Number(row.total) || 0,
      messages: Number(row.messages) || 0,
      summaries: Number(row.summaries) || 0,
    })),
  };
}

export function getMemorySettings(): MemorySettingsResult {
  const config = readConfig();

  return {
    config,
    modelStatus: getModelStatus(config),
    stats: getMemoryStats(),
  };
}

export async function saveMemorySettings(payload: MemorySettingsSavePayload): Promise<MemorySettingsSaveResult> {
  const config = normalizeConfig(payload);
  saveConfig(config);
  await disposeEmbedding();

  return getMemorySettings();
}

export async function restoreDefaultMemorySettings(): Promise<MemorySettingsRestoreDefaultResult> {
  saveConfig(DEFAULT_MEMORY_CONFIG);
  await disposeEmbedding();

  return getMemorySettings();
}

export function validateMemoryModelPath(payload: MemorySettingsValidateModelPathPayload): MemorySettingsValidateModelPathResult {
  const modelOnnxFile = assertModelOnnxFile(payload.modelOnnxFile);
  const relativePath = getModelRelativePath(modelOnnxFile);

  return {
    available: existsSync(safeJoin(getRuntimeDirectories().models, relativePath)),
    relativePath,
  };
}

function normalizeClearType(value: MemoryClearType | undefined): MemoryClearType {
  if (value === 'message' || value === 'summary') {
    return value;
  }

  return 'all';
}

export function clearMemoryBySettings(payload: MemorySettingsClearPayload): MemorySettingsClearResult {
  if (payload.scope === 'all') {
    if (payload.confirmText !== '清空全部记忆') {
      throw createError(VT_STATUS.INVALID_PARAMS, '请输入确认短语：清空全部记忆');
    }

    const result = withTransaction((database) => {
      const deleted = database.prepare('DELETE FROM memories').run().changes;
      return { deleted, updated: 0 };
    });

    return { ...result, stats: getMemoryStats() };
  }

  const agentType = payload.agentType ?? 'scriptAgent';
  if (payload.projectId === undefined || payload.projectId === null || String(payload.projectId).trim() === '') {
    throw createError(VT_STATUS.INVALID_PARAMS, '项目 ID 不能为空');
  }

  const result = clearMemory({
    projectId: payload.projectId,
    agentType,
    episodesId: payload.episodesId,
    type: normalizeClearType(payload.type),
  });

  return { ...result, stats: getMemoryStats() };
}
