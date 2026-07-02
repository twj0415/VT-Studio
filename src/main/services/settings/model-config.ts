import { randomUUID } from 'node:crypto';
import { VT_STATUS } from '@shared/constants/status';
import type {
  ApiConnection,
  ApiConnectionDeletePayload,
  ApiConnectionDeleteResult,
  ApiConnectionDraft,
  ApiConnectionListResult,
  ApiConnectionSavePayload,
  ApiConnectionSaveResult,
  ApiConnectionTestPayload,
  ApiConnectionTestResult,
  ApiProtocolType,
  ApiServiceType,
  CapabilityBindingMap,
  CapabilitySummary,
  ModelCapability,
  RegisteredModel,
  ResourceBindingSavePayload,
  ResourceBindingSaveResult,
  ResourceConfigResult,
  ResourceTestPayload,
  ResourceTestResult,
} from '@shared/types/model-config';
import { getDatabase } from '../database';
import { createError } from '../result';
import type { VendorModelConfig } from '../model/types';
import { getVendorRows, parseJsonObject, parseModelList, upsertVendorRecord } from '../model/storage';
import { getBuiltinVendorDefinition } from '../model/builtin-vendors';
import { runVendorTextTest } from './vendor';

const CONNECTIONS_KEY = 'modelConnections.v1';
const BINDINGS_KEY = 'modelCapabilityBindings.v1';
const HIDDEN_ADAPTER_KEY = '__adapterVendorId';
const HIDDEN_CONNECTION_NAME_KEY = '__connectionName';

const CAPABILITY_LABELS: Record<ModelCapability, string> = {
  text: '文本生成',
  image: '图片生成',
  video: '视频生成',
  tts: '语音生成',
};

const SERVICE_META: Record<
  ApiServiceType,
  {
    name: string;
    protocolType: ApiProtocolType;
    adapterVendorId: string;
    defaultBaseUrl: string;
    capabilities: ModelCapability[];
    models: RegisteredModel[];
  }
> = {
  'openai-official': {
    name: 'OpenAI 官方',
    protocolType: 'openai-official',
    adapterVendorId: 'openai',
    defaultBaseUrl: 'https://api.openai.com/v1',
    capabilities: ['text', 'image', 'tts'],
    models: [
      { id: 'gpt-5.5', displayName: 'GPT-5.5', modelName: 'gpt-5.5', type: 'text', think: true },
      { id: 'gpt-4.1-mini', displayName: 'GPT-4.1 mini', modelName: 'gpt-4.1-mini', type: 'text', think: false },
      { id: 'gpt-image-1', displayName: 'GPT Image 1', modelName: 'gpt-image-1', type: 'image' },
    ],
  },
  'openai-gateway': {
    name: 'OpenAI 中转',
    protocolType: 'openai-compatible',
    adapterVendorId: 'atlascloud',
    defaultBaseUrl: '',
    capabilities: ['text', 'image'],
    models: [
      { id: 'gpt-5.5', displayName: 'GPT-5.5', modelName: 'gpt-5.5', type: 'text', think: true },
      { id: 'gpt-5.4', displayName: 'GPT-5.4', modelName: 'gpt-5.4', type: 'text', think: false },
      { id: 'gpt-image-2', displayName: 'GPT Image 2', modelName: 'gpt-image-2', type: 'image' },
    ],
  },
  claude: {
    name: 'Claude',
    protocolType: 'anthropic',
    adapterVendorId: 'anthropic',
    defaultBaseUrl: 'https://api.anthropic.com/v1',
    capabilities: ['text'],
    models: [{ id: 'claude-sonnet-4-5', displayName: 'Claude Sonnet 4.5', modelName: 'claude-sonnet-4-5', type: 'text', think: true }],
  },
  deepseek: {
    name: 'DeepSeek',
    protocolType: 'deepseek',
    adapterVendorId: 'deepseek',
    defaultBaseUrl: 'https://api.deepseek.com',
    capabilities: ['text'],
    models: [{ id: 'deepseek-chat', displayName: 'DeepSeek Chat', modelName: 'deepseek-chat', type: 'text', think: false }],
  },
  gemini: {
    name: 'Gemini',
    protocolType: 'gemini',
    adapterVendorId: 'gemini',
    defaultBaseUrl: 'https://generativelanguage.googleapis.com/v1beta',
    capabilities: ['text', 'image', 'video'],
    models: [{ id: 'gemini-2.5-flash', displayName: 'Gemini 2.5 Flash', modelName: 'gemini-2.5-flash', type: 'text', think: true }],
  },
  'local-workflow': {
    name: '本地工作流',
    protocolType: 'workflow',
    adapterVendorId: 'comfyui',
    defaultBaseUrl: 'http://127.0.0.1:8188',
    capabilities: ['image'],
    models: [{ id: 'comfyui-workflow', displayName: 'ComfyUI Workflow', modelName: 'comfyui-workflow', type: 'image' }],
  },
  advanced: {
    name: '其他高级接入',
    protocolType: 'custom-adapter',
    adapterVendorId: 'atlascloud',
    defaultBaseUrl: '',
    capabilities: ['text'],
    models: [{ id: 'custom-chat-model', displayName: '自定义文本模型', modelName: 'custom-chat-model', type: 'text', think: false }],
  },
};

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

