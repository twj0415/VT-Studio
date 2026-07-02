import { VT_STATUS } from '@shared/constants/status';
import {
  TEXT_AGENT_KEYS,
  type AgentConfigGroup,
  type AgentConfigItem,
  type AgentConfigResult,
  type AgentConfigSavePayload,
  type AgentConfigSaveResult,
  type AgentEffectiveModel,
  type AgentGlobalSettings,
  type AgentTextModelOption,
  type TextAgentKey,
} from '@shared/types/agent-config';
import { getDatabase } from '../database';
import type { AgentModelConfig } from '../model/types';
import { createError } from '../result';
import { getResourceConfig } from './model-config';

const AGENT_GLOBAL_SETTINGS_KEY = 'agentGlobalSettings.v1';
const DEFAULT_GLOBAL_SETTINGS: AgentGlobalSettings = {
  temperature: 0.7,
  maxOutputTokens: 0,
};

const AGENT_GROUPS: Record<TextAgentKey, AgentConfigGroup> = {
  scriptAgent: 'main',
  productionAgent: 'main',
  universalAi: 'main',
  'scriptAgent:decisionAgent': 'script',
  'scriptAgent:supervisionAgent': 'script',
  'scriptAgent:storySkeletonAgent': 'script',
  'scriptAgent:adaptationStrategyAgent': 'script',
  'scriptAgent:scriptAgent': 'script',
  'productionAgent:decisionAgent': 'production',
  'productionAgent:supervisionAgent': 'production',
  'productionAgent:deriveAssetsAgent': 'production',
  'productionAgent:generateAssetsAgent': 'production',
  'productionAgent:directorPlanAgent': 'production',
  'productionAgent:storyboardGenAgent': 'production',
  'productionAgent:storyboardPanelAgent': 'production',
  'productionAgent:storyboardTableAgent': 'production',
};

const AGENT_LABELS: Record<TextAgentKey, string> = {
  scriptAgent: '剧本 Agent',
  productionAgent: '生产 Agent',
  universalAi: '通用 AI',
  'scriptAgent:decisionAgent': '剧本决策',
  'scriptAgent:supervisionAgent': '剧本监督',
  'scriptAgent:storySkeletonAgent': '故事骨架',
  'scriptAgent:adaptationStrategyAgent': '改编策略',
  'scriptAgent:scriptAgent': '剧本写作',
  'productionAgent:decisionAgent': '生产决策',
  'productionAgent:supervisionAgent': '生产监督',
  'productionAgent:deriveAssetsAgent': '资产衍生',
  'productionAgent:generateAssetsAgent': '资产生成',
  'productionAgent:directorPlanAgent': '导演计划',
  'productionAgent:storyboardGenAgent': '分镜生成',
  'productionAgent:storyboardPanelAgent': '分镜面板',
  'productionAgent:storyboardTableAgent': '分镜表格',
};

interface AgentModelRow {
  id: number;
  key: string;
  name: string | null;
  description: string | null;
  model_label: string | null;
  model_id: string | null;
  vendor_id: string | null;
  temperature: number | null;
  max_output_tokens: number | null;
  disabled: number;
}

function isTextAgentKey(value: string): value is TextAgentKey {
  return TEXT_AGENT_KEYS.includes(value as TextAgentKey);
}

function getSettingJson<T>(key: string, fallback: T): T {
  const row = getDatabase().prepare<[string], { value: string }>('SELECT value FROM app_settings WHERE key = ? LIMIT 1').get(key);
  if (!row?.value) {
    return fallback;
  }

  try {
    return JSON.parse(row.value) as T;
  } catch {
    return fallback;
  }
}

function saveSettingJson(key: string, value: unknown): void {
  const now = Date.now();
  const serialized = JSON.stringify(value);
  const existing = getDatabase().prepare<[string], { key: string } | undefined>('SELECT key FROM app_settings WHERE key = ? LIMIT 1').get(key);
  if (existing) {
    getDatabase().prepare<[string, number, string]>('UPDATE app_settings SET value = ?, updated_at = ? WHERE key = ?').run(serialized, now, key);
    return;
  }

  getDatabase()
    .prepare<[string, string, number, number]>('INSERT INTO app_settings (key, value, created_at, updated_at) VALUES (?, ?, ?, ?)')
    .run(key, serialized, now, now);
}

function normalizeGlobalSettings(value: Partial<AgentGlobalSettings> | null | undefined): AgentGlobalSettings {
  const temperature = Number(value?.temperature ?? DEFAULT_GLOBAL_SETTINGS.temperature);
  const maxOutputTokens = Number(value?.maxOutputTokens ?? DEFAULT_GLOBAL_SETTINGS.maxOutputTokens);

  return {
    temperature: clampTemperature(temperature),
    maxOutputTokens: normalizeMaxOutputTokens(maxOutputTokens),
  };
}

