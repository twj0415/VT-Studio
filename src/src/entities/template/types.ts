export type TemplateType = 'frame' | 'cover' | 'subtitle' | 'transition' | 'layout'
export type TemplateSourceType = 'builtin' | 'user'
export type TemplateParamType = 'text' | 'number' | 'color' | 'bool' | 'select' | 'image' | 'font' | 'range' | 'json'

export interface ListTemplateManifestsRequest {
  aspectRatio?: string
  templateType?: TemplateType
  sourceType?: TemplateSourceType
}

export interface TemplateViewportDto {
  width: number
  height: number
}

export interface TemplateParamSchemaDto {
  name: string
  paramType: TemplateParamType
  defaultValue: unknown
  required: boolean
  dictionaryCode?: string | null
  min?: number | null
  max?: number | null
}

export interface TemplateManifestDto {
  templateId: string
  templateType: TemplateType
  sourceType: TemplateSourceType
  displayName: string
  displayNameKey: string
  version: string
  aspectRatio: string
  entryPath: string
  viewport: TemplateViewportDto
  params: TemplateParamSchemaDto[]
}

export interface ValidateTemplateParamsRequest {
  manifest: TemplateManifestDto
  params: Record<string, unknown>
}

export interface TemplateParamValidationResultDto {
  valid: boolean
  normalizedParams: Record<string, unknown>
  errors: string[]
}

export interface TemplateRenderDataDto {
  title?: string
  narration?: string
  subtitleChunks?: string[]
  imagePath?: string
  videoFramePath?: string
  characterNames?: string[]
}

export interface PreviewTemplateRequest {
  templateId: string
  aspectRatio: string
  templateType: TemplateType
  params: Record<string, unknown>
  data: TemplateRenderDataDto
}

export interface RenderTemplateRequest extends PreviewTemplateRequest {
  outputPath: string
}

export interface PreviewTemplateResponseDto {
  previewPath: string
  width: number
  height: number
}

export interface RenderTemplateResponseDto {
  renderedFramePath: string
  width: number
  height: number
}

export interface TemplateSidecarBinaryStatusDto {
  name: string
  relativePath: string
  exists: boolean
  executable: boolean
  version?: string | null
  errorCode?: string | null
  message?: string | null
}

export interface TemplateSidecarStatusDto {
  node: TemplateSidecarBinaryStatusDto
  chromium: TemplateSidecarBinaryStatusDto
  playwrightDriver: TemplateSidecarBinaryStatusDto
  ready: boolean
  checkedAt: string
}
