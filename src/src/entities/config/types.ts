import type { AppLocale, LayoutDensity, ThemePreset } from '@/shared/stores/appStore'
import type { ModelCapability, ProviderAuthType, ProviderKind, ProviderStatus, ProviderVendor } from '@/shared/enums/generated'
import type { TemplateSidecarStatusDto } from '@/entities/template/types'

export interface AppConfigDto {
  appLocale: AppLocale
  themePreset: ThemePreset
  layoutDensity: LayoutDensity
}

export interface SaveProviderSecretRequest {
  providerId: string
  authType: string
  keyAlias?: string
  secret: string
}

export interface ProviderSecretHandleDto {
  providerId: string
  authType: string
  keyAlias: string
  hasSecret: boolean
}

export interface ProviderSecretStatusDto {
  keyAlias: string
  hasSecret: boolean
}

export interface ProviderConfigDto {
  providerId: string
  providerKind: ProviderKind
  vendor: ProviderVendor | string
  displayName: string
  baseUrl?: string
  authType: ProviderAuthType | 'oauth'
  keyAlias?: string
  status: ProviderStatus | 'unconfigured' | 'ready' | 'testing' | 'failed'
  isEnabled: boolean
  config: Record<string, unknown>
}

export interface ListProviderConfigsRequest {
  providerKind?: ProviderKind
}

export interface DeleteProviderConfigRequest {
  providerId: string
}

export interface ProviderModelDto {
  modelId: string
  providerId: string
  providerKind: Exclude<ProviderKind, 'workflow'>
  vendor: ProviderVendor | string
  providerModelId: string
  modelName: string
  displayName: string
  abilityTypes: ModelCapability[] | string[]
  inputModalities: string[]
  outputModalities: string[]
  featureFlags: string[]
  limits: Record<string, unknown>
  inputRequirements: Record<string, unknown> | unknown[]
  apiContractVerified: boolean
  status: 'unconfigured' | 'ready' | 'testing' | 'failed' | 'disabled'
  isEnabled: boolean
  config: Record<string, unknown>
}

export interface ListProviderModelsRequest {
  providerId?: string
  providerKind?: ProviderKind
}

export interface DeleteProviderModelRequest {
  modelId: string
}

export interface WorkflowPresetDto {
  workflowPresetId: string
  providerId: string
  vendor: 'comfyui' | 'runninghub'
  workflowKey: string
  workflowId?: string
  displayName: string
  workflowVersion: string
  abilityTypes: string[]
  inputModalities: string[]
  outputModalities: string[]
  limits: Record<string, unknown>
  paramSchema: Record<string, unknown>
  nodeMap: Record<string, string>
  outputMap: Record<string, string>
  defaultParams: Record<string, unknown>
  status: 'unconfigured' | 'ready' | 'testing' | 'failed' | 'disabled'
  isBuiltin: boolean
  isEnabled: boolean
  config: Record<string, unknown>
}

export interface ListWorkflowPresetsRequest {
  providerId?: string
  vendor?: 'comfyui' | 'runninghub'
  abilityType?: string
}

export interface DeleteWorkflowPresetRequest {
  workflowPresetId: string
}

export interface MediaInputRequirementDto {
  inputKey: string
  inputGroup: 'text' | 'image' | 'video' | 'audio' | 'workflow_param' | string
  ownerType?: string
  ownerId?: string
  requirement: 'required' | 'optional' | 'unused' | string
  sourceOptions: string[]
  missingReason?: string
  uiSchema: Record<string, unknown>
  constraints: Record<string, unknown>
  normalizedParams: Record<string, unknown>
}

export interface MediaInputPlanDto {
  planKind: 'image' | 'video' | 'audio' | 'text' | string
  abilityType: string
  imageKind?: string
  assetKind?: string
  items: MediaInputRequirementDto[]
  requiredCount: number
  optionalCount: number
  unusedCount: number
}

export interface ExecutableMediaOptionDto {
  optionId: string
  sourceType: 'provider_model' | 'workflow_preset'
  sourceId: string
  label: string
  providerId: string
  providerKind: ProviderKind
  vendor: ProviderVendor | string
  kind: 'provider_model' | 'workflow_preset'
  capability: string
  capabilities: string[]
  constraints: Record<string, unknown>
  inputPlan: MediaInputPlanDto
  status: string
  providerModelId?: string
  workflowPresetId?: string
  enabled: boolean
  disabledReason?: string
  normalizedParams: Record<string, unknown>
}

