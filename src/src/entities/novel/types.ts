export interface NovelChapterDto {
  novelChapterId: string
  projectId: string
  chapterIndex: number
  volumeTitle: string | null
  chapterTitle: string
  chapterContent: string
  structuredEvent: Record<string, unknown>
  eventStatus: 'pending' | 'running' | 'succeeded' | 'failed' | string
  errorReason: string | null
  retryCount: number
  createdAt: string
  updatedAt: string
}

export interface ImportNovelRequest {
  projectId: string
  rawText: string
}

export interface ImportNovelResultDto {
  projectId: string
  sourceTextPath: string
  chapters: NovelChapterDto[]
}

export interface UpdateNovelChapterEventRequest {
  novelChapterId: string
  structuredEvent: Record<string, unknown>
}

export interface MarkNovelChapterEventFailedRequest {
  novelChapterId: string
  errorReason: string
}

export interface RetryNovelChapterEventRequest {
  novelChapterId: string
}
