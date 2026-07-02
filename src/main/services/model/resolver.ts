import { VT_STATUS } from '@shared/constants/status';
import { getDatabase } from '../database';
import { createError } from '../result';
import { AGENT_MODEL_KEYS, AGENT_USE_MODE, MODEL_SETTING_KEYS, MODEL_TYPES, type AgentModelKey, type ModelType } from './constants';
import type { AgentModelConfig, EnabledModelItem, ResolvedModelKey, VendorModelConfig } from './types';
import { getVendor, getVendorModelList } from './vendor-service';
import { isTextAgentModelKey, resolveTextAgentModel } from '../settings/agent-config';

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

function isAgentModelKey(value: string): value is AgentModelKey {
  return AGENT_MODEL_KEYS.includes(value as AgentModelKey);
}

function mapAgentRow(row: AgentModelRow): AgentModelConfig {
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

function getSettingValue(key: string): string | null {
  const row = getDatabase().prepare<[string], { value: string }>('SELECT value FROM app_settings WHERE key = ? LIMIT 1').get(key);
  return row?.value ?? null;
}

function getAgentConfigByKey(key: string): AgentModelConfig | null {
  const row = getDatabase().prepare<[string], AgentModelRow>('SELECT * FROM agent_model_configs WHERE key = ? LIMIT 1').get(key);
  return row ? mapAgentRow(row) : null;
}

function getMainAgentKey(key: string): string {
  return key.split(/:(.+)/)[0] ?? key;
}

function assertModelId(modelId: string | null | undefined, key: string): string {
  if (!modelId) {
    throw createError(VT_STATUS.MODEL_NOT_CONFIGURED, `未找到部署配置 ${key}`);
  }

  return modelId;
}

export function splitModelId(modelId: string): { vendorId: string; modelName: string } {
  const [vendorId, modelName] = modelId.split(/:(.+)/);

  if (!vendorId || !modelName) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模型 ID 格式必须是 vendorId:modelName');
  }

  return { vendorId, modelName };
}

export function resolveModelKey(modelKey: AgentModelKey | string): ResolvedModelKey {
  if (!isAgentModelKey(modelKey)) {
    splitModelId(modelKey);
    return {
      inputKey: modelKey,
      modelId: modelKey,
      agentConfig: null,
    };
  }

  if (isTextAgentModelKey(modelKey)) {
    const resolved = resolveTextAgentModel(modelKey);
    return {
      inputKey: modelKey,
      modelId: resolved.modelId,
      agentConfig: resolved.agentConfig,
    };
  }

  const agentUseMode = getSettingValue(MODEL_SETTING_KEYS.agentUseMode);

  if (agentUseMode === AGENT_USE_MODE.ADVANCED) {
    const config = getAgentConfigByKey(modelKey);
    if (!config) {
      throw createError(VT_STATUS.MODEL_NOT_CONFIGURED, `高级配置模式下，未找到对应的模型配置 ${modelKey}`);
    }

    return { inputKey: modelKey, modelId: assertModelId(config.modelId, modelKey), agentConfig: config };
  }

  if (agentUseMode === AGENT_USE_MODE.SIMPLE) {
    const mainKey = getMainAgentKey(modelKey);
    const config = getAgentConfigByKey(mainKey);
    if (!config) {
      throw createError(VT_STATUS.MODEL_NOT_CONFIGURED, `简易配置模式下，未找到部署配置 ${modelKey}`);
    }

    return { inputKey: modelKey, modelId: assertModelId(config.modelId, modelKey), agentConfig: config };
  }

  const config = getAgentConfigByKey(modelKey);
  if (config?.modelId) {
    return { inputKey: modelKey, modelId: config.modelId, agentConfig: config };
  }

  const fallbackConfig = getAgentConfigByKey(getMainAgentKey(modelKey));
  if (!fallbackConfig) {
    throw createError(VT_STATUS.MODEL_NOT_CONFIGURED, `未找到部署配置 ${modelKey}`);
  }

  return { inputKey: modelKey, modelId: assertModelId(fallbackConfig.modelId, modelKey), agentConfig: fallbackConfig };
}

export function getModelDetail(modelId: string): VendorModelConfig {
  const { vendorId, modelName } = splitModelId(modelId);
  const model = getVendorModelList(vendorId).find((item) => item.modelName === modelName);

  if (!model) {
    throw createError(VT_STATUS.MODEL_NOT_FOUND, `未找到模型 ${modelName}`);
  }

  return model;
}

export function getEnabledModelList(type: ModelType = MODEL_TYPES.ALL): EnabledModelItem[] {
  const rows = getDatabase().prepare<[], { id: string }>('SELECT id FROM model_vendors WHERE enabled = 1 ORDER BY id ASC').all();
  const result: EnabledModelItem[] = [];

  for (const row of rows) {
    const vendor = getVendor(row.id);
    const models = vendor.models.filter((model) => {
      if (type === MODEL_TYPES.ALL) {
        return model.type !== MODEL_TYPES.VIDEO;
      }

      return model.type === type;
    });

    for (const model of models) {
      result.push({
        vendorId: vendor.id,
        vendorName: vendor.manifest.name,
        label: model.name,
        value: model.modelName,
        modelId: `${vendor.id}:${model.modelName}`,
        type: model.type,
      });
    }
  }

  return result;
}

export function getAgentModelDetail(key: 'scriptAgent' | 'productionAgent'): VendorModelConfig {
  return getModelDetail(resolveModelKey(key).modelId);
}
