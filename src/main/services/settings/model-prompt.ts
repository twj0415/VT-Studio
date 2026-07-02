import { VT_STATUS } from '@shared/constants/status';
import type {
  ModelPromptBindPayload,
  ModelPromptBindResult,
  ModelPromptBinding,
  ModelPromptClearBindingPayload,
  ModelPromptClearBindingResult,
  ModelPromptConfigResult,
  ModelPromptConnectionGroup,
  ModelPromptInvalidMapping,
  ModelPromptModelItem,
  ModelPromptModelType,
  ModelPromptTemplate,
  ModelPromptTemplateDeletePayload,
  ModelPromptTemplateDeleteResult,
  ModelPromptTemplateSavePayload,
  ModelPromptTemplateSaveResult,
  ModelPromptTemplateType,
} from '@shared/types/model-prompt';
import type { ApiConnection, RegisteredModel } from '@shared/types/model-config';
import { getDatabase } from '../database';
import { createError } from '../result';
import { getResourceConfig } from './model-config';

interface TemplateRow {
  id: number;
  name: string;
  type: string;
  content: string;
  is_builtin: number;
  created_at: number;
  updated_at: number;
  reference_count?: number;
}

interface MappingRow {
  id: number;
  connection_id: string;
  model_name: string;
  model_type: string;
  model_mode: string;
  template_id: number;
  created_at: number;
  updated_at: number;
}

const MODEL_TYPES: ModelPromptModelType[] = ['image', 'video'];
const TEMPLATE_TYPES: ModelPromptTemplateType[] = ['imagePrompt', 'videoPrompt'];

function isModelPromptModelType(value: string): value is ModelPromptModelType {
  return MODEL_TYPES.includes(value as ModelPromptModelType);
}

function isModelPromptTemplateType(value: string): value is ModelPromptTemplateType {
  return TEMPLATE_TYPES.includes(value as ModelPromptTemplateType);
}

function templateTypeForModel(modelType: ModelPromptModelType): ModelPromptTemplateType {
  return modelType === 'image' ? 'imagePrompt' : 'videoPrompt';
}

function modelTypeForTemplate(templateType: ModelPromptTemplateType): ModelPromptModelType {
  return templateType === 'imagePrompt' ? 'image' : 'video';
}

function normalizeText(value: string | null | undefined): string {
  return (value ?? '').replace(/\r\n/g, '\n').trim();
}

function normalizeMode(value: string | null | undefined): string {
  return normalizeText(value);
}

function toTemplate(row: TemplateRow): ModelPromptTemplate {
  if (!isModelPromptTemplateType(row.type)) {
    throw createError(VT_STATUS.DATABASE_ERROR, `模型模板类型无效：${row.type}`);
  }

  return {
    id: row.id,
    name: row.name,
    type: row.type,
    content: row.content,
    isBuiltin: row.is_builtin === 1,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
    referenceCount: row.reference_count ?? 0,
  };
}

