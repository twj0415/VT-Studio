import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type {
  ListTemplateManifestsRequest,
  PreviewTemplateRequest,
  PreviewTemplateResponseDto,
  RenderTemplateRequest,
  RenderTemplateResponseDto,
  TemplateManifestDto,
  TemplateParamValidationResultDto,
  TemplateSidecarStatusDto,
  ValidateTemplateParamsRequest,
} from './types'

export function listTemplateManifests(request: ListTemplateManifestsRequest = {}): Promise<TemplateManifestDto[]> {
  return callCommand<TemplateManifestDto[], { request: ListTemplateManifestsRequest }>(
    tauriCommands.listTemplateManifests,
    { request },
  )
}

export function validateTemplateParams(request: ValidateTemplateParamsRequest): Promise<TemplateParamValidationResultDto> {
  return callCommand<TemplateParamValidationResultDto, { request: ValidateTemplateParamsRequest }>(
    tauriCommands.validateTemplateParams,
    { request },
  )
}

export function previewTemplate(request: PreviewTemplateRequest): Promise<PreviewTemplateResponseDto> {
  return callCommand<PreviewTemplateResponseDto, { request: PreviewTemplateRequest }>(
    tauriCommands.previewTemplate,
    { request },
  )
}

export function renderTemplate(request: RenderTemplateRequest): Promise<RenderTemplateResponseDto> {
  return callCommand<RenderTemplateResponseDto, { request: RenderTemplateRequest }>(
    tauriCommands.renderTemplate,
    { request },
  )
}

export function checkTemplateSidecars(): Promise<TemplateSidecarStatusDto> {
  return callCommand<TemplateSidecarStatusDto>(tauriCommands.checkTemplateSidecars)
}
