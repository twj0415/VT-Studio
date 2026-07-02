import { VT_STATUS } from '@shared/constants/status';
import type {
  AuthCurrentUserPayload,
  AuthLoginPayload,
  AuthLoginResult,
  AuthUpdateLocalUserPayload,
  AuthUpdateLocalUserResult,
  AuthUser,
  AuthValidateSessionPayload,
  AuthValidateSessionResult,
} from '@shared/types/auth';
import { getDatabase } from '../database';
import { createError } from '../result';

interface UserRow {
  id: number;
  name: string;
  password: string;
}

function readTrimmedString(value: unknown): string {
  return typeof value === 'string' ? value.trim() : '';
}

function readPositiveInteger(value: unknown): number | null {
  if (typeof value !== 'number' || !Number.isInteger(value) || value <= 0) {
    return null;
  }

  return value;
}

function getTokenKey(): string {
  const row = getDatabase()
    .prepare<[], { value: string }>("SELECT value FROM app_settings WHERE key = 'tokenKey' LIMIT 1")
    .get();

  if (!row?.value) {
    throw createError(VT_STATUS.DATABASE_ERROR, '本地登录密钥未初始化，请重新初始化默认数据');
  }

  return row.value;
}

function toAuthUser(row: Pick<UserRow, 'id' | 'name'>): AuthUser {
  return {
    id: row.id,
    name: row.name,
  };
}

function getDefaultUser(): UserRow {
  const row = getDatabase()
    .prepare<[], UserRow>('SELECT id, name, password FROM users ORDER BY id ASC LIMIT 1')
    .get();

  if (!row) {
    throw createError(VT_STATUS.NOT_FOUND, '本地默认用户缺失，请重新初始化默认数据');
  }

  return row;
}

function getUserById(id: number): UserRow {
  const row = getDatabase()
    .prepare<[number], UserRow>('SELECT id, name, password FROM users WHERE id = ? LIMIT 1')
    .get(id);

  if (!row) {
    throw createError(VT_STATUS.NOT_FOUND, '本地用户不存在');
  }

  return row;
}

function assertSession(payload?: AuthCurrentUserPayload | AuthValidateSessionPayload): UserRow {
  const token = readTrimmedString(payload?.token);
  const userId = readPositiveInteger(payload?.userId);

  if (!token || token !== getTokenKey() || !userId) {
    throw createError(VT_STATUS.UNAUTHORIZED, '登录已失效，请重新登录');
  }

  return getUserById(userId);
}

function validateUserForm(name: unknown, password: unknown): { name: string; password: string } {
  const normalizedName = readTrimmedString(name);
  const normalizedPassword = readTrimmedString(password);

  if (normalizedName.length < 2 || normalizedName.length > 20) {
    throw createError(VT_STATUS.INVALID_PARAMS, '用户名长度必须为 2-20 个字符');
  }

  if (normalizedPassword.length < 6 || normalizedPassword.length > 20) {
    throw createError(VT_STATUS.INVALID_PARAMS, '密码长度必须为 6-20 个字符');
  }

  return {
    name: normalizedName,
    password: normalizedPassword,
  };
}

export function login(payload: AuthLoginPayload): AuthLoginResult {
  const username = readTrimmedString(payload?.username);
  const password = readTrimmedString(payload?.password);

  if (!username || !password) {
    throw createError(VT_STATUS.INVALID_PARAMS, '用户名和密码不能为空');
  }

  const row = getDatabase()
    .prepare<[string], UserRow>('SELECT id, name, password FROM users WHERE name = ? LIMIT 1')
    .get(username);

  if (!row || row.password !== password) {
    throw createError(VT_STATUS.UNAUTHORIZED, '用户名或密码错误');
  }

  return {
    token: getTokenKey(),
    user: toAuthUser(row),
  };
}

export function getCurrentUser(payload?: AuthCurrentUserPayload): AuthUser {
  return toAuthUser(assertSession(payload));
}

export function updateLocalUser(payload: AuthUpdateLocalUserPayload): AuthUpdateLocalUserResult {
  const id = readPositiveInteger(payload?.id);
  if (!id) {
    throw createError(VT_STATUS.INVALID_PARAMS, '用户 ID 无效');
  }

  getUserById(id);
  const { name, password } = validateUserForm(payload?.name, payload?.password);
  const now = Date.now();

  getDatabase()
    .prepare<[string, string, number, number]>(
      `
      UPDATE users
      SET name = ?, password = ?, updated_at = ?
      WHERE id = ?
      `,
    )
    .run(name, password, now, id);

  return {
    user: toAuthUser(getUserById(id)),
  };
}

export function logout(): Record<string, never> {
  return {};
}

export function validateSession(payload?: AuthValidateSessionPayload): AuthValidateSessionResult {
  const user = assertSession(payload);

  return {
    valid: true,
    user: toAuthUser(user),
  };
}

export function getInitialLocalUser(): AuthUser {
  return toAuthUser(getDefaultUser());
}
