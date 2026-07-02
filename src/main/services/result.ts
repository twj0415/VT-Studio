import { getStatusMsg, VT_STATUS, type VtStatusCode } from '@shared/constants/status';
import { isVtError, normalizeUnknownError, VtError } from '@shared/errors';
import type { EmptyData, VtResponse } from '@shared/types/response';
import { logger } from './logger';

const emptyData = {} as EmptyData;

export function createSuccessResponse<TData extends object>(data: TData, msg = getStatusMsg(VT_STATUS.OK)): VtResponse<TData> {
  return { code: VT_STATUS.OK, data, msg };
}

export function createFailResponse(msg = getStatusMsg(VT_STATUS.FAIL)): VtResponse<EmptyData> {
  return { code: VT_STATUS.FAIL, data: emptyData, msg };
}

export function createError(statusCode: VtStatusCode, msg?: string, detail?: unknown): VtError {
  return new VtError({ statusCode, msg, detail });
}

export function errorToResponse(error: unknown): VtResponse<EmptyData> {
  if (isVtError(error)) {
    return createFailResponse(error.message || getStatusMsg(error.statusCode));
  }

  return createFailResponse(getStatusMsg(VT_STATUS.SYSTEM_ERROR));
}

export function logServiceError(scope: string, error: unknown): void {
  logger.error(scope, '服务调用失败', normalizeUnknownError(error));
}
