import type { Migration } from '../database/migrations';

export const taskMigrations: Migration[] = [
  {
    id: '0002_create_tasks',
    name: 'create tasks table',
    statements: [
      `
      CREATE TABLE IF NOT EXISTS tasks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id INTEGER NULL,
        category TEXT NOT NULL,
        related_objects TEXT NULL,
        model_name TEXT NULL,
        description TEXT NULL,
        status TEXT NOT NULL,
        started_at INTEGER NOT NULL,
        finished_at INTEGER NULL,
        error_reason TEXT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
      `,
      'CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id)',
      'CREATE INDEX IF NOT EXISTS idx_tasks_category ON tasks(category)',
      'CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)',
      'CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at)',
    ],
  },
];
