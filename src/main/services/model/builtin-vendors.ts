import { createAnthropic } from '@ai-sdk/anthropic';
import { createDeepSeek } from '@ai-sdk/deepseek';
import { createGoogleGenerativeAI } from '@ai-sdk/google';
import { createOpenAI } from '@ai-sdk/openai';
import { createOpenAICompatible } from '@ai-sdk/openai-compatible';
import { VT_STATUS } from '@shared/constants/status';
import { createError } from '../result';
import type {
  ImageGenerateInput,
  ImageModelConfig,
  TextModelConfig,
  TtsGenerateInput,
  TtsModelConfig,
  VendorManifest,
  VendorRuntime,
  VideoGenerateInput,
  VideoModelConfig,
} from './types';

type TextProviderFactory = (inputValues: Record<string, string>) => (modelName: string) => unknown;

interface BuiltinVendorDefinition {
  manifest: VendorManifest;
  capabilities: Array<'text' | 'image' | 'video' | 'tts' | 'workflow'>;
  createTextProvider?: TextProviderFactory;
}

function missingAdapter(type: string): never {
  throw createError(VT_STATUS.MODEL_VENDOR_INVALID, `内置供应商暂不支持${type}模型测试，请使用高级 adapter 接入`);
}

function getRequiredInput(inputValues: Record<string, string>, key: string, label: string): string {
  const value = inputValues[key]?.trim();
  if (!value) {
    throw createError(VT_STATUS.MODEL_API_KEY_MISSING, `${label}缺失`);
  }

  return value;
}

function getOptionalInput(inputValues: Record<string, string>, key: string): string | undefined {
  return inputValues[key]?.trim() || undefined;
}

function normalizeOfficialDeepSeekBaseURL(baseURL: string | undefined): string | undefined {
  if (!baseURL) {
    return undefined;
  }

  try {
    const parsed = new URL(baseURL);
    const normalizedPath = parsed.pathname.replace(/\/+$/, '');
    if (parsed.hostname === 'api.deepseek.com' && normalizedPath === '/v1') {
      return parsed.origin;
    }
  } catch {
    return baseURL;
  }

  return baseURL;
}

function normalizeOfficialOpenAIBaseURL(baseURL: string | undefined): string | undefined {
  if (!baseURL) {
    return undefined;
  }

  try {
    const parsed = new URL(baseURL);
    const normalizedPath = parsed.pathname.replace(/\/+$/, '').replace(/\/(chat\/completions|responses|completions)$/, '');
    if (parsed.hostname === 'api.openai.com' && ['', '/'].includes(normalizedPath || '/')) {
      return `${parsed.origin}/v1`;
    }

    if (parsed.hostname === 'api.openai.com') {
      return `${parsed.origin}${normalizedPath}`;
    }
  } catch {
    return baseURL;
  }

  return baseURL;
}

function normalizeOpenAICompatibleBaseURL(
  baseURL: string | undefined,
  options: {
    appendV1WhenEmpty?: boolean;
  } = {},
): string | undefined {
  if (!baseURL) {
    return undefined;
  }

  try {
    const parsed = new URL(baseURL);
    let normalizedPath = parsed.pathname.replace(/\/+$/, '');
    normalizedPath = normalizedPath.replace(/\/(chat\/completions|responses|completions)$/, '');

    if (!normalizedPath && options.appendV1WhenEmpty) {
      normalizedPath = '/v1';
    }

    return `${parsed.origin}${normalizedPath}`;
  } catch {
    return baseURL;
  }
}

function isOfficialOpenAIBaseURL(baseURL: string | undefined): boolean {
  if (!baseURL) {
    return true;
  }

  try {
    return new URL(baseURL).hostname === 'api.openai.com';
  } catch {
    return false;
  }
}

const openAiCompatibleTextProvider =
  (name: string, defaultBaseURL: string): TextProviderFactory =>
  (inputValues) => {
    const apiKey = getRequiredInput(inputValues, 'apiKey', 'API Key');
    const baseURL = getOptionalInput(inputValues, 'baseUrl') ?? defaultBaseURL;
    const provider = createOpenAICompatible({ name, apiKey, baseURL });
    return (modelName: string) => provider(modelName);
  };