function createConnectionId(): string {
  return `conn_${randomUUID().replace(/-/g, '').slice(0, 16)}`;
}

function normalizeModels(models: RegisteredModel[], serviceType: ApiServiceType): RegisteredModel[] {
  const fallback = SERVICE_META[serviceType].models;
  const source = models.length > 0 ? models : fallback;
  const seen = new Set<string>();

  return source
    .map((model) => ({
      id: model.id?.trim() || model.modelName.trim(),
      displayName: model.displayName?.trim() || model.modelName.trim(),
      modelName: model.modelName.trim(),
      type: model.type,
      think: model.type === 'text' ? Boolean(model.think) : undefined,
    }))
    .filter((model) => {
      if (!model.modelName || seen.has(model.modelName)) {
        return false;
      }

      seen.add(model.modelName);
      return true;
    });
}

function deriveCapabilitiesFromModels(models: RegisteredModel[]): ModelCapability[] {
  return [...new Set(models.map((model) => model.type))] as ModelCapability[];
}

function toVendorModel(model: RegisteredModel): VendorModelConfig {
  const base = {
    name: model.displayName,
    modelName: model.modelName,
  };

  if (model.type === 'text') {
    return { ...base, type: 'text', think: Boolean(model.think) };
  }

  if (model.type === 'image') {
    return { ...base, type: 'image', mode: ['text'] };
  }

  if (model.type === 'video') {
    return {
      ...base,
      type: 'video',
      mode: ['text'],
      audio: 'optional',
      durationResolutionMap: [{ duration: [5], resolution: ['720p'] }],
    };
  }

  return {
    ...base,
    type: 'tts',
    voices: [{ title: 'Default', voice: 'default' }],
  };
}

function getConnectionStatus(connection: Pick<ApiConnection, 'apiKey' | 'serviceType' | 'baseUrl' | 'models'>): Pick<ApiConnection, 'status' | 'statusText'> {
  if (!connection.apiKey.trim() && connection.serviceType !== 'local-workflow') {
    return { status: 'incomplete', statusText: '缺少 API Key' };
  }

  if (connection.serviceType === 'openai-gateway' && !connection.baseUrl.trim()) {
    return { status: 'incomplete', statusText: '中转服务需要 Base URL' };
  }

  if (connection.models.length === 0) {
    return { status: 'incomplete', statusText: '至少需要登记一个模型' };
  }

  return { status: 'ready', statusText: '配置完整' };
}

function normalizeDraft(draft: ApiConnectionDraft, previous?: ApiConnection): ApiConnection {
  const now = Date.now();
  const serviceType = draft.serviceType;
  const meta = SERVICE_META[serviceType];
  const models = normalizeModels(draft.models ?? [], serviceType);
  const capabilities = deriveCapabilitiesFromModels(models);
  const base = {
    id: draft.id?.trim() || previous?.id || createConnectionId(),
    name: draft.name.trim() || meta.name,
    serviceType,
    protocolType: meta.protocolType,
    baseUrl: draft.baseUrl.trim(),
    apiKey: draft.apiKey.trim(),
    capabilities,
    models,
    createdAt: previous?.createdAt ?? now,
    updatedAt: now,
  };

  return {
    ...base,
    ...getConnectionStatus(base),
  };
}

