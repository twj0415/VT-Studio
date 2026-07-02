import { VT_STATUS } from '@shared/constants/status';
import { normalizeUnknownError } from '@shared/errors';
import { createTask, failTask, succeedTask } from '../task';
import { createError } from '../result';
import { MODEL_TYPES } from './constants';
import { resolveModelKey, splitModelId } from './resolver';
import type {
  AudioGenerateInput,
  ImageGenerateInput,
  ImageModelConfig,
  ModelTaskOptions,
  ReferenceItem,
  TtsModelConfig,
  VendorModelConfig,
  VideoGenerateInput,
  VideoModelConfig,
} from './types';
import { getVendor, getVendorRuntime } from './vendor-service';

async function urlToBase64(url: string): Promise<string> {
  const response = await fetch(url);

  if (!response.ok) {
    throw createError(VT_STATUS.MODEL_ERROR, `下载模型结果失败：${response.status}`);
  }

  const contentType = response.headers.get('content-type') ?? 'application/octet-stream';
  const buffer = Buffer.from(await response.arrayBuffer());
  return `data:${contentType};base64,${buffer.toString('base64')}`;
}

function normalizeLegacyReferenceInput(vendorId: string, input: ImageGenerateInput | VideoGenerateInput | AudioGenerateInput): void {
  const vendor = getVendor(vendorId);
  const version = Number.parseFloat(vendor.manifest.version ?? '1.0');

  if (Number.isFinite(version) && version >= 2) {
    return;
  }

  const legacyInput = input as ImageGenerateInput & { imageBase64?: string[] };
  if (legacyInput.referenceList?.length) {
    legacyInput.imageBase64 = legacyInput.referenceList.filter((item) => item.type === 'image').map((item) => item.base64);
  }
}

function ensureBase64SourceType<T extends { referenceList?: ReferenceItem[] }>(input: T): T {
  if (!input.referenceList) {
    return input;
  }

  input.referenceList = input.referenceList.map((item) => ({
    ...item,
    sourceType: 'base64',
  }));

  return input;
}

async function runWithTask<T>(task: ModelTaskOptions | undefined, modelName: string, runner: () => Promise<T>): Promise<T> {
  if (!task) {
    return runner();
  }

  const createdTask = createTask({
    projectId: task.projectId,
    category: task.category,
    modelName,
    description: task.description,
    relatedObjects: task.relatedObjects,
  });

  try {
    const result = await runner();
    succeedTask(createdTask.taskId);
    return result;
  } catch (error) {
    failTask(createdTask.taskId, normalizeUnknownError(error).message);
    throw error;
  }
}

function resolveMediaModel<TModel extends VendorModelConfig>(modelKey: string, expectedType: TModel['type']) {
  const resolved = resolveModelKey(modelKey);
  const { vendorId, modelName } = splitModelId(resolved.modelId);
  const runtime = getVendorRuntime(vendorId);
  const model = runtime.vendor?.models.find((item) => item.modelName === modelName);

  if (!model || model.type !== expectedType) {
    throw createError(VT_STATUS.MODEL_NOT_FOUND, `未找到${expectedType}模型 ${modelName}`);
  }

  return { runtime, vendorId, modelName, model: model as TModel };
}

export async function generateImageByModel(modelKey: string, input: ImageGenerateInput): Promise<string> {
  const { runtime, vendorId, modelName, model } = resolveMediaModel<ImageModelConfig>(modelKey, MODEL_TYPES.IMAGE);

  if (!runtime.imageRequest) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '供应商未导出 imageRequest');
  }

  return runWithTask(input.task, modelName, async () => {
    ensureBase64SourceType(input);
    normalizeLegacyReferenceInput(vendorId, input);
    const result = await runtime.imageRequest!(input, model);
    return result.startsWith('http') ? urlToBase64(result) : result;
  });
}

export async function generateVideoByModel(modelKey: string, input: VideoGenerateInput): Promise<string> {
  const { runtime, vendorId, modelName, model } = resolveMediaModel<VideoModelConfig>(modelKey, MODEL_TYPES.VIDEO);

  if (!runtime.videoRequest) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '供应商未导出 videoRequest');
  }

  return runWithTask(input.task, modelName, async () => {
    ensureBase64SourceType(input);
    normalizeLegacyReferenceInput(vendorId, input);
    const result = await runtime.videoRequest!(input, model);
    return result.startsWith('http') ? urlToBase64(result) : result;
  });
}

export async function generateAudioByModel(modelKey: string, input: AudioGenerateInput): Promise<string> {
  const { runtime, vendorId, modelName, model } = resolveMediaModel<TtsModelConfig>(modelKey, MODEL_TYPES.TTS);

  if (!runtime.ttsRequest) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '供应商未导出 ttsRequest');
  }

  return runWithTask(input.task, modelName, async () => {
    ensureBase64SourceType(input);
    normalizeLegacyReferenceInput(vendorId, input);
    const result = await runtime.ttsRequest!(input, model);
    return result.startsWith('http') ? urlToBase64(result) : result;
  });
}
