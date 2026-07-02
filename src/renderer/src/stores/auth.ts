import { defineStore } from 'pinia';
import { VT_STATUS } from '@shared/constants/status';
import type { AuthUser } from '@shared/types/auth';
import { useAppStore } from './app';

const AUTH_TOKEN_KEY = 'token';
const AUTH_USER_KEY = 'user';
const AUTH_USER_ID_KEY = 'userId';

interface AuthState {
  token: string | null;
  user: AuthUser | null;
  restored: boolean;
  loading: boolean;
  error: string | null;
}

function readStoredUser(): AuthUser | null {
  const raw = window.localStorage.getItem(AUTH_USER_KEY);
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as Partial<AuthUser>;
    if (typeof parsed.id === 'number' && typeof parsed.name === 'string') {
      return {
        id: parsed.id,
        name: parsed.name,
      };
    }
  } catch {
    return null;
  }

  return null;
}

function readStoredUserId(): number | null {
  const raw = window.localStorage.getItem(AUTH_USER_ID_KEY);
  if (!raw) {
    return null;
  }

  const userId = Number(raw);
  return Number.isInteger(userId) && userId > 0 ? userId : null;
}

function persistAuth(token: string, user: AuthUser): void {
  window.localStorage.setItem(AUTH_TOKEN_KEY, token);
  window.localStorage.setItem(AUTH_USER_KEY, JSON.stringify(user));
  window.localStorage.setItem(AUTH_USER_ID_KEY, String(user.id));
}

function clearStoredAuth(): void {
  window.localStorage.removeItem(AUTH_TOKEN_KEY);
  window.localStorage.removeItem(AUTH_USER_KEY);
  window.localStorage.removeItem(AUTH_USER_ID_KEY);
}

function getUnknownErrorMessage(error: unknown, fallback: string): string {
  return error instanceof Error && error.message ? error.message : fallback;
}

function getAuthApi() {
  return window.vtStudio?.auth ?? null;
}

export const useAuthStore = defineStore('auth', {
  state: (): AuthState => ({
    token: window.localStorage.getItem(AUTH_TOKEN_KEY),
    user: readStoredUser(),
    restored: false,
    loading: false,
    error: null,
  }),
  getters: {
    isLoggedIn: (state): boolean => Boolean(state.token && state.user),
    userId: (state): number | null => state.user?.id ?? readStoredUserId(),
  },
  actions: {
    setAuth(token: string, user: AuthUser): void {
      this.token = token;
      this.user = user;
      this.error = null;
      persistAuth(token, user);
    },
    clearAuth(): void {
      this.token = null;
      this.user = null;
      this.error = null;
      clearStoredAuth();
      useAppStore().clearCurrentProject();
    },
    async login(username: string, password: string): Promise<boolean> {
      this.loading = true;
      this.error = null;

      try {
        const authApi = getAuthApi();
        if (!authApi?.login) {
          this.error = '登录服务未加载，请重启应用后重试';
          return false;
        }

        const response = await authApi.login({ username, password });
        if (response.code !== VT_STATUS.OK) {
          this.error = response.msg;
          return false;
        }

        this.setAuth(response.data.token, response.data.user);
        return true;
      } catch (error) {
        this.error = getUnknownErrorMessage(error, '登录失败');
        return false;
      } finally {
        this.loading = false;
      }
    },
    async restoreSession(): Promise<boolean> {
      if (this.restored) {
        return this.isLoggedIn;
      }

      this.restored = true;
      const token = this.token ?? window.localStorage.getItem(AUTH_TOKEN_KEY);
      const userId = this.user?.id ?? readStoredUserId();

      if (!token || !userId) {
        this.clearAuth();
        return false;
      }

      try {
        const authApi = getAuthApi();
        if (!authApi?.validateSession) {
          this.clearAuth();
          this.error = '登录服务未加载，请重启应用后重试';
          return false;
        }

        const response = await authApi.validateSession({ token, userId });
        if (response.code !== VT_STATUS.OK || !response.data.valid) {
          this.clearAuth();
          this.error = response.msg;
          return false;
        }

        this.setAuth(token, response.data.user);
        return true;
      } catch (error) {
        this.clearAuth();
        this.error = getUnknownErrorMessage(error, '登录状态恢复失败');
        return false;
      }
    },
    async loadCurrentUser(): Promise<boolean> {
      const token = this.token ?? window.localStorage.getItem(AUTH_TOKEN_KEY);
      const userId = this.user?.id ?? readStoredUserId();

      if (!token || !userId) {
        this.clearAuth();
        return false;
      }

      try {
        const authApi = getAuthApi();
        if (!authApi?.getCurrentUser) {
          this.clearAuth();
          this.error = '登录服务未加载，请重启应用后重试';
          return false;
        }

        const response = await authApi.getCurrentUser({ token, userId });
        if (response.code !== VT_STATUS.OK) {
          this.clearAuth();
          this.error = response.msg;
          return false;
        }

        this.setAuth(token, response.data);
        return true;
      } catch (error) {
        this.clearAuth();
        this.error = getUnknownErrorMessage(error, '读取当前用户失败');
        return false;
      }
    },
    async updateLocalUser(name: string, password: string): Promise<boolean> {
      if (!this.user) {
        this.error = '登录已失效，请重新登录';
        return false;
      }

      this.loading = true;
      this.error = null;

      try {
        const authApi = getAuthApi();
        if (!authApi?.updateLocalUser) {
          this.error = '登录服务未加载，请重启应用后重试';
          return false;
        }

        const response = await authApi.updateLocalUser({
          id: this.user.id,
          name,
          password,
        });
        if (response.code !== VT_STATUS.OK) {
          this.error = response.msg;
          return false;
        }

        if (this.token) {
          this.setAuth(this.token, response.data.user);
        }
        return true;
      } catch (error) {
        this.error = getUnknownErrorMessage(error, '保存用户配置失败');
        return false;
      } finally {
        this.loading = false;
      }
    },
    async logout(): Promise<void> {
      const authApi = getAuthApi();
      if (authApi?.logout) {
        await authApi.logout();
      }
      this.clearAuth();
    },
  },
});
