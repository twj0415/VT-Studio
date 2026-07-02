import { createHash } from 'node:crypto';
import type Database from 'better-sqlite3';
import { memoryMigrations } from '../memory/migrations';
import { modelMigrations } from '../model/migrations';
import { modelPromptMigrations } from '../settings/model-prompt-migrations';
import { taskMigrations } from '../task/migrations';
import { getDatabase } from './connection';
import { seedMigrations } from './seed-migrations';
import { runSeed } from './seed';

export interface Migration {
  id: string;
  name: string;
  statements: string[];
}

const migrations: Migration[] = [
  {
    id: '0001_create_schema_migrations',
    name: 'create schema migrations table',
    statements: [
      `
      CREATE TABLE IF NOT EXISTS schema_migrations (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        checksum TEXT NOT NULL,
        applied_at INTEGER NOT NULL
      )
      `,
    ],
  },
  ...taskMigrations,
  ...modelMigrations,
  ...seedMigrations,
  ...memoryMigrations,
  ...modelPromptMigrations,
];

function createChecksum(migration: Migration): string {
  return createHash('sha256').update(migration.id).update(migration.name).update(migration.statements.join('\n')).digest('hex');
}

function hasSchemaMigrationsTable(database: Database.Database): boolean {
  const row = database
    .prepare<[], { name: string }>(
      "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations' LIMIT 1",
    )
    .get();

  return Boolean(row);
}

function getAppliedMigrationIds(database: Database.Database): Set<string> {
  if (!hasSchemaMigrationsTable(database)) {
    return new Set();
  }

  const rows = database.prepare<[], { id: string }>('SELECT id FROM schema_migrations').all();

  return new Set(rows.map((row) => row.id));
}

function runMigration(database: Database.Database, migration: Migration): void {
  const transaction = database.transaction(() => {
    for (const statement of migration.statements) {
      database.exec(statement);
    }

    database
      .prepare<[string, string, string, number]>(
        `
        INSERT INTO schema_migrations (id, name, checksum, applied_at)
        VALUES (?, ?, ?, ?)
        `,
      )
      .run(migration.id, migration.name, createChecksum(migration), Date.now());
  });

  transaction();
}

export function runMigrations(database = getDatabase()): void {
  const appliedMigrationIds = getAppliedMigrationIds(database);

  for (const migration of migrations) {
    if (appliedMigrationIds.has(migration.id)) {
      continue;
    }

    runMigration(database, migration);
    appliedMigrationIds.add(migration.id);
  }

  // CORE-009: 迁移完成后写入默认数据（幂等，已存在则跳过）
  runSeed(database);
}
