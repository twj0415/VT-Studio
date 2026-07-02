import { VT_STATUS } from '@shared/constants/status';
import type { AgentNamespace } from '@shared/types/socket';
import { getDatabase } from '../database';
import type { AgentSocketData } from './types';

function getTokenKey(): string {
  const row = getDatabase()
    .prepare<[], { value: string }>("SELECT value FROM app_settings WHERE key = 'tokenKey' LIMIT 1")
    .get();

  if (!row?.value) {
    throw new Error('tokenKey 未初始化');
  }

  return row.value;
}

function readAuthString(auth: Record<string, unknown>, key: string): string | null {
  const value = auth[key];
  if (typeof value === 'string' && value.trim()) {
    return value.trim();
  }
  if (typeof value === 'number') {
    return String(value);
  }
  return null;
}

export function authenticateAgentSocket(auth: unknown, namespace: AgentNamespace): AgentSocketData {
  if (!auth || typeof auth !== 'object') {
    throw new Error('Socket 鉴权参数缺失');
  }

  const authRecord = auth as Record<string, unknown>;
  const token = readAuthString(authRecord, 'token');
  const isolationKey = readAuthString(authRecord, 'isolationKey');
  const projectId = readAuthString(authRecord, 'projectId');
  const scriptId = readAuthString(authRecord, 'scriptId');

  if (!token || token !== getTokenKey()) {
    throw Object.assign(new Error('Socket token 无效'), { code: VT_STATUS.UNAUTHORIZED });
  }
  if (!isolationKey) {
    throw new Error('isolationKey 缺失');
  }
  if (!projectId) {
    throw new Error('projectId 缺失');
  }
  if (namespace === 'productionAgent' && !scriptId) {
    throw new Error('scriptId 缺失');
  }

  return { token, isolationKey, projectId, scriptId: scriptId ?? undefined };
}
