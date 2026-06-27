import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

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

export function upsertLocalMemoryEntry(request: UpsertLocalMemoryEntryRequest): Promise<LocalMemoryEntryDto> {
  return callCommand<LocalMemoryEntryDto, { request: UpsertLocalMemoryEntryRequest }>(
    tauriCommands.upsertLocalMemoryEntry,
    { request },
  )
}

export function listLocalMemoryEntries(request: ListLocalMemoryEntriesRequest): Promise<LocalMemoryEntryDto[]> {
  return callCommand<LocalMemoryEntryDto[], { request: ListLocalMemoryEntriesRequest }>(
    tauriCommands.listLocalMemoryEntries,
    { request },
  )
}

export function createLocalMemoryRetrieval(
  request: CreateLocalMemoryRetrievalRequest,
): Promise<LocalMemoryRetrievalDto> {
  return callCommand<LocalMemoryRetrievalDto, { request: CreateLocalMemoryRetrievalRequest }>(
    tauriCommands.createLocalMemoryRetrieval,
    { request },
  )
}

export function approveLocalMemoryCandidate(
  request: LocalMemoryCandidateDecisionRequest,
): Promise<LocalMemoryRetrievalCandidateDto> {
  return callCommand<LocalMemoryRetrievalCandidateDto, { request: LocalMemoryCandidateDecisionRequest }>(
    tauriCommands.approveLocalMemoryCandidate,
    { request },
  )
}

export function rejectLocalMemoryCandidate(
  request: LocalMemoryCandidateDecisionRequest,
): Promise<LocalMemoryRetrievalCandidateDto> {
  return callCommand<LocalMemoryRetrievalCandidateDto, { request: LocalMemoryCandidateDecisionRequest }>(
    tauriCommands.rejectLocalMemoryCandidate,
    { request },
  )
}

export function buildLocalMemoryContext(request: BuildLocalMemoryContextRequest): Promise<LocalMemoryContextDto> {
  return callCommand<LocalMemoryContextDto, { request: BuildLocalMemoryContextRequest }>(
    tauriCommands.buildLocalMemoryContext,
    { request },
  )
}
