import { createHmac, randomBytes, timingSafeEqual } from 'node:crypto';
import type { MediaMode, MediaRoot, MediaThumbnailSize } from '@shared/types/media';
import { isMediaRoot } from './path';

const mediaSecret = randomBytes(32).toString('hex');

export interface MediaResource {
  root: MediaRoot;
  relativePath: string;
}

export interface SignedMediaPayload extends MediaResource {
  mode: MediaMode;
  size?: MediaThumbnailSize;
  expires: number;
}

function toBase64Url(value: Buffer | string): string {
  return Buffer.from(value).toString('base64url');
}

function fromBase64Url(value: string): string {
  return Buffer.from(value, 'base64url').toString('utf-8');
}

function signParts(encodedResource: string, mode: MediaMode, size: string, expires: number): string {
  return createHmac('sha256', mediaSecret)
    .update(encodedResource)
    .update('\n')
    .update(mode)
    .update('\n')
    .update(size)
    .update('\n')
    .update(String(expires))
    .digest('base64url');
}

function isSafeEqual(left: string, right: string): boolean {
  const leftBuffer = Buffer.from(left);
  const rightBuffer = Buffer.from(right);

  if (leftBuffer.length !== rightBuffer.length) {
    return false;
  }

  return timingSafeEqual(leftBuffer, rightBuffer);
}

export function encodeMediaResource(resource: MediaResource): string {
  return toBase64Url(JSON.stringify({ root: resource.root, relativePath: resource.relativePath }));
}

export function decodeMediaResource(encodedResource: string): MediaResource {
  const parsed = JSON.parse(fromBase64Url(encodedResource)) as Partial<MediaResource>;

  if (!isMediaRoot(parsed.root) || typeof parsed.relativePath !== 'string' || !parsed.relativePath.trim()) {
    throw new Error('媒体资源无效');
  }

  return { root: parsed.root, relativePath: parsed.relativePath };
}

export function signMediaUrl(input: {
  encodedResource: string;
  mode: MediaMode;
  size?: MediaThumbnailSize;
  expires: number;
}): string {
  return signParts(input.encodedResource, input.mode, input.size ?? '', input.expires);
}

export function verifySignedMediaUrl(input: {
  encodedResource: string;
  mode: MediaMode;
  size?: MediaThumbnailSize;
  expires: number;
  token: string;
}): SignedMediaPayload {
  if (!Number.isFinite(input.expires) || input.expires < Math.floor(Date.now() / 1000)) {
    throw new Error('媒体 URL 已过期');
  }

  const expected = signMediaUrl(input);
  if (!input.token || !isSafeEqual(expected, input.token)) {
    throw new Error('媒体 URL 签名无效');
  }

  return {
    ...decodeMediaResource(input.encodedResource),
    mode: input.mode,
    size: input.size,
    expires: input.expires,
  };
}
