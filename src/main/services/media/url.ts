import { statSync } from 'node:fs';
import { getLocalServerUrl } from '../../app/server';
import { VT_STATUS } from '@shared/constants/status';
import type {
  MediaCreateThumbnailUrlPayload,
  MediaCreateThumbnailUrlResult,
  MediaCreateUrlPayload,
  MediaCreateUrlResult,
  MediaGetOriginalUrlPayload,
  MediaGetOriginalUrlResult,
  MediaMode,
  MediaResolveUrlPayload,
  MediaResolveUrlResult,
  MediaThumbnailSize,
} from '@shared/types/media';
import { createError } from '../result';
import { getMediaMimeInfo } from './mime';
import { resolveMediaPath } from './path';
import { encodeMediaResource, signMediaUrl, verifySignedMediaUrl } from './security';
import { normalizeThumbnailSize } from './thumbnail';

const DEFAULT_EXPIRES_IN_SECONDS = 60 * 60 * 12;

function assertMode(value: string | null): MediaMode {
  if (value === 'thumbnail') {
    return 'thumbnail';
  }

  return 'original';
}

function assertSize(value: string | null): MediaThumbnailSize | undefined {
  if (!value) {
    return undefined;
  }

  if (value === 'small' || value === 'list' || value === 'detail') {
    return value;
  }

  throw createError(VT_STATUS.INVALID_PARAMS, '缩略图尺寸无效');
}

function assertPositiveInteger(value: unknown, fallback: number): number {
  const parsed = typeof value === 'number' ? value : Number(value);

  if (!Number.isFinite(parsed) || parsed <= 0) {
    return fallback;
  }

  return Math.floor(parsed);
}

function assertReadableFile(root: MediaCreateUrlPayload['root'], relativePath: string): string {
  try {
    const filePath = resolveMediaPath(root, relativePath);
    const stat = statSync(filePath);

    if (!stat.isFile()) {
      throw createError(VT_STATUS.FILE_PATH_INVALID, '目标不是文件');
    }

    return filePath;
  } catch (error) {
    if (error instanceof Error && error.message === '目标不是文件') {
      throw error;
    }

    throw createError(VT_STATUS.FILE_NOT_FOUND, '文件不存在', error);
  }
}

function createSignedMediaUrl(input: {
  root: MediaCreateUrlPayload['root'];
  relativePath: string;
  mode: MediaMode;
  size?: MediaThumbnailSize;
  expiresInSeconds?: number;
}): string {
  const baseUrl = getLocalServerUrl();
  const encodedResource = encodeMediaResource({ root: input.root, relativePath: input.relativePath });
  const expires = Math.floor(Date.now() / 1000) + assertPositiveInteger(input.expiresInSeconds, DEFAULT_EXPIRES_IN_SECONDS);
  const token = signMediaUrl({
    encodedResource,
    mode: input.mode,
    size: input.size,
    expires,
  });
  const params = new URLSearchParams({
    mode: input.mode,
    expires: String(expires),
    token,
  });

  if (input.size) {
    params.set('size', input.size);
  }

  return `${baseUrl}/media/${encodeURIComponent(encodedResource)}?${params.toString()}`;
}

export function createMediaUrl(payload: MediaCreateUrlPayload): MediaCreateUrlResult {
  if (!payload.relativePath?.trim()) {
    throw createError(VT_STATUS.INVALID_PARAMS, '媒体路径不能为空');
  }

  assertReadableFile(payload.root, payload.relativePath);

  return {
    url: createSignedMediaUrl({
      root: payload.root,
      relativePath: payload.relativePath,
      mode: 'original',
      expiresInSeconds: payload.expiresInSeconds,
    }),
  };
}

export function createThumbnailMediaUrl(payload: MediaCreateThumbnailUrlPayload): MediaCreateThumbnailUrlResult {
  if (!payload.relativePath?.trim()) {
    throw createError(VT_STATUS.INVALID_PARAMS, '媒体路径不能为空');
  }

  const filePath = assertReadableFile(payload.root, payload.relativePath);
  const mime = getMediaMimeInfo(filePath);

  if (mime?.kind !== 'image') {
    throw createError(VT_STATUS.UNSUPPORTED_FILE_TYPE, '只有图片支持缩略图');
  }

  return {
    url: createSignedMediaUrl({
      root: payload.root,
      relativePath: payload.relativePath,
      mode: 'thumbnail',
      size: normalizeThumbnailSize(payload.size),
      expiresInSeconds: payload.expiresInSeconds,
    }),
    fallback: false,
  };
}

export function resolveMediaUrlToPath(payload: MediaResolveUrlPayload): MediaResolveUrlResult {
  if (!payload.url?.trim()) {
    throw createError(VT_STATUS.INVALID_PARAMS, '媒体 URL 不能为空');
  }

  const parsedUrl = new URL(payload.url, getLocalServerUrl());
  const pathname = decodeURIComponent(parsedUrl.pathname);

  if (pathname.startsWith('/media/')) {
    const encodedResource = decodeURIComponent(pathname.slice('/media/'.length));
    const mode = assertMode(parsedUrl.searchParams.get('mode'));
    const size = assertSize(parsedUrl.searchParams.get('size'));
    const expires = Number(parsedUrl.searchParams.get('expires'));
    const token = parsedUrl.searchParams.get('token') ?? '';
    const verified = verifySignedMediaUrl({ encodedResource, mode, size, expires, token });

    resolveMediaPath(verified.root, verified.relativePath);

    return {
      root: verified.root,
      relativePath: verified.relativePath,
      mode: verified.mode,
      size: verified.size,
    };
  }

  const legacyRelativePath = resolveLegacyPath(pathname);
  if (legacyRelativePath) {
    resolveMediaPath('project', legacyRelativePath);
    return {
      root: 'project',
      relativePath: legacyRelativePath,
      mode: 'original',
    };
  }

  throw createError(VT_STATUS.INVALID_PARAMS, '无法反解媒体 URL');
}

export function getOriginalMediaUrl(payload: MediaGetOriginalUrlPayload): MediaGetOriginalUrlResult {
  if (payload.url) {
    const resolved = resolveMediaUrlToPath({ url: payload.url });

    return createMediaUrl({
      root: resolved.root,
      relativePath: resolved.relativePath,
      expiresInSeconds: payload.expiresInSeconds,
    });
  }

  if (!payload.root || !payload.relativePath) {
    throw createError(VT_STATUS.INVALID_PARAMS, '媒体路径不能为空');
  }

  return createMediaUrl({
    root: payload.root,
    relativePath: payload.relativePath,
    expiresInSeconds: payload.expiresInSeconds,
  });
}

function resolveLegacyPath(pathname: string): string | null {
  const normalized = pathname.replace(/\\/g, '/');
  const legacyPrefixes = ['/oss/', '/smallImage/'];
  const prefix = legacyPrefixes.find((item) => normalized.startsWith(item));

  if (!prefix) {
    return null;
  }

  return normalized.slice(prefix.length).replace(/^\/+/, '');
}
