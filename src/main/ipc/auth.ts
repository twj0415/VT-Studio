import type {
  AuthCurrentUserPayload,
  AuthLoginPayload,
  AuthLoginResult,
  AuthUpdateLocalUserPayload,
  AuthUpdateLocalUserResult,
  AuthValidateSessionPayload,
  AuthValidateSessionResult,
} from '@shared/types/auth';
import {
  getCurrentUser,
  login,
  logout,
  updateLocalUser,
  validateSession,
} from '../services/auth';
import { handleIpc } from './handle';

function readObjectArg<T extends object>(value: unknown): T {
  return value && typeof value === 'object' ? (value as T) : ({} as T);
}

export function registerAuthIpc(): void {
  handleIpc<AuthLoginResult>('auth:login', (_event, payload) => login(readObjectArg<AuthLoginPayload>(payload)));
  handleIpc('auth:get-current-user', (_event, payload) => getCurrentUser(readObjectArg<AuthCurrentUserPayload>(payload)));
  handleIpc<AuthUpdateLocalUserResult>('auth:update-local-user', (_event, payload) => updateLocalUser(readObjectArg<AuthUpdateLocalUserPayload>(payload)));
  handleIpc<Record<string, never>>('auth:logout', () => logout());
  handleIpc<AuthValidateSessionResult>('auth:validate-session', (_event, payload) => validateSession(readObjectArg<AuthValidateSessionPayload>(payload)));
}
