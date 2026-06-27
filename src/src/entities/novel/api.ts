import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'
import { createMockId } from '@/shared/mock/ids'

import type { ImportNovelRequest, ImportNovelResultDto, MarkNovelChapterEventFailedRequest, NovelChapterDto, RetryNovelChapterEventRequest, UpdateNovelChapterEventRequest } from './types'

const MOCK_NOW = '2026-06-22 10:00'
const novelChaptersByProjectId = new Map<string, NovelChapterDto[]>()

export async function importNovel(request: ImportNovelRequest): Promise<ImportNovelResultDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ImportNovelResultDto, { request: ImportNovelRequest }>(tauriCommands.importNovel, { request })
  }

  const chapters = splitNovelChapters(request.projectId, request.rawText)
  novelChaptersByProjectId.set(request.projectId, chapters)
  return {
    projectId: request.projectId,
    sourceTextPath: `projects/${request.projectId}/input/source.txt`,
    chapters,
  }
}

export async function listNovelChapters(projectId: string): Promise<NovelChapterDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<NovelChapterDto[]>(tauriCommands.listNovelChapters, { projectId })
  }

  return novelChaptersByProjectId.get(projectId) ?? []
}

export async function updateNovelChapterEvent(request: UpdateNovelChapterEventRequest): Promise<NovelChapterDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<NovelChapterDto, { request: UpdateNovelChapterEventRequest }>(tauriCommands.updateNovelChapterEvent, { request })
  }

  return updateMockChapter(request.novelChapterId, (chapter) => ({
    ...chapter,
    structuredEvent: request.structuredEvent,
    eventStatus: 'succeeded',
    errorReason: null,
    updatedAt: MOCK_NOW,
  }))
}

export async function markNovelChapterEventFailed(request: MarkNovelChapterEventFailedRequest): Promise<NovelChapterDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<NovelChapterDto, { request: MarkNovelChapterEventFailedRequest }>(tauriCommands.markNovelChapterEventFailed, { request })
  }

  return updateMockChapter(request.novelChapterId, (chapter) => ({
    ...chapter,
    eventStatus: 'failed',
    errorReason: request.errorReason,
    retryCount: chapter.retryCount + 1,
    updatedAt: MOCK_NOW,
  }))
}

export async function retryNovelChapterEvent(request: RetryNovelChapterEventRequest): Promise<NovelChapterDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<NovelChapterDto, { request: RetryNovelChapterEventRequest }>(tauriCommands.retryNovelChapterEvent, { request })
  }

  return updateMockChapter(request.novelChapterId, (chapter) => ({
    ...chapter,
    eventStatus: 'pending',
    errorReason: null,
    updatedAt: MOCK_NOW,
  }))
}

function splitNovelChapters(projectId: string, rawText: string): NovelChapterDto[] {
  const lines = rawText.split(/\r?\n/)
  const sections: Array<{ title: string, content: string[] }> = []
  let current: { title: string, content: string[] } = { title: '', content: [] }

  for (const line of lines) {
    const trimmed = line.trim()
    if (isChapterHeading(trimmed)) {
      if (current.title || current.content.length > 0) sections.push(current)
      current = { title: trimmed, content: [] }
    } else {
      current.content.push(line)
    }
  }
  if (current.title || current.content.length > 0) sections.push(current)

  const usable = sections.filter((section) => section.content.join('\n').trim())
  const finalSections = usable.length > 1 ? usable : [{ title: 'Chapter 1', content: [rawText] }]
  return finalSections.map((section, index) => ({
    novelChapterId: createMockId('chapter'),
    projectId,
    chapterIndex: index + 1,
    volumeTitle: null,
    chapterTitle: section.title || `Chapter ${index + 1}`,
    chapterContent: section.content.join('\n').trim(),
    structuredEvent: {},
    eventStatus: 'pending',
    errorReason: null,
    retryCount: 0,
    createdAt: MOCK_NOW,
    updatedAt: MOCK_NOW,
  }))
}

function isChapterHeading(line: string): boolean {
  const lower = line.toLowerCase()
  return line.length <= 80 && ((line.startsWith('第') && /[章节回]/.test(line)) || lower.startsWith('chapter '))
}

function updateMockChapter(chapterId: string, updater: (chapter: NovelChapterDto) => NovelChapterDto): NovelChapterDto {
  for (const [projectId, chapters] of novelChaptersByProjectId.entries()) {
    const index = chapters.findIndex((chapter) => chapter.novelChapterId === chapterId)
    if (index >= 0) {
      const next = updater(chapters[index])
      chapters[index] = next
      novelChaptersByProjectId.set(projectId, chapters)
      return next
    }
  }
  throw new Error(`Novel chapter not found: ${chapterId}`)
}
