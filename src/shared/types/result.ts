import type { VtError } from '../errors';

export type VtResult<TData> = { success: true; data: TData } | { success: false; error: VtError };

export function createResult<TData>(data: TData): VtResult<TData> {
  return { success: true, data };
}

export function createErrorResult<TData = never>(error: VtError): VtResult<TData> {
  return { success: false, error };
}
