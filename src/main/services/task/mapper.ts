import { TASK_STATUS_VALUES, type TaskStatus } from './constants';
import type { TaskRecord } from './types';

export interface TaskRow {
  id: number;
  project_id: number | null;
  category: string;
  related_objects: string | null;
  model_name: string | null;
  description: string | null;
  status: string;
  started_at: number;
  finished_at: number | null;
  error_reason: string | null;
  created_at: number;
  updated_at: number;
}

export function isTaskStatus(value: unknown): value is TaskStatus {
  return typeof value === 'string' && TASK_STATUS_VALUES.includes(value as TaskStatus);
}

export function mapTaskRow(row: TaskRow): TaskRecord {
  return {
    id: row.id,
    projectId: row.project_id,
    category: row.category,
    relatedObjects: row.related_objects,
    modelName: row.model_name,
    description: row.description,
    status: isTaskStatus(row.status) ? row.status : 'failed',
    startedAt: row.started_at,
    finishedAt: row.finished_at,
    errorReason: row.error_reason,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}
