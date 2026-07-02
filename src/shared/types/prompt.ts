export type BuiltinPromptType =
  | 'eventExtraction'
  | 'scriptAssetExtraction'
  | 'videoPromptGeneration'
  | 'audioBindPrompt';

export interface PromptItem {
  id: number;
  name: string;
  type: string;
  data: string;
  useData: string;
  effectiveData: string;
  isCustomized: boolean;
  updatedAt: number;
}

export interface PromptValidationWarning {
  code: string;
  message: string;
}

export interface PromptListResult {
  prompts: PromptItem[];
}

export interface PromptUpdatePayload {
  id: number;
  useData: string;
  force?: boolean;
}

export interface PromptUpdateResult {
  saved: boolean;
  prompt: PromptItem | null;
  warnings: PromptValidationWarning[];
}

export interface PromptRestoreDefaultPayload {
  id: number;
}

export interface PromptRestoreDefaultResult {
  prompt: PromptItem;
}
