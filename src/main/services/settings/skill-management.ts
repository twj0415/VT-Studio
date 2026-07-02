import { createHash } from 'node:crypto';
import { existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import { VT_STATUS } from '@shared/constants/status';
import type {
  SkillEmbeddingStatus,
  SkillFileStatus,
  SkillManagementContentResult,
  SkillManagementGetContentPayload,
  SkillManagementItem,
  SkillManagementListPayload,
  SkillManagementListResult,
  SkillManagementSaveContentPayload,
  SkillManagementSaveContentResult,
  SkillManagementValidationWarning,
  SkillType,
} from '@shared/types/skill-management';
import { getDatabase } from '../database';
import { getRuntimeDirectories, safeJoin } from '../file-system';
import { createError } from '../result';

interface SkillRow {
  id: string;
  md5: string;
  path: string;
  name: string;
  description: string;
  embedding: string;
  type: SkillType;
  created_at: number;
  updated_at: number;
  state: number;
}

interface AttributionRow {
  skill_id: string;
  attribution: string;
}

interface ParsedSkillFrontmatter {
  name?: string;
  description?: string;
}

function normalizeKeyword(keyword?: string): string {
  return keyword?.trim().toLowerCase() ?? '';
}

function resolveSkillPath(relativePath: string): string {
  return safeJoin(getRuntimeDirectories().skills, relativePath);
}

function getSkillRow(id: string): SkillRow {
  if (!id?.trim()) {
    throw createError(VT_STATUS.INVALID_PARAMS, 'Skill ID 不能为空');
  }

  const row = getDatabase()
    .prepare<[string], SkillRow>('SELECT * FROM skill_list WHERE id = ?')
    .get(id);

  if (!row) {
    throw createError(VT_STATUS.NOT_FOUND, 'Skill 不存在');
  }

  resolveSkillPath(row.path);

  return row;
}

function getAttributions(): Map<string, string[]> {
  const rows = getDatabase()
    .prepare<[], AttributionRow>('SELECT skill_id, attribution FROM skill_attributions ORDER BY attribution ASC')
    .all();
  const map = new Map<string, string[]>();

  for (const row of rows) {
    const list = map.get(row.skill_id) ?? [];
    list.push(row.attribution);
    map.set(row.skill_id, list);
  }

  return map;
}

function getFileStatus(path: string): SkillFileStatus {
  return existsSync(resolveSkillPath(path)) ? 'ready' : 'missing';
}

function getEmbeddingStatus(row: SkillRow): SkillEmbeddingStatus {
  if (row.type !== 'references') {
    return 'not-applicable';
  }

  return row.state === 1 && row.embedding.trim() ? 'ready' : 'expired';
}

function mapSkill(row: SkillRow, attributions: Map<string, string[]>): SkillManagementItem {
  return {
    id: row.id,
    md5: row.md5,
    path: row.path,
    name: row.name,
    description: row.description,
    type: row.type,
    attributions: attributions.get(row.id) ?? [],
    fileStatus: getFileStatus(row.path),
    embeddingStatus: getEmbeddingStatus(row),
    state: row.state,
    updatedAt: row.updated_at,
  };
}

function skillMatches(item: SkillManagementItem, keyword: string): boolean {
  if (!keyword) {
    return true;
  }

  return [
    item.name,
    item.description,
    item.path,
    item.type,
    ...item.attributions,
  ].some((value) => value.toLowerCase().includes(keyword));
}

function parseFrontmatterLight(content: string): ParsedSkillFrontmatter | null {
  const match = content.match(/^\uFEFF?---[ \t]*\r?\n([\s\S]*?)\r?\n---[ \t]*(?:\r?\n|$)/);
  if (!match?.[1]) {
    return null;
  }

  const result: ParsedSkillFrontmatter = {};
  for (const line of match[1].split(/\r?\n/)) {
    const keyMatch = line.match(/^([A-Za-z0-9_-]+)\s*:\s*(.*)$/);
    if (!keyMatch) {
      continue;
    }

    const key = keyMatch[1].trim();
    const value = (keyMatch[2] ?? '').trim().replace(/^(['"])([\s\S]*)\1$/, '$2');

    if (key === 'name' || key === 'description') {
      result[key] = value;
    }
  }

  return result;
}

function validateSkillContent(row: SkillRow, content: string): SkillManagementValidationWarning[] {
  const warnings: SkillManagementValidationWarning[] = [];

  if (!content.trim()) {
    throw createError(VT_STATUS.INVALID_PARAMS, 'Skill 内容不能为空');
  }

  const frontmatter = parseFrontmatterLight(content);
  if (!frontmatter) {
    warnings.push({ code: 'frontmatter-missing', message: 'Skill 缺少 frontmatter，Agent 可能无法识别名称和描述' });
    return warnings;
  }

  if (!frontmatter.name?.trim()) {
    warnings.push({ code: 'frontmatter-name-missing', message: 'Skill frontmatter 缺少 name' });
  }

  if (!frontmatter.description?.trim()) {
    warnings.push({ code: 'frontmatter-description-missing', message: 'Skill frontmatter 缺少 description' });
  }

  if (row.type === 'references' && !frontmatter.description?.trim()) {
    warnings.push({ code: 'reference-description-empty', message: 'references Skill 缺少 description，会影响后续向量检索质量' });
  }

  return warnings;
}

function createMd5(content: string): string {
  return createHash('md5').update(content).digest('hex');
}

export function getSkillManagementList(payload: SkillManagementListPayload = {}): SkillManagementListResult {
  const rows = getDatabase()
    .prepare<[], SkillRow>('SELECT * FROM skill_list ORDER BY type ASC, name ASC')
    .all();
  const attributions = getAttributions();
  const keyword = normalizeKeyword(payload.keyword);
  const skills = rows
    .map((row) => mapSkill(row, attributions))
    .filter((item) => skillMatches(item, keyword));

  return { skills };
}

export async function getSkillContent(payload: SkillManagementGetContentPayload): Promise<SkillManagementContentResult> {
  const row = getSkillRow(payload.id);
  const filePath = resolveSkillPath(row.path);

  if (!existsSync(filePath)) {
    throw createError(VT_STATUS.FILE_NOT_FOUND, `Skill 文件不存在：${row.path}`);
  }

  return {
    skill: mapSkill(row, getAttributions()),
    content: await readFile(filePath, 'utf-8'),
  };
}

export async function saveSkillContent(payload: SkillManagementSaveContentPayload): Promise<SkillManagementSaveContentResult> {
  const row = getSkillRow(payload.id);
  const filePath = resolveSkillPath(row.path);

  if (!existsSync(filePath)) {
    throw createError(VT_STATUS.FILE_NOT_FOUND, `Skill 文件不存在：${row.path}`);
  }

  const content = payload.content ?? '';
  const warnings = validateSkillContent(row, content);

  if (warnings.length > 0 && !payload.force) {
    return { saved: false, warnings };
  }

  const frontmatter = parseFrontmatterLight(content);
  const now = Date.now();
  const nextName = frontmatter?.name?.trim() || row.name;
  const nextDescription = frontmatter?.description?.trim() || row.description;
  const nextMd5 = createMd5(content);
  const nextState = row.type === 'references' ? -1 : row.state;

  await writeFile(filePath, content, 'utf-8');
  getDatabase()
    .prepare<[string, string, string, number, number, string]>(
      `
      UPDATE skill_list
      SET md5 = ?, name = ?, description = ?, updated_at = ?, state = ?
      WHERE id = ?
      `,
    )
    .run(nextMd5, nextName, nextDescription, now, nextState, row.id);

  return {
    saved: true,
    warnings,
    skill: mapSkill({ ...row, md5: nextMd5, name: nextName, description: nextDescription, updated_at: now, state: nextState }, getAttributions()),
    content,
  };
}
