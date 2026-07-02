import { existsSync, readFileSync } from 'node:fs';
import { basename } from 'node:path';
import { VT_STATUS } from '@shared/constants/status';
import { getDatabase } from '../database';
import { getRuntimeDirectories, writeManagedFile } from '../file-system';
import { createError } from '../result';
import type { VendorModelConfig } from './types';

export interface VendorRow {
  id: string;
  input_values: string;
  models: string;
  enabled: number;
  created_at: number;
  updated_at: number;
}

function normalizeVendorId(vendorId: string): string {
  const normalized = vendorId.trim();

  if (!normalized || normalized.includes(':') || basename(normalized) !== normalized) {
    throw createError(VT_STATUS.INVALID_PARAMS, '供应商 ID 无效');
  }

  return normalized;
}

export function getVendorCodeRelativePath(vendorId: string): string {
  return `${normalizeVendorId(vendorId)}.ts`;
}

export function getVendorCodePath(vendorId: string): string {
  return `${getRuntimeDirectories().vendors}\\${getVendorCodeRelativePath(vendorId)}`;
}

export function readVendorCode(vendorId: string): string {
  const targetPath = getVendorCodePath(vendorId);

  if (!existsSync(targetPath)) {
    throw createError(VT_STATUS.MODEL_VENDOR_NOT_FOUND, `供应商代码不存在：${vendorId}`);
  }

  return readFileSync(targetPath, 'utf-8');
}

export function writeVendorCode(vendorId: string, code: string): string {
  return writeManagedFile(getRuntimeDirectories().vendors, getVendorCodeRelativePath(vendorId), code);
}

export function getVendorRow(vendorId: string): VendorRow {
  const row = getDatabase().prepare<[string], VendorRow>('SELECT * FROM model_vendors WHERE id = ? LIMIT 1').get(normalizeVendorId(vendorId));

  if (!row) {
    throw createError(VT_STATUS.MODEL_VENDOR_NOT_FOUND);
  }

  return row;
}

export function getVendorRows(): VendorRow[] {
  return getDatabase().prepare<[], VendorRow>("SELECT * FROM model_vendors ORDER BY CASE WHEN id = 'toonflow' THEN 0 ELSE 1 END, id ASC").all();
}

export function upsertVendorRecord(input: {
  id: string;
  inputValues: Record<string, string>;
  models?: VendorModelConfig[];
  enabled?: boolean;
}): void {
  const now = Date.now();
  const existing = getDatabase().prepare<[string], Pick<VendorRow, 'id'> | undefined>('SELECT id FROM model_vendors WHERE id = ? LIMIT 1').get(input.id);

  if (existing) {
    getDatabase()
      .prepare<[string, string, number, number, string]>(
        'UPDATE model_vendors SET input_values = ?, models = ?, enabled = ?, updated_at = ? WHERE id = ?',
      )
      .run(JSON.stringify(input.inputValues), JSON.stringify(input.models ?? []), input.enabled ? 1 : 0, now, input.id);
    return;
  }

  getDatabase()
    .prepare<[string, string, string, number, number, number]>(
      'INSERT INTO model_vendors (id, input_values, models, enabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)',
    )
    .run(input.id, JSON.stringify(input.inputValues), JSON.stringify(input.models ?? []), input.enabled ? 1 : 0, now, now);
}

export function parseJsonObject(value: string | null | undefined): Record<string, string> {
  if (!value) {
    return {};
  }

  try {
    const parsed = JSON.parse(value);
    if (typeof parsed === 'object' && parsed !== null && !Array.isArray(parsed)) {
      return Object.fromEntries(Object.entries(parsed).map(([key, val]) => [key, typeof val === 'string' ? val : String(val ?? '')]));
    }
  } catch {
    return {};
  }

  return {};
}

export function parseModelList(value: string | null | undefined): VendorModelConfig[] {
  if (!value) {
    return [];
  }

  try {
    const parsed = JSON.parse(value);
    return Array.isArray(parsed) ? (parsed as VendorModelConfig[]) : [];
  } catch {
    return [];
  }
}