export interface SidecarBinaryStatusDto {
  name: string
  relativePath: string
  exists: boolean
  executable: boolean
  version?: string
  errorCode?: string
  message?: string
}

export interface FfmpegSidecarStatusDto {
  ffmpeg: SidecarBinaryStatusDto
  ffprobe: SidecarBinaryStatusDto
  ready: boolean
  checkedAt: string
}

export interface AppReleaseInfoDto {
  appName: string
  productName: string
  version: string
  identifier: string
  targetPlatform: string
  updateChannel: string
  autoUpdateEnabled: boolean
  updateFeedUrl?: string | null
  updateRequiresHttps: boolean
  updateSignatureRequired: boolean
  installerSignatureRequired: boolean
  signedInstallerVerified: boolean
}

export interface RuntimeCheckItemDto {
  key: string
  ready: boolean
  skipped: boolean
  errorCode?: string | null
  message?: string | null
}

export interface RuntimeSelfCheckDto {
  ready: boolean
  checkedAt: string
  workspace: RuntimeCheckItemDto
  sqlite: RuntimeCheckItemDto
  ffmpeg: FfmpegSidecarStatusDto
  templateSidecar: TemplateSidecarStatusDto
  templateManifest: RuntimeCheckItemDto
  templatePreview: RuntimeCheckItemDto
}

export interface ProbeMediaRequest {
  relativePath: string
  mediaKind?: string
}

export interface MediaProbeDto {
  path: string
  mediaKind: string
  container?: string
  formatName?: string
  durationSeconds: number
  width?: number
  height?: number
  fps?: number
  videoCodec?: string
  pixelFormat?: string
  audioCodec?: string
  sampleRate?: number
  channels?: number
  bitRate?: number
  hasVideoStream: boolean
  hasAudioStream: boolean
}

export interface AssetPreviewRequest {
  assetId: string
}

export interface AssetPreviewDto {
  assetId: string
  relativePath: string
  mediaKind: string
  mimeType: string
  previewKind: 'image' | 'video_frame' | string
  bytes: number[]
}

export interface ProviderDryRunRequest {
  providerId: string
  providerKind: ProviderKind
  providerModelId?: string
  workflowPresetId?: string
  simulateFailure?: boolean
  simulateCancelled?: boolean
}

export interface ProviderDryRunResponse {
  traceId: string
  providerId: string
  providerKind: ProviderKind
  status: 'succeeded'
  message: string
  outputSummary: Record<string, unknown>
}

export type ProviderGenerationTestMode = 'dry_run' | 'real_generate'

export interface ProviderGenerationTestRequest {
  providerId: string
  providerKind: ProviderKind
  providerModelId?: string
  workflowPresetId?: string
  testMode: ProviderGenerationTestMode
  realGenerateConfirmed?: boolean
  confirmToken?: string
  simulateFailure?: boolean
  simulateCancelled?: boolean
}

export interface ProviderGenerationTestResponse {
  traceId: string
  testMode: ProviderGenerationTestMode
  providerId: string
  providerKind: ProviderKind
  status: 'succeeded'
  message: string
  outputSummary: Record<string, unknown>
  billable: boolean
  realGenerateConfirmed: boolean
}

export interface CreativeRuleDto {
  ruleId: string
  key: string
  name: string
  module: string
  ruleType: string
  providerKind: ProviderKind
  version: string
  outputSchema: Record<string, unknown>
  paramsSchema: Record<string, unknown>
  description: string
  sourceType: 'builtin' | 'user'
  enabled: boolean
  body: string
  relativePath: string
  contentHash: string
  schemaHash: string
  referenceCounts: CreativeRuleReferenceCountsDto
}

export interface CreativeRuleReferenceCountsDto {
  videoPacks: number
  projects: number
  taskSteps: number
  generationContexts: number
}

export interface CreativeRuleRefDto {
  slot: string
  ruleKey: string
  ruleId: string
  sourceType: 'builtin' | 'user'
  ruleType: string
  module: string
  name: string
  version: string
  contentHash: string
  schemaHash: string
  enabled: boolean
}