function getStoredConnections(): ApiConnection[] {
  const connections = getSettingJson<ApiConnection[]>(CONNECTIONS_KEY, []);
  if (Array.isArray(connections) && connections.length > 0) {
    return connections;
  }

  const migrated = migrateLegacyVendorConnections();
  if (migrated.length > 0) {
    saveConnections(migrated);
  }

  return migrated;
}

function saveConnections(connections: ApiConnection[]): void {
  saveSettingJson(CONNECTIONS_KEY, connections);
}

function getServiceTypeFromVendor(vendorId: string, inputValues: Record<string, string>): ApiServiceType | null {
  if (vendorId === 'openai') {
    const baseUrl = inputValues.baseUrl?.trim() ?? '';
    return baseUrl && !baseUrl.includes('api.openai.com') ? 'openai-gateway' : 'openai-official';
  }

  const map: Partial<Record<string, ApiServiceType>> = {
    anthropic: 'claude',
    deepseek: 'deepseek',
    gemini: 'gemini',
    comfyui: 'local-workflow',
    atlascloud: 'openai-gateway',
  };

  return map[vendorId] ?? null;
}

function migrateLegacyVendorConnections(): ApiConnection[] {
  const now = Date.now();
  const connections: ApiConnection[] = [];

  for (const row of getVendorRows()) {
    const inputValues = parseJsonObject(row.input_values);
    if (!inputValues.apiKey?.trim() && !inputValues.baseUrl?.trim() && row.enabled !== 1) {
      continue;
    }

    const serviceType = getServiceTypeFromVendor(row.id, inputValues);
    if (!serviceType) {
      continue;
    }

    const meta = SERVICE_META[serviceType];
    const builtin = getBuiltinVendorDefinition(meta.adapterVendorId);
    const storedModels = parseModelList(row.models);
    const sourceModels = storedModels.length > 0 ? storedModels : builtin?.manifest.models ?? [];
    const models = sourceModels.map((model) => ({
      id: model.modelName,
      displayName: model.name,
      modelName: model.modelName,
      type: model.type,
      think: model.type === 'text' ? model.think : undefined,
    }));
    const base = {
      id: `conn_migrated_${row.id}`,
      name: `${meta.name}（已迁移）`,
      serviceType,
      protocolType: meta.protocolType,
      baseUrl: inputValues.baseUrl ?? inputValues.endpoint ?? meta.defaultBaseUrl,
      apiKey: inputValues.apiKey ?? '',
      capabilities: deriveCapabilitiesFromModels(models),
      models,
      createdAt: row.created_at || now,
      updatedAt: now,
    };
    const connection = {
      ...base,
      ...getConnectionStatus(base),
    };

    syncConnectionToVendor(connection);
    connections.push(connection);
  }

  return connections;
}

function getBindings(): CapabilityBindingMap {
  const bindings = getSettingJson<CapabilityBindingMap>(BINDINGS_KEY, {});
  return bindings && typeof bindings === 'object' ? bindings : {};
}

function saveBindings(bindings: CapabilityBindingMap): void {
  saveSettingJson(BINDINGS_KEY, bindings);
}

function isBindingValid(connections: ApiConnection[], capability: ModelCapability, binding: CapabilityBindingMap[ModelCapability]): boolean {
  if (!binding) {
    return false;
  }

  const connection = connections.find((item) => item.id === binding.connectionId);
  const model = connection?.models.find((item) => item.modelName === binding.modelName);
  return Boolean(connection && connection.status === 'ready' && model?.type === capability);
}

function findDefaultBinding(connections: ApiConnection[], capability: ModelCapability): CapabilityBindingMap[ModelCapability] {
  for (const connection of connections) {
    if (connection.status !== 'ready') {
      continue;
    }

    const model = connection.models.find((item) => item.type === capability);
    if (model) {
      return {
        connectionId: connection.id,
        modelName: model.modelName,
      };
    }
  }

  return undefined;
}

