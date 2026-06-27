import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type { ListLongContentPlansRequest, LongContentPlanDto, LongContentPlanIdRequest, SaveLongContentPlanRequest } from './types'

export function saveLongContentPlan(request: SaveLongContentPlanRequest): Promise<LongContentPlanDto> {
  return callCommand<LongContentPlanDto, { request: SaveLongContentPlanRequest }>(tauriCommands.saveLongContentPlan, { request })
}

export function listLongContentPlans(request: ListLongContentPlansRequest): Promise<LongContentPlanDto[]> {
  return callCommand<LongContentPlanDto[], { request: ListLongContentPlansRequest }>(tauriCommands.listLongContentPlans, { request })
}

export function approveLongContentPlan(request: LongContentPlanIdRequest): Promise<LongContentPlanDto> {
  return callCommand<LongContentPlanDto, { request: LongContentPlanIdRequest }>(tauriCommands.approveLongContentPlan, { request })
}

export function rejectLongContentPlan(request: LongContentPlanIdRequest): Promise<LongContentPlanDto> {
  return callCommand<LongContentPlanDto, { request: LongContentPlanIdRequest }>(tauriCommands.rejectLongContentPlan, { request })
}
