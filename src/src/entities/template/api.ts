import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'

import type {
  ListTemplateManifestsRequest,
  PreviewTemplateRequest,
  PreviewTemplateResponseDto,
  RenderTemplateRequest,
  RenderTemplateResponseDto,
  TemplateManifestDto,
  TemplateParamSchemaDto,
  TemplateParamValidationResultDto,
  TemplateSidecarStatusDto,
  ValidateTemplateParamsRequest,
} from './types'

const mockManifests: TemplateManifestDto[] = [
  {
    templateId: 'knowledge_bold',
    templateType: 'cover',
    sourceType: 'builtin',
    displayName: 'knowledge_bold',
    displayNameKey: 'template.cover.knowledge_bold',
    version: '1.0.0',
    aspectRatio: 'vertical_9_16',
    entryPath: 'templates/builtin/cover/vertical_9_16/knowledge_bold.html',
    viewport: { width: 1080, height: 1920 },
    params: [
      param('cover_title', 'text', ''),
      param('accent_color', 'color', '#FFD54A'),
      param('position', 'select', 'bottom', 'templatePosition'),
    ],
  },
  {
    templateId: 'karaoke_basic',
    templateType: 'subtitle',
    sourceType: 'builtin',
    displayName: 'karaoke_basic',
    displayNameKey: 'template.subtitle.karaoke_basic',
    version: '1.0.0',
    aspectRatio: 'vertical_9_16',
    entryPath: 'templates/builtin/subtitle/vertical_9_16/karaoke_basic.html',
    viewport: { width: 1080, height: 1920 },
    params: [
      param('subtitle_color', 'color', '#ffffff'),
      param('subtitle_safe_bottom', 'number', 240),
      param('subtitle_position', 'select', 'bottom', 'templatePosition'),
    ],
  },
]

export async function listTemplateManifests(request: ListTemplateManifestsRequest = {}): Promise<TemplateManifestDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TemplateManifestDto[], { request: ListTemplateManifestsRequest }>(
      tauriCommands.listTemplateManifests,
      { request },
    )
  }

  return mockManifests.filter((manifest) => {
    if (request.aspectRatio && normalizeAspectRatio(request.aspectRatio) !== manifest.aspectRatio) return false
    if (request.templateType && request.templateType !== manifest.templateType) return false
    if (request.sourceType && request.sourceType !== manifest.sourceType) return false
    return true
  })
}

export async function validateTemplateParams(request: ValidateTemplateParamsRequest): Promise<TemplateParamValidationResultDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TemplateParamValidationResultDto, { request: ValidateTemplateParamsRequest }>(
      tauriCommands.validateTemplateParams,
      { request },
    )
  }

  const normalizedParams: Record<string, unknown> = {}
  const errors: string[] = []
  for (const schema of request.manifest.params) {
    const value = request.params[schema.name] ?? schema.defaultValue
    const error = validateMockParam(schema, value)
    if (error) errors.push(error)
    else normalizedParams[schema.name] = value
  }
  return { valid: errors.length === 0, normalizedParams, errors }
}

export async function previewTemplate(request: PreviewTemplateRequest): Promise<PreviewTemplateResponseDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<PreviewTemplateResponseDto, { request: PreviewTemplateRequest }>(
      tauriCommands.previewTemplate,
      { request },
    )
  }

  return {
    previewPath: `cache/template_preview_${request.templateType}_${request.templateId}.png`,
    width: viewportForAspectRatio(request.aspectRatio).width,
    height: viewportForAspectRatio(request.aspectRatio).height,
  }
}

export async function renderTemplate(request: RenderTemplateRequest): Promise<RenderTemplateResponseDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<RenderTemplateResponseDto, { request: RenderTemplateRequest }>(
      tauriCommands.renderTemplate,
      { request },
    )
  }

  return {
    renderedFramePath: request.outputPath,
    width: viewportForAspectRatio(request.aspectRatio).width,
    height: viewportForAspectRatio(request.aspectRatio).height,
  }
}

export async function checkTemplateSidecars(): Promise<TemplateSidecarStatusDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<TemplateSidecarStatusDto>(tauriCommands.checkTemplateSidecars)
  }

  return {
    ready: false,
    checkedAt: new Date().toISOString(),
    node: mockTemplateSidecar('node.exe'),
    chromium: mockTemplateSidecar('chromium.exe'),
    playwrightDriver: mockTemplateSidecar('playwright-driver.js'),
  }
}

function mockTemplateSidecar(name: string) {
  return {
    name,
    relativePath: `sidecars/${name}`,
    exists: false,
    executable: false,
    errorCode: 'template.sidecar_missing',
    message: 'Mock adapter does not include bundled template sidecars.',
  }
}

function param(name: string, paramType: TemplateParamSchemaDto['paramType'], defaultValue: unknown, dictionaryCode?: string): TemplateParamSchemaDto {
  return {
    name,
    paramType,
    defaultValue,
    required: defaultValue === '',
    dictionaryCode,
  }
}

function validateMockParam(schema: TemplateParamSchemaDto, value: unknown) {
  if (schema.paramType === 'number' || schema.paramType === 'range') {
    return typeof value === 'number' ? undefined : `template.param_invalid: ${schema.name} must be a number.`
  }
  if (schema.paramType === 'bool') {
    return typeof value === 'boolean' ? undefined : `template.param_invalid: ${schema.name} must be boolean.`
  }
  if (schema.paramType === 'color') {
    return typeof value === 'string' && /^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.test(value)
      ? undefined
      : `template.param_invalid: ${schema.name} must be a hex color.`
  }
  if (schema.paramType === 'json') return undefined
  return typeof value === 'string' ? undefined : `template.param_invalid: ${schema.name} must be text.`
}

function normalizeAspectRatio(value: string) {
  if (value === '9:16') return 'vertical_9_16'
  if (value === '16:9') return 'horizontal_16_9'
  if (value === '1:1') return 'square_1_1'
  return value
}

function viewportForAspectRatio(value: string) {
  const aspectRatio = normalizeAspectRatio(value)
  if (aspectRatio === 'horizontal_16_9') return { width: 1920, height: 1080 }
  if (aspectRatio === 'square_1_1') return { width: 1080, height: 1080 }
  return { width: 1080, height: 1920 }
}
