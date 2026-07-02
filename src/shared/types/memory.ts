import type { AgentNamespace } from './socket';

export type MemoryType = 'message' | 'summary';
export type MemoryClearType = MemoryType | 'all';
export type MemoryRole = 'user' | 'assistant' | `assistant:${string}`;

export interface MemoryIsolationInput {
  projectId: number | string;
  agentType: AgentNamespace;
  episodesId?: number | string | null;
}

export interface MemoryMetadata {
  attachments?: Array<{
    id?: string;
    type: 'image' | 'video' | 'audio' | 'document' | 'file';
    relativePath?: string;
    mimeType?: string;
    size?: number;
    source?: string;
    summary?: string;
  }>;
  source?: string;
  tokenEstimate?: number;
  [key: string]: unknown;
}

export interface AddMemoryInput {
  projectId?: number | string;
  agentType?: AgentNamespace;
  episodesId?: number | string | null;
  isolationKey?: string;
  role?: MemoryRole;
  name?: string;
  content: string;
  createdAt?: number;
  metadata?: MemoryMetadata | null;
}

export interface MemoryItem {
  id: string;
  role: MemoryRole | null;
  name: string | null;
  content: string;
  metadata: MemoryMetadata | null;
  createdAt: number;
}

export interface MemorySummaryItem {
  id: string;
  content: string;
  relatedMessageIds: string[];
  metadata: MemoryMetadata | null;
  createdAt: number;
}

export interface MemoryRagItem {
  id: string;
  content: string;
  similarity: number;
  metadata: MemoryMetadata | null;
  createdAt: number;
}

export interface MemoryContextResult {
  shortTerm: MemoryItem[];
  summaries: MemorySummaryItem[];
  rag: MemoryRagItem[];
  warning?: string;
}

export interface HistoryMessage {
  id: string;
  role: 'user' | 'assistant';
  name?: string;
  status: 'complete';
  datetime: string;
  content: Array<{
    type: 'markdown';
    status: 'complete';
    data: string;
  }>;
  createTime: number;
}

export interface ClearMemoryInput extends MemoryIsolationInput {
  type?: MemoryClearType;
}

export interface ClearMemoryResult {
  deleted: number;
  updated: number;
}

export interface DeepRetrieveResult {
  id: string;
  content: string;
  metadata: MemoryMetadata | null;
  createdAt: number;
}
