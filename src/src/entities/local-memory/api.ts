import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'
import { createMockId } from '@/shared/mock/ids'

import type {
  BuildLocalMemoryContextRequest,
  CreateLocalMemoryRetrievalRequest,
  ListLocalMemoryEntriesRequest,
  LocalMemoryCandidateDecisionRequest,
  LocalMemoryContextDto,
  LocalMemoryEntryDto,
  LocalMemoryRetrievalCandidateDto,
  LocalMemoryRetrievalDto,
  UpsertLocalMemoryEntryRequest,
} from './types'

const MOCK_NOW = '2026-06-22 10:00'
const entries = new Map<string, LocalMemoryEntryDto>()
const retrievals = new Map<string, LocalMemoryRetrievalDto>()

export async function upsertLocalMemoryEntry(request: UpsertLocalMemoryEntryRequest): Promise<LocalMemoryEntryDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LocalMemoryEntryDto, { request: UpsertLocalMemoryEntryRequest }>(
      tauriCommands.upsertLocalMemoryEntry,
      { request },
    )
  }

  const memoryId = request.memoryId?.trim() || createMockId('memory')
  const sourceLabel = request.sourceLabel?.trim() || request.sourceId
  const entry: LocalMemoryEntryDto = {
    memoryId,
    projectId: request.projectId ?? null,
    sourceKind: request.sourceKind,
    sourceId: request.sourceId,
    sourceLabel,
    contentSummary: request.contentSummary,
    contentHash: createContentHash(`${request.sourceKind}\n${request.sourceId}\n${request.contentSummary}`),
    embeddingProviderId: request.embeddingProviderId ?? null,
    embeddingModelId: request.embeddingModelId ?? null,
    embeddingVectorPath: request.embeddingVectorPath ?? null,
    metadata: request.metadata ?? {},
    lifecycle: 'active',
    expiresAt: request.expiresAt ?? null,
    createdAt: entries.get(memoryId)?.createdAt ?? MOCK_NOW,
    updatedAt: MOCK_NOW,
  }
  entries.set(memoryId, entry)
  return entry
}

export async function listLocalMemoryEntries(request: ListLocalMemoryEntriesRequest): Promise<LocalMemoryEntryDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LocalMemoryEntryDto[], { request: ListLocalMemoryEntriesRequest }>(
      tauriCommands.listLocalMemoryEntries,
      { request },
    )
  }

  return [...entries.values()].filter((entry) => {
    if (entry.lifecycle !== 'active') return false
    if (!request.projectId) return true
    if (entry.projectId === request.projectId) return true
    return request.includeGlobal !== false && entry.projectId === null
  })
}

export async function createLocalMemoryRetrieval(
  request: CreateLocalMemoryRetrievalRequest,
): Promise<LocalMemoryRetrievalDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LocalMemoryRetrievalDto, { request: CreateLocalMemoryRetrievalRequest }>(
      tauriCommands.createLocalMemoryRetrieval,
      { request },
    )
  }

  const retrievalId = createMockId('memory_retrieval')
  const maxResults = request.maxResults ?? 8
  const seen = new Set<string>()
  const candidates: LocalMemoryRetrievalCandidateDto[] = request.candidates
    .slice()
    .sort((left, right) => right.similarity - left.similarity)
    .filter((candidate) => {
      if (seen.has(candidate.memoryId)) return false
      seen.add(candidate.memoryId)
      return true
    })
    .slice(0, maxResults)
    .map((candidate) => {
      const entry = entries.get(candidate.memoryId)
      if (!entry) throw new Error(`Local memory entry not found: ${candidate.memoryId}`)
      const belowThreshold = candidate.similarity < request.minSimilarity
      return {
        candidateId: createMockId('memory_candidate'),
        retrievalId,
        memoryId: candidate.memoryId,
        similarity: candidate.similarity,
        status: belowThreshold ? 'rejected' : 'waiting_user',
        reason: belowThreshold
          ? `below_min_similarity:${candidate.similarity}<${request.minSimilarity}`
          : candidate.reason ?? null,
        citation: {
          memoryId: entry.memoryId,
          sourceKind: entry.sourceKind,
          sourceId: entry.sourceId,
          sourceLabel: entry.sourceLabel,
          contentHash: entry.contentHash,
        },
        createdAt: MOCK_NOW,
        updatedAt: MOCK_NOW,
      }
    })
  const retrieval: LocalMemoryRetrievalDto = {
    retrievalId,
    projectId: request.projectId,
    queryText: request.queryText,
    minSimilarity: request.minSimilarity,
    maxResults,
    status: 'waiting_user',
    candidates,
    createdAt: MOCK_NOW,
    updatedAt: MOCK_NOW,
  }
  retrievals.set(retrievalId, retrieval)
  return retrieval
}

