import type {
  AgentConfigSavePayload,
} from '@shared/types/agent-config';
import type {
  PromptRestoreDefaultPayload,
  PromptUpdatePayload,
} from '@shared/types/prompt';
import type {
  SkillManagementGetContentPayload,
  SkillManagementListPayload,
  SkillManagementSaveContentPayload,
} from '@shared/types/skill-management';
import type {
  ApiConnectionDeletePayload,
  ApiConnectionSavePayload,
  ApiConnectionTestPayload,
  ResourceBindingSavePayload,
  ResourceTestPayload,
} from '@shared/types/model-config';
import type {
  ModelPromptBindPayload,
  ModelPromptClearBindingPayload,
  ModelPromptTemplateDeletePayload,
  ModelPromptTemplateSavePayload,
} from '@shared/types/model-prompt';
import type {
  MemorySettingsClearPayload,
  MemorySettingsSavePayload,
  MemorySettingsValidateModelPathPayload,
} from '@shared/types/memory-settings';
import type {
  DatabaseClearAllPayload,
  DatabaseClearTablePayload,
  DatabaseImportPayload,
} from '@shared/types/database-management';
import type { FileManagementOpenPayload } from '@shared/types/file-management';
import type {
  VendorAddCodePayload,
  VendorCodePayload,
  VendorDeleteModelPayload,
  VendorDeletePayload,
  VendorModelPayload,
  VendorSetEnabledPayload,
  VendorTestImagePayload,
  VendorTestTextPayload,
  VendorTestVideoPayload,
  VendorUpdateInputsPayload,
} from '@shared/types/vendor';
import {
  addVendorCode,
  deleteVendor,
  deleteVendorModel,
  getVendorCode,
  getVendorList,
  runVendorImageTest,
  runVendorTextTest,
  runVendorVideoTest,
  saveVendorCode,
  saveVendorEnabled,
  saveVendorInputs,
  saveVendorModel,
} from '../services/settings/vendor';
import {
  getAgentConfig,
  saveAgentConfig,
} from '../services/settings/agent-config';
import {
  deleteApiConnection,
  getApiConnectionList,
  getConnectionTemplates,
  getResourceConfig,
  saveApiConnection,
  saveResourceBinding,
  testApiConnection,
  testResourceBinding,
} from '../services/settings/model-config';
import {
  bindModelPromptTemplate,
  clearModelPromptBinding,
  deleteModelPromptTemplate,
  getModelPromptConfig,
  saveModelPromptTemplate,
} from '../services/settings/model-prompt';
import {
  getPromptList,
  restorePromptDefault,
  updatePrompt,
} from '../services/settings/prompt';
import {
  clearMemoryBySettings,
  getMemorySettings,
  restoreDefaultMemorySettings,
  saveMemorySettings,
  validateMemoryModelPath,
} from '../services/settings/memory-settings';
import { listOpenableDirectories, openDirectory } from '../services/settings/file-management';
import {
  checkDatabaseRunningTasks,
  clearAllDatabaseData,
  clearDatabaseTable,
  exportDatabaseBackup,
  getDatabaseManagementInfo,
  importDatabaseBackup,
  listDatabaseBackups,
  listDatabaseTables,
} from '../services/settings/database-management';
import {
  getSkillContent,
  getSkillManagementList,
  saveSkillContent,
} from '../services/settings/skill-management';
import { handleIpc } from './handle';

function readObjectArg<T extends object>(value: unknown): T {
  return value && typeof value === 'object' ? (value as T) : ({} as T);
}

