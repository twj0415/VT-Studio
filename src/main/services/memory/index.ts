import { randomUUID } from 'node:crypto';
import { VT_STATUS } from '@shared/constants/status';
import { normalizeUnknownError } from '@shared/errors';
import type {
  AddMemoryInput,
  ClearMemoryInput,
  ClearMemoryResult,
  DeepRetrieveResult,
  HistoryMessage,
  MemoryContextResult,
  MemoryIsolationInput,
  MemoryItem,
  MemoryMetadata,
  MemoryRagItem,
  MemoryRole,
  MemorySummaryItem,
} from '@shared/types/memory';
import type { AgentNamespace } from '@shared/types/socket';
import { cosineSimilarity, embedText } from '../embedding';
import { getDatabase, withTransaction } from '../database';
import { logger } from '../logger';
import { createError } from '../result';

const MEMORY_DEFAULTS = {
  messagesPerSummary: 10,
  summaryMaxLength: 500,
  shortTermLimit: 5,
  summaryLimit: 10,
  ragLimit: 3,
  deepRetrieveSummaryLimit: 5,
};

interface MemoryRow {
  id: string;
  isolation_key: string;
  type: 'message' | 'summary';
  role: string | null;
  name: string | null;
  content: string;
  embedding: string;
  metadata: string | null;
  related_message_ids: string | null;
  summarized: number;
  created_at: number;
}

function readNumberSetting(key: keyof typeof MEMORY_DEFAULTS): number {
  const row = getDatabase().prepare<[string], { value: string }>('SELECT value FROM app_settings WHERE key = ? LIMIT 1').get(key);
  const value = Number(row?.value);

  return Number.isFinite(value) ? value : MEMORY_DEFAULTS[key];
}

function parseJsonArray(value: string | null): string[] {
  if (!value) {
    return [];
  }

  try {
    const parsed = JSON.parse(value);
    return Array.isArray(parsed) ? parsed.map(String) : [];
  } catch {
    return [];
  }
}

function parseMetadata(value: string | null): MemoryMetadata | null {
  if (!value) {
    return null;
  }

  try {
    const parsed = JSON.parse(value);
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? (parsed as MemoryMetadata) : null;
  } catch {
    return null;
  }
}

function stringifyMetadata(metadata: MemoryMetadata | null | undefined): string | null {
  return metadata ? JSON.stringify(metadata) : null;
}

function parseEmbedding(row: Pick<MemoryRow, 'id' | 'embedding'>): number[] | null {
  if (!row.embedding) {
    return null;
  }

  try {
    const parsed = JSON.parse(row.embedding);
    if (!Array.isArray(parsed) || parsed.some((value) => typeof value !== 'number')) {
      return null;
    }

    return parsed;
  } catch {
    logger.warn('记忆功能', `记忆向量解析失败，已跳过：${row.id}`);
    return null;
  }
}

function normalizeHistoryRole(role?: string | null): 'user' | 'assistant' {
  return role?.startsWith('assistant') ? 'assistant' : 'user';
}

function mapMessageRow(row: MemoryRow): MemoryItem {
  return {
    id: row.id,
    role: row.role as MemoryRole | null,
    name: row.name,
    content: row.content,
    metadata: parseMetadata(row.metadata),
    createdAt: row.created_at,
  };
}

function mapSummaryRow(row: MemoryRow): MemorySummaryItem {
  return {
    id: row.id,
    content: row.content,
    relatedMessageIds: parseJsonArray(row.related_message_ids),
    metadata: parseMetadata(row.metadata),
    createdAt: row.created_at,
  };
}

function assertAgentType(value: AgentNamespace): AgentNamespace {
  if (value !== 'scriptAgent' && value !== 'productionAgent') {
    throw createError(VT_STATUS.INVALID_PARAMS, 'Agent 类型无效');
  }

  return value;
}

function readRequiredId(value: number | string | null | undefined, label: string): string {
  const text = String(value ?? '').trim();
  if (!text) {
    throw createError(VT_STATUS.INVALID_PARAMS, `${label}不能为空`);
  }

  return text;
}

