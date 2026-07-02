import { isVtError } from './vt-error';

export interface NormalizedError {
  name: string;
  message: string;
  statusCode?: number;
  errorKey?: string;
  stack?: string;
  detail?: unknown;
  cause?: NormalizedError;
  raw?: unknown;
}

function getObjectRecord(value: unknown): Record<string, unknown> | null {
  if (typeof value !== 'object' || value === null) {
    return null;
  }

  return value as Record<string, unknown>;
}

export function normalizeUnknownError(error: unknown): NormalizedError {
  if (isVtError(error)) {
    return {
      name: error.name,
      message: error.message,
      statusCode: error.statusCode,
      errorKey: error.errorKey,
      stack: error.stack,
      detail: error.detail,
      cause: error.cause ? normalizeUnknownError(error.cause) : undefined,
    };
  }

  if (error instanceof Error) {
    return {
      name: error.name || 'Error',
      message: error.message || '未知错误',
      stack: error.stack,
      cause: error.cause ? normalizeUnknownError(error.cause) : undefined,
    };
  }

  const record = getObjectRecord(error);

  if (record) {
    const message = typeof record.message === 'string' ? record.message : '未知错误';
    const name = typeof record.name === 'string' ? record.name : 'UnknownError';

    return {
      name,
      message,
      detail: record,
    };
  }

  return {
    name: 'UnknownError',
    message: String(error),
    raw: error,
  };
}