export async function approveLocalMemoryCandidate(
  request: LocalMemoryCandidateDecisionRequest,
): Promise<LocalMemoryRetrievalCandidateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LocalMemoryRetrievalCandidateDto, { request: LocalMemoryCandidateDecisionRequest }>(
      tauriCommands.approveLocalMemoryCandidate,
      { request },
    )
  }

  return updateCandidate(request.candidateId, 'approved', request.reason ?? null)
}

export async function rejectLocalMemoryCandidate(
  request: LocalMemoryCandidateDecisionRequest,
): Promise<LocalMemoryRetrievalCandidateDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LocalMemoryRetrievalCandidateDto, { request: LocalMemoryCandidateDecisionRequest }>(
      tauriCommands.rejectLocalMemoryCandidate,
      { request },
    )
  }

  return updateCandidate(request.candidateId, 'rejected', request.reason ?? null)
}

export async function buildLocalMemoryContext(request: BuildLocalMemoryContextRequest): Promise<LocalMemoryContextDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LocalMemoryContextDto, { request: BuildLocalMemoryContextRequest }>(
      tauriCommands.buildLocalMemoryContext,
      { request },
    )
  }

  const retrieval = retrievals.get(request.retrievalId)
  if (!retrieval) throw new Error(`Local memory retrieval not found: ${request.retrievalId}`)
  const citations = retrieval.candidates
    .filter((candidate) => candidate.status === 'approved' && candidate.similarity >= retrieval.minSimilarity)
    .map((candidate) => {
      const entry = entries.get(candidate.memoryId)
      if (!entry) throw new Error(`Local memory entry not found: ${candidate.memoryId}`)
      return {
        candidateId: candidate.candidateId,
        memoryId: entry.memoryId,
        sourceKind: entry.sourceKind,
        sourceId: entry.sourceId,
        sourceLabel: entry.sourceLabel,
        similarity: candidate.similarity,
        contentSummary: entry.contentSummary,
        citation: candidate.citation,
      }
    })
  return {
    retrievalId: retrieval.retrievalId,
    projectId: retrieval.projectId,
    minSimilarity: retrieval.minSimilarity,
    usable: citations.length > 0,
    citations,
  }
}

function updateCandidate(
  candidateId: string,
  status: LocalMemoryRetrievalCandidateDto['status'],
  reason: string | null,
): LocalMemoryRetrievalCandidateDto {
  for (const [retrievalId, retrieval] of retrievals.entries()) {
    const index = retrieval.candidates.findIndex((candidate) => candidate.candidateId === candidateId)
    if (index < 0) continue
    const candidate = retrieval.candidates[index]
    if (status === 'approved' && candidate.similarity < retrieval.minSimilarity) {
      throw new Error('candidate similarity is below retrieval threshold and cannot be approved.')
    }
    const nextCandidate = {
      ...candidate,
      status,
      reason: reason ?? candidate.reason,
      updatedAt: MOCK_NOW,
    }
    const nextRetrieval = {
      ...retrieval,
      candidates: [
        ...retrieval.candidates.slice(0, index),
        nextCandidate,
        ...retrieval.candidates.slice(index + 1),
      ],
    }
    retrievals.set(retrievalId, nextRetrieval)
    return nextCandidate
  }
  throw new Error(`Local memory retrieval candidate not found: ${candidateId}`)
}

function createContentHash(value: string): string {
  let hash = 2166136261
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index)
    hash = Math.imul(hash, 16777619)
  }
  return (hash >>> 0).toString(16).padStart(8, '0')
}
