import { existsSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { basename } from 'node:path';
import { VT_STATUS } from '@shared/constants/status';
import { normalizeUnknownError } from '@shared/errors';
import { cosineSimilarity, embedText, getEmbeddingModelStatus } from '../embedding';
import { getDatabase } from '../database';
import { getRuntimeDirectories, safeJoin } from '../file-system';
import { logger } from '../logger';
import { createError } from '../result';

export interface SkillRecord {
  id: string;
  md5: string;
  path: string;
  name: string;
  description: string;
  embedding: string;
  type: 'main' | 'references';
  created_at: number;
  updated_at: number;
  state: number;
}

export interface ResolvedSkill {
  id: string;
  path: string;
  name: string;
  description: string;
  type: 'main' | 'references';
  similarity?: number;
}

export interface ResolveSkillsInput {
  attribution: string;
  query?: string;
  limit?: number;
}

export interface ResolveSkillsResult {
  mainSkills: ResolvedSkill[];
  referenceSkills: ResolvedSkill[];
  warning?: string;
}

export interface RebuildSkillEmbeddingsResult {
  total: number;
  succeeded: number;
  failed: number;
  failedSkillIds: string[];
}

function normalizeAttribution(attribution: string): string {
  const normalized = attribution.trim();
  if (!normalized || normalized.includes('\\') || normalized.includes('/') || normalized.includes('\0')) {
    throw createError(VT_STATUS.INVALID_PARAMS, 'Skill attribution 无效');
  }

  return normalized.endsWith('.md') ? normalized : `${normalized}.md`;
}

function parseEmbedding(value: string): number[] | null {
  if (!value) {
    return null;
  }

  try {
    const parsed = JSON.parse(value);
    return Array.isArray(parsed) && parsed.every((item) => typeof item === 'number') ? parsed : null;
  } catch {
    return null;
  }
}

function mapSkill(row: SkillRecord, similarity?: number): ResolvedSkill {
  return {
    id: row.id,
    path: row.path,
    name: row.name,
    description: row.description,
    type: row.type,
    ...(similarity === undefined ? {} : { similarity }),
  };
}

export function parseFrontmatter(content: string): { name: string; description: string } {
  const match = content.match(/^\uFEFF?---[ \t]*\r?\n([\s\S]*?)\r?\n---[ \t]*(?:\r?\n|$)/);
  if (!match?.[1]) {
    throw createError(VT_STATUS.SKILL_ERROR, 'Skill 文件缺少 frontmatter');
  }

  const result: Record<string, string> = {};
  for (const line of match[1].split(/\r?\n/)) {
    const keyMatch = line.match(/^([A-Za-z0-9_-]+)\s*:\s*(.*)$/);
    if (!keyMatch) {
      continue;
    }

    const key = keyMatch[1].trim();
    const value = (keyMatch[2] ?? '').trim().replace(/^(['"])([\s\S]*)\1$/, '$2');
    result[key] = value;
  }

  if (!result.name || !result.description) {
    throw createError(VT_STATUS.SKILL_ERROR, 'Skill frontmatter 缺少 name 或 description');
  }

  return { name: result.name, description: result.description };
}

export async function readSkillFile(relativePath: string): Promise<{ path: string; content: string }> {
  const normalizedPath = relativePath.trim();
  if (!normalizedPath) {
    throw createError(VT_STATUS.INVALID_PARAMS, 'Skill 文件路径不能为空');
  }

  const fullPath = safeJoin(getRuntimeDirectories().skills, normalizedPath);
  if (!existsSync(fullPath)) {
    throw createError(VT_STATUS.FILE_NOT_FOUND, `Skill 文件不存在：${normalizedPath}`);
  }

  return {
    path: normalizedPath,
    content: await readFile(fullPath, 'utf-8'),
  };
}

function getMainSkills(attribution: string): SkillRecord[] {
  const attributionName = basename(attribution, '.md');
  return getDatabase()
    .prepare<[string, string], SkillRecord>(
      `
      SELECT * FROM skill_list
      WHERE type = 'main' AND (path = ? OR name = ?)
      ORDER BY name ASC
      `,
    )
    .all(attribution, attributionName);
}

function getReferenceSkills(attribution: string): SkillRecord[] {
  return getDatabase()
    .prepare<[string], SkillRecord>(
      `
      SELECT s.*
      FROM skill_list s
      INNER JOIN skill_attributions a ON a.skill_id = s.id
      WHERE a.attribution = ? AND s.type = 'references'
      ORDER BY s.name ASC
      `,
    )
    .all(attribution);
}

export async function resolveSkillsForAgent(input: ResolveSkillsInput): Promise<ResolveSkillsResult> {
  const attribution = normalizeAttribution(input.attribution);
  const mainSkills = getMainSkills(attribution).map((row) => mapSkill(row));
  const referenceRows = getReferenceSkills(attribution);
  const limit = Math.max(1, input.limit ?? 5);
  const query = input.query?.trim() ?? '';

  if (!query || referenceRows.length === 0) {
    return {
      mainSkills,
      referenceSkills: referenceRows.slice(0, limit).map((row) => mapSkill(row)),
    };
  }

  try {
    const queryEmbedding = await embedText(query);
    const scored = referenceRows
      .map((row) => {
        const embedding = parseEmbedding(row.embedding);
        if (!embedding || embedding.length !== queryEmbedding.length) {
          return null;
        }

        return mapSkill(row, cosineSimilarity(queryEmbedding, embedding));
      })
      .filter((row): row is ResolvedSkill => row !== null && typeof row.similarity === 'number' && Number.isFinite(row.similarity))
      .sort((a, b) => (b.similarity ?? 0) - (a.similarity ?? 0))
      .slice(0, limit);

    if (scored.length > 0) {
      return { mainSkills, referenceSkills: scored };
    }

    return {
      mainSkills,
      referenceSkills: referenceRows.slice(0, limit).map((row) => mapSkill(row)),
      warning: 'Skill 向量为空，已按 attribution 顺序返回',
    };
  } catch (error) {
    const normalized = normalizeUnknownError(error);
    logger.warn('Skill', 'Skill 语义检索已降级为 attribution 顺序');
    logger.detail('Skill', 'Skill 语义检索降级详情', normalized);

    return {
      mainSkills,
      referenceSkills: referenceRows.slice(0, limit).map((row) => mapSkill(row)),
      warning: normalized.message,
    };
  }
}

export async function rebuildSkillEmbeddings(skillId?: string): Promise<RebuildSkillEmbeddingsResult> {
  const status = getEmbeddingModelStatus();
  if (!status.available) {
    throw createError(VT_STATUS.EMBEDDING_MODEL_NOT_FOUND, '本地向量模型文件不存在，无法重建 Skill embedding');
  }

  const rows = skillId
    ? getDatabase()
        .prepare<[string], SkillRecord>('SELECT * FROM skill_list WHERE id = ? AND type = "references"')
        .all(skillId)
    : getDatabase()
        .prepare<[], SkillRecord>('SELECT * FROM skill_list WHERE type = "references" ORDER BY name ASC')
        .all();

  const failedSkillIds: string[] = [];
  let succeeded = 0;

  for (const row of rows) {
    if (!row.description.trim()) {
      failedSkillIds.push(row.id);
      continue;
    }

    try {
      const embedding = await embedText(row.description);
      getDatabase()
        .prepare<[string, number, number, string]>(
          'UPDATE skill_list SET embedding = ?, state = ?, updated_at = ? WHERE id = ?',
        )
        .run(JSON.stringify(embedding), 1, Date.now(), row.id);
      succeeded += 1;
    } catch (error) {
      failedSkillIds.push(row.id);
      logger.warn('Skill', `Skill embedding 生成失败：${row.name}`);
      logger.detail('Skill', 'Skill embedding 生成失败详情', {
        skillId: row.id,
        skillName: row.name,
        error: normalizeUnknownError(error),
      });
    }
  }

  return {
    total: rows.length,
    succeeded,
    failed: failedSkillIds.length,
    failedSkillIds,
  };
}
