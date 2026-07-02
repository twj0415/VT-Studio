import { is } from '@electron-toolkit/utils';
import { APP_NAME, APP_VERSION } from '@shared/constants/app';
import type { AppInfo } from '@shared/types/app';
import { getUserDataPath } from '../app/runtime';
import { handleIpc } from './handle';

export function registerAppIpc(): void {
  handleIpc<AppInfo>('app:get-info', () => {
    return {
      name: APP_NAME,
      version: APP_VERSION,
      platform: process.platform,
      isDev: is.dev,
      userDataPath: getUserDataPath(),
    };
  });
}
