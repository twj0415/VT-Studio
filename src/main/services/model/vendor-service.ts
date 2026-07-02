import { VT_STATUS } from '@shared/constants/status';
import { getDatabase } from '../database';
import { createError } from '../result';
import { parseJsonObject, parseModelList, readVendorCode, upsertVendorRecord, writeVendorCode, type VendorRow } from './storage';
import type { VendorManifest, VendorModelConfig, VendorRecord, VendorRuntime } from './types';
import { assertVendorRequiredInputs, normalizeVendorManifest } from './validation';
import { runVendorCode, validateVendorRuntime } from './vendor-runner';
import { createBuiltinConnectionRuntime, createBuiltinVendorRuntime, getBuiltinVendorDefinition, isBuiltinVendor } from './builtin-vendors';

function mergeModels(baseModels: VendorModelConfig[], customModels: VendorModelConfig[]): VendorModelConfig[] {
  const map = new Map<string, VendorModelConfig>();

  for (const model of [...baseModels, ...customModels]) {
    map.set(model.modelName, model);
  }

  return [...map.values()];
}

function getVendorRuntimeFromRow(row: VendorRow): { code: string; runtime: VendorRuntime; codeReady: boolean } {
  try {
    const code = readVendorCode(row.id);
    return {
      code,
      codeReady: true,
      runtime: validateVendorRuntime(runVendorCode(code)),
    };
  } catch (error) {
    const builtin = getBuiltinVendorDefinition(row.id);
    if (!builtin) {
      const inputValues = parseJsonObject(row.input_values);
      const adapterVendorId = inputValues.__adapterVendorId;
      const connectionName = inputValues.__connectionName || row.id;
      const connectionRuntime = adapterVendorId
        ? createBuiltinConnectionRuntime({
            connectionId: row.id,
            connectionName,
            adapterVendorId,
            inputValues,
            models: parseModelList(row.models),
          })
        : null;

      if (connectionRuntime) {
        return {
          code: '',
          codeReady: false,
          runtime: connectionRuntime,
        };
      }

      throw error;
    }

    return {
      code: '',
      codeReady: false,
      runtime: createBuiltinVendorRuntime(row.id, parseJsonObject(row.input_values), mergeModels(builtin.manifest.models, parseModelList(row.models)))!,
    };
  }
}

function rowToVendorRecord(row: VendorRow): VendorRecord {
  const { code, runtime, codeReady } = getVendorRuntimeFromRow(row);
  const manifest = normalizeVendorManifest(runtime.vendor);
  const inputValues = { ...manifest.inputValues, ...parseJsonObject(row.input_values) };
  const models = mergeModels(manifest.models, parseModelList(row.models));

  return {
    id: row.id,
    inputValues,
    models,
    enabled: row.enabled === 1,
    code,
    codeReady,
    builtin: isBuiltinVendor(row.id),
    manifest: {
      ...manifest,
      inputValues,
      models,
    },
  };
}

export function validateVendorCode(code: string): VendorManifest {
  return validateVendorRuntime(runVendorCode(code)).vendor!;
}

export function addVendorFromCode(code: string): VendorManifest {
  const manifest = validateVendorCode(code);
  const existing = getDatabase().prepare<[string], { id: string } | undefined>('SELECT id FROM model_vendors WHERE id = ? LIMIT 1').get(manifest.id);

  if (existing) {
    throw createError(VT_STATUS.CONFLICT, '供应商 ID 已存在');
  }

  writeVendorCode(manifest.id, code);
  upsertVendorRecord({
    id: manifest.id,
    inputValues: manifest.inputValues,
    models: [],
    enabled: manifest.id === 'toonflow',
  });

  return manifest;
}

export function updateVendorCode(vendorId: string, code: string): VendorManifest {
  const manifest = validateVendorCode(code);
  const row = getVendorRowRequired(vendorId);

  if (isBuiltinVendor(row.id) && manifest.id !== row.id) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '内置供应商 adapter 的 id 不能变更');
  }

  writeVendorCode(row.id, code);
  getDatabase()
    .prepare<[string, number, string]>('UPDATE model_vendors SET models = ?, updated_at = ? WHERE id = ?')
    .run(JSON.stringify(manifest.models ?? []), Date.now(), row.id);

  return manifest;
}

export function getVendorRowRequired(vendorId: string): VendorRow {
  const row = getDatabase().prepare<[string], VendorRow>('SELECT * FROM model_vendors WHERE id = ? LIMIT 1').get(vendorId);

  if (!row) {
    throw createError(VT_STATUS.MODEL_VENDOR_NOT_FOUND);
  }

  return row;
}

export function getVendor(vendorId: string): VendorRecord {
  return rowToVendorRecord(getVendorRowRequired(vendorId));
}

export function getVendorRuntime(vendorId: string): VendorRuntime {
  const vendor = getVendor(vendorId);
  const runtime = vendor.codeReady
    ? validateVendorRuntime(runVendorCode(vendor.code))
    : createBuiltinVendorRuntime(vendor.id, vendor.inputValues, vendor.models) ??
      createBuiltinConnectionRuntime({
        connectionId: vendor.id,
        connectionName: vendor.inputValues.__connectionName || vendor.manifest.name,
        adapterVendorId: vendor.inputValues.__adapterVendorId,
        inputValues: vendor.inputValues,
        models: vendor.models,
      });

  if (!runtime) {
    throw createError(VT_STATUS.MODEL_VENDOR_NOT_FOUND);
  }

  runtime.vendor = {
    ...vendor.manifest,
    inputValues: vendor.inputValues,
    models: vendor.models,
  };

  assertVendorRequiredInputs(runtime.vendor, vendor.inputValues);

  return runtime;
}

export function getVendorModelList(vendorId: string): VendorModelConfig[] {
  return getVendor(vendorId).models;
}

export function listVendors(): VendorRecord[] {
  const rows = getDatabase().prepare<[], VendorRow>('SELECT * FROM model_vendors').all();
  return rows.map(rowToVendorRecord);
}

export function updateVendorInputs(vendorId: string, inputValues: Record<string, string>): void {
  const row = getVendorRowRequired(vendorId);
  const vendor = rowToVendorRecord(row);
  const nextInputValues = { ...vendor.inputValues, ...inputValues };

  getDatabase()
    .prepare<[string, number, string]>('UPDATE model_vendors SET input_values = ?, updated_at = ? WHERE id = ?')
    .run(JSON.stringify(nextInputValues), Date.now(), vendorId);
}

export function setVendorEnabled(vendorId: string, enabled: boolean): void {
  getVendorRowRequired(vendorId);
  getDatabase()
    .prepare<[number, number, string]>('UPDATE model_vendors SET enabled = ?, updated_at = ? WHERE id = ?')
    .run(enabled ? 1 : 0, Date.now(), vendorId);
}
