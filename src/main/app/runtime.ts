import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { app } from 'electron';
import { is } from '@electron-toolkit/utils';

export function resolveUserDataPath(): string {
  if (process.env.VT_STUDIO_USER_DATA) {
    return process.env.VT_STUDIO_USER_DATA;
  }

  const productionRoot = process.env.LOCALAPPDATA ?? tmpdir();

  if (is.dev) {
    return join(tmpdir(), 'VT Studio Dev', 'user-data');
  }

  return join(productionRoot, 'VT Studio', 'user-data');
}

export function configureRuntime(): string {
  const userDataPath = resolveUserDataPath();
  app.setPath('userData', userDataPath);

  return userDataPath;
}

export function getUserDataPath(): string {
  return app.getPath('userData');
}
