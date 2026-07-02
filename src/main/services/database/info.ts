import { existsSync, statSync } from 'node:fs';
import { getDatabase } from './connection';
import { resolveDatabaseDirectory, resolveDatabaseFilePath } from './path';

interface SqliteVersionRow {
  version: string;
}

interface CountRow {
  count: number;
}

export interface DatabaseInfo {
  directory: string;
  filePath: string;
  exists: boolean;
  sizeBytes: number;
  tableCount: number;
  migrationCount: number;
  sqliteVersion: string;
}

function getFileSize(filePath: string): number {
  if (!existsSync(filePath)) {
    return 0;
  }

  return statSync(filePath).size;
}

export function getDatabaseInfo(): DatabaseInfo {
  const database = getDatabase();
  const filePath = resolveDatabaseFilePath();
  const versionRow = database.prepare<[], SqliteVersionRow>('SELECT sqlite_version() AS version').get();
  const tableCountRow = database
    .prepare<[], CountRow>("SELECT COUNT(*) AS count FROM sqlite_master WHERE type = 'table'")
    .get();
  const migrationCountRow = database.prepare<[], CountRow>('SELECT COUNT(*) AS count FROM schema_migrations').get();

  return {
    directory: resolveDatabaseDirectory(),
    filePath,
    exists: existsSync(filePath),
    sizeBytes: getFileSize(filePath),
    tableCount: tableCountRow?.count ?? 0,
    migrationCount: migrationCountRow?.count ?? 0,
    sqliteVersion: versionRow?.version ?? '',
  };
}