function ensureDefaultBindings(connections: ApiConnection[]): void {
  const bindings = getBindings();
  let changed = false;

  for (const capability of Object.keys(CAPABILITY_LABELS) as ModelCapability[]) {
    if (isBindingValid(connections, capability, bindings[capability])) {
      continue;
    }

    const fallback = findDefaultBinding(connections, capability);
    if (fallback) {
      bindings[capability] = fallback;
      changed = true;
    } else if (bindings[capability]) {
      delete bindings[capability];
      changed = true;
    }
  }

  if (changed) {
    saveBindings(bindings);
  }
}

function syncConnectionToVendor(connection: ApiConnection): void {
  const meta = SERVICE_META[connection.serviceType];
  upsertVendorRecord({
    id: connection.id,
    inputValues: {
      apiKey: connection.apiKey,
      baseUrl: connection.baseUrl,
      endpoint: connection.baseUrl,
      [HIDDEN_ADAPTER_KEY]: meta.adapterVendorId,
      [HIDDEN_CONNECTION_NAME_KEY]: connection.name,
    },
    models: connection.models.map(toVendorModel),
    enabled: connection.status === 'ready',
  });
}

function assertConnectionReady(connection: ApiConnection): void {
  if (connection.status !== 'ready') {
    throw createError(VT_STATUS.MODEL_NOT_CONFIGURED, connection.statusText);
  }
}

function findConnection(connectionId: string): ApiConnection {
  const connection = getStoredConnections().find((item) => item.id === connectionId);
  if (!connection) {
    throw createError(VT_STATUS.NOT_FOUND, '连接不存在');
  }

  return connection;
}

function findModel(connection: ApiConnection, modelName: string): RegisteredModel {
  const model = connection.models.find((item) => item.modelName === modelName);
  if (!model) {
    throw createError(VT_STATUS.MODEL_NOT_FOUND, '模型不存在');
  }

  return model;
}

function buildCapabilitySummaries(connections: ApiConnection[], bindings: CapabilityBindingMap): CapabilitySummary[] {
  return (Object.keys(CAPABILITY_LABELS) as ModelCapability[]).map((capability) => {
    const binding = bindings[capability] ?? null;
    const connection = binding ? connections.find((item) => item.id === binding.connectionId) : null;
    const model = connection && binding ? connection.models.find((item) => item.modelName === binding.modelName) : null;

    if (!binding) {
      return {
        capability,
        label: CAPABILITY_LABELS[capability],
        binding: null,
        connectionName: '未选择',
        modelDisplayName: '未选择',
        modelName: '',
        status: 'missing',
        statusText: '未配置',
      };
    }

    if (!connection || !model) {
      return {
        capability,
        label: CAPABILITY_LABELS[capability],
        binding,
        connectionName: connection?.name ?? '连接不存在',
        modelDisplayName: model?.displayName ?? '模型不存在',
        modelName: binding.modelName,
        status: 'unsupported',
        statusText: '绑定已失效',
      };
    }

    if (model.type !== capability) {
      return {
        capability,
        label: CAPABILITY_LABELS[capability],
        binding,
        connectionName: connection.name,
        modelDisplayName: model.displayName,
        modelName: model.modelName,
        status: 'unsupported',
        statusText: '模型能力不匹配',
      };
    }

    return {
      capability,
      label: CAPABILITY_LABELS[capability],
      binding,
      connectionName: connection.name,
      modelDisplayName: model.displayName,
      modelName: model.modelName,
      status: connection.status === 'ready' ? 'configured' : 'missing',
      statusText: connection.status === 'ready' ? '已配置' : connection.statusText,
    };
  });
}

function assertNoAgentReferences(connectionId: string): void {
  const row = getDatabase()
    .prepare<[string, string], { name: string | null; key: string }>('SELECT key, name FROM agent_model_configs WHERE vendor_id = ? OR model_id LIKE ? LIMIT 1')
    .get(connectionId, `${connectionId}:%`);

  if (row) {
    throw createError(VT_STATUS.CONFLICT, `当前连接仍被 ${row.name ?? row.key} 引用，请先解除引用`);
  }
}