function getGlobalSettings(): AgentGlobalSettings {
  return normalizeGlobalSettings(getSettingJson<Partial<AgentGlobalSettings>>(AGENT_GLOBAL_SETTINGS_KEY, DEFAULT_GLOBAL_SETTINGS));
}

function saveGlobalSettings(settings: AgentGlobalSettings): AgentGlobalSettings {
  const normalized = normalizeGlobalSettings(settings);
  saveSettingJson(AGENT_GLOBAL_SETTINGS_KEY, normalized);
  return normalized;
}

function clampTemperature(value: number): number {
  if (!Number.isFinite(value)) {
    return DEFAULT_GLOBAL_SETTINGS.temperature;
  }

  return Math.min(2, Math.max(0, value));
}

function normalizeMaxOutputTokens(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    return DEFAULT_GLOBAL_SETTINGS.maxOutputTokens;
  }

  return Math.floor(value);
}

function normalizeNullableTemperature(value: number | null): number | null {
  return value === null ? null : clampTemperature(Number(value));
}

function normalizeNullableMaxOutputTokens(value: number | null): number | null {
  return value === null ? null : normalizeMaxOutputTokens(Number(value));
}

function toAgentModelConfig(row: AgentModelRow): AgentModelConfig {
  return {
    id: row.id,
    key: row.key,
    name: row.name,
    description: row.description,
    modelLabel: row.model_label,
    modelId: row.model_id,
    vendorId: row.vendor_id,
    temperature: row.temperature,
    maxOutputTokens: row.max_output_tokens,
    disabled: row.disabled === 1,
  };
}

function getAgentRows(): AgentModelRow[] {
  return getDatabase()
    .prepare<[], AgentModelRow>('SELECT * FROM agent_model_configs ORDER BY id ASC')
    .all()
    .filter((row) => isTextAgentKey(row.key));
}

function getAgentRowByKey(key: TextAgentKey): AgentModelRow | null {
  const row = getDatabase().prepare<[string], AgentModelRow>('SELECT * FROM agent_model_configs WHERE key = ? LIMIT 1').get(key);
  return row ?? null;
}

function ensureAgentRow(key: TextAgentKey): AgentModelRow {
  const existing = getAgentRowByKey(key);
  if (existing) {
    return existing;
  }

  const now = Date.now();
  getDatabase()
    .prepare<[string, string, number, number, number]>('INSERT INTO agent_model_configs (key, name, disabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?)')
    .run(key, AGENT_LABELS[key], 0, now, now);

  const created = getAgentRowByKey(key);
  if (!created) {
    throw createError(VT_STATUS.DATABASE_ERROR, `Agent 配置初始化失败：${key}`);
  }

  return created;
}

function getAvailableTextModels(): AgentTextModelOption[] {
  const resource = getResourceConfig();
  const options: AgentTextModelOption[] = [];

  for (const connection of resource.connections) {
    if (connection.status !== 'ready') {
      continue;
    }

    for (const model of connection.models) {
      if (model.type !== 'text') {
        continue;
      }

      options.push({
        modelId: `${connection.id}:${model.modelName}`,
        connectionId: connection.id,
        connectionName: connection.name,
        modelName: model.modelName,
        modelDisplayName: model.displayName,
      });
    }
  }

  return options;
}

function getDefaultTextModel(): { model: AgentEffectiveModel | null; status: AgentConfigResult['defaultTextStatus']; statusText: string } {
  const resource = getResourceConfig();
  const text = resource.capabilities.find((item) => item.capability === 'text') ?? null;
  if (!text || !text.binding) {
    return { model: null, status: 'missing', statusText: '默认文本模型未配置' };
  }

  if (text.status !== 'configured') {
    return { model: null, status: text.status, statusText: text.statusText };
  }

  return {
    model: {
      modelId: `${text.binding.connectionId}:${text.binding.modelName}`,
      connectionId: text.binding.connectionId,
      connectionName: text.connectionName,
      modelName: text.modelName,
      modelDisplayName: text.modelDisplayName,
    },
    status: 'configured',
    statusText: text.statusText,
  };
}

function getOptionByModelId(modelId: string | null | undefined, options: AgentTextModelOption[]): AgentTextModelOption | null {
  if (!modelId) {
    return null;
  }

  return options.find((item) => item.modelId === modelId) ?? null;
}

function toEffectiveModel(option: AgentTextModelOption): AgentEffectiveModel {
  return {
    modelId: option.modelId,
    connectionId: option.connectionId,
    connectionName: option.connectionName,
    modelName: option.modelName,
    modelDisplayName: option.modelDisplayName,
  };
}