export function registerSettingsIpc(): void {
  handleIpc('settings:api:list', () => getApiConnectionList());
  handleIpc('settings:api:templates', () => getConnectionTemplates());
  handleIpc('settings:api:save', (_event, payload) => saveApiConnection(readObjectArg<ApiConnectionSavePayload>(payload)));
  handleIpc('settings:api:delete', (_event, payload) => deleteApiConnection(readObjectArg<ApiConnectionDeletePayload>(payload)));
  handleIpc('settings:api:test', (_event, payload) => testApiConnection(readObjectArg<ApiConnectionTestPayload>(payload)));
  handleIpc('settings:resource:get', () => getResourceConfig());
  handleIpc('settings:resource:save-binding', (_event, payload) => saveResourceBinding(readObjectArg<ResourceBindingSavePayload>(payload)));
  handleIpc('settings:resource:test', (_event, payload) => testResourceBinding(readObjectArg<ResourceTestPayload>(payload)));
  handleIpc('settings:agent-config:get', () => getAgentConfig());
  handleIpc('settings:agent-config:save', (_event, payload) => saveAgentConfig(readObjectArg<AgentConfigSavePayload>(payload)));
  handleIpc('settings:model-prompt:get', () => getModelPromptConfig());
  handleIpc('settings:model-prompt:save-template', (_event, payload) => saveModelPromptTemplate(readObjectArg<ModelPromptTemplateSavePayload>(payload)));
  handleIpc('settings:model-prompt:delete-template', (_event, payload) => deleteModelPromptTemplate(readObjectArg<ModelPromptTemplateDeletePayload>(payload)));
  handleIpc('settings:model-prompt:bind', (_event, payload) => bindModelPromptTemplate(readObjectArg<ModelPromptBindPayload>(payload)));
  handleIpc('settings:model-prompt:clear-binding', (_event, payload) => clearModelPromptBinding(readObjectArg<ModelPromptClearBindingPayload>(payload)));
  handleIpc('settings:prompt:list', () => getPromptList());
  handleIpc('settings:prompt:update', (_event, payload) => updatePrompt(readObjectArg<PromptUpdatePayload>(payload)));
  handleIpc('settings:prompt:restore-default', (_event, payload) => restorePromptDefault(readObjectArg<PromptRestoreDefaultPayload>(payload)));
  handleIpc('settings:memory:get', () => getMemorySettings());
  handleIpc('settings:memory:save', (_event, payload) => saveMemorySettings(readObjectArg<MemorySettingsSavePayload>(payload)));
  handleIpc('settings:memory:restore-default', () => restoreDefaultMemorySettings());
  handleIpc('settings:memory:validate-model-path', (_event, payload) => validateMemoryModelPath(readObjectArg<MemorySettingsValidateModelPathPayload>(payload)));
  handleIpc('settings:memory:clear', (_event, payload) => clearMemoryBySettings(readObjectArg<MemorySettingsClearPayload>(payload)));
  handleIpc('settings:files:list-openable-dirs', () => listOpenableDirectories());
  handleIpc('settings:files:open-dir', (_event, payload) => openDirectory(readObjectArg<FileManagementOpenPayload>(payload)));
  handleIpc('settings:database:info', () => getDatabaseManagementInfo());
  handleIpc('settings:database:list-backups', () => listDatabaseBackups());
  handleIpc('settings:database:export', () => exportDatabaseBackup());
  handleIpc('settings:database:import', (_event, payload) => importDatabaseBackup(readObjectArg<DatabaseImportPayload>(payload)));
  handleIpc('settings:database:list-tables', () => listDatabaseTables());
  handleIpc('settings:database:clear-table', (_event, payload) => clearDatabaseTable(readObjectArg<DatabaseClearTablePayload>(payload)));
  handleIpc('settings:database:clear-all', (_event, payload) => clearAllDatabaseData(readObjectArg<DatabaseClearAllPayload>(payload)));
  handleIpc('settings:database:check-running-tasks', () => checkDatabaseRunningTasks());
  handleIpc('settings:skill:list', (_event, payload) => getSkillManagementList(readObjectArg<SkillManagementListPayload>(payload)));
  handleIpc('settings:skill:get-content', (_event, payload) => getSkillContent(readObjectArg<SkillManagementGetContentPayload>(payload)));
  handleIpc('settings:skill:save-content', (_event, payload) => saveSkillContent(readObjectArg<SkillManagementSaveContentPayload>(payload)));
  handleIpc('settings:vendor:list', () => getVendorList());
  handleIpc('settings:vendor:update-inputs', (_event, payload) => saveVendorInputs(readObjectArg<VendorUpdateInputsPayload>(payload)));
  handleIpc('settings:vendor:set-enabled', (_event, payload) => saveVendorEnabled(readObjectArg<VendorSetEnabledPayload>(payload)));
  handleIpc('settings:vendor:save-model', (_event, payload) => saveVendorModel(readObjectArg<VendorModelPayload>(payload)));
  handleIpc('settings:vendor:delete-model', (_event, payload) => deleteVendorModel(readObjectArg<VendorDeleteModelPayload>(payload)));
  handleIpc('settings:vendor:get-code', (_event, payload) => getVendorCode(readObjectArg<VendorDeletePayload>(payload)));
  handleIpc('settings:vendor:add-code', (_event, payload) => addVendorCode(readObjectArg<VendorAddCodePayload>(payload)));
  handleIpc('settings:vendor:update-code', (_event, payload) => saveVendorCode(readObjectArg<VendorCodePayload>(payload)));
  handleIpc('settings:vendor:delete', (_event, payload) => deleteVendor(readObjectArg<VendorDeletePayload>(payload)));
  handleIpc('settings:vendor:test-text', (_event, payload) => runVendorTextTest(readObjectArg<VendorTestTextPayload>(payload)));
  handleIpc('settings:vendor:test-image', (_event, payload) => runVendorImageTest(readObjectArg<VendorTestImagePayload>(payload)));
  handleIpc('settings:vendor:test-video', (_event, payload) => runVendorVideoTest(readObjectArg<VendorTestVideoPayload>(payload)));
}
