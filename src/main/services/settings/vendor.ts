import { existsSync } from 'node:fs';
import { VT_STATUS } from '@shared/constants/status';
import type {
  VendorAddCodePayload,
  VendorAddCodeResult,
  VendorCodePayload,
  VendorCodeResult,
  VendorDeleteModelPayload,
  VendorDeleteModelResult,
  VendorDeletePayload,
  VendorDeleteResult,
  VendorListItem,
  VendorListResult,
  VendorModelPayload,
  VendorModelSaveResult,
  VendorSetEnabledPayload,
  VendorSetEnabledResult,
  VendorTestImagePayload,
  VendorTestMediaResult,
  VendorTestTextPayload,
  VendorTestTextResult,
  VendorTestVideoPayload,
  VendorUpdateCodeResult,
  VendorUpdateInputsPayload,
  VendorUpdateInputsResult,
} from '@shared/types/vendor';
import { getDatabase } from '../database';
import { deleteManagedFile, getRuntimeDirectories } from '../file-system';
import { createError } from '../result';
import {
  addVendorFromCode,
  getVendor,
  setVendorEnabled,
  testImageModel,
  testTextModel,
  testVideoModel,
  updateVendorCode,
  updateVendorInputs,
  validateVendorCode,
} from '../model';
import { getBuiltinVendorDefinition, getBuiltinVendorIds, isBuiltinVendor } from '../model/builtin-vendors';
import { getVendorCodePath, getVendorCodeRelativePath, getVendorRows, parseJsonObject, parseModelList } from '../model/storage';
import type { VendorManifest, VendorModelConfig, VideoMode } from '../model/types';
import { normalizeVendorManifest } from '../model/validation';

interface ReferenceItem {
  type: 'agent' | 'project' | 'modelPrompt';
  name: string;
  detail: string;
}

function ensureBuiltinVendorRows(): void {
  const now = Date.now();
  const stmt = getDatabase().prepare<[string, string, string, number, number, number]>(
    'INSERT INTO model_vendors (id, input_values, models, enabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)',
  );

  for (const id of getBuiltinVendorIds()) {
    const exists = getDatabase().prepare<[string], { n: number }>('SELECT COUNT(*) as n FROM model_vendors WHERE id = ?').get(id);
    if (!exists || exists.n === 0) {
      stmt.run(id, '{}', '[]', 0, now, now);
    }
  }
}

function toStatusText(error: unknown): string {
  return error instanceof Error ? error.message : '供应商配置不可用';
}

function getVendorCapabilities(vendorId: string, models: VendorModelConfig[]): VendorListItem['capabilities'] {
  const builtin = getBuiltinVendorDefinition(vendorId);
  if (builtin) {
    return [...builtin.capabilities];
  }

  return [...new Set(models.map((model) => model.type))] as VendorListItem['capabilities'];
}

function manifestToListItem(manifest: VendorManifest, row: { enabled: number; input_values: string; models: string }, status: VendorListItem['status'], codeReady: boolean, statusText: string): VendorListItem {
  const inputValues = { ...manifest.inputValues, ...parseJsonObject(row.input_values) };
  const models = parseModelList(row.models);
  const mergedModels = mergeModels(manifest.models, models);
  const builtin = isBuiltinVendor(manifest.id);

  return {
    id: manifest.id,
    name: manifest.name,
    description: manifest.description,
    icon: manifest.icon,
    author: manifest.author,
    version: manifest.version,
    enabled: row.enabled === 1,
    builtin,
    codeEditable: !builtin || codeReady,
    codeReady,
    status,
    statusText,
    capabilities: getVendorCapabilities(manifest.id, mergedModels),
    inputs: manifest.inputs,
    inputValues,
    models: mergedModels,
  };
}

function mergeModels(baseModels: VendorModelConfig[], customModels: VendorModelConfig[]): VendorModelConfig[] {
  const map = new Map<string, VendorModelConfig>();
  for (const model of [...baseModels, ...customModels]) {
    map.set(model.modelName, model);
  }

  return [...map.values()];
}

