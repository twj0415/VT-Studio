import type Database from 'better-sqlite3';
import { getDatabase } from './connection';

export function withTransaction<T>(handler: (database: Database.Database) => T): T {
  const database = getDatabase();
  const transaction = database.transaction(() => handler(database));

  return transaction();
}