export interface ListCreativeRulesRequest {
  sourceType?: 'builtin' | 'user'
  module?: string
}

export interface CreativeRuleIdRequest {
  ruleId: string
}

export interface SaveCreativeRuleRequest {
  key: string
  name: string
  module: string
  ruleType: string
  providerKind: ProviderKind
  version?: string
  outputSchema: Record<string, unknown>
  paramsSchema?: Record<string, unknown>
  description: string
  enabled: boolean
  body: string
}

export interface SetCreativeRuleEnabledRequest {
  ruleId: string
  enabled: boolean
}

export interface ValidateStructuredOutputRequest {
  rawOutput: string
  outputSchema: Record<string, unknown>
  expectedCount?: number
  repairAttemptCount?: number
  maxRepairAttempts?: number
}

export interface StructuredOutputValidationResult {
  valid: boolean
  parsedJson?: unknown
  errors: string[]
  repairNeeded: boolean
  attemptCount: number
  maxAttempts: number
}

export interface VideoPackDto {
  packId: string
  sourceType: 'builtin' | 'user'
  name: string
  description: string
  applicableInputTypes: string[]
  contentCategory?: string
  defaultTone?: string
  defaultAspectRatio: string
  defaultDurationSeconds: number
  defaultSceneCount: number
  ruleRefs: Record<string, CreativeRuleRefDto>
  recommendedExecutableRefs: Record<string, unknown>
  assetRefs: unknown[]
  isEnabled: boolean
  createdAt?: string
  updatedAt?: string
  projectReferenceCount: number
}

export interface ListVideoPacksRequest {
  sourceType?: 'builtin' | 'user'
  includeDisabled?: boolean
}

export interface VideoPackIdRequest {
  packId: string
}

export interface UpsertUserVideoPackRequest {
  packId?: string
  name: string
  description: string
  applicableInputTypes: string[]
  contentCategory?: string
  defaultTone?: string
  defaultAspectRatio: string
  defaultDurationSeconds: number
  defaultSceneCount: number
  ruleRefs: Record<string, CreativeRuleRefDto>
  recommendedExecutableRefs: Record<string, unknown>
  assetRefs: unknown[]
  isEnabled: boolean
}

export interface SetVideoPackEnabledRequest {
  packId: string
  isEnabled: boolean
}

export interface SaveProjectConfigAsVideoPackRequest {
  projectId: string
  name: string
  description: string
}

export interface AssetDto {
  assetId: string
  kind: string
  relativePath: string
  sourceKind: string
  mimeType?: string
  sizeBytes?: number
  checksum?: string
  isBuiltin: boolean
  lifecycle: string
  metadata: Record<string, unknown>
  createdAt?: string
  updatedAt?: string
}

export interface ProviderMediaInputDto {
  path: string
  role: string
  weight?: number
}

export interface StyleBibleDto {
  styleBibleId: string
  projectId: string
  name: string
  stylePrompt: string
  colorPalette: string[]
  lighting: string
  composition: string
  negativePrompt: string
  referenceImagePath?: string | null
  referenceImages: Record<string, unknown>[]
  data: Record<string, unknown>
  createdAt?: string
  updatedAt?: string
}

export interface StylePresetDto {
  presetId: string
  sourceType: 'builtin' | 'user'
  name: string
  stylePrompt: string
  colorPalette: string[]
  lighting: string
  composition: string
  negativePrompt: string
  referenceImagePath?: string | null
}

export interface UpsertProjectStyleBibleRequest {
  projectId: string
  styleBibleId?: string
  name: string
  stylePrompt: string
  colorPalette: string[]
  lighting: string
  composition: string
  negativePrompt: string
  referenceImagePath?: string | null
  referenceImages?: Record<string, unknown>[]
  saveAsPreset?: boolean
  presetName?: string
}

export interface ApplyStylePresetRequest {
  projectId: string
  presetId: string
}

export interface BindStyleReferenceAssetRequest {
  projectId: string
  styleBibleId?: string
  assetId: string
}

export interface BindStyleReferenceAssetResponse {
  styleBible: StyleBibleDto
  referenceId: string
}