function toBinding(row: MappingRow, template: ModelPromptTemplate): ModelPromptBinding {
  if (!isModelPromptModelType(row.model_type)) {
    throw createError(VT_STATUS.DATABASE_ERROR, `模型类型无效：${row.model_type}`);
  }

  return {
    id: row.id,
    connectionId: row.connection_id,
    modelName: row.model_name,
    modelType: row.model_type,
    modelMode: row.model_mode,
    templateId: row.template_id,
    templateName: template.name,
    templateType: template.type,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function makeMappingKey(connectionId: string, modelName: string, modelType: string, modelMode = ''): string {
  return `${connectionId}\n${modelName}\n${modelType}\n${modelMode}`;
}

function getTemplates(): ModelPromptTemplate[] {
  const rows = getDatabase()
    .prepare<[], TemplateRow>(
      `
      SELECT
        t.id,
        t.name,
        t.type,
        t.content,
        t.is_builtin,
        t.created_at,
        t.updated_at,
        COUNT(m.id) as reference_count
      FROM model_prompt_templates t
      LEFT JOIN model_prompt_mappings m ON m.template_id = t.id
      GROUP BY t.id
      ORDER BY t.type ASC, t.id ASC
      `,
    )
    .all();

  return rows.map(toTemplate);
}

function getMappings(): MappingRow[] {
  return getDatabase()
    .prepare<[], MappingRow>('SELECT id, connection_id, model_name, model_type, model_mode, template_id, created_at, updated_at FROM model_prompt_mappings ORDER BY id ASC')
    .all();
}

function getTemplateById(id: number): ModelPromptTemplate | null {
  const row = getDatabase()
    .prepare<[number], TemplateRow>(
      `
      SELECT
        t.id,
        t.name,
        t.type,
        t.content,
        t.is_builtin,
        t.created_at,
        t.updated_at,
        COUNT(m.id) as reference_count
      FROM model_prompt_templates t
      LEFT JOIN model_prompt_mappings m ON m.template_id = t.id
      WHERE t.id = ?
      GROUP BY t.id
      LIMIT 1
      `,
    )
    .get(id);

  return row ? toTemplate(row) : null;
}

function requireTemplate(id: number): ModelPromptTemplate {
  const template = getTemplateById(id);
  if (!template) {
    throw createError(VT_STATUS.NOT_FOUND, '模型提示词模板不存在');
  }

  return template;
}

function assertTemplateNameAvailable(name: string, type: ModelPromptTemplateType, excludeId?: number): void {
  const row = getDatabase()
    .prepare<[string, string, number], { id: number }>(
      'SELECT id FROM model_prompt_templates WHERE type = ? AND lower(name) = lower(?) AND id != ? LIMIT 1',
    )
    .get(type, name, excludeId ?? 0);

  if (row) {
    throw createError(VT_STATUS.CONFLICT, '同类型模板名称已存在');
  }
}

function validateTemplatePayload(payload: ModelPromptTemplateSavePayload): { id: number | null; name: string; type: ModelPromptTemplateType; content: string } {
  const id = payload.id === undefined ? null : Number(payload.id);
  if (id !== null && (!Number.isInteger(id) || id <= 0)) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模板 id 无效');
  }

  const name = normalizeText(payload.name);
  if (!name) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模板名称不能为空');
  }

  if (!isModelPromptTemplateType(payload.type)) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模板类型只支持 imagePrompt 或 videoPrompt');
  }

  const content = normalizeText(payload.content);
  if (!content) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模板内容不能为空');
  }

  return { id, name, type: payload.type, content };
}

function getReferences(templateId: number): MappingRow[] {
  return getDatabase()
    .prepare<[number], MappingRow>('SELECT id, connection_id, model_name, model_type, model_mode, template_id, created_at, updated_at FROM model_prompt_mappings WHERE template_id = ? ORDER BY id ASC')
    .all(templateId);
}

function formatReferenceList(rows: MappingRow[]): string {
  return rows
    .slice(0, 3)
    .map((row) => `${row.connection_id}/${row.model_name}${row.model_mode ? `/${row.model_mode}` : ''}`)
    .join('、');
}

function getModel(connectionId: string, modelName: string, modelType: ModelPromptModelType): { connection: ApiConnection; model: RegisteredModel } {
  const resource = getResourceConfig();
  const connection = resource.connections.find((item) => item.id === connectionId);
  if (!connection) {
    throw createError(VT_STATUS.NOT_FOUND, '模型连接不存在');
  }

  const model = connection.models.find((item) => item.modelName === modelName && item.type === modelType);
  if (!model) {
    throw createError(VT_STATUS.MODEL_NOT_FOUND, '模型不存在或类型不匹配');
  }

  return { connection, model };
}

function buildModelItem(connection: ApiConnection, model: RegisteredModel, mappings: Map<string, MappingRow>, templates: Map<number, ModelPromptTemplate>): ModelPromptModelItem {
  const modelType = model.type as ModelPromptModelType;
  const modelMode = '';
  const mapping = mappings.get(makeMappingKey(connection.id, model.modelName, modelType, modelMode));

  if (!mapping) {
    return {
      connectionId: connection.id,
      connectionName: connection.name,
      modelName: model.modelName,
      modelDisplayName: model.displayName,
      modelType,
      modelMode,
      binding: null,
      status: 'fallback',
      statusText: modelType === 'video' ? '使用视频默认提示词 fallback' : '未绑定专用模板',
    };
  }

  const template = templates.get(mapping.template_id);
  if (!template) {
    return {
      connectionId: connection.id,
      connectionName: connection.name,
      modelName: model.modelName,
      modelDisplayName: model.displayName,
      modelType,
      modelMode,
      binding: null,
      status: 'invalid-template',
      statusText: '绑定模板不存在',
    };
  }

  if (modelTypeForTemplate(template.type) !== modelType) {
    return {
      connectionId: connection.id,
      connectionName: connection.name,
      modelName: model.modelName,
      modelDisplayName: model.displayName,
      modelType,
      modelMode,
      binding: toBinding(mapping, template),
      status: 'type-mismatch',
      statusText: '模板类型不匹配',
    };
  }

  return {
    connectionId: connection.id,
    connectionName: connection.name,
    modelName: model.modelName,
    modelDisplayName: model.displayName,
    modelType,
    modelMode,
    binding: toBinding(mapping, template),
    status: 'bound',
    statusText: '已绑定专用模板',
  };
}

