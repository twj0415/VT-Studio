export interface AuthUser {
  id: number;
  name: string;
}

export interface AuthLoginPayload {
  username: string;
  password: string;
}

export interface AuthLoginResult {
  token: string;
  user: AuthUser;
}

export interface AuthCurrentUserPayload {
  token?: string;
  userId?: number;
}

export interface AuthUpdateLocalUserPayload {
  id: number;
  name: string;
  password: string;
}

export interface AuthUpdateLocalUserResult {
  user: AuthUser;
}

export interface AuthValidateSessionPayload {
  token?: string;
  userId?: number;
}

export interface AuthValidateSessionResult {
  valid: boolean;
  user: AuthUser;
}