export function createMemoryIsolationKey(input: MemoryIsolationInput): string {
  const projectId = readRequiredId(input.projectId, '项目 ID');
  const agentType = assertAgentType(input.agentType);

  if (agentType === 'scriptAgent') {
    return `${projectId}:scriptAgent`;
  }

  const episodesId = readRequiredId(input.episodesId, '生产 Agent 分集 ID');
  return `${projectId}:productionAgent:${episodesId}`;
}

function resolveIsolationKey(input: AddMemoryInput | ClearMemoryInput): string {
  if ('isolationKey' in input && typeof input.isolationKey === 'string' && input.isolationKey.trim()) {
    return input.isolationKey.trim();
  }

  return createMemoryIsolationKey({
    projectId: input.projectId!,
    agentType: input.agentType!,
    episodesId: input.episodesId,
  });
}

function vectorSearch(rows: MemoryRow[], queryEmbedding: number[], limit: number): MemoryRagItem[] {
  return rows
    .map((row) => {
      const embedding = parseEmbedding(row);
      if (!embedding || embedding.length !== queryEmbedding.length) {
        return null;
      }

      return {
        id: row.id,
        content: row.content,
        similarity: cosineSimilarity(queryEmbedding, embedding),
        metadata: parseMetadata(row.metadata),
        createdAt: row.created_at,
      };
    })
    .filter((row): row is MemoryRagItem => row !== null && Number.isFinite(row.similarity))
    .sort((a, b) => b.similarity - a.similarity)
    .slice(0, limit);
}

async function generateSummary(agentType: AgentNamespace, contents: string[]): Promise<string> {
  const summaryMaxLength = Math.max(0, readNumberSetting('summaryMaxLength'));
  const { invokeText } = await import('../model');
  const result = await invokeText({
    modelKey: agentType,
    system: `你是一个记忆压缩助手。请将以下多条记忆内容压缩为一段简洁摘要，不超过${summaryMaxLength}个字符。只输出摘要内容，不要加前缀或解释。`,
    messages: [{ role: 'user', content: contents.map((content, index) => `${index + 1}. ${content}`).join('\n') }],
  });

  return result.text.slice(0, summaryMaxLength);
}

async function triggerSummaryIfNeeded(agentType: AgentNamespace, isolationKey: string): Promise<void> {
  const messagesPerSummary = Math.max(1, readNumberSetting('messagesPerSummary'));
  const unsummarized = getDatabase()
    .prepare<[string], MemoryRow>(
      'SELECT * FROM memories WHERE isolation_key = ? AND type = "message" AND summarized = 0 ORDER BY created_at ASC',
    )
    .all(isolationKey);

  if (unsummarized.length < messagesPerSummary) {
    return;
  }

  const batch = unsummarized.slice(0, messagesPerSummary);
  const batchIds = batch.map((row) => row.id);
  const summaryContent = await generateSummary(agentType, batch.map((row) => row.content));
  const summaryEmbedding = await embedText(summaryContent);
  const summaryId = randomUUID();
  const now = Date.now();

  withTransaction((database) => {
    database
      .prepare<[string, string, string, string, string, number, number]>(
        `
        INSERT INTO memories
          (id, isolation_key, type, content, embedding, related_message_ids, summarized, created_at)
        VALUES (?, ?, 'summary', ?, ?, ?, ?, ?)
        `,
      )
      .run(summaryId, isolationKey, summaryContent, JSON.stringify(summaryEmbedding), JSON.stringify(batchIds), 0, now);

    const updateStmt = database.prepare<[string]>('UPDATE memories SET summarized = 1 WHERE id = ?');
    for (const id of batchIds) {
      updateStmt.run(id);
    }
  });
}

