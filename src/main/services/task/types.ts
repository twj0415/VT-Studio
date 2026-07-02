import type { TaskStatus } from './constants';

export interface TaskRecord {
  id: number;
  projectId: number | null;
  category: string;
  relatedObjects: string | null;
  modelName: string | null;
  description: string | null;
  status: TaskStatus;
  startedAt: number;
  finishedAt: number | null;
  errorReason: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface CreateTaskInput {
  projectId?: number | null;
  category: string;
  relatedObjects?: unknown;
  modelName?: string | null;
  description?: string | null;
}

export interface UpdateTaskMetaInput {
  taskId: number;
  relatedObjects?: unknown;
  modelName?: string | null;
  description?: string | null;
}

export interface ListTasksInput {
  page?: number;
  limit?: number;
  projectId?: number | null;
  category?: string | null;
  status?: TaskStatus | null;
}

export interface TaskListResult {
  data: TaskRecord[];
  total: number;
  page: number;
  limit: number;
}

export interface CreateTaskResult {
  taskId: number;
  status: TaskStatus;
  startedAt: number;
}