function buildAgentItem(row: AgentModelRow, options: AgentTextModelOption[], defaultText: ReturnType<typeof getDefaultTextModel>): AgentConfigItem {
  const key = row.key as TextAgentKey;
  const overrideOption = getOptionByModelId(row.model_id, options);
  const overrideEnabled = Boolean(row.model_id);
  const disabled = row.disabled === 1;

  if (disabled) {
    return {
      ...toBaseAgentItem(row, key),
      disabled,
      overrideEnabled,
      effectiveModel: null,
      status: 'disabled',
      statusText: '已禁用',
    };
  }

  if (overrideEnabled) {
    return {
      ...toBaseAgentItem(row, key),
      disabled,
      overrideEnabled,
      effectiveModel: overrideOption ? toEffectiveModel(overrideOption) : null,
      status: overrideOption ? 'overridden' : 'invalid-override',
      statusText: overrideOption ? '高级覆盖' : '覆盖模型已失效',
    };
  }

  return {
    ...toBaseAgentItem(row, key),
    disabled,
    overrideEnabled,
    effectiveModel: defaultText.model,
    status: defaultText.model ? 'inherited' : defaultText.status === 'missing' ? 'missing-default' : 'invalid-default',
    statusText: defaultText.model ? '继承默认文本模型' : defaultText.statusText,
  };
}

function toBaseAgentItem(row: AgentModelRow, key: TextAgentKey): Omit<AgentConfigItem, 'disabled' | 'overrideEnabled' | 'effectiveModel' | 'status' | 'statusText'> {
  return {
    id: row.id,
    key,
    name: row.name ?? AGENT_LABELS[key],
    description: row.description,
    group: AGENT_GROUPS[key],
    modelLabel: row.model_label,
    modelId: row.model_id,
    vendorId: row.vendor_id,
    temperature: row.temperature,
    maxOutputTokens: row.max_output_tokens,
  };
}

function buildAgentConfigResult(): AgentConfigResult {
  for (const key of TEXT_AGENT_KEYS) {
    ensureAgentRow(key);
  }

  const options = getAvailableTextModels();
  const defaultText = getDefaultTextModel();
  const rows = getAgentRows();

  return {
    agents: rows.map((row) => buildAgentItem(row, options, defaultText)),
    availableTextModels: options,
    defaultTextModel: defaultText.model,
    defaultTextStatus: defaultText.status,
    defaultTextStatusText: defaultText.statusText,
    globalSettings: getGlobalSettings(),
  };
}

export function getAgentConfig(): AgentConfigResult {
  return buildAgentConfigResult();
}

export function saveAgentConfig(payload: AgentConfigSavePayload): AgentConfigSaveResult {
  const globalSettings = saveGlobalSettings(payload.globalSettings);
  const options = getAvailableTextModels();
  const now = Date.now();

  for (const item of payload.agents) {
    if (!isTextAgentKey(item.key)) {
      throw createError(VT_STATUS.INVALID_PARAMS, `未知 Agent：${item.key}`);
    }

    ensureAgentRow(item.key);
    const option = item.modelId ? getOptionByModelId(item.modelId, options) : null;
    if (item.modelId && !option) {
      throw createError(VT_STATUS.INVALID_PARAMS, '覆盖模型必须来自已启用文本模型');
    }

    getDatabase()
      .prepare<[string | null, string | null, string | null, number | null, number | null, number, string]>(
        `UPDATE agent_model_configs
           SET model_id = ?,
               vendor_id = ?,
               model_label = ?,
               temperature = ?,
               max_output_tokens = ?,
               updated_at = ?
         WHERE key = ?`,
      )
      .run(
        option?.modelId ?? null,
        option?.connectionId ?? null,
        option ? `${option.connectionName} / ${option.modelDisplayName}` : null,
        normalizeNullableTemperature(item.temperature),
        normalizeNullableMaxOutputTokens(item.maxOutputTokens),
        now,
        item.key,
      );
  }

  return {
    agents: buildAgentConfigResult().agents,
    globalSettings,
  };
}

export function resolveTextAgentModel(modelKey: TextAgentKey): { modelId: string; agentConfig: AgentModelConfig } {
  const row = ensureAgentRow(modelKey);
  const config = toAgentModelConfig(row);
  const globalSettings = getGlobalSettings();
  const effectiveConfig: AgentModelConfig = {
    ...config,
    temperature: config.temperature ?? globalSettings.temperature,
    maxOutputTokens: config.maxOutputTokens ?? globalSettings.maxOutputTokens,
  };

  if (config.modelId) {
    const option = getOptionByModelId(config.modelId, getAvailableTextModels());
    if (!option) {
      throw createError(VT_STATUS.MODEL_NOT_CONFIGURED, `${config.name ?? modelKey} 的覆盖模型已失效，请在 Agent 高级设置中修正`);
    }

    return { modelId: config.modelId, agentConfig: effectiveConfig };
  }

  const defaultText = getDefaultTextModel();
  if (!defaultText.model) {
    throw createError(VT_STATUS.MODEL_NOT_CONFIGURED, defaultText.statusText);
  }

  return { modelId: defaultText.model.modelId, agentConfig: effectiveConfig };
}

export function isTextAgentModelKey(value: string): value is TextAgentKey {
  return isTextAgentKey(value);
}
