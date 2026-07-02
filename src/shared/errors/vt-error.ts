import { getStatusMsg, VT_STATUS, type VtStatusCode } from '../constants/status';

export interface VtErrorOptions {
  statusCode?: VtStatusCode;
  msg?: string;
  errorKey?: string;
  detail?: unknown;
  cause?: unknown;
}

export class VtError extends Error {
  readonly statusCode: VtStatusCode;
  readonly errorKey?: string;
  readonly detail?: unknown;
  override readonly cause?: unknown;

  constructor(options: VtErrorOptions = {}) {
    const statusCode = options.statusCode ?? VT_STATUS.FAIL;

    super(options.msg ?? getStatusMsg(statusCode));
    this.name = 'VtError';
    this.statusCode = statusCode;
    this.errorKey = options.errorKey;
    this.detail = options.detail;
    this.cause = options.cause;
  }
}

export function isVtError(error: unknown): error is VtError {
  return error instanceof VtError;
}
