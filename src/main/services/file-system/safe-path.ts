import { isAbsolute, relative, resolve, sep } from 'node:path';

const WINDOWS_ABSOLUTE_PATH_PATTERN = /^[a-zA-Z]:[\\/]/;

function isInsideOrEqual(targetPath: string, rootPath: string): boolean {
  const relativePath = relative(rootPath, targetPath);

  return relativePath === '' || (!relativePath.startsWith('..') && !isAbsolute(relativePath));
}

export function normalizeRelativePath(relativePath: string): string {
  const normalizedInput = relativePath.trim().replace(/\\/g, '/').replace(/^\/+/, '');

  if (!normalizedInput || normalizedInput === '.' || normalizedInput.includes('\0')) {
    throw new Error('Invalid relative path');
  }

  if (isAbsolute(normalizedInput) || WINDOWS_ABSOLUTE_PATH_PATTERN.test(normalizedInput)) {
    throw new Error('Absolute paths are not allowed');
  }

  return normalizedInput.split('/').filter(Boolean).join(sep);
}

export function assertInsideRoot(targetPath: string, rootPath: string): string {
  const resolvedTarget = resolve(targetPath);
  const resolvedRoot = resolve(rootPath);

  if (!isInsideOrEqual(resolvedTarget, resolvedRoot)) {
    throw new Error(`Path escapes root: ${targetPath}`);
  }

  return resolvedTarget;
}

export function safeJoin(rootPath: string, relativePath: string): string {
  const normalizedRelativePath = normalizeRelativePath(relativePath);
  const targetPath = resolve(rootPath, normalizedRelativePath);

  return assertInsideRoot(targetPath, rootPath);
}
