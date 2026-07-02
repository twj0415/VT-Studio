import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, relative, resolve } from 'node:path';
import { resolveDatabaseDirectory } from '../database/path';
import { assertInsideRoot, safeJoin } from './safe-path';

export type ManagedFileData = Buffer | string;

function assertDeletionAllowed(targetPath: string, rootPath: string): string {
  const resolvedTarget = assertInsideRoot(targetPath, rootPath);
  const resolvedRoot = resolve(rootPath);
  const databaseDirectory = resolveDatabaseDirectory();
  const databaseRelativePath = relative(databaseDirectory, resolvedTarget);

  if (resolvedTarget === resolvedRoot) {
    throw new Error('Deleting a managed root directory is not allowed');
  }

  if (databaseRelativePath === '' || (!databaseRelativePath.startsWith('..') && !isAbsolute(databaseRelativePath))) {
    throw new Error('Deleting the database directory is not allowed');
  }

  return resolvedTarget;
}

export function ensureDirectory(directoryPath: string): string {
  mkdirSync(directoryPath, { recursive: true });

  return directoryPath;
}

export function pathExists(targetPath: string): boolean {
  return existsSync(targetPath);
}

export function fileExists(targetPath: string): boolean {
  if (!existsSync(targetPath)) {
    return false;
  }

  return statSync(targetPath).isFile();
}

export function copyFileToManagedPath(sourcePath: string, targetRoot: string, targetRelativePath: string): string {
  const resolvedSource = resolve(sourcePath);

  if (!fileExists(resolvedSource)) {
    throw new Error(`Source file does not exist: ${sourcePath}`);
  }

  const targetPath = safeJoin(targetRoot, targetRelativePath);
  mkdirSync(dirname(targetPath), { recursive: true });
  copyFileSync(resolvedSource, targetPath);

  return targetPath;
}

export function writeManagedFile(targetRoot: string, targetRelativePath: string, data: ManagedFileData): string {
  const targetPath = safeJoin(targetRoot, targetRelativePath);

  mkdirSync(dirname(targetPath), { recursive: true });
  writeFileSync(targetPath, data);

  return targetPath;
}

export function readManagedFile(targetRoot: string, targetRelativePath: string): Buffer {
  const targetPath = safeJoin(targetRoot, targetRelativePath);

  if (!fileExists(targetPath)) {
    throw new Error(`File does not exist: ${targetRelativePath}`);
  }

  return readFileSync(targetPath);
}

export function deleteManagedFile(targetRoot: string, targetRelativePath: string): void {
  const targetPath = assertDeletionAllowed(safeJoin(targetRoot, targetRelativePath), targetRoot);

  if (!fileExists(targetPath)) {
    throw new Error(`File does not exist: ${targetRelativePath}`);
  }

  rmSync(targetPath, { force: false });
}

export function deleteManagedDirectory(targetRoot: string, targetRelativePath: string): void {
  const targetPath = assertDeletionAllowed(safeJoin(targetRoot, targetRelativePath), targetRoot);

  if (!pathExists(targetPath)) {
    throw new Error(`Directory does not exist: ${targetRelativePath}`);
  }

  if (!statSync(targetPath).isDirectory()) {
    throw new Error(`Path is not a directory: ${targetRelativePath}`);
  }

  rmSync(targetPath, { recursive: true, force: false });
}