function buildInvalidMappings(mappings: MappingRow[], templates: Map<number, ModelPromptTemplate>, currentKeys: Set<string>): ModelPromptInvalidMapping[] {
  const invalid: ModelPromptInvalidMapping[] = [];

  for (const mapping of mappings) {
    const template = templates.get(mapping.template_id);
    const key = makeMappingKey(mapping.connection_id, mapping.model_name, mapping.model_type, mapping.model_mode);
    if (!isModelPromptModelType(mapping.model_type)) {
      invalid.push({
        id: mapping.id,
        connectionId: mapping.connection_id,
        modelName: mapping.model_name,
        modelType: 'image',
        modelMode: mapping.model_mode,
        templateId: mapping.template_id,
        templateName: template?.name ?? '模板不存在',
        reason: 'type-mismatch',
        reasonText: '映射模型类型无效',
      });
      continue;
    }

    if (!currentKeys.has(key)) {
      invalid.push({
        id: mapping.id,
        connectionId: mapping.connection_id,
        modelName: mapping.model_name,
        modelType: mapping.model_type,
        modelMode: mapping.model_mode,
        templateId: mapping.template_id,
        templateName: template?.name ?? '模板不存在',
        reason: 'model-missing',
        reasonText: '模型或连接不存在',
      });
      continue;
    }

    if (!template) {
      invalid.push({
        id: mapping.id,
        connectionId: mapping.connection_id,
        modelName: mapping.model_name,
        modelType: mapping.model_type,
        modelMode: mapping.model_mode,
        templateId: mapping.template_id,
        templateName: '模板不存在',
        reason: 'template-missing',
        reasonText: '绑定模板不存在',
      });
      continue;
    }

    if (modelTypeForTemplate(template.type) !== mapping.model_type) {
      invalid.push({
        id: mapping.id,
        connectionId: mapping.connection_id,
        modelName: mapping.model_name,
        modelType: mapping.model_type,
        modelMode: mapping.model_mode,
        templateId: mapping.template_id,
        templateName: template.name,
        reason: 'type-mismatch',
        reasonText: '模板类型和模型类型不匹配',
      });
    }
  }

  return invalid;
}

export function getModelPromptConfig(): ModelPromptConfigResult {
  const templates = getTemplates();
  const mappings = getMappings();
  const templateMap = new Map(templates.map((template) => [template.id, template]));
  const mappingMap = new Map(mappings.map((mapping) => [makeMappingKey(mapping.connection_id, mapping.model_name, mapping.model_type, mapping.model_mode), mapping]));
  const currentKeys = new Set<string>();
  const connections: ModelPromptConnectionGroup[] = [];

  for (const connection of getResourceConfig().connections) {
    const models = connection.models
      .filter((model) => isModelPromptModelType(model.type))
      .map((model) => {
        currentKeys.add(makeMappingKey(connection.id, model.modelName, model.type));
        return buildModelItem(connection, model, mappingMap, templateMap);
      });

    if (models.length === 0) {
      continue;
    }

    connections.push({
      connectionId: connection.id,
      connectionName: connection.name,
      connectionStatus: connection.status,
      connectionStatusText: connection.statusText,
      models,
    });
  }

  return {
    templates,
    connections,
    invalidMappings: buildInvalidMappings(mappings, templateMap, currentKeys),
  };
}