export async function addMemory(input: AddMemoryInput): Promise<{ id: string }> {
  const content = input.content.trim();
  if (!content) {
    throw createError(VT_STATUS.INVALID_PARAMS, '记忆内容不能为空');
  }

  const agentType = assertAgentType(input.agentType ?? 'scriptAgent');
  const isolationKey = resolveIsolationKey(input);
  const embedding = await embedText(content);
  const id = randomUUID();
  const createdAt = input.createdAt ?? Date.now();
  const role = input.role ?? 'user';

  getDatabase()
    .prepare<[string, string, string, string | null, string, string, string | null, number, number]>(
      `
      INSERT INTO memories
        (id, isolation_key, type, role, name, content, embedding, metadata, summarized, created_at)
      VALUES (?, ?, 'message', ?, ?, ?, ?, ?, ?, ?)
      `,
    )
    .run(
      id,
      isolationKey,
      role,
      input.name ?? null,
      content,
      JSON.stringify(embedding),
      stringifyMetadata(input.metadata),
      0,
      createdAt,
    );

  try {
    await triggerSummaryIfNeeded(agentType, isolationKey);
  } catch (error) {
    logger.warn('记忆功能', '自动摘要失败，已保留原始消息');
    logger.detail('记忆功能', '自动摘要失败详情', normalizeUnknownError(error));
  }

  return { id };
}

export async function getMemoryContext(input: {
  isolationKey: string;
  query: string;
}): Promise<MemoryContextResult> {
  const shortTermLimit = Math.max(0, readNumberSetting('shortTermLimit'));
  const summaryLimit = Math.max(0, readNumberSetting('summaryLimit'));
  const ragLimit = Math.max(0, readNumberSetting('ragLimit'));

  const shortTerm = getDatabase()
    .prepare<[string, number], MemoryRow>(
      `
      SELECT * FROM memories
      WHERE isolation_key = ? AND type = 'message' AND summarized = 0
      ORDER BY created_at DESC
      LIMIT ?
      `,
    )
    .all(input.isolationKey, shortTermLimit)
    .reverse()
    .map(mapMessageRow);

  const summaries = getDatabase()
    .prepare<[string, number], MemoryRow>(
      `
      SELECT * FROM memories
      WHERE isolation_key = ? AND type = 'summary'
      ORDER BY created_at DESC
      LIMIT ?
      `,
    )
    .all(input.isolationKey, summaryLimit)
    .reverse()
    .map(mapSummaryRow);

  if (ragLimit === 0 || !input.query.trim()) {
    return { shortTerm, summaries, rag: [] };
  }

  try {
    const queryEmbedding = await embedText(input.query);
    const messages = getDatabase()
      .prepare<[string], MemoryRow>("SELECT * FROM memories WHERE isolation_key = ? AND type = 'message'")
      .all(input.isolationKey);

    return {
      shortTerm,
      summaries,
      rag: vectorSearch(messages, queryEmbedding, ragLimit),
    };
  } catch (error) {
    const normalized = normalizeUnknownError(error);
    logger.warn('记忆功能', 'RAG 检索已降级为空');
    logger.detail('记忆功能', 'RAG 检索降级详情', normalized);

    return {
      shortTerm,
      summaries,
      rag: [],
      warning: normalized.message,
    };
  }
}

export function getMemoryHistory(input: MemoryIsolationInput): HistoryMessage[] {
  const isolationKey = createMemoryIsolationKey(input);
  const rows = getDatabase()
    .prepare<[string], Pick<MemoryRow, 'id' | 'role' | 'name' | 'content' | 'created_at'>>(
      `
      SELECT id, role, name, content, created_at
      FROM memories
      WHERE isolation_key = ? AND type = 'message'
      ORDER BY created_at ASC
      `,
    )
    .all(isolationKey);

  return rows.map((row) => ({
    id: row.id,
    role: normalizeHistoryRole(row.role),
    name: row.name ?? undefined,
    status: 'complete',
    datetime: new Date(row.created_at).toISOString(),
    content: [{ type: 'markdown', status: 'complete', data: row.content }],
    createTime: row.created_at,
  }));
}

