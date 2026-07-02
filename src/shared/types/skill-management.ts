export type SkillType = 'main' | 'references';

export type SkillFileStatus = 'ready' | 'missing';

export type SkillEmbeddingStatus = 'ready' | 'expired' | 'not-applicable';

export interface SkillManagementItem {
  id: string;
  md5: string;
  path: string;
  name: string;
  description: string;
  type: SkillType;
  attributions: string[];
  fileStatus: SkillFileStatus;
  embeddingStatus: SkillEmbeddingStatus;
  state: number;
  updatedAt: number;
}

export interface SkillManagementListPayload {
  keyword?: string;
}

export interface SkillManagementListResult {
  skills: SkillManagementItem[];
}

export interface SkillManagementGetContentPayload {
  id: string;
}

export interface SkillManagementContentResult {
  skill: SkillManagementItem;
  content: string;
}

export interface SkillManagementValidationWarning {
  code: string;
  message: string;
}

export interface SkillManagementSaveContentPayload {
  id: string;
  content: string;
  force?: boolean;
}

export interface SkillManagementSaveContentResult {
  saved: boolean;
  warnings: SkillManagementValidationWarning[];
  skill?: SkillManagementItem;
  content?: string;
}
