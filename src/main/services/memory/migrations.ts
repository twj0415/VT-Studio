import type { Migration } from '../database/migrations';

export const memoryMigrations: Migration[] = [
  {
    id: '0005_create_memories',
    name: 'create memories table',
    statements: [
      `
      CREATE TABLE IF NOT EXISTS memories (
        id TEXT PRIMARY KEY,
        isolation_key TEXT NOT NULL,
        type TEXT NOT NULL,
        role TEXT NULL,
        name TEXT NULL,
        content TEXT NOT NULL,
        embedding TEXT NOT NULL DEFAULT '',
        metadata TEXT NULL,
        related_message_ids TEXT NULL,
        summarized INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL
      )
      `,
      'CREATE INDEX IF NOT EXISTS idx_memories_isolation_type ON memories(isolation_key, type)',
      'CREATE INDEX IF NOT EXISTS idx_memories_isolation_summarized ON memories(isolation_key, summarized)',
      'CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at)',
    ],
  },
];
