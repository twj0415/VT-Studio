import { contextBridge, ipcRenderer } from 'electron';
import type { VtStudioApi } from '@shared/contracts/preload';

const api: VtStudioApi = {
  app: {
    getInfo: () => ipcRenderer.invoke('app:get-info'),
  },
  agent: {
    getSocketInfo: () => ipcRenderer.invoke('agent:get-socket-info'),
  },
  media: {
    createUrl: (payload) => ipcRenderer.invoke('media:create-url', payload),
    createThumbnailUrl: (payload) => ipcRenderer.invoke('media:create-thumbnail-url', payload),
    resolveUrlToPath: (payload) => ipcRenderer.invoke('media:resolve-url-to-path', payload),
    getOriginalUrl: (payload) => ipcRenderer.invoke('media:get-original-url', payload),
  },
  auth: {
    login: (payload) => ipcRenderer.invoke('auth:login', payload),
    getCurrentUser: (payload) => ipcRenderer.invoke('auth:get-current-user', payload),
    updateLocalUser: (payload) => ipcRenderer.invoke('auth:update-local-user', payload),
    logout: () => ipcRenderer.invoke('auth:logout'),
    validateSession: (payload) => ipcRenderer.invoke('auth:validate-session', payload),
  },
  settings: {
    api: {
      list: () => ipcRenderer.invoke('settings:api:list'),
      templates: () => ipcRenderer.invoke('settings:api:templates'),
      save: (payload) => ipcRenderer.invoke('settings:api:save', payload),
      delete: (payload) => ipcRenderer.invoke('settings:api:delete', payload),
      test: (payload) => ipcRenderer.invoke('settings:api:test', payload),
    },
    resource: {
      get: () => ipcRenderer.invoke('settings:resource:get'),
      saveBinding: (payload) => ipcRenderer.invoke('settings:resource:save-binding', payload),
      test: (payload) => ipcRenderer.invoke('settings:resource:test', payload),
    },
    agentConfig: {
      get: () => ipcRenderer.invoke('settings:agent-config:get'),
      save: (payload) => ipcRenderer.invoke('settings:agent-config:save', payload),
    },
    modelPrompt: {
      get: () => ipcRenderer.invoke('settings:model-prompt:get'),
      saveTemplate: (payload) => ipcRenderer.invoke('settings:model-prompt:save-template', payload),
      deleteTemplate: (payload) => ipcRenderer.invoke('settings:model-prompt:delete-template', payload),
      bind: (payload) => ipcRenderer.invoke('settings:model-prompt:bind', payload),
      clearBinding: (payload) => ipcRenderer.invoke('settings:model-prompt:clear-binding', payload),
    },
    prompt: {
      list: () => ipcRenderer.invoke('settings:prompt:list'),
      update: (payload) => ipcRenderer.invoke('settings:prompt:update', payload),
      restoreDefault: (payload) => ipcRenderer.invoke('settings:prompt:restore-default', payload),
    },
    memory: {
      get: () => ipcRenderer.invoke('settings:memory:get'),
      save: (payload) => ipcRenderer.invoke('settings:memory:save', payload),
      restoreDefault: () => ipcRenderer.invoke('settings:memory:restore-default'),
      validateModelPath: (payload) => ipcRenderer.invoke('settings:memory:validate-model-path', payload),
      clear: (payload) => ipcRenderer.invoke('settings:memory:clear', payload),
    },
    database: {
      info: () => ipcRenderer.invoke('settings:database:info'),
      listBackups: () => ipcRenderer.invoke('settings:database:list-backups'),
      export: () => ipcRenderer.invoke('settings:database:export'),
      import: (payload) => ipcRenderer.invoke('settings:database:import', payload),
      listTables: () => ipcRenderer.invoke('settings:database:list-tables'),
      clearTable: (payload) => ipcRenderer.invoke('settings:database:clear-table', payload),
      clearAll: (payload) => ipcRenderer.invoke('settings:database:clear-all', payload),
      checkRunningTasks: () => ipcRenderer.invoke('settings:database:check-running-tasks'),
    },
    files: {
      listOpenableDirs: () => ipcRenderer.invoke('settings:files:list-openable-dirs'),
      openDir: (payload) => ipcRenderer.invoke('settings:files:open-dir', payload),
    },
    business: {
      get: () => ipcRenderer.invoke('settings:business:get'),
      save: (payload) => ipcRenderer.invoke('settings:business:save', payload),
      restoreDefaultChapterReg: () => ipcRenderer.invoke('settings:business:restore-default-chapter-reg'),
    },
    skill: {
      list: (payload) => ipcRenderer.invoke('settings:skill:list', payload),
      getContent: (payload) => ipcRenderer.invoke('settings:skill:get-content', payload),
      saveContent: (payload) => ipcRenderer.invoke('settings:skill:save-content', payload),
    },
    vendor: {
      list: () => ipcRenderer.invoke('settings:vendor:list'),
      updateInputs: (payload) => ipcRenderer.invoke('settings:vendor:update-inputs', payload),
      setEnabled: (payload) => ipcRenderer.invoke('settings:vendor:set-enabled', payload),
      saveModel: (payload) => ipcRenderer.invoke('settings:vendor:save-model', payload),
      deleteModel: (payload) => ipcRenderer.invoke('settings:vendor:delete-model', payload),
      getCode: (payload) => ipcRenderer.invoke('settings:vendor:get-code', payload),
      addCode: (payload) => ipcRenderer.invoke('settings:vendor:add-code', payload),
      updateCode: (payload) => ipcRenderer.invoke('settings:vendor:update-code', payload),
      delete: (payload) => ipcRenderer.invoke('settings:vendor:delete', payload),
      testText: (payload) => ipcRenderer.invoke('settings:vendor:test-text', payload),
      testImage: (payload) => ipcRenderer.invoke('settings:vendor:test-image', payload),
      testVideo: (payload) => ipcRenderer.invoke('settings:vendor:test-video', payload),
    },
  },
};

contextBridge.exposeInMainWorld('vtStudio', api);