export interface AnalyzeStyleReferenceRequest {
  projectId: string
  styleBibleId?: string
  providerId?: string
  providerModelId?: string
}

export interface StyleReferenceAnalysisDto {
  projectId: string
  styleBibleId: string
  referenceImagePath: string
  stylePrompt: string
  colorPalette: string[]
  lighting: string
  composition: string
  negativePromptSuggestion: string
  warnings: string[]
  rawDescription: string
  providerTraceId?: string
  providerId?: string
  providerModelId?: string
}

export interface CharacterBibleDto {
  characterBibleId: string
  projectId: string
  characterId: string
  name: string
  alias: string[]
  age: string
  gender: string
  appearance: string
  clothing: string
  personality: string
  visualPrompt: string
  negativePrompt: string
  referenceImagePath?: string | null
  referenceImages: Record<string, unknown>[]
  lockFlags: Record<string, unknown>
  data: Record<string, unknown>
  createdAt?: string
  updatedAt?: string
}

export interface UpsertProjectCharacterBibleRequest {
  projectId: string
  characterId?: string
  name: string
  alias: string[]
  age: string
  gender: string
  appearance: string
  clothing: string
  personality: string
  visualPrompt?: string
  negativePrompt?: string
  referenceImagePath?: string | null
  referenceImages?: Record<string, unknown>[]
  lockFlags?: Record<string, unknown>
}

export interface CharacterBibleIdRequest {
  characterId: string
}

export interface BindCharacterReferenceAssetRequest {
  projectId: string
  characterId: string
  assetId: string
  referenceRole?: string
}

export interface BindCharacterReferenceAssetResponse {
  characterBible: CharacterBibleDto
  reference: AssetReferenceDto
}

export interface LocationBibleDto {
  locationBibleId: string
  projectId: string
  locationId: string
  name: string
  spaceDescription: string
  lighting: string
  timeOfDay: string
  props: string[]
  visualPrompt: string
  negativePrompt: string
  referenceImagePath?: string | null
  referenceImages: Record<string, unknown>[]
  variants: Record<string, unknown>[]
  data: Record<string, unknown>
  createdAt?: string
  updatedAt?: string
}

export interface UpsertProjectLocationBibleRequest {
  projectId: string
  locationId?: string
  name: string
  spaceDescription: string
  lighting: string
  timeOfDay: string
  props: string[]
  visualPrompt?: string
  negativePrompt?: string
  referenceImagePath?: string | null
  referenceImages?: Record<string, unknown>[]
  variants?: Record<string, unknown>[]
}

export interface LocationBibleIdRequest {
  locationId: string
}

export interface BindLocationReferenceAssetRequest {
  projectId: string
  locationId: string
  assetId: string
  referenceRole?: string
}

export interface BindLocationReferenceAssetResponse {
  locationBible: LocationBibleDto
  reference: AssetReferenceDto
}

export interface BuildImagePromptPreviewRequest {
  projectId: string
  itemId: string
}

export interface PromptSectionDto {
  key: string
  label: string
  content: string
}

export interface ImagePromptPreviewDto {
  projectId: string
  itemId: string
  finalPrompt: string
  finalNegativePrompt: string
  sections: PromptSectionDto[]
  referenceImages: ProviderMediaInputDto[]
  styleBible?: StyleBibleDto | null
  characterBibles: CharacterBibleDto[]
  locationBible?: LocationBibleDto | null
  negativePromptTruncated: boolean
  negativePromptMaxLength: number
}

export interface AssetReferenceDto {
  referenceId: string
  assetId: string
  ownerKind: string
  ownerId: string
  usageKind: string
  createdAt?: string
}

export interface ImportAssetRequest {
  sourcePath: string
  kind: string
  displayName?: string
  mimeType?: string
  metadata?: Record<string, unknown>
}

export interface ListAssetsRequest {
  kind?: string
  includeDeleted?: boolean
}

export interface DeleteAssetRequest {
  assetId: string
  physical?: boolean
}

export interface DeleteAssetReferenceRequest {
  referenceId: string
}

export interface CreateAssetReferenceRequest {
  assetId: string
  ownerKind: string
  ownerId: string
  usageKind: string
}
