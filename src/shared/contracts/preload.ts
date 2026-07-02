import type { AppInfo } from '@shared/types/app';
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
import type { VtResponse } from '@shared/types/response';
import type { AgentSocketInfo } from '@shared/types/socket';
import type { AgentConfigResult, AgentConfigSavePayload, AgentConfigSaveResult } from '@shared/types/agent-config';
import type {
  ApiConnectionDeletePayload,
  ApiConnectionDeleteResult,
  ApiConnectionListResult,
  ApiConnectionSavePayload,
  ApiConnectionSaveResult,
  ApiConnectionTestPayload,
  ApiConnectionTestResult,
  ResourceBindingSavePayload,
  ResourceBindingSaveResult,
  ResourceConfigResult,
  ResourceTestPayload,
  ResourceTestResult,
} from '@shared/types/model-config';
import type {
  ModelPromptBindPayload,
  ModelPromptBindResult,
  ModelPromptClearBindingPayload,
  ModelPromptClearBindingResult,
  ModelPromptConfigResult,
  ModelPromptTemplateDeletePayload,
  ModelPromptTemplateDeleteResult,
  ModelPromptTemplateSavePayload,
  ModelPromptTemplateSaveResult,
} from '@shared/types/model-prompt';
import type {
  MediaCreateThumbnailUrlPayload,
  MediaCreateThumbnailUrlResult,
  MediaCreateUrlPayload,
  MediaCreateUrlResult,
  MediaGetOriginalUrlPayload,
  MediaGetOriginalUrlResult,
  MediaResolveUrlPayload,
  MediaResolveUrlResult,
} from '@shared/types/media';
import type {
  MemorySettingsClearPayload,
  MemorySettingsClearResult,
  MemorySettingsResult,
  MemorySettingsRestoreDefaultResult,
  MemorySettingsSavePayload,
  MemorySettingsSaveResult,
  MemorySettingsValidateModelPathPayload,
  MemorySettingsValidateModelPathResult,
} from '@shared/types/memory-settings';
import type {
  DatabaseBackupListResult,
  DatabaseClearAllPayload,
  DatabaseClearAllResult,
  DatabaseClearTablePayload,
  DatabaseClearTableResult,
  DatabaseExportResult,
  DatabaseImportPayload,
  DatabaseImportResult,
  DatabaseManagementInfoResult,
  DatabaseRunningTasksResult,
  DatabaseTableListResult,
} from '@shared/types/database-management';
import type {
  FileManagementListResult,
  FileManagementOpenPayload,
  FileManagementOpenResult,
} from '@shared/types/file-management';
import type {
  PromptListResult,
  PromptRestoreDefaultPayload,
  PromptRestoreDefaultResult,
  PromptUpdatePayload,
  PromptUpdateResult,
} from '@shared/types/prompt';
import type {
  SkillManagementContentResult,
  SkillManagementGetContentPayload,
  SkillManagementListPayload,
  SkillManagementListResult,
  SkillManagementSaveContentPayload,
  SkillManagementSaveContentResult,
} from '@shared/types/skill-management';
import type {
  VendorAddCodePayload,
  VendorAddCodeResult,
  VendorCodePayload,
  VendorCodeResult,
  VendorDeleteModelPayload,
  VendorDeleteModelResult,
  VendorDeletePayload,
  VendorDeleteResult,
  VendorListResult,
  VendorModelPayload,
  VendorModelSaveResult,
  VendorSetEnabledPayload,
  VendorSetEnabledResult,
  VendorTestImagePayload,
  VendorTestMediaResult,
  VendorTestTextPayload,
  VendorTestTextResult,
  VendorTestVideoPayload,
  VendorUpdateCodeResult,
  VendorUpdateInputsPayload,
  VendorUpdateInputsResult,
} from '@shared/types/vendor';

