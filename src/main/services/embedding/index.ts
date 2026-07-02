import { existsSync } from 'node:fs';
import { dirname } from 'node:path';
import { env as transformersEnv, pipeline, type FeatureExtractionPipeline } from '@huggingface/transformers';
import { VT_STATUS } from '@shared/constants/status';
import { normalizeUnknownError } from '@shared/errors';
import { getDatabase } from '../database';
import { getRuntimeDirectories, safeJoin } from '../file-system';
import { logger } from '../logger';
import { createError } from '../result';

const DEFAULT_MODEL_ONNX_FILE = ['all-MiniLM-L6-v2', 'onnx', 'model_fp16.onnx'];
const DEFAULT_MODEL_DTYPE = 'fp16';
const MAX_EMBED_TEXT_LENGTH = 8000;

let extractor: FeatureExtractionPipeline | null = null;
let loadingPromise: Promise<FeatureExtractionPipeline> | null = null;

interface EmbeddingConfig {
  modelOnnxFile: string[];
  modelDtype: string;
  modelFilePath: string;
  modelFolder: string;
}

function readSettingValue(key: string): string | null {
  const row = getDatabase().prepare<[string], { value: string }>('SELECT value FROM app_settings WHERE key = ? LIMIT 1').get(key);
  return row?.value ?? null;
}

function parseModelOnnxFile(rawValue: string | null): string[] {
  if (!rawValue) {
    return DEFAULT_MODEL_ONNX_FILE;
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(rawValue);
  } catch (error) {
    throw createError(VT_STATUS.EMBEDDING_MODEL_INVALID, '本地向量模型路径配置不是合法 JSON', error);
  }

  if (
    !Array.isArray(parsed) ||
    parsed.length < 2 ||
    parsed.some((item) => typeof item !== 'string' || item.trim().length === 0)
  ) {
    throw createError(VT_STATUS.EMBEDDING_MODEL_INVALID, '本地向量模型路径必须是非空字符串数组');
  }

  return parsed.map((item) => item.trim());
}

function getEmbeddingConfig(): EmbeddingConfig {
  const modelOnnxFile = parseModelOnnxFile(readSettingValue('modelOnnxFile'));
  const modelDtype = readSettingValue('modelDtype') || DEFAULT_MODEL_DTYPE;
  const modelFilePath = safeJoin(getRuntimeDirectories().models, modelOnnxFile.join('/'));
  const modelFolder = modelOnnxFile[0];

  return {
    modelOnnxFile,
    modelDtype,
    modelFilePath,
    modelFolder,
  };
}

export function getEmbeddingModelStatus(): { available: boolean; modelFilePath: string; modelFolder: string; modelDtype: string } {
  const config = getEmbeddingConfig();

  return {
    available: existsSync(config.modelFilePath),
    modelFilePath: config.modelFilePath,
    modelFolder: config.modelFolder,
    modelDtype: config.modelDtype,
  };
}

function assertEmbeddingModelExists(config: EmbeddingConfig): void {
  if (!existsSync(config.modelFilePath)) {
    logger.warn('记忆功能', '本地向量模型文件未安装，embedding 功能将降级');
    logger.detail('记忆功能', '缺失向量模型路径', {
      modelFilePath: config.modelFilePath,
      modelFolder: config.modelFolder,
    });
    throw createError(
      VT_STATUS.EMBEDDING_MODEL_NOT_FOUND,
      `本地向量模型文件不存在，请放置模型文件后重试：${config.modelOnnxFile.join('/')}`,
    );
  }
}

async function createExtractor(): Promise<FeatureExtractionPipeline> {
  const config = getEmbeddingConfig();
  assertEmbeddingModelExists(config);

  transformersEnv.allowRemoteModels = false;
  transformersEnv.allowLocalModels = true;
  transformersEnv.localModelPath = `${getRuntimeDirectories().models.replace(/\\/g, '/')}/`;

  try {
    const createdExtractor = await pipeline('feature-extraction', config.modelFolder, {
      dtype: config.modelDtype,
    } as Parameters<typeof pipeline>[2]);

    logger.info('记忆功能', '本地向量模型已加载');
    logger.detail('记忆功能', '本地向量模型详情', {
      modelFolder: config.modelFolder,
      modelFileDirectory: dirname(config.modelFilePath),
      dtype: config.modelDtype,
    });

    return createdExtractor as FeatureExtractionPipeline;
  } catch (error) {
    const normalized = normalizeUnknownError(error);
    throw createError(VT_STATUS.EMBEDDING_ERROR, `本地向量模型初始化失败：${normalized.message}`, error);
  }
}

export async function initEmbedding(): Promise<void> {
  if (extractor) {
    return;
  }

  if (!loadingPromise) {
    loadingPromise = createExtractor().catch((error) => {
      loadingPromise = null;
      throw error;
    });
  }

  extractor = await loadingPromise;
}

export async function embedText(text: string): Promise<number[]> {
  const normalizedText = text.trim();
  if (!normalizedText) {
    throw createError(VT_STATUS.INVALID_PARAMS, '向量化文本不能为空');
  }

  await initEmbedding();

  const input = normalizedText.length > MAX_EMBED_TEXT_LENGTH ? normalizedText.slice(0, MAX_EMBED_TEXT_LENGTH) : normalizedText;
  const output = await extractor!(input, { pooling: 'mean', normalize: true });
  const rawData = output.data;

  if (!(rawData instanceof Float32Array) && !Array.isArray(rawData)) {
    throw createError(VT_STATUS.EMBEDDING_ERROR, '本地向量模型返回数据格式无效');
  }

  return Array.from(rawData as Float32Array | number[]);
}

export function cosineSimilarity(a: number[], b: number[]): number {
  if (a.length === 0 || a.length !== b.length) {
    return Number.NEGATIVE_INFINITY;
  }

  return a.reduce((dot, value, index) => dot + value * b[index], 0);
}

export async function disposeEmbedding(): Promise<void> {
  const currentExtractor = extractor;
  extractor = null;
  loadingPromise = null;

  await currentExtractor?.dispose?.();
}
