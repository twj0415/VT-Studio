import type {
  MediaCreateThumbnailUrlPayload,
  MediaCreateUrlPayload,
  MediaGetOriginalUrlPayload,
  MediaResolveUrlPayload,
} from '@shared/types/media';
import {
  createMediaUrl,
  createThumbnailMediaUrl,
  getOriginalMediaUrl,
  resolveMediaUrlToPath,
} from '../services/media';
import { handleIpc } from './handle';

function readObjectArg<T extends object>(value: unknown): T {
  return value && typeof value === 'object' ? (value as T) : ({} as T);
}

export function registerMediaIpc(): void {
  handleIpc('media:create-url', (_event, payload) => createMediaUrl(readObjectArg<MediaCreateUrlPayload>(payload)));
  handleIpc('media:create-thumbnail-url', (_event, payload) => createThumbnailMediaUrl(readObjectArg<MediaCreateThumbnailUrlPayload>(payload)));
  handleIpc('media:resolve-url-to-path', (_event, payload) => resolveMediaUrlToPath(readObjectArg<MediaResolveUrlPayload>(payload)));
  handleIpc('media:get-original-url', (_event, payload) => getOriginalMediaUrl(readObjectArg<MediaGetOriginalUrlPayload>(payload)));
}
