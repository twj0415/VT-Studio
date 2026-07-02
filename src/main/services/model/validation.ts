import { VT_STATUS } from '@shared/constants/status';
import { createError } from '../result';
import type { ImageMode, VendorInput, VendorManifest, VendorModelConfig, VideoMode } from './types';

const inputTypes = new Set(['text', 'password', 'url']);
const imageModes = new Set(['text', 'singleImage', 'multiReference']);
const videoSimpleModes = new Set([
  'singleImage',
  'startEndRequired',
  'endFrameOptional',
  'startFrameOptional',
  'text',
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function assertString(value: unknown, label: string): string {
  if (typeof value !== 'string' || !value.trim()) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, `${label}不能为空`);
  }

  return value;
}

function normalizeVendorInput(value: unknown): VendorInput {
  if (!isRecord(value)) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '供应商 inputs 配置无效');
  }

  const type = assertString(value.type, 'input.type');
  if (!inputTypes.has(type)) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, `不支持的供应商参数类型：${type}`);
  }

  return {
    key: assertString(value.key, 'input.key'),
    label: assertString(value.label, 'input.label'),
    type: type as VendorInput['type'],
    required: Boolean(value.required),
    placeholder: typeof value.placeholder === 'string' ? value.placeholder : undefined,
  };
}

function normalizeImageMode(value: unknown): ImageMode {
  const mode = assertString(value, 'image.mode');
  if (!imageModes.has(mode)) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, `不支持的图片模型模式：${mode}`);
  }

  return mode as ImageMode;
}

function normalizeVideoMode(value: unknown): VideoMode {
  if (Array.isArray(value)) {
    return value.map((item) => {
      const mode = assertString(item, 'video.referenceMode');
      if (!/^(videoReference|imageReference|audioReference):\d+$/.test(mode)) {
        throw createError(VT_STATUS.MODEL_VENDOR_INVALID, `不支持的视频参考模式：${mode}`);
      }

      return mode as `${'videoReference' | 'imageReference' | 'audioReference'}:${number}`;
    });
  }

  const mode = assertString(value, 'video.mode');
  if (!videoSimpleModes.has(mode)) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, `不支持的视频模型模式：${mode}`);
  }

  return mode as VideoMode;
}

function normalizeModel(value: unknown): VendorModelConfig {
  if (!isRecord(value)) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '模型配置无效');
  }

  const type = assertString(value.type, 'model.type');
  const base = {
    name: assertString(value.name, 'model.name'),
    modelName: assertString(value.modelName, 'model.modelName'),
  };

  if (type === 'text') {
    return { ...base, type, think: Boolean(value.think) };
  }

  if (type === 'image') {
    if (!Array.isArray(value.mode)) {
      throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '图片模型 mode 必须是数组');
    }

    return {
      ...base,
      type,
      mode: value.mode.map(normalizeImageMode),
      associationSkills: typeof value.associationSkills === 'string' ? value.associationSkills : undefined,
    };
  }

  if (type === 'video') {
    if (!Array.isArray(value.mode)) {
      throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '视频模型 mode 必须是数组');
    }

    if (!Array.isArray(value.durationResolutionMap)) {
      throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '视频模型 durationResolutionMap 必须是数组');
    }

    return {
      ...base,
      type,
      mode: value.mode.map(normalizeVideoMode),
      associationSkills: typeof value.associationSkills === 'string' ? value.associationSkills : undefined,
      audio: value.audio === 'optional' ? 'optional' : Boolean(value.audio),
      durationResolutionMap: value.durationResolutionMap.map((item) => {
        if (!isRecord(item) || !Array.isArray(item.duration) || !Array.isArray(item.resolution)) {
          throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '视频模型 durationResolutionMap 格式无效');
        }

        return {
          duration: item.duration.map(Number).filter((duration) => Number.isFinite(duration) && duration > 0),
          resolution: item.resolution.filter((resolution): resolution is string => typeof resolution === 'string' && Boolean(resolution)),
        };
      }),
    };
  }

  if (type === 'tts') {
    if (!Array.isArray(value.voices)) {
      throw createError(VT_STATUS.MODEL_VENDOR_INVALID, 'TTS 模型 voices 必须是数组');
    }

    return {
      ...base,
      type,
      voices: value.voices.map((voice) => {
        if (!isRecord(voice)) {
          throw createError(VT_STATUS.MODEL_VENDOR_INVALID, 'TTS 模型 voices 格式无效');
        }

        return {
          title: assertString(voice.title, 'tts.voice.title'),
          voice: assertString(voice.voice, 'tts.voice.voice'),
        };
      }),
    };
  }

  throw createError(VT_STATUS.MODEL_VENDOR_INVALID, `不支持的模型类型：${type}`);
}

export function normalizeVendorManifest(value: unknown): VendorManifest {
  if (!isRecord(value)) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, 'vendor 必须是对象');
  }

  if (!Array.isArray(value.inputs)) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, 'vendor.inputs 必须是数组');
  }

  if (!isRecord(value.inputValues)) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, 'vendor.inputValues 必须是对象');
  }

  if (!Array.isArray(value.models)) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, 'vendor.models 必须是数组');
  }

  const id = assertString(value.id, 'vendor.id');
  if (id.includes(':')) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '供应商 id 不能包含英文冒号');
  }

  return {
    id,
    author: assertString(value.author, 'vendor.author'),
    description: typeof value.description === 'string' ? value.description : undefined,
    name: assertString(value.name, 'vendor.name'),
    icon: typeof value.icon === 'string' ? value.icon : undefined,
    inputs: value.inputs.map(normalizeVendorInput),
    inputValues: Object.fromEntries(Object.entries(value.inputValues).map(([key, val]) => [key, typeof val === 'string' ? val : String(val ?? '')])),
    models: value.models.map(normalizeModel),
    version: typeof value.version === 'string' ? value.version : undefined,
  };
}

export function assertVendorRequiredInputs(manifest: VendorManifest, inputValues: Record<string, string>): void {
  for (const input of manifest.inputs) {
    if (!input.required) {
      continue;
    }

    if (!inputValues[input.key]?.trim()) {
      const statusCode = /api.?key|token|secret/i.test(input.key) ? VT_STATUS.MODEL_API_KEY_MISSING : VT_STATUS.MODEL_VENDOR_INPUT_MISSING;
      throw createError(statusCode, `${input.label} 缺失`);
    }
  }
}