export const BUILTIN_VENDOR_DEFINITIONS: Record<string, BuiltinVendorDefinition> = {
  openai: {
    capabilities: ['text', 'image', 'tts'],
    manifest: {
      id: 'openai',
      name: 'OpenAI',
      author: 'VT Studio',
      version: '1.0.0',
      description: 'OpenAI 官方接口，支持 GPT 文本模型，并预留图片和语音能力。',
      icon: 'openai',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: true, placeholder: 'sk-...' },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: false, placeholder: 'https://api.openai.com/v1' },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [
        { name: 'GPT-5.5', modelName: 'gpt-5.5', type: 'text', think: true },
        { name: 'GPT-4.1', modelName: 'gpt-4.1', type: 'text', think: false },
        { name: 'GPT-4.1 mini', modelName: 'gpt-4.1-mini', type: 'text', think: false },
        { name: 'GPT-4o', modelName: 'gpt-4o', type: 'text', think: false },
        { name: 'GPT Image 1', modelName: 'gpt-image-1', type: 'image', mode: ['text', 'singleImage'] },
        {
          name: 'TTS 1',
          modelName: 'tts-1',
          type: 'tts',
          voices: [
            { title: 'Alloy', voice: 'alloy' },
            { title: 'Verse', voice: 'verse' },
          ],
        },
      ],
    },
    createTextProvider: (inputValues) => {
      const apiKey = getRequiredInput(inputValues, 'apiKey', 'API Key');
      const baseURL = normalizeOfficialOpenAIBaseURL(getOptionalInput(inputValues, 'baseUrl'));
      if (!isOfficialOpenAIBaseURL(baseURL)) {
        const compatibleBaseURL = normalizeOpenAICompatibleBaseURL(baseURL, { appendV1WhenEmpty: true });
        const provider = createOpenAICompatible({ name: 'openai-compatible', apiKey, baseURL: compatibleBaseURL! });
        return (modelName: string) => provider(modelName);
      }

      const provider = createOpenAI({ apiKey, ...(baseURL ? { baseURL } : {}) });
      return (modelName: string) => provider(modelName);
    },
  },
  deepseek: {
    capabilities: ['text'],
    manifest: {
      id: 'deepseek',
      name: 'DeepSeek',
      author: 'VT Studio',
      version: '1.0.0',
      description: 'DeepSeek 文本模型服务，适合作为通用 Agent 文本模型。',
      icon: 'deepseek',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: true, placeholder: 'sk-...' },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: false, placeholder: 'https://api.deepseek.com' },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [
        { name: 'DeepSeek Chat', modelName: 'deepseek-chat', type: 'text', think: false },
        { name: 'DeepSeek Reasoner', modelName: 'deepseek-reasoner', type: 'text', think: true },
      ],
    },
    createTextProvider: (inputValues) => {
      const apiKey = getRequiredInput(inputValues, 'apiKey', 'API Key');
      const baseURL = normalizeOfficialDeepSeekBaseURL(getOptionalInput(inputValues, 'baseUrl'));
      const provider = createDeepSeek({ apiKey, ...(baseURL ? { baseURL } : {}) });
      return (modelName: string) => provider(modelName);
    },
  },
  anthropic: {
    capabilities: ['text'],
    manifest: {
      id: 'anthropic',
      name: 'Anthropic',
      author: 'VT Studio',
      version: '1.0.0',
      description: 'Claude 文本模型服务，适合长文本和复杂推理场景。',
      icon: 'anthropic',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: true, placeholder: 'sk-ant-...' },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: false, placeholder: 'https://api.anthropic.com/v1' },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [
        { name: 'Claude Sonnet 4.5', modelName: 'claude-sonnet-4-5', type: 'text', think: true },
        { name: 'Claude Haiku 4.5', modelName: 'claude-haiku-4-5', type: 'text', think: false },
      ],
    },
    createTextProvider: (inputValues) => {
      const apiKey = getRequiredInput(inputValues, 'apiKey', 'API Key');
      const baseURL = getOptionalInput(inputValues, 'baseUrl');
      const provider = createAnthropic({ apiKey, ...(baseURL ? { baseURL } : {}) });
      return (modelName: string) => provider(modelName);
    },
  },
  gemini: {
    capabilities: ['text', 'image', 'video'],
    manifest: {
      id: 'gemini',
      name: 'Gemini',
      author: 'VT Studio',
      version: '1.0.0',
      description: 'Google Gemini 模型服务，文本可直接测试，图片/视频能力预留增强 adapter。',
      icon: 'gemini',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: true, placeholder: 'Google AI API Key' },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: false, placeholder: 'https://generativelanguage.googleapis.com/v1beta' },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [
        { name: 'Gemini 2.5 Pro', modelName: 'gemini-2.5-pro', type: 'text', think: true },
        { name: 'Gemini 2.5 Flash', modelName: 'gemini-2.5-flash', type: 'text', think: true },
        { name: 'Imagen 4', modelName: 'imagen-4.0-generate-001', type: 'image', mode: ['text'] },
      ],
    },
    createTextProvider: (inputValues) => {
      const apiKey = getRequiredInput(inputValues, 'apiKey', 'API Key');
      const baseURL = getOptionalInput(inputValues, 'baseUrl');
      const provider = createGoogleGenerativeAI({ apiKey, ...(baseURL ? { baseURL } : {}) });
      return (modelName: string) => provider(modelName);
    },
  },
  toonflow: {
    capabilities: ['text', 'image', 'video', 'tts'],
    manifest: {
      id: 'toonflow',
      name: 'Toonflow',
      author: 'VT Studio',
      version: '1.0.0',
      description: '参考项目默认供应商，保留为兼容入口。需要高级 adapter 才能真实调用。',
      icon: 'toonflow',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: false },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: false },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [],
    },
  },
  atlascloud: {
    capabilities: ['text', 'image', 'video'],
    manifest: {
      id: 'atlascloud',
      name: 'AtlasCloud',
      author: 'VT Studio',
      version: '1.0.0',
      description: 'OpenAI Compatible 协议入口，适合中转服务或私有网关。',
      icon: 'atlascloud',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: true },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: true, placeholder: 'https://example.com/v1' },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [{ name: '自定义文本模型', modelName: 'custom-chat-model', type: 'text', think: false }],
    },
    createTextProvider: openAiCompatibleTextProvider('atlascloud', ''),
  },
  volcengine: {
    capabilities: ['text', 'image', 'video'],
    manifest: {
      id: 'volcengine',
      name: '火山引擎',
      author: 'VT Studio',
      version: '1.0.0',
      description: '火山引擎/豆包等模型服务入口，默认按 OpenAI Compatible 文本协议接入。',
      icon: 'volcengine',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: true },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: true, placeholder: 'https://ark.cn-beijing.volces.com/api/v3' },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [{ name: 'Doubao 文本模型', modelName: 'doubao-seed-1-6', type: 'text', think: false }],
    },
    createTextProvider: openAiCompatibleTextProvider('volcengine', 'https://ark.cn-beijing.volces.com/api/v3'),
  },
  minimax: {
    capabilities: ['text', 'image', 'video', 'tts'],
    manifest: {
      id: 'minimax',
      name: 'MiniMax',
      author: 'VT Studio',
      version: '1.0.0',
      description: 'MiniMax 文本/音视频能力入口，第一版按 OpenAI Compatible 文本协议接入。',
      icon: 'minimax',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: true },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: false, placeholder: 'https://api.minimax.io/v1' },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [{ name: 'MiniMax Text', modelName: 'MiniMax-Text-01', type: 'text', think: false }],
    },
    createTextProvider: openAiCompatibleTextProvider('minimax', 'https://api.minimax.io/v1'),
  },
  klingai: {
    capabilities: ['image', 'video'],
    manifest: {
      id: 'klingai',
      name: '可灵',
      author: 'VT Studio',
      version: '1.0.0',
      description: '可灵图片/视频生成入口，需要高级 adapter 接入任务、轮询和结果解析。',
      icon: 'klingai',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: true },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: false },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [
        {
          name: 'Kling Video',
          modelName: 'kling-video',
          type: 'video',
          mode: ['singleImage', 'text'],
          audio: 'optional',
          durationResolutionMap: [{ duration: [5, 10], resolution: ['720p', '1080p'] }],
        },
      ],
    },
  },
  vidu: {
    capabilities: ['image', 'video'],
    manifest: {
      id: 'vidu',
      name: 'Vidu',
      author: 'VT Studio',
      version: '1.0.0',
      description: 'Vidu 图片/视频生成入口，需要高级 adapter 接入任务、轮询和结果解析。',
      icon: 'vidu',
      inputs: [
        { key: 'apiKey', label: 'API Key', type: 'password', required: true },
        { key: 'baseUrl', label: 'Base URL', type: 'url', required: false },
      ],
      inputValues: { apiKey: '', baseUrl: '' },
      models: [
        {
          name: 'Vidu Video',
          modelName: 'vidu-video',
          type: 'video',
          mode: ['singleImage', 'text'],
          audio: 'optional',
          durationResolutionMap: [{ duration: [4, 8], resolution: ['720p', '1080p'] }],
        },
      ],
    },
  },
  comfyui: {
    capabilities: ['image', 'workflow'],
    manifest: {
      id: 'comfyui',
      name: 'ComfyUI',
      author: 'VT Studio',
      version: '1.0.0',
      description: '本地 ComfyUI 工作流增强 adapter，后续接 workflow、节点映射、队列和历史轮询。',
      icon: 'comfyui',
      inputs: [
        { key: 'endpoint', label: 'Endpoint', type: 'url', required: true, placeholder: 'http://127.0.0.1:8188' },
        { key: 'workflow', label: 'Workflow JSON', type: 'text', required: false },
      ],
      inputValues: { endpoint: '', workflow: '' },
      models: [{ name: 'ComfyUI Workflow', modelName: 'comfyui-workflow', type: 'image', mode: ['text', 'singleImage', 'multiReference'] }],
    },
  },
};

