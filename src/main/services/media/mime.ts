import { extname } from 'node:path';

export type MediaKind = 'image' | 'video' | 'audio';

export interface MediaMimeInfo {
  contentType: string;
  kind: MediaKind;
}

const MIME_BY_EXTENSION: Record<string, MediaMimeInfo> = {
  '.jpg': { contentType: 'image/jpeg', kind: 'image' },
  '.jpeg': { contentType: 'image/jpeg', kind: 'image' },
  '.png': { contentType: 'image/png', kind: 'image' },
  '.webp': { contentType: 'image/webp', kind: 'image' },
  '.gif': { contentType: 'image/gif', kind: 'image' },
  '.bmp': { contentType: 'image/bmp', kind: 'image' },
  '.svg': { contentType: 'image/svg+xml', kind: 'image' },
  '.mp4': { contentType: 'video/mp4', kind: 'video' },
  '.webm': { contentType: 'video/webm', kind: 'video' },
  '.mov': { contentType: 'video/quicktime', kind: 'video' },
  '.mp3': { contentType: 'audio/mpeg', kind: 'audio' },
  '.wav': { contentType: 'audio/wav', kind: 'audio' },
  '.m4a': { contentType: 'audio/mp4', kind: 'audio' },
  '.aac': { contentType: 'audio/aac', kind: 'audio' },
  '.ogg': { contentType: 'audio/ogg', kind: 'audio' },
};

export function getMediaMimeInfo(filePath: string): MediaMimeInfo | null {
  return MIME_BY_EXTENSION[extname(filePath).toLowerCase()] ?? null;
}

export function isImageMime(filePath: string): boolean {
  return getMediaMimeInfo(filePath)?.kind === 'image';
}
