import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'
import { createMockId } from '@/shared/mock/ids'

import type { ListLongContentPlansRequest, LongContentPlanDto, LongContentPlanIdRequest, SaveLongContentPlanRequest } from './types'

const MOCK_NOW = '2026-06-22 10:00'
const plansByProjectId = new Map<string, LongContentPlanDto[]>()

export async function saveLongContentPlan(request: SaveLongContentPlanRequest): Promise<LongContentPlanDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LongContentPlanDto, { request: SaveLongContentPlanRequest }>(tauriCommands.saveLongContentPlan, { request })
  }

  const content = JSON.parse(request.rawOutput) as Record<string, unknown>
  const plan: LongContentPlanDto = {
    planId: createMockId('long_plan'),
    projectId: request.projectId,
    planKind: request.planKind,
    parentPlanId: request.parentPlanId ?? null,
    chapterIds: [...request.chapterIds],
    content,
    status: 'waiting_user',
    schemaVersion: 1,
    createdAt: MOCK_NOW,
    updatedAt: MOCK_NOW,
  }
  plansByProjectId.set(request.projectId, [...(plansByProjectId.get(request.projectId) ?? []), plan])
  return plan
}

export async function listLongContentPlans(request: ListLongContentPlansRequest): Promise<LongContentPlanDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LongContentPlanDto[], { request: ListLongContentPlansRequest }>(tauriCommands.listLongContentPlans, { request })
  }

  const plans = plansByProjectId.get(request.projectId) ?? []
  return request.planKind ? plans.filter((plan) => plan.planKind === request.planKind) : plans
}

export async function approveLongContentPlan(request: LongContentPlanIdRequest): Promise<LongContentPlanDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LongContentPlanDto, { request: LongContentPlanIdRequest }>(tauriCommands.approveLongContentPlan, { request })
  }

  return updateMockPlan(request.planId, 'approved')
}

export async function rejectLongContentPlan(request: LongContentPlanIdRequest): Promise<LongContentPlanDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LongContentPlanDto, { request: LongContentPlanIdRequest }>(tauriCommands.rejectLongContentPlan, { request })
  }

  return updateMockPlan(request.planId, 'rejected')
}

function updateMockPlan(planId: string, status: LongContentPlanDto['status']): LongContentPlanDto {
  for (const [projectId, plans] of plansByProjectId.entries()) {
    const index = plans.findIndex((plan) => plan.planId === planId)
    if (index >= 0) {
      const next = { ...plans[index], status, updatedAt: MOCK_NOW }
      plans[index] = next
      plansByProjectId.set(projectId, plans)
      return next
    }
  }
  throw new Error(`Long content plan not found: ${planId}`)
}
