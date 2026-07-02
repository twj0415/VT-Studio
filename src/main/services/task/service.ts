import { VT_STATUS } from '@shared/constants/status';
import { normalizeUnknownError, VtError } from '@shared/errors';
import { getDatabase } from '../database';
import { createError } from '../result';
import { DEFAULT_TASK_CANCEL_REASON, DEFAULT_TASK_RECOVER_REASON, TASK_STATUS, type TaskStatus } from './constants';
import { isTaskStatus, mapTaskRow, type TaskRow } from './mapper';
import type { CreateTaskInput, CreateTaskResult, ListTasksInput, TaskListResult, TaskRecord, UpdateTaskMetaInput } from './types';

const DEFAULT_PAGE = 1;
const DEFAULT_LIMIT = 20;
const MAX_LIMIT = 100;

function normalizeCategory(category: string): string {
  const normalized = category.trim();

  if (!normalized) {
    throw createError(VT_STATUS.INVALID_PARAMS, '任务分类不能为空');
  }

  return normalized;
}

function normalizeId(taskId: number): number {
  if (!Number.isInteger(taskId) || taskId <= 0) {
    throw createError(VT_STATUS.INVALID_PARAMS, '任务 ID 无效');
  }

  return taskId;
}

function serializeRelatedObjects(value: unknown): string | null {
  if (value === undefined || value === null) {
    return null;
  }

  if (typeof value === 'string') {
    return value;
  }

  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

function getTaskRow(taskId: number): TaskRow {
  const row = getDatabase().prepare<[number], TaskRow>('SELECT * FROM tasks WHERE id = ? LIMIT 1').get(taskId);

  if (!row) {
    throw createError(VT_STATUS.TASK_NOT_FOUND);
  }

  return row;
}

function getTask(taskId: number): TaskRecord {
  return mapTaskRow(getTaskRow(normalizeId(taskId)));
}

function assertRunning(task: TaskRecord, action: string): void {
  if (task.status !== TASK_STATUS.RUNNING) {
    throw createError(VT_STATUS.TASK_STATUS_CONFLICT, `任务当前状态不允许${action}`);
  }
}

function normalizePage(value: number | undefined): number {
  const page = value ?? DEFAULT_PAGE;

  if (!Number.isInteger(page) || page <= 0) {
    throw createError(VT_STATUS.INVALID_PARAMS, '分页页码无效');
  }

  return page;
}

function normalizeLimit(value: number | undefined): number {
  const limit = value ?? DEFAULT_LIMIT;

  if (!Number.isInteger(limit) || limit <= 0 || limit > MAX_LIMIT) {
    throw createError(VT_STATUS.INVALID_PARAMS, `分页数量必须在 1-${MAX_LIMIT} 之间`);
  }

  return limit;
}

function normalizeErrorReason(error: unknown): string {
  const normalized = normalizeUnknownError(error);
  return normalized.message || '任务处理失败';
}

function toNullableText(value: string | null | undefined): string | null {
  if (value === undefined || value === null) {
    return null;
  }

  const normalized = value.trim();
  return normalized || null;
}

export function createTask(input: CreateTaskInput): CreateTaskResult {
  const now = Date.now();
  const category = normalizeCategory(input.category);
  const result = getDatabase()
    .prepare<[number | null, string, string | null, string | null, string | null, TaskStatus, number, number, number]>(
      `
      INSERT INTO tasks (
        project_id, category, related_objects, model_name, description,
        status, started_at, created_at, updated_at
      )
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      `,
    )
    .run(
      input.projectId ?? null,
      category,
      serializeRelatedObjects(input.relatedObjects),
      toNullableText(input.modelName),
      toNullableText(input.description),
      TASK_STATUS.RUNNING,
      now,
      now,
      now,
    );

  return {
    taskId: Number(result.lastInsertRowid),
    status: TASK_STATUS.RUNNING,
    startedAt: now,
  };
}

export function succeedTask(taskId: number): TaskRecord {
  const task = getTask(taskId);
  assertRunning(task, '完成');

  const now = Date.now();
  getDatabase()
    .prepare<[TaskStatus, number, number, null, number]>(
      'UPDATE tasks SET status = ?, finished_at = ?, updated_at = ?, error_reason = ? WHERE id = ?',
    )
    .run(TASK_STATUS.SUCCEEDED, now, now, null, task.id);

  return getTask(task.id);
}

export function failTask(taskId: number, error: unknown): TaskRecord {
  const task = getTask(taskId);
  assertRunning(task, '失败');

  const now = Date.now();
  getDatabase()
    .prepare<[TaskStatus, string, number, number, number]>(
      'UPDATE tasks SET status = ?, error_reason = ?, finished_at = ?, updated_at = ? WHERE id = ?',
    )
    .run(TASK_STATUS.FAILED, normalizeErrorReason(error), now, now, task.id);

  return getTask(task.id);
}

export function cancelTask(taskId: number, reason = DEFAULT_TASK_CANCEL_REASON): TaskRecord {
  const task = getTask(taskId);
  assertRunning(task, '取消');

  const now = Date.now();
  getDatabase()
    .prepare<[TaskStatus, string, number, number, number]>(
      'UPDATE tasks SET status = ?, error_reason = ?, finished_at = ?, updated_at = ? WHERE id = ?',
    )
    .run(TASK_STATUS.CANCELLED, reason.trim() || DEFAULT_TASK_CANCEL_REASON, now, now, task.id);

  return getTask(task.id);
}

export function isTaskCancelled(taskId: number): boolean {
  return getTask(taskId).status === TASK_STATUS.CANCELLED;
}

export function getTaskDetail(taskId: number): TaskRecord {
  return getTask(taskId);
}

export function listTasks(input: ListTasksInput = {}): TaskListResult {
  const page = normalizePage(input.page);
  const limit = normalizeLimit(input.limit);
  const where: string[] = [];
  const params: Array<string | number> = [];

  if (input.projectId !== undefined && input.projectId !== null) {
    if (!Number.isInteger(input.projectId) || input.projectId <= 0) {
      throw createError(VT_STATUS.INVALID_PARAMS, '项目 ID 无效');
    }

    where.push('project_id = ?');
    params.push(input.projectId);
  }

  if (input.category) {
    where.push('category = ?');
    params.push(normalizeCategory(input.category));
  }

  if (input.status) {
    if (!isTaskStatus(input.status)) {
      throw createError(VT_STATUS.INVALID_PARAMS, '任务状态无效');
    }

    where.push('status = ?');
    params.push(input.status);
  }

  const whereSql = where.length > 0 ? `WHERE ${where.join(' AND ')}` : '';
  const totalRow = getDatabase().prepare<Array<string | number>, { total: number }>(`SELECT COUNT(*) AS total FROM tasks ${whereSql}`).get(...params);
  const rows = getDatabase()
    .prepare<Array<string | number>, TaskRow>(`SELECT * FROM tasks ${whereSql} ORDER BY id DESC LIMIT ? OFFSET ?`)
    .all(...params, limit, (page - 1) * limit);

  return {
    data: rows.map(mapTaskRow),
    total: totalRow?.total ?? 0,
    page,
    limit,
  };
}

export function getTaskCategories(): string[] {
  const rows = getDatabase()
    .prepare<[], { category: string }>('SELECT category FROM tasks WHERE category != "" GROUP BY category ORDER BY category ASC')
    .all();

  return rows.map((row) => row.category);
}

export function updateTaskMeta(input: UpdateTaskMetaInput): TaskRecord {
  const task = getTask(input.taskId);
  assertRunning(task, '更新');

  const nextRelatedObjects = input.relatedObjects === undefined ? task.relatedObjects : serializeRelatedObjects(input.relatedObjects);
  const nextModelName = input.modelName === undefined ? task.modelName : toNullableText(input.modelName);
  const nextDescription = input.description === undefined ? task.description : toNullableText(input.description);

  getDatabase()
    .prepare<[string | null, string | null, string | null, number, number]>(
      'UPDATE tasks SET related_objects = ?, model_name = ?, description = ?, updated_at = ? WHERE id = ?',
    )
    .run(nextRelatedObjects, nextModelName, nextDescription, Date.now(), task.id);

  return getTask(task.id);
}

export function recoverRunningTasks(reason = DEFAULT_TASK_RECOVER_REASON): number {
  const now = Date.now();

  try {
    const result = getDatabase()
      .prepare<[TaskStatus, string, number, number, TaskStatus]>(
        'UPDATE tasks SET status = ?, error_reason = ?, finished_at = ?, updated_at = ? WHERE status = ?',
      )
      .run(TASK_STATUS.FAILED, reason, now, now, TASK_STATUS.RUNNING);

    return result.changes;
  } catch (error) {
    throw new VtError({
      statusCode: VT_STATUS.DATABASE_ERROR,
      msg: '恢复运行中任务失败',
      detail: error,
      cause: error,
    });
  }
}
