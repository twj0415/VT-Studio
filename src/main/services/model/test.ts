import { VT_STATUS } from '@shared/constants/status';
import { isVtError, normalizeUnknownError } from '@shared/errors';
import { writeManagedFile, getRuntimeDirectories } from '../file-system';
import { createError } from '../result';
import { MODEL_TEST_FILE_NAMES } from './constants';
import { getModelDetail } from './resolver';
import type { ModelTestImageInput, ModelTestTextInput, ModelTestVideoInput, VideoModelConfig } from './types';
import { generateImageByModel, generateVideoByModel } from './media';
import { invokeText } from './text';

function formatTextTestErrorMessage(message: string): string {
  if (/invalid json response/i.test(message)) {
    return '接口返回了非 JSON。中转地址通常应填到 /v1，不要填完整的 /chat/completions 或 /responses；也可能是网关返回了网页错误页';
  }

  return message;
}

export async function testTextModel(input: ModelTestTextInput): Promise<{ thinking?: string; content: string }> {
  try {
    const result = await invokeText({
      modelKey: `${input.vendorId}:${input.modelName}`,
      messages: input.messages,
    });

    return {
      thinking: typeof result.reasoningText === 'string' ? result.reasoningText : undefined,
      content: result.text,
    };
  } catch (error) {
    if (isVtError(error)) {
      throw error;
    }

    const normalized = normalizeUnknownError(error);
    throw createError(VT_STATUS.MODEL_ERROR, `模型测试失败：${formatTextTestErrorMessage(normalized.message)}`, error);
  }
}

export async function testImageModel(input: ModelTestImageInput): Promise<{ filePath: string; content: string }> {
  const content = await generateImageByModel(`${input.vendorId}:${input.modelName}`, {
    prompt: input.prompt,
    referenceList: input.imageBase64 ? [{ type: 'image', sourceType: 'base64', base64: input.imageBase64 }] : [],
    size: '1K',
    aspectRatio: '16:9',
  });
  const filePath = writeManagedFile(getRuntimeDirectories().modelTest, MODEL_TEST_FILE_NAMES.image, content);

  return { filePath, content };
}

export async function testVideoModel(input: ModelTestVideoInput): Promise<{ filePath: string; content: string }> {
  const model = getModelDetail(`${input.vendorId}:${input.modelName}`) as VideoModelConfig;
  const firstDurationResolution = model.durationResolutionMap[0];
  const duration = firstDurationResolution?.duration[0] ?? 5;
  const resolution = firstDurationResolution?.resolution[0] ?? '720p';
  const content = await generateVideoByModel(`${input.vendorId}:${input.modelName}`, {
    duration,
    resolution,
    aspectRatio: '16:9',
    prompt: input.prompt,
    referenceList: [...input.images, ...input.videos, ...input.audios],
    audio: typeof model.audio === 'boolean' ? model.audio : true,
    mode: input.mode,
  });
  const filePath = writeManagedFile(getRuntimeDirectories().modelTest, MODEL_TEST_FILE_NAMES.video, content);

  return { filePath, content };
}
