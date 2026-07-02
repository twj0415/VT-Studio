import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { getUserDataPath } from '../../app/runtime';

const DATABASE_DIRECTORY_NAME = 'database';
const DATABASE_FILE_NAME = 'vt-studio.sqlite';

export function resolveDatabaseDirectory(): string {
  return join(getUserDataPath(), DATABASE_DIRECTORY_NAME);
}

export function resolveDatabaseFilePath(): string {
  return join(resolveDatabaseDirectory(), DATABASE_FILE_NAME);
}

export function ensureDatabaseDirectory(): string {
  const directory = resolveDatabaseDirectory();
  mkdirSync(directory, { recursive: true });

  return directory;
}