export interface VtStudioApi {
  app: {
    getInfo: () => Promise<VtResponse<AppInfo>>;
  };
  agent: {
    getSocketInfo: () => Promise<VtResponse<AgentSocketInfo>>;
  };
  media: {
    createUrl: (payload: MediaCreateUrlPayload) => Promise<VtResponse<MediaCreateUrlResult>>;
    createThumbnailUrl: (payload: MediaCreateThumbnailUrlPayload) => Promise<VtResponse<MediaCreateThumbnailUrlResult>>;
    resolveUrlToPath: (payload: MediaResolveUrlPayload) => Promise<VtResponse<MediaResolveUrlResult>>;
    getOriginalUrl: (payload: MediaGetOriginalUrlPayload) => Promise<VtResponse<MediaGetOriginalUrlResult>>;
  };
  auth: {
    login: (payload: AuthLoginPayload) => Promise<VtResponse<AuthLoginResult>>;
    getCurrentUser: (payload: AuthCurrentUserPayload) => Promise<VtResponse<AuthUser>>;
    updateLocalUser: (payload: AuthUpdateLocalUserPayload) => Promise<VtResponse<AuthUpdateLocalUserResult>>;
    logout: () => Promise<VtResponse<Record<string, never>>>;
    validateSession: (payload: AuthValidateSessionPayload) => Promise<VtResponse<AuthValidateSessionResult>>;
  };
  settings: {
    api: {
      list: () => Promise<VtResponse<ApiConnectionListResult>>;
      templates: () => Promise<
        VtResponse<{
          services: Array<{
            serviceType: string;
            name: string;
            defaultBaseUrl: string;
            capabilities: string[];
            models: Array<{
              id: string;
              displayName: string;
              modelName: string;
              type: string;
              think?: boolean;
            }>;
          }>;
        }>
      >;
      save: (payload: ApiConnectionSavePayload) => Promise<VtResponse<ApiConnectionSaveResult>>;
      delete: (payload: ApiConnectionDeletePayload) => Promise<VtResponse<ApiConnectionDeleteResult>>;
      test: (payload: ApiConnectionTestPayload) => Promise<VtResponse<ApiConnectionTestResult>>;
    };
    resource: {
      get: () => Promise<VtResponse<ResourceConfigResult>>;
      saveBinding: (payload: ResourceBindingSavePayload) => Promise<VtResponse<ResourceBindingSaveResult>>;
      test: (payload: ResourceTestPayload) => Promise<VtResponse<ResourceTestResult>>;
    };
    agentConfig: {
      get: () => Promise<VtResponse<AgentConfigResult>>;
      save: (payload: AgentConfigSavePayload) => Promise<VtResponse<AgentConfigSaveResult>>;
    };
    modelPrompt: {
      get: () => Promise<VtResponse<ModelPromptConfigResult>>;
      saveTemplate: (payload: ModelPromptTemplateSavePayload) => Promise<VtResponse<ModelPromptTemplateSaveResult>>;
      deleteTemplate: (payload: ModelPromptTemplateDeletePayload) => Promise<VtResponse<ModelPromptTemplateDeleteResult>>;
      bind: (payload: ModelPromptBindPayload) => Promise<VtResponse<ModelPromptBindResult>>;
      clearBinding: (payload: ModelPromptClearBindingPayload) => Promise<VtResponse<ModelPromptClearBindingResult>>;
    };
    prompt: {
      list: () => Promise<VtResponse<PromptListResult>>;
      update: (payload: PromptUpdatePayload) => Promise<VtResponse<PromptUpdateResult>>;
      restoreDefault: (payload: PromptRestoreDefaultPayload) => Promise<VtResponse<PromptRestoreDefaultResult>>;
    };
    memory: {
      get: () => Promise<VtResponse<MemorySettingsResult>>;
      save: (payload: MemorySettingsSavePayload) => Promise<VtResponse<MemorySettingsSaveResult>>;
      restoreDefault: () => Promise<VtResponse<MemorySettingsRestoreDefaultResult>>;
      validateModelPath: (payload: MemorySettingsValidateModelPathPayload) => Promise<VtResponse<MemorySettingsValidateModelPathResult>>;
      clear: (payload: MemorySettingsClearPayload) => Promise<VtResponse<MemorySettingsClearResult>>;
    };
    database: {
      info: () => Promise<VtResponse<DatabaseManagementInfoResult>>;
      listBackups: () => Promise<VtResponse<DatabaseBackupListResult>>;
      export: () => Promise<VtResponse<DatabaseExportResult>>;
      import: (payload: DatabaseImportPayload) => Promise<VtResponse<DatabaseImportResult>>;
      listTables: () => Promise<VtResponse<DatabaseTableListResult>>;
      clearTable: (payload: DatabaseClearTablePayload) => Promise<VtResponse<DatabaseClearTableResult>>;
      clearAll: (payload: DatabaseClearAllPayload) => Promise<VtResponse<DatabaseClearAllResult>>;
      checkRunningTasks: () => Promise<VtResponse<DatabaseRunningTasksResult>>;
    };
    files: {
      listOpenableDirs: () => Promise<VtResponse<FileManagementListResult>>;
      openDir: (payload: FileManagementOpenPayload) => Promise<VtResponse<FileManagementOpenResult>>;
    };
    skill: {
      list: (payload?: SkillManagementListPayload) => Promise<VtResponse<SkillManagementListResult>>;
      getContent: (payload: SkillManagementGetContentPayload) => Promise<VtResponse<SkillManagementContentResult>>;
      saveContent: (payload: SkillManagementSaveContentPayload) => Promise<VtResponse<SkillManagementSaveContentResult>>;
    };
    vendor: {
      list: () => Promise<VtResponse<VendorListResult>>;
      updateInputs: (payload: VendorUpdateInputsPayload) => Promise<VtResponse<VendorUpdateInputsResult>>;
      setEnabled: (payload: VendorSetEnabledPayload) => Promise<VtResponse<VendorSetEnabledResult>>;
      saveModel: (payload: VendorModelPayload) => Promise<VtResponse<VendorModelSaveResult>>;
      deleteModel: (payload: VendorDeleteModelPayload) => Promise<VtResponse<VendorDeleteModelResult>>;
      getCode: (payload: VendorDeletePayload) => Promise<VtResponse<VendorCodeResult>>;
      addCode: (payload: VendorAddCodePayload) => Promise<VtResponse<VendorAddCodeResult>>;
      updateCode: (payload: VendorCodePayload) => Promise<VtResponse<VendorUpdateCodeResult>>;
      delete: (payload: VendorDeletePayload) => Promise<VtResponse<VendorDeleteResult>>;
      testText: (payload: VendorTestTextPayload) => Promise<VtResponse<VendorTestTextResult>>;
      testImage: (payload: VendorTestImagePayload) => Promise<VtResponse<VendorTestMediaResult>>;
      testVideo: (payload: VendorTestVideoPayload) => Promise<VtResponse<VendorTestMediaResult>>;
    };
  };
}
