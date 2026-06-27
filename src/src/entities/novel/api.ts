import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type { ImportNovelRequest, ImportNovelResultDto, MarkNovelChapterEventFailedRequest, NovelChapterDto, RetryNovelChapterEventRequest, UpdateNovelChapterEventRequest } from './types'

export function importNovel(request: ImportNovelRequest): Promise<ImportNovelResultDto> {
  return callCommand<ImportNovelResultDto, { request: ImportNovelRequest }>(tauriCommands.importNovel, { request })
}

export function listNovelChapters(projectId: string): Promise<NovelChapterDto[]> {
  return callCommand<NovelChapterDto[]>(tauriCommands.listNovelChapters, { projectId })
}

export function updateNovelChapterEvent(request: UpdateNovelChapterEventRequest): Promise<NovelChapterDto> {
  return callCommand<NovelChapterDto, { request: UpdateNovelChapterEventRequest }>(tauriCommands.updateNovelChapterEvent, { request })
}

export function markNovelChapterEventFailed(request: MarkNovelChapterEventFailedRequest): Promise<NovelChapterDto> {
  return callCommand<NovelChapterDto, { request: MarkNovelChapterEventFailedRequest }>(tauriCommands.markNovelChapterEventFailed, { request })
}

export function retryNovelChapterEvent(request: RetryNovelChapterEventRequest): Promise<NovelChapterDto> {
  return callCommand<NovelChapterDto, { request: RetryNovelChapterEventRequest }>(tauriCommands.retryNovelChapterEvent, { request })
}
