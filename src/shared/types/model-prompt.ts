export type ModelPromptModelType = 'image' | 'video';
export type ModelPromptTemplateType = 'imagePrompt' | 'videoPrompt';
export type ModelPromptBindingStatus = 'bound' | 'fallback' | 'invalid-template' | 'type-mismatch';
export type ModelPromptInvalidReason = 'model-missing' | 'template-missing' | 'type-mismatch';

export interface ModelPromptTemplate {
  id: number;
  name: string;
  type: ModelPromptTemplateType;
  content: string;
  isBuiltin: boolean;
  createdAt: number;
  updatedAt: number;
  referenceCount: number;
}

export interface ModelPromptBinding {
  id: number;
  connectionId: string;
  modelName: string;
  modelType: ModelPromptModelType;
  modelMode: string;
  templateId: number;
  templateName: string;
  templateType: ModelPromptTemplateType;
  createdAt: number;
  updatedAt: number;
}

export interface ModelPromptModelItem {
  connectionId: string;
  connectionName: string;
  modelName: string;
  modelDisplayName: string;
  modelType: ModelPromptModelType;
  modelMode: string;
  binding: ModelPromptBinding | null;
  status: ModelPromptBindingStatus;
  statusText: string;
}

export interface ModelPromptConnectionGroup {
  connectionId: string;
  connectionName: string;
  connectionStatus: string;
  connectionStatusText: string;
  models: ModelPromptModelItem[];
}

export interface ModelPromptInvalidMapping {
  id: number;
  connectionId: string;
  modelName: string;
  modelType: ModelPromptModelType;
  modelMode: string;
  templateId: number;
  templateName: string;
  reason: ModelPromptInvalidReason;
  reasonText: string;
}

export interface ModelPromptConfigResult {
  templates: ModelPromptTemplate[];
  connections: ModelPromptConnectionGroup[];
  invalidMappings: ModelPromptInvalidMapping[];
}

export interface ModelPromptTemplateSavePayload {
  id?: number;
  name: string;
  type: ModelPromptTemplateType;
  content: string;
}

export interface ModelPromptTemplateSaveResult {
  template: ModelPromptTemplate;
}

export interface ModelPromptTemplateDeletePayload {
  id: number;
}

export interface ModelPromptTemplateDeleteResult {
  templateId: number;
}

export interface ModelPromptBindPayload {
  connectionId: string;
  modelName: string;
  modelType: ModelPromptModelType;
  modelMode?: string;
  templateId: number;
}

export interface ModelPromptBindResult {
  binding: ModelPromptBinding;
}

export interface ModelPromptClearBindingPayload {
  connectionId: string;
  modelName: string;
  modelType: ModelPromptModelType;
  modelMode?: string;
}

export interface ModelPromptClearBindingResult {
  cleared: boolean;
}
