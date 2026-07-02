export const TEXT_AGENT_KEYS = [
  'scriptAgent',
  'productionAgent',
  'universalAi',
  'scriptAgent:decisionAgent',
  'scriptAgent:supervisionAgent',
  'scriptAgent:storySkeletonAgent',
  'scriptAgent:adaptationStrategyAgent',
  'scriptAgent:scriptAgent',
  'productionAgent:decisionAgent',
  'productionAgent:supervisionAgent',
  'productionAgent:deriveAssetsAgent',
  'productionAgent:generateAssetsAgent',
  'productionAgent:directorPlanAgent',
  'productionAgent:storyboardGenAgent',
  'productionAgent:storyboardPanelAgent',
  'productionAgent:storyboardTableAgent',
] as const;

export type TextAgentKey = (typeof TEXT_AGENT_KEYS)[number];
export type AgentConfigGroup = 'main' | 'script' | 'production';
export type AgentConfigStatus = 'inherited' | 'overridden' | 'missing-default' | 'invalid-default' | 'invalid-override' | 'disabled';

export interface AgentTextModelOption {
  modelId: string;
  connectionId: string;
  connectionName: string;
  modelName: string;
  modelDisplayName: string;
}

export interface AgentEffectiveModel {
  modelId: string;
  connectionId: string;
  connectionName: string;
  modelName: string;
  modelDisplayName: string;
}

export interface AgentGlobalSettings {
  temperature: number;
  maxOutputTokens: number;
}

export interface AgentConfigItem {
  id: number;
  key: TextAgentKey;
  name: string;
  description: string | null;
  group: AgentConfigGroup;
  modelLabel: string | null;
  modelId: string | null;
  vendorId: string | null;
  overrideEnabled: boolean;
  temperature: number | null;
  maxOutputTokens: number | null;
  disabled: boolean;
  effectiveModel: AgentEffectiveModel | null;
  status: AgentConfigStatus;
  statusText: string;
}

export interface AgentConfigResult {
  agents: AgentConfigItem[];
  availableTextModels: AgentTextModelOption[];
  defaultTextModel: AgentEffectiveModel | null;
  defaultTextStatus: 'configured' | 'missing' | 'unsupported';
  defaultTextStatusText: string;
  globalSettings: AgentGlobalSettings;
}

export interface AgentConfigSavePayload {
  globalSettings: AgentGlobalSettings;
  agents: Array<{
    key: TextAgentKey;
    modelId: string | null;
    temperature: number | null;
    maxOutputTokens: number | null;
  }>;
}

export interface AgentConfigSaveResult {
  agents: AgentConfigItem[];
  globalSettings: AgentGlobalSettings;
}
