import type { AgentNamespace } from './socket';
import type { MemoryClearType } from './memory';

export type MemoryModelDtype = 'fp32' | 'fp16' | 'q8';

export interface MemorySettingsConfig {
  modelOnnxFile: string[];
  modelDtype: MemoryModelDtype;
  messagesPerSummary: number;
  shortTermLimit: number;
  summaryMaxLength: number;
  summaryLimit: number;
  ragLimit: number;
  deepRetrieveSummaryLimit: number;
}

export interface MemoryModelStatus {
  available: boolean;
  relativePath: string;
  modelFolder: string;
  modelDtype: string;
}

export interface MemoryStatsItem {
  isolationKey: string;
  total: number;
  messages: number;
  summaries: number;
}

export interface MemoryStatsResult {
  total: number;
  messages: number;
  summaries: number;
  isolations: MemoryStatsItem[];
}

export interface MemorySettingsResult {
  config: MemorySettingsConfig;
  modelStatus: MemoryModelStatus;
  stats: MemoryStatsResult;
}

export interface MemorySettingsSavePayload extends MemorySettingsConfig {}

export interface MemorySettingsSaveResult extends MemorySettingsResult {}

export interface MemorySettingsRestoreDefaultResult extends MemorySettingsResult {}

export interface MemorySettingsValidateModelPathPayload {
  modelOnnxFile: string[];
}

export interface MemorySettingsValidateModelPathResult {
  available: boolean;
  relativePath: string;
}

export type MemoryClearScope = 'isolation' | 'all';

export interface MemorySettingsClearPayload {
  scope: MemoryClearScope;
  type?: MemoryClearType;
  projectId?: number | string;
  agentType?: AgentNamespace;
  episodesId?: number | string;
  confirmText?: string;
}

export interface MemorySettingsClearResult {
  deleted: number;
  updated: number;
  stats: MemoryStatsResult;
}
