import { extractReasoningMiddleware, generateText, stepCountIs, streamText, wrapLanguageModel } from 'ai';
import { VT_STATUS } from '@shared/constants/status';
import { createError } from '../result';
import { MODEL_TYPES } from './constants';
import { splitModelId, resolveModelKey } from './resolver';
import type { TextInvokeInput, TextModelConfig, TextStreamInput } from './types';
import { getVendorRuntime } from './vendor-service';

function resolveTextRequest(modelKey: string, think?: boolean, thinkLevel: 0 | 1 | 2 | 3 = 0) {
  const resolved = resolveModelKey(modelKey);
  const { vendorId, modelName } = splitModelId(resolved.modelId);
  const runtime = getVendorRuntime(vendorId);
  const model = runtime.vendor?.models.find((item) => item.modelName === modelName);

  if (!model || model.type !== MODEL_TYPES.TEXT) {
    throw createError(VT_STATUS.MODEL_NOT_FOUND, `未找到文本模型 ${modelName}`);
  }

  if (!runtime.textRequest) {
    throw createError(VT_STATUS.MODEL_VENDOR_INVALID, '供应商未导出 textRequest');
  }

  const effectiveThink = think ?? Boolean((model as TextModelConfig).think);
  const baseModel = runtime.textRequest(model as TextModelConfig, effectiveThink, thinkLevel);

  return { baseModel, agentConfig: resolved.agentConfig };
}

function withCommonTextSettings<TInput extends TextInvokeInput | TextStreamInput>(input: TInput) {
  const { modelKey, think, thinkLevel, ...rest } = input;
  const { baseModel, agentConfig } = resolveTextRequest(modelKey, think, thinkLevel);
  const toolCount = rest.tools ? Object.keys(rest.tools).length : 0;

  return {
    settings: {
      ...rest,
      model: baseModel,
      ...(toolCount > 0 ? { stopWhen: stepCountIs(toolCount * 50) } : {}),
      ...(agentConfig?.temperature !== null && agentConfig?.temperature !== undefined ? { temperature: agentConfig.temperature } : {}),
      ...(agentConfig?.maxOutputTokens !== null && agentConfig?.maxOutputTokens !== undefined && agentConfig.maxOutputTokens > 0 ? { maxOutputTokens: agentConfig.maxOutputTokens } : {}),
    },
    baseModel,
  };
}

export function invokeText(input: TextInvokeInput) {
  const { settings } = withCommonTextSettings(input);
  return generateText(settings as Parameters<typeof generateText>[0]);
}

export function streamModelText(input: TextStreamInput) {
  const { settings, baseModel } = withCommonTextSettings(input);
  const wrappedModel = wrapLanguageModel({
    model: baseModel as Parameters<typeof wrapLanguageModel>[0]['model'],
    middleware: extractReasoningMiddleware({ tagName: 'reasoning_content', separator: '\n' }),
  });

  return streamText({
    ...settings,
    model: wrappedModel,
  } as Parameters<typeof streamText>[0]);
}
