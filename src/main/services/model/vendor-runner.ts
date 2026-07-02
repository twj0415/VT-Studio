import { Buffer } from 'node:buffer';
import { createHash } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';
import { createAnthropic } from '@ai-sdk/anthropic';
import { createDeepSeek } from '@ai-sdk/deepseek';
import { createGoogleGenerativeAI } from '@ai-sdk/google';
import { createOpenAI } from '@ai-sdk/openai';
import { createOpenAICompatible } from '@ai-sdk/openai-compatible';
import { createXai } from '@ai-sdk/xai';
import { VT_STATUS } from '@shared/constants/status';
import axios from 'axios';
import FormData from 'form-data';
import jsonwebtoken from 'jsonwebtoken';
import { createQwen } from 'qwen-ai-provider-v5';
import sharp from 'sharp';
import { transform } from 'sucrase';
import { createMinimax } from 'vercel-minimax-ai-provider';
import { VM } from 'vm2';
import { createZhipu } from 'zhipu-ai-provider';
import { logger as mainLogger } from '../logger';
import { createError } from '../result';
import { normalizeVendorManifest } from './validation';
import type { VendorRuntime } from './types';

function splitDataUrl(base64: string): { mime: string; data: string } {
  const match = /^data:([^;]+);base64,(.+)$/s.exec(base64);

  if (!match) {
    return { mime: 'image/png', data: base64 };
  }

  return { mime: match[1], data: match[2] };
}

function toDataUrl(buffer: Buffer, mime = 'image/png'): string {
  return `data:${mime};base64,${buffer.toString('base64')}`;
}

async function zipImage(base64: string, size: number): Promise<string> {
  const { data } = splitDataUrl(base64);
  const maxBytes = size * 1024;
  let quality = 90;
  let buffer = await sharp(Buffer.from(data, 'base64')).jpeg({ quality }).toBuffer();

  while (buffer.byteLength > maxBytes && quality > 30) {
    quality -= 10;
    buffer = await sharp(Buffer.from(data, 'base64')).jpeg({ quality }).toBuffer();
  }

  return toDataUrl(buffer, 'image/jpeg');
}

async function zipImageResolution(base64: string, width: number, height: number): Promise<string> {
  const { data } = splitDataUrl(base64);
  const buffer = await sharp(Buffer.from(data, 'base64')).resize(width, height, { fit: 'inside' }).png().toBuffer();
  return toDataUrl(buffer, 'image/png');
}

async function mergeImages(base64Arr: string[], maxSize = '1024x1024'): Promise<string> {
  const [widthText, heightText] = maxSize.split('x');
  const width = Number.parseInt(widthText ?? '1024', 10) || 1024;
  const height = Number.parseInt(heightText ?? '1024', 10) || 1024;
  const itemWidth = Math.max(1, Math.floor(width / Math.max(1, base64Arr.length)));
  const composites = await Promise.all(
    base64Arr.map(async (base64, index) => {
      const { data } = splitDataUrl(base64);
      const input = await sharp(Buffer.from(data, 'base64')).resize(itemWidth, height, { fit: 'inside' }).png().toBuffer();
      return { input, left: index * itemWidth, top: 0 };
    }),
  );
  const buffer = await sharp({
    create: {
      width,
      height,
      channels: 4,
      background: { r: 255, g: 255, b: 255, alpha: 0 },
    },
  })
    .composite(composites)
    .png()
    .toBuffer();

  return toDataUrl(buffer, 'image/png');
}

async function urlToBase64(url: string): Promise<string> {
  const response = await fetch(url);

  if (!response.ok) {
    throw createError(VT_STATUS.MODEL_ERROR, `下载模型结果失败：${response.status}`);
  }

  const contentType = response.headers.get('content-type') ?? 'application/octet-stream';
  const buffer = Buffer.from(await response.arrayBuffer());
  return `data:${contentType};base64,${buffer.toString('base64')}`;
}

async function pollTask(
  fn: () => Promise<{ completed: boolean; data?: string; error?: string }>,
  interval = 3000,
  timeout = 3000000,
): Promise<{ completed: boolean; data?: string; error?: string }> {
  const start = Date.now();

  while (Date.now() - start < timeout) {
    const result = await fn();

    if (result.completed || result.error) {
      return result;
    }

    await delay(interval);
  }

  return { completed: false, error: 'timeout' };
}

function logger(value: unknown): void {
  mainLogger.detail('供应商脚本', '脚本日志', value);
}

export function runVendorCode(code: string): VendorRuntime {
  try {
    const jsCode = transform(code.replace(/export\s*\{\s*\};?/g, ''), { transforms: ['typescript'] }).code;
    const exports = {};
    const sandbox = {
      exports,
      fetch,
      Buffer,
      crypto: { createHash },
      createOpenAI,
      createDeepSeek,
      createZhipu,
      createQwen,
      createAnthropic,
      createOpenAICompatible,
      createXai,
      createMinimax,
      createGoogleGenerativeAI,
      axios,
      FormData,
      jsonwebtoken,
      zipImage,
      zipImageResolution,
      mergeImages,
      urlToBase64,
      pollTask,
      logger,
    };
    const vm = new VM({
      timeout: 0,
      sandbox,
      compiler: 'javascript',
      eval: false,
      wasm: false,
    });

    vm.run(jsCode);

    return exports as VendorRuntime;
  } catch (error) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '供应商代码运行失败', error);
  }
}

export function validateVendorRuntime(runtime: VendorRuntime): VendorRuntime {
  if (!runtime.vendor) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '脚本文件必须导出 vendor 对象');
  }

  if (!runtime.textRequest) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '脚本文件必须导出 textRequest');
  }

  if (!runtime.imageRequest) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '脚本文件必须导出 imageRequest');
  }

  if (!runtime.videoRequest) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '脚本文件必须导出 videoRequest');
  }

  if (!runtime.ttsRequest) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '脚本文件必须导出 ttsRequest');
  }

  runtime.vendor = normalizeVendorManifest(runtime.vendor);

  return runtime;
}
