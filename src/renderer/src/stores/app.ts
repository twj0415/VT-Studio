import { defineStore } from 'pinia';
import { VT_STATUS } from '@shared/constants/status';
import type { AppInfo } from '@shared/types/app';

interface CurrentProject {
  id: string;
  name: string;
  type: 'novel' | 'script';
  status: 'mock';
}

interface AppState {
  appInfo: AppInfo | null;
  currentProject: CurrentProject | null;
}

export const useAppStore = defineStore('app', {
  state: (): AppState => ({
    appInfo: null,
    currentProject: {
      id: 'placeholder',
      name: '未选择项目',
      type: 'novel',
      status: 'mock',
    },
  }),
  actions: {
    async loadAppInfo(): Promise<void> {
      const response = await window.vtStudio.app.getInfo();
      this.appInfo = response.code === VT_STATUS.OK ? response.data : null;
    },
    clearCurrentProject(): void {
      this.currentProject = null;
    },
  },
});
