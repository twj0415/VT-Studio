import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import sharp from 'sharp';
import type { MediaRoot, MediaThumbnailSize } from '@shared/types/media';
import { normalizeUnknownError } from '@shared/errors';
import { getRuntimeDirectories } from '../file-system';
import { logger } from '../logger';
import { getMediaMimeInfo } from './mime';
import { resolveMediaPath } from './path';

const THUMBNAIL_SIZES: Record<MediaThumbnailSize, number> = {
  small: 96,
  list: 240,
  detail: 640,
};

export function normalizeThumbnailSize(size?: MediaThumbnailSize): MediaThumbnailSize {
  return size && size in THUMBNAIL_SIZES ? size : 'list';
}

export function getThumbnailDimension(size: MediaThumbnailSize): number {
  return THUMBNAIL_SIZES[size];
}

export async function ensureThumbnailFile(root: MediaRoot, relativePath: string, size?: MediaThumbnailSize): Promise<string | null> {
  const normalizedSize = normalizeThumbnailSize(size);
  const sourcePath = resolveMediaPath(root, relativePath);
  const sourceStat = statSync(sourcePath);

  if (!sourceStat.isFile()) {
    return null;
  }

  const mime = getMediaMimeInfo(sourcePath);
  if (mime?.kind !== 'image') {
    return null;
  }

  const hash = createHash('sha256')
    .update(root)
    .update('\n')
    .update(relativePath)
    .update('\n')
    .update(String(sourceStat.mtimeMs))
    .update('\n')
    .update(normalizedSize)
    .digest('hex')
    .slice(0, 32);
  const thumbnailDirectory = getRuntimeDirectories().thumbnails;
  const thumbnailPath = join(thumbnailDirectory, `${hash}-${normalizedSize}.webp`);

  if (existsSync(thumbnailPath)) {
    return thumbnailPath;
  }

  mkdirSync(thumbnailDirectory, { recursive: true });

  try {
    await sharp(sourcePath)
      .resize(getThumbnailDimension(normalizedSize), getThumbnailDimension(normalizedSize), {
        fit: 'inside',
        withoutEnlargement: true,
      })
      .webp({ quality: 78 })
      .toFile(thumbnailPath);

    return thumbnailPath;
  } catch (error) {
    logger.warn('媒体服务', '缩略图生成失败，降级使用原图');
    logger.detail('媒体服务', '缩略图失败详情', normalizeUnknownError(error));
    return null;
  }
}
