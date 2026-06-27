import type { ErrorKind } from '@/shared/enums/generated'

export type RecoverAction =
  | 'retry'
  | 'resume_task'
  | 'update_secret'
  | 'manual_check'
  | 'change_provider_or_plan'
  | 'edit_input'
  | 'choose_controlled_path'
  | 'start_composition'
  | 'configure_sidecar'
  | 'check_media_file'
  | 'regenerate_or_replace_media'
  | 'restart_app_or_check_database'
  | 'remove_secret_and_retry'
  | 'reject_package'
  | 'grant_media_permission'

export interface AppErrorDto {
  code: string
  kind: ErrorKind
  message: string
  detail?: Record<string, unknown>
  isRetryable: boolean
  recoverAction?: RecoverAction
  traceId: string
}

export class AppApiError extends Error {
  readonly code: string
  readonly kind: ErrorKind
  readonly detail?: Record<string, unknown>
  readonly isRetryable: boolean
  readonly recoverAction?: RecoverAction
  readonly traceId: string

  constructor(error: AppErrorDto) {
    super(error.message)
    this.name = 'AppApiError'
    this.code = error.code
    this.kind = error.kind
    this.detail = error.detail
    this.isRetryable = error.isRetryable
    this.recoverAction = error.recoverAction
    this.traceId = error.traceId
  }
}

export function normalizeApiError(error: unknown): AppApiError {
  if (error instanceof AppApiError) return error

  if (typeof error === 'object' && error !== null && 'code' in error && 'message' in error) {
    const dto = error as Partial<AppErrorDto> & {
      recoverable?: boolean
      details?: Record<string, unknown>
    }
    return new AppApiError({
      code: String(dto.code),
      kind: dto.kind ?? 'unknown',
      message: String(dto.message),
      detail: dto.detail ?? dto.details,
      isRetryable: dto.isRetryable ?? dto.recoverable ?? true,
      recoverAction: normalizeRecoverAction(dto.recoverAction),
      traceId: dto.traceId ?? 'trace_frontend_unknown',
    })
  }

  if (error instanceof Error) {
    return new AppApiError({ code: 'app.unknown', kind: 'unknown', message: error.message, isRetryable: true, recoverAction: 'retry', traceId: 'trace_frontend_error' })
  }

  return new AppApiError({ code: 'app.unknown', kind: 'unknown', message: '未知错误', isRetryable: true, recoverAction: 'retry', traceId: 'trace_frontend_unknown' })
}

export function getApiErrorI18nKey(error: Pick<AppErrorDto, 'code'>) {
  return `errors.${error.code}`
}

export function getRecoverActionI18nKey(action: RecoverAction) {
  return `recoverActions.${action}`
}

function normalizeRecoverAction(action: unknown): RecoverAction | undefined {
  if (typeof action !== 'string') return undefined
  const known: RecoverAction[] = [
    'retry',
    'resume_task',
    'update_secret',
    'manual_check',
    'change_provider_or_plan',
    'edit_input',
    'choose_controlled_path',
    'start_composition',
    'configure_sidecar',
    'check_media_file',
    'regenerate_or_replace_media',
    'restart_app_or_check_database',
    'remove_secret_and_retry',
    'reject_package',
    'grant_media_permission',
  ]
  return known.includes(action as RecoverAction) ? (action as RecoverAction) : 'manual_check'
}