function getFallbackManifest(vendorId: string): VendorManifest {
  const builtin = getBuiltinVendorDefinition(vendorId);
  if (builtin) {
    return normalizeVendorManifest(builtin.manifest);
  }

  return {
    id: vendorId,
    name: vendorId,
    author: 'Unknown',
    description: '供应商 adapter 不可用',
    inputs: [],
    inputValues: {},
    models: [],
  };
}

function getVendorReferences(vendorId: string, modelName?: string): ReferenceItem[] {
  const references: ReferenceItem[] = [];
  const rows = getDatabase()
    .prepare<[string, string], { key: string; name: string | null; model_id: string | null }>(
      'SELECT key, name, model_id FROM agent_model_configs WHERE vendor_id = ? OR model_id LIKE ?',
    )
    .all(vendorId, `${vendorId}:%`);

  for (const row of rows) {
    if (modelName && row.model_id !== `${vendorId}:${modelName}`) {
      continue;
    }

    references.push({
      type: 'agent',
      name: row.name ?? row.key,
      detail: row.model_id ?? vendorId,
    });
  }

  return references;
}

function assertNoReferences(vendorId: string, modelName?: string): void {
  const references = getVendorReferences(vendorId, modelName);
  if (references.length === 0) {
    return;
  }

  const first = references[0];
  throw createError(VT_STATUS.CONFLICT, `当前配置仍被 ${first.name} 引用，请先解除引用后再删除`);
}

function assertEditableVendor(vendorId: string): void {
  if (isBuiltinVendor(vendorId)) {
    throw createError(VT_STATUS.FORBIDDEN, '内置供应商不能删除，请使用禁用');
  }
}

function assertModelPayload(model: VendorModelConfig): VendorModelConfig {
  const normalized = normalizeVendorManifest({
    id: 'payload',
    name: 'payload',
    author: 'payload',
    inputs: [],
    inputValues: {},
    models: [model],
  });

  return normalized.models[0];
}

function normalizeModelPayload(model: VendorModelPayload['model']): VendorModelConfig {
  if (model.type !== 'video') {
    return assertModelPayload(model as VendorModelConfig);
  }

  const videoModel: VendorModelConfig = {
    ...model,
    mode: model.mode.map((mode) => {
      if (Array.isArray(mode)) {
        return mode as VideoMode;
      }

      return mode as VideoMode;
    }),
  };

  return assertModelPayload(videoModel);
}

function updateVendorModels(vendorId: string, updater: (models: VendorModelConfig[]) => VendorModelConfig[]): void {
  const vendor = getVendor(vendorId);
  const nextModels = updater(vendor.models);
  getDatabase()
    .prepare<[string, number, string]>('UPDATE model_vendors SET models = ?, updated_at = ? WHERE id = ?')
    .run(JSON.stringify(nextModels), Date.now(), vendorId);
}

export function getVendorList(): VendorListResult {
  ensureBuiltinVendorRows();

  const rows = getVendorRows();
  const vendors: VendorListItem[] = rows.map((row) => {
    try {
      const vendor = getVendor(row.id);
      return manifestToListItem(vendor.manifest, row, 'ready', vendor.codeReady, vendor.codeReady ? 'adapter 已加载' : '内置 adapter');
    } catch (error) {
      const codePath = getVendorCodePath(row.id);
      return manifestToListItem(
        getFallbackManifest(row.id),
        row,
        existsSync(codePath) ? 'invalid' : 'missing-code',
        false,
        toStatusText(error),
      );
    }
  });

  return { vendors };
}

export function saveVendorInputs(payload: VendorUpdateInputsPayload): VendorUpdateInputsResult {
  const vendor = getVendor(payload.vendorId);
  const allowedKeys = new Set(vendor.manifest.inputs.map((input) => input.key));
  const sanitized = Object.fromEntries(
    Object.entries(payload.inputValues).filter(([key, value]) => allowedKeys.has(key) && typeof value === 'string'),
  );

  updateVendorInputs(payload.vendorId, sanitized);
  return { vendorId: payload.vendorId };
}

