import { VT_STATUS } from '@shared/constants/status';
import type {
  BuiltinPromptType,
  PromptItem,
  PromptListResult,
  PromptRestoreDefaultPayload,
  PromptRestoreDefaultResult,
  PromptUpdatePayload,
  PromptUpdateResult,
  PromptValidationWarning,
} from '@shared/types/prompt';
import { getDatabase } from '../database';
import { createError } from '../result';

interface PromptRow {
  id: number;
  name: string;
  type: string;
  data: string;
  use_data: string;
  updated_at: number;
}

const PROMPT_ORDER: BuiltinPromptType[] = [
  'eventExtraction',
  'scriptAssetExtraction',
  'videoPromptGeneration',
  'audioBindPrompt',
];

const PROMPT_TYPE_LABELS: Record<BuiltinPromptType, string> = {
  eventExtraction: '事件提取',
  scriptAssetExtraction: '剧本资产提取',
  videoPromptGeneration: '视频提示词生成',
  audioBindPrompt: '音色绑定',
};

function isBuiltinPromptType(value: string): value is BuiltinPromptType {
  return PROMPT_ORDER.includes(value as BuiltinPromptType);
}

function getPromptSortIndex(type: string): number {
  const index = PROMPT_ORDER.indexOf(type as BuiltinPromptType);
  return index >= 0 ? index : PROMPT_ORDER.length;
}

function normalizePromptText(value: string): string {
  return value.replace(/\r\n/g, '\n').trim();
}

function toPromptItem(row: PromptRow): PromptItem {
  const useData = row.use_data ?? '';
  const effectiveData = useData.trim() ? useData : row.data;
  return {
    id: row.id,
    name: row.name || (isBuiltinPromptType(row.type) ? PROMPT_TYPE_LABELS[row.type] : row.type),
    type: row.type,
    data: row.data,
    useData,
    effectiveData,
    isCustomized: Boolean(useData.trim()),
    updatedAt: row.updated_at,
  };
}

function getPromptRowById(id: number): PromptRow | null {
  const row = getDatabase()
    .prepare<[number], PromptRow>('SELECT id, name, type, data, use_data, updated_at FROM prompts WHERE id = ? LIMIT 1')
    .get(id);
  return row ?? null;
}

function getPromptItemById(id: number): PromptItem {
  const row = getPromptRowById(id);
  if (!row) {
    throw createError(VT_STATUS.NOT_FOUND, '提示词不存在');
  }

  return toPromptItem(row);
}

function hasAll(content: string, tokens: string[]): boolean {
  return tokens.every((token) => content.includes(token));
}

function validatePromptStructure(type: string, content: string): PromptValidationWarning[] {
  const warnings: PromptValidationWarning[] = [];

  if (content.length < 20) {
    warnings.push({ code: 'too-short', message: '内容过短，可能不是完整提示词' });
  }

  if (type === 'eventExtraction' && !hasAll(content, ['输出格式', '| 第X章', '核心事件', '预估集长'])) {
    warnings.push({ code: 'event-format-risk', message: '事件提取提示词缺少表格输出格式或关键字段' });
  }

  if (type === 'scriptAssetExtraction' && !hasAll(content, ['resultTool', 'assetsList', 'role', 'scene', 'tool'])) {
    warnings.push({ code: 'asset-result-tool-risk', message: '剧本资产提取提示词缺少 resultTool 或资产类型结构' });
  }

  if (type === 'videoPromptGeneration' && !hasAll(content, ['<storyboardItem', 'videoDesc', 'Seedance 2.0', 'Wan 2.6'])) {
    warnings.push({ code: 'video-structure-risk', message: '视频提示词生成提示词缺少分镜 XML 或模型模式结构' });
  }

  if (type === 'audioBindPrompt' && !hasAll(content, ['音色', '候选音频', 'audioId'])) {
    warnings.push({ code: 'audio-bind-risk', message: '音色绑定提示词缺少候选音频或 audioId 约束' });
  }

  return warnings;
}

export function getPromptList(): PromptListResult {
  const rows = getDatabase()
    .prepare<[], PromptRow>('SELECT id, name, type, data, use_data, updated_at FROM prompts ORDER BY id ASC')
    .all();

  return {
    prompts: rows
      .map(toPromptItem)
      .sort((left, right) => getPromptSortIndex(left.type) - getPromptSortIndex(right.type) || left.id - right.id),
  };
}

export function updatePrompt(payload: PromptUpdatePayload): PromptUpdateResult {
  const id = Number(payload.id);
  if (!Number.isInteger(id) || id <= 0) {
    throw createError(VT_STATUS.INVALID_PARAMS, '提示词 id 无效');
  }

  const current = getPromptItemById(id);
  const useData = normalizePromptText(payload.useData ?? '');
  if (!useData) {
    throw createError(VT_STATUS.INVALID_PARAMS, '提示词内容不能为空');
  }

  const warnings = validatePromptStructure(current.type, useData);
  if (warnings.length > 0 && !payload.force) {
    return {
      saved: false,
      prompt: current,
      warnings,
    };
  }

  getDatabase()
    .prepare<[string, number, number]>('UPDATE prompts SET use_data = ?, updated_at = ? WHERE id = ?')
    .run(useData, Date.now(), id);

  return {
    saved: true,
    prompt: getPromptItemById(id),
    warnings,
  };
}

export function restorePromptDefault(payload: PromptRestoreDefaultPayload): PromptRestoreDefaultResult {
  const id = Number(payload.id);
  if (!Number.isInteger(id) || id <= 0) {
    throw createError(VT_STATUS.INVALID_PARAMS, '提示词 id 无效');
  }

  const current = getPromptItemById(id);
  if (!current.data.trim()) {
    throw createError(VT_STATUS.NOT_FOUND, '默认提示词不存在');
  }

  getDatabase()
    .prepare<[string, number, number]>('UPDATE prompts SET use_data = ?, updated_at = ? WHERE id = ?')
    .run('', Date.now(), id);

  return {
    prompt: getPromptItemById(id),
  };
}

export function getEffectivePromptByType(type: BuiltinPromptType): string {
  const row = getDatabase()
    .prepare<[string], PromptRow>('SELECT id, name, type, data, use_data, updated_at FROM prompts WHERE type = ? LIMIT 1')
    .get(type);
  if (!row) {
    throw createError(VT_STATUS.NOT_FOUND, `提示词不存在：${type}`);
  }

  return toPromptItem(row).effectiveData;
}