export function saveModelPromptTemplate(payload: ModelPromptTemplateSavePayload): ModelPromptTemplateSaveResult {
  const draft = validateTemplatePayload(payload);
  assertTemplateNameAvailable(draft.name, draft.type, draft.id ?? undefined);
  const now = Date.now();

  if (draft.id === null) {
    const result = getDatabase()
      .prepare<[string, string, string, number, number, number]>(
        'INSERT INTO model_prompt_templates (name, type, content, is_builtin, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)',
      )
      .run(draft.name, draft.type, draft.content, 0, now, now);

    return { template: requireTemplate(Number(result.lastInsertRowid)) };
  }

  const current = requireTemplate(draft.id);
  if (current.isBuiltin) {
    throw createError(VT_STATUS.FORBIDDEN, '内置模型模板不能编辑');
  }

  const references = getReferences(current.id);
  if (references.length > 0 && current.type !== draft.type) {
    throw createError(VT_STATUS.CONFLICT, '模板已被模型引用，不能修改类型');
  }

  getDatabase()
    .prepare<[string, string, string, number, number]>('UPDATE model_prompt_templates SET name = ?, type = ?, content = ?, updated_at = ? WHERE id = ?')
    .run(draft.name, draft.type, draft.content, now, draft.id);

  return { template: requireTemplate(draft.id) };
}

export function deleteModelPromptTemplate(payload: ModelPromptTemplateDeletePayload): ModelPromptTemplateDeleteResult {
  const id = Number(payload.id);
  if (!Number.isInteger(id) || id <= 0) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模板 id 无效');
  }

  const template = requireTemplate(id);
  if (template.isBuiltin) {
    throw createError(VT_STATUS.FORBIDDEN, '内置模型模板不能删除');
  }

  const references = getReferences(id);
  if (references.length > 0) {
    throw createError(VT_STATUS.CONFLICT, `模板正在被 ${formatReferenceList(references)} 引用，请先清除绑定`);
  }

  getDatabase().prepare<[number]>('DELETE FROM model_prompt_templates WHERE id = ?').run(id);
  return { templateId: id };
}

export function bindModelPromptTemplate(payload: ModelPromptBindPayload): ModelPromptBindResult {
  if (!isModelPromptModelType(payload.modelType)) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模型类型只支持 image/video');
  }

  const connectionId = normalizeText(payload.connectionId);
  const modelName = normalizeText(payload.modelName);
  const modelMode = normalizeMode(payload.modelMode);
  if (!connectionId || !modelName) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模型连接和模型名称不能为空');
  }

  getModel(connectionId, modelName, payload.modelType);
  const template = requireTemplate(Number(payload.templateId));
  if (template.type !== templateTypeForModel(payload.modelType)) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模板类型和模型类型不匹配');
  }

  const now = Date.now();
  getDatabase()
    .prepare<[string, string, string, string, number, number, number]>(
      `
      INSERT INTO model_prompt_mappings
        (connection_id, model_name, model_type, model_mode, template_id, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(connection_id, model_name, model_type, model_mode)
      DO UPDATE SET template_id = excluded.template_id, updated_at = excluded.updated_at
      `,
    )
    .run(connectionId, modelName, payload.modelType, modelMode, template.id, now, now);

  const row = getDatabase()
    .prepare<[string, string, string, string], MappingRow>(
      'SELECT id, connection_id, model_name, model_type, model_mode, template_id, created_at, updated_at FROM model_prompt_mappings WHERE connection_id = ? AND model_name = ? AND model_type = ? AND model_mode = ? LIMIT 1',
    )
    .get(connectionId, modelName, payload.modelType, modelMode);
  if (!row) {
    throw createError(VT_STATUS.DATABASE_ERROR, '模型模板绑定保存失败');
  }

  return { binding: toBinding(row, template) };
}

export function clearModelPromptBinding(payload: ModelPromptClearBindingPayload): ModelPromptClearBindingResult {
  if (!isModelPromptModelType(payload.modelType)) {
    throw createError(VT_STATUS.INVALID_PARAMS, '模型类型只支持 image/video');
  }

  const result = getDatabase()
    .prepare<[string, string, string, string]>(
      'DELETE FROM model_prompt_mappings WHERE connection_id = ? AND model_name = ? AND model_type = ? AND model_mode = ?',
    )
    .run(normalizeText(payload.connectionId), normalizeText(payload.modelName), payload.modelType, normalizeMode(payload.modelMode));

  return { cleared: result.changes > 0 };
}