export function getApiConnectionList(): ApiConnectionListResult {
  return { connections: getStoredConnections() };
}

export function saveApiConnection(payload: ApiConnectionSavePayload): ApiConnectionSaveResult {
  const connections = getStoredConnections();
  const previous = payload.connection.id ? connections.find((item) => item.id === payload.connection.id) : undefined;
  const connection = normalizeDraft(payload.connection, previous);
  const nextConnections = previous
    ? connections.map((item) => (item.id === connection.id ? connection : item))
    : [...connections, connection];

  syncConnectionToVendor(connection);
  saveConnections(nextConnections);
  ensureDefaultBindings(nextConnections);

  return { connection };
}

export function deleteApiConnection(payload: ApiConnectionDeletePayload): ApiConnectionDeleteResult {
  const connections = getStoredConnections();
  const connection = connections.find((item) => item.id === payload.connectionId);
  if (!connection) {
    throw createError(VT_STATUS.NOT_FOUND, '连接不存在');
  }

  const bindings = getBindings();
  const referencedCapability = (Object.keys(bindings) as ModelCapability[]).find((capability) => bindings[capability]?.connectionId === connection.id);
  if (referencedCapability) {
    throw createError(VT_STATUS.CONFLICT, `当前连接仍被${CAPABILITY_LABELS[referencedCapability]}使用，请先更换绑定`);
  }

  assertNoAgentReferences(connection.id);
  getDatabase().prepare<[string]>('DELETE FROM model_vendors WHERE id = ?').run(connection.id);
  saveConnections(connections.filter((item) => item.id !== connection.id));

  return { connectionId: connection.id };
}

export async function testApiConnection(payload: ApiConnectionTestPayload): Promise<ApiConnectionTestResult> {
  const connection = findConnection(payload.connectionId);
  const model = findModel(connection, payload.modelName);
  if (model.type !== 'text') {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '第一版仅支持文本模型测试，媒体测试会在后续能力任务接入');
  }

  assertConnectionReady(connection);
  syncConnectionToVendor(connection);

  return runVendorTextTest({
    vendorId: connection.id,
    modelName: model.modelName,
    prompt: payload.prompt,
  });
}

export function getResourceConfig(): ResourceConfigResult {
  const connections = getStoredConnections();
  const bindings = getBindings();
  return {
    connections,
    bindings,
    capabilities: buildCapabilitySummaries(connections, bindings),
  };
}

export function saveResourceBinding(payload: ResourceBindingSavePayload): ResourceBindingSaveResult {
  const bindings = getBindings();
  if (!payload.binding) {
    delete bindings[payload.capability];
    saveBindings(bindings);
    return { bindings };
  }

  const connection = findConnection(payload.binding.connectionId);
  const model = findModel(connection, payload.binding.modelName);
  if (model.type !== payload.capability) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模型类型和能力不匹配');
  }

  bindings[payload.capability] = payload.binding;
  saveBindings(bindings);
  return { bindings };
}

export async function testResourceBinding(payload: ResourceTestPayload): Promise<ResourceTestResult> {
  const binding = getBindings()[payload.capability];
  if (!binding) {
    throw createError(VT_STATUS.MODEL_NOT_CONFIGURED, `${CAPABILITY_LABELS[payload.capability]}未配置`);
  }

  return testApiConnection({
    connectionId: binding.connectionId,
    modelName: binding.modelName,
    prompt: payload.prompt,
  });
}

export function getConnectionTemplates(): { services: Array<{ serviceType: ApiServiceType; name: string; defaultBaseUrl: string; capabilities: ModelCapability[]; models: RegisteredModel[] }> } {
  return {
    services: (Object.keys(SERVICE_META) as ApiServiceType[]).map((serviceType) => {
      const meta = SERVICE_META[serviceType];
      return {
        serviceType,
        name: meta.name,
        defaultBaseUrl: meta.defaultBaseUrl,
        capabilities: meta.capabilities,
        models: meta.models,
      };
    }),
  };
}
