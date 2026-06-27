export type LongContentPlanKind =
  | 'story_skeleton'
  | 'adaptation_strategy'
  | 'episode_script'
  | 'storyboard_table'
  | 'asset_extraction'

export interface LongContentPlanDto {
  planId: string
  projectId: string
  planKind: LongContentPlanKind | string
  parentPlanId: string | null
  chapterIds: string[]
  content: Record<string, unknown>
  status: 'waiting_user' | 'approved' | 'rejected' | string
  schemaVersion: number
  createdAt: string
  updatedAt: string
}

export interface SaveLongContentPlanRequest {
  projectId: string
  planKind: LongContentPlanKind
  parentPlanId?: string | null
  chapterIds: string[]
  rawOutput: string
}

export interface ListLongContentPlansRequest {
  projectId: string
  planKind?: LongContentPlanKind
}

export interface LongContentPlanIdRequest {
  planId: string
}