export function saveVendorEnabled(payload: VendorSetEnabledPayload): VendorSetEnabledResult {
  getVendor(payload.vendorId);
  setVendorEnabled(payload.vendorId, payload.enabled);
  return { vendorId: payload.vendorId, enabled: payload.enabled };
}

export function saveVendorModel(payload: VendorModelPayload): VendorModelSaveResult {
  const model = normalizeModelPayload(payload.model);

  updateVendorModels(payload.vendorId, (models) => {
    const originalModelName = payload.originalModelName ?? model.modelName;
    const duplicate = models.find((item) => item.modelName === model.modelName && item.modelName !== originalModelName);
    if (duplicate) {
      throw createError(VT_STATUS.CONFLICT, '同一供应商下模型 ID 已存在');
    }

    const exists = models.some((item) => item.modelName === originalModelName);
    if (!exists) {
      return [...models, model];
    }

    return models.map((item) => (item.modelName === originalModelName ? model : item));
  });

  return { vendorId: payload.vendorId, modelName: model.modelName };
}

export function deleteVendorModel(payload: VendorDeleteModelPayload): VendorDeleteModelResult {
  assertNoReferences(payload.vendorId, payload.modelName);

  updateVendorModels(payload.vendorId, (models) => {
    if (!models.some((item) => item.modelName === payload.modelName)) {
      throw createError(VT_STATUS.MODEL_NOT_FOUND, '模型不存在');
    }

    return models.filter((item) => item.modelName !== payload.modelName);
  });

  return { vendorId: payload.vendorId, modelName: payload.modelName };
}

export function getVendorCode(payload: VendorDeletePayload): VendorCodeResult {
  const vendor = getVendor(payload.vendorId);
  if (!vendor.codeReady && isBuiltinVendor(vendor.id)) {
    throw createError(VT_STATUS.FORBIDDEN, '内置供应商当前使用固定 adapter，不提供代码编辑');
  }

  return {
    vendorId: vendor.id,
    code: vendor.code,
    editable: !isBuiltinVendor(vendor.id) || vendor.codeReady,
  };
}

export function addVendorCode(payload: VendorAddCodePayload): VendorAddCodeResult {
  const manifest = validateVendorCode(payload.code);
  addVendorFromCode(payload.code);
  return { vendorId: manifest.id };
}

export function saveVendorCode(payload: VendorCodePayload): VendorUpdateCodeResult {
  updateVendorCode(payload.vendorId, payload.code);
  return { vendorId: payload.vendorId };
}

export function deleteVendor(payload: VendorDeletePayload): VendorDeleteResult {
  assertEditableVendor(payload.vendorId);
  assertNoReferences(payload.vendorId);
  getVendor(payload.vendorId);

  getDatabase().prepare<[string]>('DELETE FROM model_vendors WHERE id = ?').run(payload.vendorId);
  const codePath = getVendorCodePath(payload.vendorId);
  if (existsSync(codePath)) {
    deleteManagedFile(getRuntimeDirectories().vendors, getVendorCodeRelativePath(payload.vendorId));
  }
  return { vendorId: payload.vendorId };
}

export async function runVendorTextTest(payload: VendorTestTextPayload): Promise<VendorTestTextResult> {
  const startedAt = Date.now();
  const result = await testTextModel({
    vendorId: payload.vendorId,
    modelName: payload.modelName,
    messages: [{ role: 'user', content: payload.prompt }],
  });

  return { ...result, durationMs: Date.now() - startedAt };
}

export async function runVendorImageTest(payload: VendorTestImagePayload): Promise<VendorTestMediaResult> {
  const startedAt = Date.now();
  const result = await testImageModel(payload);
  return { ...result, durationMs: Date.now() - startedAt };
}

export async function runVendorVideoTest(payload: VendorTestVideoPayload): Promise<VendorTestMediaResult> {
  const startedAt = Date.now();
  const result = await testVideoModel({
    vendorId: payload.vendorId,
    modelName: payload.modelName,
    mode: payload.mode,
    prompt: payload.prompt,
    images: [],
    videos: [],
    audios: [],
  });
  return { ...result, durationMs: Date.now() - startedAt };
}