export function clearMemory(input: ClearMemoryInput): ClearMemoryResult {
  const isolationKey = createMemoryIsolationKey(input);
  const type = input.type ?? 'all';

  return withTransaction((database) => {
    if (type === 'all') {
      const deleted = database.prepare<[string]>('DELETE FROM memories WHERE isolation_key = ?').run(isolationKey).changes;
      return { deleted, updated: 0 };
    }

    if (type === 'message') {
      const deletedMessages = database
        .prepare<[string]>("DELETE FROM memories WHERE isolation_key = ? AND type = 'message'")
        .run(isolationKey).changes;
      const deletedSummaries = database
        .prepare<[string]>("DELETE FROM memories WHERE isolation_key = ? AND type = 'summary'")
        .run(isolationKey).changes;
      return { deleted: deletedMessages + deletedSummaries, updated: 0 };
    }

    const updated = database
      .prepare<[string]>("UPDATE memories SET summarized = 0 WHERE isolation_key = ? AND type = 'message' AND summarized = 1")
      .run(isolationKey).changes;
    const deleted = database.prepare<[string]>("DELETE FROM memories WHERE isolation_key = ? AND type = 'summary'").run(isolationKey).changes;
    return { deleted, updated };
  });
}

async function judgeSummaryRelevance(
  agentType: AgentNamespace,
  keyword: string,
  summaries: Array<{ id: string; content: string }>,
): Promise<string[]> {
  const list = summaries.map((summary) => `[${summary.id}] ${summary.content}`).join('\n');
  const { invokeText } = await import('../model');
  const result = await invokeText({
    modelKey: agentType,
    system:
      '你是一个信息检索助手。用户会给你一个关键词和一组摘要，请判断哪些摘要可能包含与关键词相关的详细信息。只返回相关摘要 id 的 JSON 数组，不要解释。',
    messages: [{ role: 'user', content: `关键词: ${keyword}\n\n摘要列表:\n${list}` }],
  });

  try {
    const parsed = JSON.parse(result.text);
    return Array.isArray(parsed) ? parsed.map(String) : [];
  } catch {
    return [];
  }
}

export async function deepRetrieve(input: {
  isolationKey: string;
  agentType: AgentNamespace;
  keyword: string;
}): Promise<DeepRetrieveResult[]> {
  const keyword = input.keyword.trim();
  if (!keyword) {
    return [];
  }

  const allSummaries = getDatabase()
    .prepare<[string], MemoryRow>("SELECT * FROM memories WHERE isolation_key = ? AND type = 'summary'")
    .all(input.isolationKey);

  if (allSummaries.length === 0) {
    return [];
  }

  try {
    const limit = Math.max(0, readNumberSetting('deepRetrieveSummaryLimit'));
    const queryEmbedding = await embedText(keyword);
    const topSummaries = vectorSearch(allSummaries, queryEmbedding, limit);
    const relevantIds = await judgeSummaryRelevance(
      input.agentType,
      keyword,
      topSummaries.map((summary) => ({ id: summary.id, content: summary.content })),
    );

    if (relevantIds.length === 0) {
      return [];
    }

    const messageIds = topSummaries
      .filter((summary) => relevantIds.includes(summary.id))
      .flatMap((summary) => {
        const row = allSummaries.find((item) => item.id === summary.id);
        return parseJsonArray(row?.related_message_ids ?? null);
      });

    if (messageIds.length === 0) {
      return [];
    }

    const placeholders = messageIds.map(() => '?').join(',');
    const rows = getDatabase()
      .prepare<string[], MemoryRow>(`SELECT * FROM memories WHERE id IN (${placeholders}) ORDER BY created_at ASC`)
      .all(...messageIds);

    return rows.map((row) => ({
      id: row.id,
      content: row.content,
      metadata: parseMetadata(row.metadata),
      createdAt: row.created_at,
    }));
  } catch (error) {
    logger.warn('记忆功能', '深度检索失败，已降级为空');
    logger.detail('记忆功能', '深度检索失败详情', normalizeUnknownError(error));
    return [];
  }
}
