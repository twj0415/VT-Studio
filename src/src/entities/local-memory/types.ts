export interface LocalMemoryEntryDto {
  memoryId: string
  projectId: string | null
  sourceKind: string
  sourceId: string
  sourceLabel: string
  contentSummary: string
  contentHash: string
  embeddingProviderId: string | null
  embeddingModelId: string | null
  embeddingVectorPath: string | null
  metadata: Record<string, unknown>
  lifecycle: 'active' | 'expired' | 'disabled' | string
  expiresAt: string | null
  createdAt: string
  updatedAt: string
}

export interface LocalMemoryRetrievalCandidateDto {
  candidateId: string
  retrievalId: string
  memoryId: string
  similarity: number
  status: 'waiting_user' | 'approved' | 'rejected' | string
  reason: string | null
  citation: Record<string, unknown>
  createdAt: string
  updatedAt: string
}

export interface LocalMemoryRetrievalDto {
  retrievalId: string
  projectId: string
  queryText: string
  minSimilarity: number
  maxResults: number
  status: 'waiting_user' | 'approved' | 'rejected' | 'expired' | string
  candidates: LocalMemoryRetrievalCandidateDto[]
  createdAt: string
  updatedAt: string
}

export interface LocalMemoryContextCitationDto {
  candidateId: string
  memoryId: string
  sourceKind: string
  sourceId: string
  sourceLabel: string
  similarity: number
  contentSummary: string
  citation: Record<string, unknown>
}

export interface LocalMemoryContextDto {
  retrievalId: string
  projectId: string
  minSimilarity: number
  usable: boolean
  citations: LocalMemoryContextCitationDto[]
}

export interface UpsertLocalMemoryEntryRequest {
  memoryId?: string | null
  projectId?: string | null
  sourceKind: string
  sourceId: string
  sourceLabel?: string | null
  contentSummary: string
  embeddingProviderId?: string | null
  embeddingModelId?: string | null
  embeddingVectorPath?: string | null
  metadata?: Record<string, unknown>
  expiresAt?: string | null
}

export interface ListLocalMemoryEntriesRequest {
  projectId?: string | null
  includeGlobal?: boolean
}

export interface CreateLocalMemoryRetrievalRequest {
  projectId: string
  queryText: string
  minSimilarity: number
  maxResults?: number
  candidates: LocalMemoryRetrievalCandidateInput[]
}

export interface LocalMemoryRetrievalCandidateInput {
  memoryId: string
  similarity: number
  reason?: string | null
}

export interface LocalMemoryCandidateDecisionRequest {
  candidateId: string
  reason?: string | null
}

export interface BuildLocalMemoryContextRequest {
  retrievalId: string
}
