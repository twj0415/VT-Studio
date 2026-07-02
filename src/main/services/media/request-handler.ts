import { createReadStream, statSync } from 'node:fs';
import type { IncomingMessage, ServerResponse } from 'node:http';
import { normalizeUnknownError } from '@shared/errors';
import type { MediaMode, MediaThumbnailSize } from '@shared/types/media';
import { logger } from '../logger';
import { getMediaMimeInfo } from './mime';
import { resolveMediaPath } from './path';
import { verifySignedMediaUrl } from './security';
import { ensureThumbnailFile, normalizeThumbnailSize } from './thumbnail';

interface ParsedRange {
  start: number;
  end: number;
}

function writeJson(res: ServerResponse, statusCode: number, msg: string): void {
  res.statusCode = statusCode;
  res.setHeader('content-type', 'application/json; charset=utf-8');
  res.end(JSON.stringify({ code: statusCode, msg }));
}

function parseMode(value: string | null): MediaMode {
  return value === 'thumbnail' ? 'thumbnail' : 'original';
}

function parseSize(value: string | null): MediaThumbnailSize | undefined {
  if (!value) {
    return undefined;
  }

  if (value === 'small' || value === 'list' || value === 'detail') {
    return value;
  }

  throw new Error('缩略图尺寸无效');
}

function parseRangeHeader(rangeHeader: string, fileSize: number): ParsedRange | null {
  const match = /^bytes=(\d*)-(\d*)$/.exec(rangeHeader.trim());

  if (!match) {
    return null;
  }

  const [, rawStart, rawEnd] = match;
  let start: number;
  let end: number;

  if (!rawStart && !rawEnd) {
    return null;
  }

  if (!rawStart) {
    const suffixLength = Number(rawEnd);
    if (!Number.isInteger(suffixLength) || suffixLength <= 0) {
      return null;
    }
    start = Math.max(fileSize - suffixLength, 0);
    end = fileSize - 1;
  } else {
    start = Number(rawStart);
    end = rawEnd ? Number(rawEnd) : fileSize - 1;
  }

  if (!Number.isInteger(start) || !Number.isInteger(end) || start < 0 || end < start || start >= fileSize) {
    return null;
  }

  return { start, end: Math.min(end, fileSize - 1) };
}

function streamFile(req: IncomingMessage, res: ServerResponse, filePath: string, contentType: string): void {
  const stat = statSync(filePath);

  if (!stat.isFile()) {
    writeJson(res, 400, '目标不是文件');
    return;
  }

  const fileSize = stat.size;
  const rangeHeader = req.headers.range;

  res.setHeader('accept-ranges', 'bytes');
  res.setHeader('content-type', contentType);
  res.setHeader('x-content-type-options', 'nosniff');

  if (rangeHeader) {
    const range = parseRangeHeader(rangeHeader, fileSize);

    if (!range) {
      res.statusCode = 416;
      res.setHeader('content-range', `bytes */${fileSize}`);
      res.end();
      return;
    }

    res.statusCode = 206;
    res.setHeader('content-range', `bytes ${range.start}-${range.end}/${fileSize}`);
    res.setHeader('content-length', String(range.end - range.start + 1));

    if (req.method === 'HEAD') {
      res.end();
      return;
    }

    createReadStream(filePath, { start: range.start, end: range.end }).pipe(res);
    return;
  }

  res.statusCode = 200;
  res.setHeader('content-length', String(fileSize));

  if (req.method === 'HEAD') {
    res.end();
    return;
  }

  createReadStream(filePath).pipe(res);
}

export async function handleMediaRequest(req: IncomingMessage, res: ServerResponse): Promise<boolean> {
  const requestUrl = new URL(req.url ?? '/', 'http://127.0.0.1');

  if (!requestUrl.pathname.startsWith('/media/')) {
    return false;
  }

  if (req.method !== 'GET' && req.method !== 'HEAD') {
    writeJson(res, 405, 'Method Not Allowed');
    return true;
  }

  try {
    const encodedResource = decodeURIComponent(requestUrl.pathname.slice('/media/'.length));
    const mode = parseMode(requestUrl.searchParams.get('mode'));
    const size = parseSize(requestUrl.searchParams.get('size'));
    const expires = Number(requestUrl.searchParams.get('expires'));
    const token = requestUrl.searchParams.get('token') ?? '';
    const verified = verifySignedMediaUrl({ encodedResource, mode, size, expires, token });
    const sourcePath = resolveMediaPath(verified.root, verified.relativePath);
    let responsePath = sourcePath;

    if (verified.mode === 'thumbnail') {
      responsePath = (await ensureThumbnailFile(verified.root, verified.relativePath, normalizeThumbnailSize(verified.size))) ?? sourcePath;
    }

    const mime = getMediaMimeInfo(responsePath);
    if (!mime) {
      writeJson(res, 415, '不支持的媒体类型');
      return true;
    }

    streamFile(req, res, responsePath, mime.contentType);
    return true;
  } catch (error) {
    const normalized = normalizeUnknownError(error);
    logger.warn('媒体服务', '媒体请求失败');
    logger.detail('媒体服务', '媒体请求失败详情', normalized);

    if (normalized.message.includes('签名') || normalized.message.includes('过期')) {
      writeJson(res, 401, '媒体 URL 无效或已过期');
      return true;
    }

    if (normalized.message.includes('ENOENT')) {
      writeJson(res, 404, '文件不存在');
      return true;
    }

    if (normalized.message.includes('Range')) {
      writeJson(res, 416, 'Range 不合法');
      return true;
    }

    writeJson(res, 400, '媒体请求无效');
    return true;
  }
}