export function getBuiltinVendorIds(): string[] {
  return Object.keys(BUILTIN_VENDOR_DEFINITIONS);
}

export function getBuiltinVendorDefinition(vendorId: string): BuiltinVendorDefinition | null {
  return BUILTIN_VENDOR_DEFINITIONS[vendorId] ?? null;
}

export function createBuiltinVendorRuntime(vendorId: string, inputValues: Record<string, string>, models = BUILTIN_VENDOR_DEFINITIONS[vendorId]?.manifest.models ?? []): VendorRuntime | null {
  const definition = getBuiltinVendorDefinition(vendorId);
  if (!definition) {
    return null;
  }

  return {
    vendor: {
      ...definition.manifest,
      inputValues: { ...definition.manifest.inputValues, ...inputValues },
      models,
    },
    textRequest: (model: TextModelConfig) => {
      if (!definition.createTextProvider) {
        missingAdapter('文本');
      }

      return definition.createTextProvider(inputValues)(model.modelName);
    },
    imageRequest: async (_config: ImageGenerateInput, _model: ImageModelConfig) => missingAdapter('图片'),
    videoRequest: async (_config: VideoGenerateInput, _model: VideoModelConfig) => missingAdapter('视频'),
    ttsRequest: async (_config: TtsGenerateInput, _model: TtsModelConfig) => missingAdapter('TTS'),
  };
}

export function createBuiltinConnectionRuntime(input: {
  connectionId: string;
  connectionName: string;
  adapterVendorId: string;
  inputValues: Record<string, string>;
  models: VendorManifest['models'];
}): VendorRuntime | null {
  const runtime = createBuiltinVendorRuntime(input.adapterVendorId, input.inputValues, input.models);
  if (!runtime?.vendor) {
    return runtime;
  }

  return {
    ...runtime,
    vendor: {
      ...runtime.vendor,
      id: input.connectionId,
      name: input.connectionName,
      models: input.models,
      inputValues: input.inputValues,
    },
  };
}

export function isBuiltinVendor(vendorId: string): boolean {
  return Boolean(getBuiltinVendorDefinition(vendorId));
}
