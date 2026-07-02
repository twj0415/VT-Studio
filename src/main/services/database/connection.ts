import Database from 'better-sqlite3';
import { ensureDatabaseDirectory, resolveDatabaseFilePath } from './path';

let database: Database.Database | null = null;

function configureConnection(connection: Database.Database): void {
  connection.pragma('journal_mode = WAL');
  connection.pragma('foreign_keys = ON');
  connection.pragma('busy_timeout = 5000');
}

export function getDatabase(): Database.Database {
  if (database) {
    return database;
  }

  ensureDatabaseDirectory();

  database = new Database(resolveDatabaseFilePath());
  configureConnection(database);

  return database;
}

export function closeDatabase(): void {
  if (!database) {
    return;
  }

  if (database.open) {
    database.close();
  }

  database = null;
}
