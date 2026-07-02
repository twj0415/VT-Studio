import { statSync } from 'node:fs';
import type { MediaRoot } from '@shared/types/media';
import { getRuntimeDirectories, safeJoin } from '../file-system';

const MEDIA_ROOTS: MediaRoot[] = ['project', 'cache', 'temp', 'exports'];

export function isMediaRoot(value: unknown): value is MediaRoot {
  return typeof value === 'string' && (MEDIA_ROOTS as string[]).includes(value);
}

export function getMediaRootPath(root: MediaRoot): string {
  const directories = getRuntimeDirectories();

  switch (root) {
    case 'project':
      return directories.projects;
    case 'cache':
      return directories.cache;
    case 'temp':
      return directories.temp;
    case 'exports':
      return directories.exports;
    default:
      throw new Error('不支持的媒体根目录');
  }
}

export function resolveMediaPath(root: MediaRoot, relativePath: string): string {
  return safeJoin(getMediaRootPath(root), relativePath);
}

export function assertMediaFile(root: MediaRoot, relativePath: string): string {
  const filePath = resolveMediaPath(root, relativePath);
  const stat = statSync(filePath);

  if (!stat.isFile()) {
    throw new Error('目标不是文件');
  }

  return filePath;
}
