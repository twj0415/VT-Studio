export const TASK_STATUS = {
  RUNNING: 'running',
  SUCCEEDED: 'succeeded',
  FAILED: 'failed',
  CANCELLED: 'cancelled',
} as const;

export type TaskStatus = (typeof TASK_STATUS)[keyof typeof TASK_STATUS];

export const TASK_STATUS_VALUES = Object.values(TASK_STATUS);

export const DEFAULT_TASK_CANCEL_REASON = '任务已取消';
export const DEFAULT_TASK_RECOVER_REASON = '软件退出导致失败';
