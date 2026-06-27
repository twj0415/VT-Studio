import { callCommand } from '@/shared/api/client'
import { getApiAdapter } from '@/shared/api/invoke'
import { tauriCommands } from '@/shared/api/commands'

import type {
  AppConfigDto,
  AppReleaseInfoDto,
  AssetDto,
  AssetPreviewDto,
  AssetPreviewRequest,
  AssetReferenceDto,
  BindCharacterReferenceAssetRequest,
  BindCharacterReferenceAssetResponse,
  BindLocationReferenceAssetRequest,
  BindLocationReferenceAssetResponse,
  CreateAssetReferenceRequest,
  CharacterBibleDto,
  CharacterBibleIdRequest,
  CreativeRuleDto,
  CreativeRuleIdRequest,
  DeleteAssetReferenceRequest,
  DeleteProviderConfigRequest,
  DeleteProviderModelRequest,
  DeleteWorkflowPresetRequest,
  DeleteAssetRequest,
  ExecutableMediaOptionDto,
  FfmpegSidecarStatusDto,
  ImportAssetRequest,
  ListProviderConfigsRequest,
  ListProviderModelsRequest,
  ListWorkflowPresetsRequest,
  ListAssetsRequest,
  ListCreativeRulesRequest,
  LocationBibleDto,
  LocationBibleIdRequest,
  ProviderDryRunRequest,
  ProviderDryRunResponse,
  ProviderGenerationTestRequest,
  ProviderGenerationTestResponse,
  MediaProbeDto,
  ProbeMediaRequest,
  ProviderConfigDto,
  ProviderModelDto,
  ProviderSecretHandleDto,
  ProviderSecretStatusDto,
  RuntimeSelfCheckDto,
  SaveProviderSecretRequest,
  SaveCreativeRuleRequest,
  SetCreativeRuleEnabledRequest,
  StructuredOutputValidationResult,
  ListVideoPacksRequest,
  ApplyStylePresetRequest,
  AnalyzeStyleReferenceRequest,
  BindStyleReferenceAssetRequest,
  BindStyleReferenceAssetResponse,
  BuildImagePromptPreviewRequest,
  SaveProjectConfigAsVideoPackRequest,
  ValidateStructuredOutputRequest,
  SetVideoPackEnabledRequest,
  ImagePromptPreviewDto,
  StyleBibleDto,
  StylePresetDto,
  StyleReferenceAnalysisDto,
  UpsertUserVideoPackRequest,
  UpsertProjectCharacterBibleRequest,
  UpsertProjectLocationBibleRequest,
  UpsertProjectStyleBibleRequest,
  VideoPackDto,
  VideoPackIdRequest,
  WorkflowPresetDto,
} from './types'

let config: AppConfigDto = {
  appLocale: 'zh-CN',
  themePreset: 'graphite',
  layoutDensity: 'comfortable',
}

const providerSecretsByAlias = new Map<string, string>()

export async function getAppConfig(): Promise<AppConfigDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AppConfigDto>(tauriCommands.getAppConfig)
  }

  return config
}

export async function updateAppConfig(patch: Partial<AppConfigDto>): Promise<AppConfigDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AppConfigDto, { config: AppConfigDto }>(tauriCommands.updateAppConfig, { config: { ...config, ...patch } })
  }

  config = { ...config, ...patch }
  return config
}

export async function saveProviderSecret(
  request: SaveProviderSecretRequest,
): Promise<ProviderSecretHandleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderSecretHandleDto, { request: SaveProviderSecretRequest }>(tauriCommands.saveProviderSecret, { request })
  }

  const keyAlias = request.keyAlias?.trim() || `${request.providerId}:${request.authType}`
  providerSecretsByAlias.set(keyAlias, request.secret)
  return {
    providerId: request.providerId,
    authType: request.authType,
    keyAlias,
    hasSecret: request.secret.length > 0,
  }
}

export async function deleteProviderSecret(keyAlias: string): Promise<ProviderSecretStatusDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderSecretStatusDto, { request: { keyAlias: string } }>(tauriCommands.deleteProviderSecret, { request: { keyAlias } })
  }

  providerSecretsByAlias.delete(keyAlias)
  return { keyAlias, hasSecret: false }
}

export async function hasProviderSecret(keyAlias: string): Promise<ProviderSecretStatusDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderSecretStatusDto, { request: { keyAlias: string } }>(tauriCommands.hasProviderSecret, { request: { keyAlias } })
  }

  return { keyAlias, hasSecret: providerSecretsByAlias.has(keyAlias) }
}

const providerConfigs: ProviderConfigDto[] = []
const providerModels: ProviderModelDto[] = []
const workflowPresets: WorkflowPresetDto[] = []
const creativeRules: CreativeRuleDto[] = []
const videoPacks: VideoPackDto[] = []
const styleBiblesByProjectId = new Map<string, StyleBibleDto>()
const characterBiblesByProjectId = new Map<string, CharacterBibleDto[]>()
const locationBiblesByProjectId = new Map<string, LocationBibleDto[]>()
const userStylePresets: StylePresetDto[] = []

export async function checkFfmpegSidecars(): Promise<FfmpegSidecarStatusDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<FfmpegSidecarStatusDto>(tauriCommands.checkFfmpegSidecars)
  }

  return {
    ready: false,
    checkedAt: new Date().toISOString(),
    ffmpeg: {
      name: 'ffmpeg.exe',
      relativePath: 'sidecars/ffmpeg.exe',
      exists: false,
      executable: false,
      errorCode: 'ffmpeg.not_found',
      message: 'Mock adapter does not include bundled FFmpeg sidecars.',
    },
    ffprobe: {
      name: 'ffprobe.exe',
      relativePath: 'sidecars/ffprobe.exe',
      exists: false,
      executable: false,
      errorCode: 'ffmpeg.not_found',
      message: 'Mock adapter does not include bundled FFmpeg sidecars.',
    },
  }
}

export async function getAppReleaseInfo(): Promise<AppReleaseInfoDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AppReleaseInfoDto>(tauriCommands.getAppReleaseInfo)
  }

  return {
    appName: 'vt-ai-short-video-maker-app',
    productName: 'VT AI Short Video Maker',
    version: '0.1.0',
    identifier: 'com.vt-ai-short-video-maker.app',
    targetPlatform: 'windows-x64',
    updateChannel: 'manual',
    autoUpdateEnabled: false,
    updateFeedUrl: null,
    updateRequiresHttps: true,
    updateSignatureRequired: true,
    installerSignatureRequired: true,
    signedInstallerVerified: false,
  }
}

export async function runRuntimeSelfCheck(): Promise<RuntimeSelfCheckDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<RuntimeSelfCheckDto>(tauriCommands.runRuntimeSelfCheck)
  }

  const checkedAt = new Date().toISOString()
  const { checkTemplateSidecars } = await import('@/entities/template/api')
  return {
    ready: false,
    checkedAt,
    workspace: mockRuntimeCheckItem('workspace', true),
    sqlite: mockRuntimeCheckItem('sqlite', true),
    ffmpeg: await checkFfmpegSidecars(),
    templateSidecar: await checkTemplateSidecars(),
    templateManifest: mockRuntimeCheckItem('templateManifest', true),
    templatePreview: {
      key: 'templatePreview',
      ready: false,
      skipped: true,
      errorCode: 'template.sidecar_missing',
      message: 'Mock adapter does not include bundled template sidecars.',
    },
  }
}

function mockRuntimeCheckItem(key: string, ready: boolean) {
  return {
    key,
    ready,
    skipped: false,
    errorCode: ready ? undefined : `runtime.${key}_failed`,
    message: ready ? undefined : `${key} check failed.`,
  }
}

function mockTemplateBinary(name: string) {
  return {
    name,
    relativePath: `sidecars/${name}`,
    exists: false,
    executable: false,
    errorCode: 'template.sidecar_missing',
    message: 'Mock adapter does not include bundled template sidecars.',
  }
}

export async function probeMedia(request: ProbeMediaRequest): Promise<MediaProbeDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<MediaProbeDto, { request: ProbeMediaRequest }>(tauriCommands.probeMedia, { request })
  }

  return {
    path: request.relativePath,
    mediaKind: request.mediaKind ?? 'unknown',
    container: 'mp4',
    formatName: 'mp4',
    durationSeconds: 4,
    width: 720,
    height: 1280,
    fps: 30,
    videoCodec: 'h264',
    audioCodec: undefined,
    hasVideoStream: true,
    hasAudioStream: false,
  }
}

export async function listProviderConfigs(
  request: ListProviderConfigsRequest = {},
): Promise<ProviderConfigDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderConfigDto[], { request: ListProviderConfigsRequest }>(
      tauriCommands.listProviderConfigs,
      { request },
    )
  }

  return request.providerKind
    ? providerConfigs.filter((provider) => provider.providerKind === request.providerKind)
    : providerConfigs
}

export async function upsertProviderConfig(config: ProviderConfigDto): Promise<ProviderConfigDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderConfigDto, { config: ProviderConfigDto }>(tauriCommands.upsertProviderConfig, { config })
  }

  const existingIndex = providerConfigs.findIndex((provider) => provider.providerId === config.providerId)
  if (existingIndex >= 0) {
    providerConfigs.splice(existingIndex, 1, config)
  } else {
    providerConfigs.push(config)
  }
  return config
}

export async function deleteProviderConfig(request: DeleteProviderConfigRequest): Promise<ProviderConfigDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderConfigDto, { request: DeleteProviderConfigRequest }>(tauriCommands.deleteProviderConfig, { request })
  }

  const existingIndex = providerConfigs.findIndex((provider) => provider.providerId === request.providerId)
  if (existingIndex < 0) throw new Error(`provider.not_found: ${request.providerId}`)
  const [deleted] = providerConfigs.splice(existingIndex, 1)
  for (let index = providerModels.length - 1; index >= 0; index -= 1) {
    if (providerModels[index].providerId === request.providerId) providerModels.splice(index, 1)
  }
  for (let index = workflowPresets.length - 1; index >= 0; index -= 1) {
    if (workflowPresets[index].providerId === request.providerId) workflowPresets.splice(index, 1)
  }
  return deleted
}

export async function listProviderModels(
  request: ListProviderModelsRequest = {},
): Promise<ProviderModelDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderModelDto[], { request: ListProviderModelsRequest }>(
      tauriCommands.listProviderModels,
      { request },
    )
  }

  return providerModels.filter((model) => {
    if (request.providerId && model.providerId !== request.providerId) return false
    if (request.providerKind && model.providerKind !== request.providerKind) return false
    return true
  })
}

export async function upsertProviderModel(model: ProviderModelDto): Promise<ProviderModelDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderModelDto, { model: ProviderModelDto }>(tauriCommands.upsertProviderModel, { model })
  }

  const provider = providerConfigs.find((item) => item.providerId === model.providerId)
  if (!provider) throw new Error(`provider.not_found: ${model.providerId}`)
  if (provider.providerKind === 'workflow') throw new Error('provider_models cannot register workflow providers')
  if (provider.providerKind !== model.providerKind) throw new Error('provider_kind must match provider')
  const existingIndex = providerModels.findIndex((item) => item.modelId === model.modelId)
  if (existingIndex >= 0) {
    providerModels.splice(existingIndex, 1, model)
  } else {
    providerModels.push(model)
  }
  return model
}

export async function deleteProviderModel(request: DeleteProviderModelRequest): Promise<ProviderModelDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderModelDto, { request: DeleteProviderModelRequest }>(tauriCommands.deleteProviderModel, { request })
  }

  const existingIndex = providerModels.findIndex((model) => model.modelId === request.modelId)
  if (existingIndex < 0) throw new Error(`provider.model_not_found: ${request.modelId}`)
  const [deleted] = providerModels.splice(existingIndex, 1)
  return deleted
}

export async function listWorkflowPresets(
  request: ListWorkflowPresetsRequest = {},
): Promise<WorkflowPresetDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<WorkflowPresetDto[], { request: ListWorkflowPresetsRequest }>(
      tauriCommands.listWorkflowPresets,
      { request },
    )
  }

  return workflowPresets.filter((preset) => {
    if (request.providerId && preset.providerId !== request.providerId) return false
    if (request.vendor && preset.vendor !== request.vendor) return false
    if (request.abilityType && !preset.abilityTypes.includes(request.abilityType)) return false
    return true
  })
}

export async function upsertWorkflowPreset(preset: WorkflowPresetDto): Promise<WorkflowPresetDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<WorkflowPresetDto, { preset: WorkflowPresetDto }>(tauriCommands.upsertWorkflowPreset, { preset })
  }

  const provider = providerConfigs.find((item) => item.providerId === preset.providerId)
  if (!provider) throw new Error(`provider.not_found: ${preset.providerId}`)
  if (provider.providerKind !== 'workflow') throw new Error('workflow_presets require ProviderKind=workflow')
  if (provider.vendor !== preset.vendor) throw new Error('workflow preset vendor must match provider vendor')
  const existingIndex = workflowPresets.findIndex((item) => item.workflowPresetId === preset.workflowPresetId)
  if (existingIndex >= 0) {
    workflowPresets.splice(existingIndex, 1, preset)
  } else {
    workflowPresets.push(preset)
  }
  return preset
}

export async function deleteWorkflowPreset(request: DeleteWorkflowPresetRequest): Promise<WorkflowPresetDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<WorkflowPresetDto, { request: DeleteWorkflowPresetRequest }>(tauriCommands.deleteWorkflowPreset, { request })
  }

  const existingIndex = workflowPresets.findIndex((preset) => preset.workflowPresetId === request.workflowPresetId)
  if (existingIndex < 0) throw new Error(`workflow.preset_not_found: ${request.workflowPresetId}`)
  const [deleted] = workflowPresets.splice(existingIndex, 1)
  return deleted
}

export async function listExecutableMediaOptions(): Promise<ExecutableMediaOptionDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ExecutableMediaOptionDto[]>(tauriCommands.listExecutableMediaOptions)
  }

  const options: ExecutableMediaOptionDto[] = [
    ...providerModels.map((model) => providerModelToExecutableOption(model)),
    ...workflowPresets.map((preset) => workflowPresetToExecutableOption(preset)),
  ]
  return options.sort((left, right) => left.sourceType.localeCompare(right.sourceType) || left.label.localeCompare(right.label))
}

export async function listVideoPacks(
  request: ListVideoPacksRequest = {},
): Promise<VideoPackDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoPackDto[], { request: ListVideoPacksRequest }>(
      tauriCommands.listVideoPacks,
      { request },
    )
  }

  ensureMockVideoPacks()
  return videoPacks.filter((pack) => {
    if (request.sourceType && pack.sourceType !== request.sourceType) return false
    if (!request.includeDisabled && !pack.isEnabled) return false
    return true
  })
}

export async function getVideoPack(request: VideoPackIdRequest): Promise<VideoPackDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoPackDto, { request: VideoPackIdRequest }>(tauriCommands.getVideoPack, { request })
  }

  ensureMockVideoPacks()
  const pack = videoPacks.find((item) => item.packId === request.packId)
  if (!pack) throw new Error(`video_pack.not_found: ${request.packId}`)
  return pack
}

export async function cloneVideoPackToUser(request: VideoPackIdRequest): Promise<VideoPackDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoPackDto, { request: VideoPackIdRequest }>(tauriCommands.cloneVideoPackToUser, { request })
  }

  const source = await getVideoPack(request)
  if (source.sourceType !== 'builtin') throw new Error('video_pack.clone_source_invalid')
  const pack: VideoPackDto = {
    ...source,
    packId: `pack_${Date.now()}`,
    sourceType: 'user',
    name: `${source.name} Copy`,
    ruleRefs: { ...source.ruleRefs },
    isEnabled: false,
    projectReferenceCount: 0,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  videoPacks.push(pack)
  refreshMockCreativeRuleReferenceCounts()
  return pack
}

export async function upsertUserVideoPack(request: UpsertUserVideoPackRequest): Promise<VideoPackDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoPackDto, { request: UpsertUserVideoPackRequest }>(tauriCommands.upsertUserVideoPack, { request })
  }

  ensureMockVideoPacks()
  const packId = request.packId || `pack_${Date.now()}`
  const existing = videoPacks.find((pack) => pack.packId === packId)
  if (existing?.sourceType === 'builtin') throw new Error('video_pack.builtin_readonly')
  const pack: VideoPackDto = {
    packId,
    sourceType: 'user',
    name: request.name,
    description: request.description,
    applicableInputTypes: request.applicableInputTypes,
    contentCategory: request.contentCategory,
    defaultTone: request.defaultTone,
    defaultAspectRatio: request.defaultAspectRatio,
    defaultDurationSeconds: request.defaultDurationSeconds,
    defaultSceneCount: request.defaultSceneCount,
    ruleRefs: request.ruleRefs,
    recommendedExecutableRefs: request.recommendedExecutableRefs,
    assetRefs: request.assetRefs,
    isEnabled: request.isEnabled,
    projectReferenceCount: existing?.projectReferenceCount ?? 0,
    createdAt: existing?.createdAt ?? new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  const existingIndex = videoPacks.findIndex((item) => item.packId === packId)
  if (existingIndex >= 0) videoPacks.splice(existingIndex, 1, pack)
  else videoPacks.push(pack)
  refreshMockCreativeRuleReferenceCounts()
  return pack
}

export async function setVideoPackEnabled(request: SetVideoPackEnabledRequest): Promise<VideoPackDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoPackDto, { request: SetVideoPackEnabledRequest }>(tauriCommands.setVideoPackEnabled, { request })
  }

  const pack = await getVideoPack({ packId: request.packId })
  if (pack.sourceType !== 'user') throw new Error('video_pack.builtin_readonly')
  pack.isEnabled = request.isEnabled
  pack.updatedAt = new Date().toISOString()
  return pack
}

export async function deleteUserVideoPack(request: VideoPackIdRequest): Promise<VideoPackDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoPackDto, { request: VideoPackIdRequest }>(tauriCommands.deleteUserVideoPack, { request })
  }

  ensureMockVideoPacks()
  const existingIndex = videoPacks.findIndex((pack) => pack.packId === request.packId)
  if (existingIndex < 0) throw new Error(`video_pack.not_found: ${request.packId}`)
  if (videoPacks[existingIndex].sourceType !== 'user') throw new Error('video_pack.builtin_readonly')
  if (videoPacks[existingIndex].projectReferenceCount > 0) throw new Error('video_pack.in_use')
  const [deleted] = videoPacks.splice(existingIndex, 1)
  refreshMockCreativeRuleReferenceCounts()
  return deleted
}

export async function saveProjectConfigAsVideoPack(
  request: SaveProjectConfigAsVideoPackRequest,
): Promise<VideoPackDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<VideoPackDto, { request: SaveProjectConfigAsVideoPackRequest }>(
      tauriCommands.saveProjectConfigAsVideoPack,
      { request },
    )
  }

  ensureMockVideoPacks()
  const pack: VideoPackDto = {
    packId: `pack_${Date.now()}`,
    sourceType: 'user',
    name: request.name,
    description: request.description,
    applicableInputTypes: ['topic', 'paste', 'article'],
    contentCategory: 'knowledge',
    defaultTone: undefined,
    defaultAspectRatio: '9:16',
    defaultDurationSeconds: 60,
    defaultSceneCount: 8,
    ruleRefs: {},
    recommendedExecutableRefs: {},
    assetRefs: [],
    isEnabled: true,
    projectReferenceCount: 0,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  videoPacks.push(pack)
  refreshMockCreativeRuleReferenceCounts()
  return pack
}

export async function getProjectStyleBible(projectId: string): Promise<StyleBibleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StyleBibleDto, { projectId: string }>(tauriCommands.getProjectStyleBible, { projectId })
  }

  return ensureMockStyleBible(projectId)
}

export async function listProjectCharacterBibles(projectId: string): Promise<CharacterBibleDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CharacterBibleDto[], { projectId: string }>(tauriCommands.listProjectCharacterBibles, { projectId })
  }

  return ensureMockCharacterBibles(projectId)
}

export async function upsertProjectCharacterBible(
  request: UpsertProjectCharacterBibleRequest,
): Promise<CharacterBibleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CharacterBibleDto, { request: UpsertProjectCharacterBibleRequest }>(
      tauriCommands.upsertProjectCharacterBible,
      { request },
    )
  }

  const items = await ensureMockCharacterBibles(request.projectId)
  const characterId = request.characterId?.trim() || `character_${Date.now()}`
  const existing = items.find((item) => item.characterId === characterId)
  const character: CharacterBibleDto = {
    characterBibleId: characterId,
    projectId: request.projectId,
    characterId,
    name: request.name.trim(),
    alias: normalizeStringList(request.alias),
    age: request.age.trim(),
    gender: request.gender.trim(),
    appearance: request.appearance.trim(),
    clothing: request.clothing.trim(),
    personality: request.personality.trim(),
    visualPrompt: request.visualPrompt?.trim() || [request.name, request.age, request.gender, request.appearance, request.clothing, request.personality].map((value) => value.trim()).filter(Boolean).join(', '),
    negativePrompt: request.negativePrompt?.trim() || existing?.negativePrompt || '',
    referenceImagePath: request.referenceImagePath ?? existing?.referenceImagePath ?? null,
    referenceImages: request.referenceImages ?? existing?.referenceImages ?? [],
    lockFlags: request.lockFlags ?? existing?.lockFlags ?? {},
    data: {
      character_id: characterId,
      alias: normalizeStringList(request.alias),
      age: request.age.trim(),
      gender: request.gender.trim(),
      appearance: request.appearance.trim(),
      clothing: request.clothing.trim(),
      personality: request.personality.trim(),
      visual_prompt: request.visualPrompt?.trim() || [request.name, request.age, request.gender, request.appearance, request.clothing, request.personality].map((value) => value.trim()).filter(Boolean).join(', '),
      negative_prompt: request.negativePrompt?.trim() || existing?.negativePrompt || '',
      reference_image_path: request.referenceImagePath ?? existing?.referenceImagePath ?? null,
      reference_images_json: request.referenceImages ?? existing?.referenceImages ?? [],
      lock_flags: request.lockFlags ?? existing?.lockFlags ?? {},
    },
    createdAt: existing?.createdAt ?? new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  const existingIndex = items.findIndex((item) => item.characterId === characterId)
  if (existingIndex >= 0) items.splice(existingIndex, 1, character)
  else items.push(character)
  characterBiblesByProjectId.set(request.projectId, [...items])
  return character
}

export async function deleteProjectCharacterBible(
  request: CharacterBibleIdRequest,
): Promise<CharacterBibleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CharacterBibleDto, { request: CharacterBibleIdRequest }>(
      tauriCommands.deleteProjectCharacterBible,
      { request },
    )
  }

  for (const [projectId, items] of characterBiblesByProjectId.entries()) {
    const index = items.findIndex((item) => item.characterId === request.characterId)
    if (index < 0) continue
    const hasAssetReference = assetReferences.some((reference) => reference.ownerKind === 'character_bible' && reference.ownerId === request.characterId)
    if (hasAssetReference) throw new Error('character_bible.in_use')
    const [deleted] = items.splice(index, 1)
    characterBiblesByProjectId.set(projectId, [...items])
    return deleted
  }
  throw new Error(`character_bible.not_found: ${request.characterId}`)
}

export async function bindCharacterReferenceAsset(
  request: BindCharacterReferenceAssetRequest,
): Promise<BindCharacterReferenceAssetResponse> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<BindCharacterReferenceAssetResponse, { request: BindCharacterReferenceAssetRequest }>(
      tauriCommands.bindCharacterReferenceAsset,
      { request },
    )
  }

  const items = await ensureMockCharacterBibles(request.projectId)
  const character = items.find((item) => item.characterId === request.characterId)
  if (!character) throw new Error(`character_bible.not_found: ${request.characterId}`)
  const asset = assets.find((item) => item.assetId === request.assetId)
  if (!asset) throw new Error(`asset.not_found: ${request.assetId}`)
  if (!asset.relativePath.startsWith('assets/')) throw new Error('character reference must use assets/ relativePath')
  const reference: AssetReferenceDto = {
    referenceId: `asset_ref_${Date.now()}`,
    assetId: asset.assetId,
    ownerKind: 'character_bible',
    ownerId: character.characterId,
    usageKind: 'character_reference',
  }
  assetReferences.push(reference)
  const entry = {
    assetId: asset.assetId,
    referenceId: reference.referenceId,
    role: request.referenceRole || 'character_front_view',
    imageKind: 'character_reference',
    assetKind: asset.kind,
    relativePath: asset.relativePath,
    source: asset.sourceKind,
  }
  const updated: CharacterBibleDto = {
    ...character,
    referenceImagePath: asset.relativePath,
    referenceImages: [...character.referenceImages, entry],
    data: {
      ...character.data,
      reference_image_path: asset.relativePath,
      reference_images_json: [...character.referenceImages, entry],
    },
    updatedAt: new Date().toISOString(),
  }
  characterBiblesByProjectId.set(request.projectId, items.map((item) => (item.characterId === updated.characterId ? updated : item)))
  return { characterBible: updated, reference }
}

export async function listProjectLocationBibles(projectId: string): Promise<LocationBibleDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LocationBibleDto[], { projectId: string }>(tauriCommands.listProjectLocationBibles, { projectId })
  }

  return ensureMockLocationBibles(projectId)
}

export async function upsertProjectLocationBible(
  request: UpsertProjectLocationBibleRequest,
): Promise<LocationBibleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LocationBibleDto, { request: UpsertProjectLocationBibleRequest }>(
      tauriCommands.upsertProjectLocationBible,
      { request },
    )
  }

  const items = await ensureMockLocationBibles(request.projectId)
  const locationId = request.locationId?.trim() || `location_${Date.now()}`
  const existing = items.find((item) => item.locationId === locationId)
  const visualPrompt = request.visualPrompt?.trim() || [request.name, request.spaceDescription, request.lighting, request.timeOfDay, request.props.join(', ')].map((value) => value.trim()).filter(Boolean).join(', ')
  const location: LocationBibleDto = {
    locationBibleId: locationId,
    projectId: request.projectId,
    locationId,
    name: request.name.trim(),
    spaceDescription: request.spaceDescription.trim(),
    lighting: request.lighting.trim(),
    timeOfDay: request.timeOfDay.trim(),
    props: normalizeStringList(request.props),
    visualPrompt,
    negativePrompt: request.negativePrompt?.trim() || existing?.negativePrompt || '',
    referenceImagePath: request.referenceImagePath ?? existing?.referenceImagePath ?? null,
    referenceImages: request.referenceImages ?? existing?.referenceImages ?? [],
    variants: request.variants ?? existing?.variants ?? [],
    data: {
      location_id: locationId,
      space_description: request.spaceDescription.trim(),
      lighting: request.lighting.trim(),
      time_of_day: request.timeOfDay.trim(),
      props: normalizeStringList(request.props),
      visual_prompt: visualPrompt,
      negative_prompt: request.negativePrompt?.trim() || existing?.negativePrompt || '',
      reference_image_path: request.referenceImagePath ?? existing?.referenceImagePath ?? null,
      reference_images_json: request.referenceImages ?? existing?.referenceImages ?? [],
      variants: request.variants ?? existing?.variants ?? [],
    },
    createdAt: existing?.createdAt ?? new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  const existingIndex = items.findIndex((item) => item.locationId === locationId)
  if (existingIndex >= 0) items.splice(existingIndex, 1, location)
  else items.push(location)
  locationBiblesByProjectId.set(request.projectId, [...items])
  return location
}

export async function deleteProjectLocationBible(
  request: LocationBibleIdRequest,
): Promise<LocationBibleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<LocationBibleDto, { request: LocationBibleIdRequest }>(
      tauriCommands.deleteProjectLocationBible,
      { request },
    )
  }

  for (const [projectId, items] of locationBiblesByProjectId.entries()) {
    const index = items.findIndex((item) => item.locationId === request.locationId)
    if (index < 0) continue
    const hasAssetReference = assetReferences.some((reference) => reference.ownerKind === 'location_bible' && reference.ownerId === request.locationId)
    if (hasAssetReference) throw new Error('location_bible.in_use')
    const [deleted] = items.splice(index, 1)
    locationBiblesByProjectId.set(projectId, [...items])
    return deleted
  }
  throw new Error(`location_bible.not_found: ${request.locationId}`)
}

export async function bindLocationReferenceAsset(
  request: BindLocationReferenceAssetRequest,
): Promise<BindLocationReferenceAssetResponse> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<BindLocationReferenceAssetResponse, { request: BindLocationReferenceAssetRequest }>(
      tauriCommands.bindLocationReferenceAsset,
      { request },
    )
  }

  const items = await ensureMockLocationBibles(request.projectId)
  const location = items.find((item) => item.locationId === request.locationId)
  if (!location) throw new Error(`location_bible.not_found: ${request.locationId}`)
  const asset = assets.find((item) => item.assetId === request.assetId)
  if (!asset) throw new Error(`asset.not_found: ${request.assetId}`)
  if (!asset.relativePath.startsWith('assets/')) throw new Error('location reference must use assets/ relativePath')
  const reference: AssetReferenceDto = {
    referenceId: `asset_ref_${Date.now()}`,
    assetId: asset.assetId,
    ownerKind: 'location_bible',
    ownerId: location.locationId,
    usageKind: 'location_reference',
  }
  assetReferences.push(reference)
  const entry = {
    assetId: asset.assetId,
    referenceId: reference.referenceId,
    role: request.referenceRole || 'scene_wide_view',
    imageKind: 'scene_reference',
    assetKind: asset.kind,
    relativePath: asset.relativePath,
    source: asset.sourceKind,
  }
  const updated: LocationBibleDto = {
    ...location,
    referenceImagePath: asset.relativePath,
    referenceImages: [...location.referenceImages, entry],
    data: {
      ...location.data,
      reference_image_path: asset.relativePath,
      reference_images_json: [...location.referenceImages, entry],
    },
    updatedAt: new Date().toISOString(),
  }
  locationBiblesByProjectId.set(request.projectId, items.map((item) => (item.locationId === updated.locationId ? updated : item)))
  return { locationBible: updated, reference }
}

export async function analyzeStyleReferenceImage(
  request: AnalyzeStyleReferenceRequest,
): Promise<StyleReferenceAnalysisDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StyleReferenceAnalysisDto, { request: AnalyzeStyleReferenceRequest }>(
      tauriCommands.analyzeStyleReferenceImage,
      { request },
    )
  }

  const styleBible = await ensureMockStyleBible(request.projectId)
  const referenceImagePath = styleBible.referenceImagePath
    || stringFromRecord(styleBible.referenceImages[0], ['relativePath', 'relative_path'])
  if (!referenceImagePath) throw new Error('validation.missing_input: Style reference image is required before analysis.')
  if (!referenceImagePath.startsWith('assets/')) throw new Error('storage.path_denied: Style reference analysis only accepts controlled assets/ paths.')
  return {
    projectId: request.projectId,
    styleBibleId: request.styleBibleId || styleBible.styleBibleId,
    referenceImagePath,
    stylePrompt: 'clean cinematic short-video still, refined texture, consistent subject-background separation',
    colorPalette: ['warm neutral', 'muted green', 'charcoal accent'],
    lighting: 'soft directional light with gentle shadow falloff',
    composition: 'vertical frame, clear focal zone, balanced negative space',
    negativePromptSuggestion: 'low resolution, distorted geometry, watermark, logo, private details',
    warnings: ['style_reference.analysis_scope_only'],
    rawDescription: 'Style reference analysis focused on reusable visual treatment; identity, logo, and private details were not returned.',
    providerTraceId: `trace_style_mock_${Date.now()}`,
    providerId: request.providerId || 'provider_controlled_fake_vlm',
    providerModelId: request.providerModelId,
  }
}

export async function listStylePresets(): Promise<StylePresetDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StylePresetDto[]>(tauriCommands.listStylePresets)
  }

  return [...builtinStylePresets(), ...userStylePresets]
}

export async function upsertProjectStyleBible(request: UpsertProjectStyleBibleRequest): Promise<StyleBibleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StyleBibleDto, { request: UpsertProjectStyleBibleRequest }>(tauriCommands.upsertProjectStyleBible, { request })
  }

  const existing = await ensureMockStyleBible(request.projectId)
  const style: StyleBibleDto = {
    ...existing,
    styleBibleId: request.styleBibleId || existing.styleBibleId,
    name: request.name || existing.name,
    stylePrompt: request.stylePrompt.trim(),
    colorPalette: request.colorPalette.map((item) => item.trim()).filter(Boolean),
    lighting: request.lighting.trim(),
    composition: request.composition.trim(),
    negativePrompt: mergeNegativePrompts([request.negativePrompt]),
    referenceImagePath: request.referenceImagePath ?? null,
    referenceImages: request.referenceImages ?? existing.referenceImages,
    data: {
      ...existing.data,
      style_prompt: request.stylePrompt.trim(),
      color_palette: request.colorPalette.map((item) => item.trim()).filter(Boolean),
      lighting: request.lighting.trim(),
      composition: request.composition.trim(),
      negative_prompt: mergeNegativePrompts([request.negativePrompt]),
      reference_image_path: request.referenceImagePath ?? null,
      reference_images_json: request.referenceImages ?? existing.referenceImages,
    },
    updatedAt: new Date().toISOString(),
  }
  styleBiblesByProjectId.set(request.projectId, style)
  if (request.saveAsPreset) {
    userStylePresets.push({
      presetId: `style_preset_${Date.now()}`,
      sourceType: 'user',
      name: request.presetName?.trim() || style.name,
      stylePrompt: style.stylePrompt,
      colorPalette: style.colorPalette,
      lighting: style.lighting,
      composition: style.composition,
      negativePrompt: style.negativePrompt,
      referenceImagePath: style.referenceImagePath,
    })
  }
  return style
}

export async function applyStylePreset(request: ApplyStylePresetRequest): Promise<StyleBibleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StyleBibleDto, { request: ApplyStylePresetRequest }>(tauriCommands.applyStylePreset, { request })
  }

  const preset = (await listStylePresets()).find((item) => item.presetId === request.presetId)
  if (!preset) throw new Error(`style_preset.not_found: ${request.presetId}`)
  return upsertProjectStyleBible({
    projectId: request.projectId,
    name: preset.name,
    stylePrompt: preset.stylePrompt,
    colorPalette: preset.colorPalette,
    lighting: preset.lighting,
    composition: preset.composition,
    negativePrompt: preset.negativePrompt,
    referenceImagePath: preset.referenceImagePath ?? null,
  })
}

export async function bindStyleReferenceAsset(request: BindStyleReferenceAssetRequest): Promise<BindStyleReferenceAssetResponse> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<BindStyleReferenceAssetResponse, { request: BindStyleReferenceAssetRequest }>(tauriCommands.bindStyleReferenceAsset, { request })
  }

  const asset = assets.find((item) => item.assetId === request.assetId)
  if (!asset) throw new Error(`asset.not_found: ${request.assetId}`)
  const styleBible = await ensureMockStyleBible(request.projectId)
  const referenceId = `asset_ref_${Date.now()}`
  const entry = {
    assetId: asset.assetId,
    referenceId,
    role: 'style_reference',
    imageKind: 'style_reference',
    assetKind: asset.kind,
    relativePath: asset.relativePath,
    source: asset.sourceKind,
  }
  assetReferences.push({
    referenceId,
    assetId: asset.assetId,
    ownerKind: 'style_bible',
    ownerId: request.styleBibleId || styleBible.styleBibleId,
    usageKind: 'style_reference',
  })
  const next: StyleBibleDto = {
    ...styleBible,
    referenceImagePath: asset.relativePath,
    referenceImages: [...styleBible.referenceImages, entry],
    data: {
      ...styleBible.data,
      reference_image_path: asset.relativePath,
      reference_images_json: [...styleBible.referenceImages, entry],
    },
    updatedAt: new Date().toISOString(),
  }
  styleBiblesByProjectId.set(request.projectId, next)
  return { styleBible: next, referenceId }
}

export async function buildImagePromptPreview(request: BuildImagePromptPreviewRequest): Promise<ImagePromptPreviewDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ImagePromptPreviewDto, { request: BuildImagePromptPreviewRequest }>(tauriCommands.buildImagePromptPreview, { request })
  }

  const { getStoryboard } = await import('@/entities/storyboard/api')
  const storyboard = await getStoryboard(request.projectId)
  const item = storyboard.items.find((entry) => entry.itemId === request.itemId)
  if (!item) throw new Error(`storyboard_item.not_found: ${request.itemId}`)
  const styleBible = await ensureMockStyleBible(request.projectId)
  const characters = await ensureMockCharacterBibles(request.projectId)
  const characterBibles = item.characterIds.length > 0 ? characters.filter((character) => item.characterIds.includes(character.characterId)) : []
  const locations = await ensureMockLocationBibles(request.projectId)
  const locationBible = item.locationId ? locations.find((location) => location.locationId === item.locationId) : undefined
  const characterContent = characterBibles.length > 0 ? characterBibles.map((character) => [character.name, character.visualPrompt, character.appearance, character.clothing].map((value) => value.trim()).filter(Boolean).join('; ')).join('; ') : item.characters.join(', ')
  const sceneContent = locationBible
    ? [locationBible.name, locationBible.visualPrompt, locationBible.spaceDescription, locationBible.lighting, locationBible.timeOfDay, locationBible.props.join(', ')].map((value) => value.trim()).filter(Boolean).join('; ')
    : item.sceneDescription
  const sections = [
    { key: 'visualDescription', label: 'Visual description', content: item.visualDescription },
    { key: 'imagePrompt', label: 'Storyboard image prompt', content: item.imagePrompt },
    { key: 'characters', label: 'Characters', content: characterContent },
    { key: 'scene', label: 'Scene', content: sceneContent },
    { key: 'stylePrompt', label: 'Style Bible', content: styleBible.stylePrompt },
    { key: 'colorPalette', label: 'Color palette', content: styleBible.colorPalette.join(', ') },
    { key: 'lighting', label: 'Lighting', content: styleBible.lighting },
    { key: 'composition', label: 'Composition', content: styleBible.composition },
  ].filter((section) => section.content.trim())
  const finalNegativePrompt = mergeNegativePrompts([item.negativePrompt, styleBible.negativePrompt, ...characterBibles.map((character) => character.negativePrompt), locationBible?.negativePrompt ?? ''])
  const referenceImages = styleBible.referenceImagePath ? [{ path: styleBible.referenceImagePath, role: 'style_reference', weight: 0.65 }] : []
  for (const character of characterBibles) {
    if (character.referenceImagePath) referenceImages.push({ path: character.referenceImagePath, role: `character_front_view:${character.characterId}`, weight: 0.8 })
  }
  if (locationBible?.referenceImagePath) {
    referenceImages.push({ path: locationBible.referenceImagePath, role: `scene_wide_view:${locationBible.locationId}`, weight: 0.72 })
  }
  return {
    projectId: request.projectId,
    itemId: request.itemId,
    finalPrompt: sections.map((section) => section.content.trim()).join(', '),
    finalNegativePrompt,
    sections,
    referenceImages,
    styleBible,
    characterBibles,
    locationBible,
    negativePromptTruncated: false,
    negativePromptMaxLength: 800,
  }
}

export async function providerDryRun(request: ProviderDryRunRequest): Promise<ProviderDryRunResponse> {
  const response = await providerGenerationTest({
    ...request,
    testMode: 'dry_run',
    realGenerateConfirmed: false,
  })
  return {
    traceId: response.traceId,
    providerId: response.providerId,
    providerKind: response.providerKind,
    status: response.status,
    message: response.message,
    outputSummary: response.outputSummary,
  }
}

export async function providerGenerationTest(
  request: ProviderGenerationTestRequest,
): Promise<ProviderGenerationTestResponse> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ProviderGenerationTestResponse, { request: ProviderGenerationTestRequest }>(
      tauriCommands.providerGenerationTest,
      { request },
    )
  }

  if (request.testMode !== 'dry_run' && request.testMode !== 'real_generate') {
    throw new Error('provider.test_mode_unsupported: Provider test mode must be dry_run or real_generate')
  }
  if (request.testMode === 'real_generate' && !request.realGenerateConfirmed) {
    throw new Error('provider.real_generate_confirmation_required: real_generate requires explicit user confirmation')
  }
  if (request.testMode === 'real_generate' && request.providerKind === 'video' && request.confirmToken !== 'REAL_GENERATE_VIDEO') {
    throw new Error('provider.video_real_generate_confirmation_required: video real_generate requires confirmation token')
  }
  if (request.providerModelId && request.workflowPresetId) {
    throw new Error('provider.source_conflict: provider_model_id and workflow_preset_id cannot both be set')
  }
  if (request.simulateCancelled) {
    throw new Error(`provider.cancelled: dummy provider ${request.testMode} was cancelled`)
  }
  if (request.simulateFailure) {
    throw new Error('provider.server_error: dummy provider simulated failure')
  }

  const config = providerConfigs.find((provider) => provider.providerId === request.providerId)
  if (!config) throw new Error(`provider.not_found: ${request.providerId}`)
  if (!config.isEnabled || config.status === 'disabled') throw new Error(`provider.disabled: ${request.providerId}`)
  if (config.providerKind !== request.providerKind) throw new Error(`provider.capability_unsupported: ${request.providerKind}`)
  if (request.providerModelId) {
    const model = providerModels.find((item) => item.providerId === request.providerId && (item.modelId === request.providerModelId || item.providerModelId === request.providerModelId))
    if (!model) throw new Error(`provider.model_not_found: ${request.providerModelId}`)
    if (!model.isEnabled || model.status === 'disabled') throw new Error(`provider.model_disabled: ${request.providerModelId}`)
  }
  if (request.workflowPresetId) {
    const preset = workflowPresets.find((item) => item.providerId === request.providerId && item.workflowPresetId === request.workflowPresetId)
    if (!preset) throw new Error(`workflow.preset_not_found: ${request.workflowPresetId}`)
    if (!preset.isEnabled || preset.status === 'disabled') throw new Error(`workflow.preset_disabled: ${request.workflowPresetId}`)
    if (config.providerKind === 'workflow' && config.vendor !== preset.vendor) {
      throw new Error(`workflow.provider_unavailable: ${request.workflowPresetId}`)
    }
  }

  return {
    traceId: `trace_${Date.now()}`,
    testMode: request.testMode,
    providerId: request.providerId,
    providerKind: request.providerKind,
    status: 'succeeded',
    message: request.testMode === 'real_generate'
      ? 'Dummy provider real_generate test succeeded without external network calls.'
      : 'Dummy provider dry_run succeeded without external network calls.',
    outputSummary: {
      adapter: 'dummy',
      testMode: request.testMode,
      providerId: request.providerId,
      providerKind: request.providerKind,
      providerModelId: request.providerModelId,
      workflowPresetId: request.workflowPresetId,
      billable: false,
      realGenerateConfirmed: request.testMode === 'real_generate' && request.realGenerateConfirmed === true,
      externalNetwork: false,
    },
    billable: false,
    realGenerateConfirmed: request.testMode === 'real_generate' && request.realGenerateConfirmed === true,
  }
}

function providerModelToExecutableOption(model: ProviderModelDto): ExecutableMediaOptionDto {
  const provider = providerConfigs.find((item) => item.providerId === model.providerId)
  const disabledReason = getProviderModelDisabledReason(model, provider)
  const capability = String(model.abilityTypes[0] ?? 'text_to_image')
  const inputPlan = createMockInputPlan(capability, model.providerKind, model.inputRequirements, model.limits, {}, {}, readStringConfig(model.config, 'imageKind', 'image_kind'), readStringConfig(model.config, 'assetKind', 'asset_kind'))
  return {
    optionId: `provider_model:${model.modelId}`,
    sourceType: 'provider_model',
    sourceId: model.modelId,
    label: model.displayName,
    providerId: model.providerId,
    providerKind: model.providerKind,
    vendor: model.vendor,
    kind: 'provider_model',
    capability,
    capabilities: model.abilityTypes.map(String),
    constraints: {
      limits: model.limits,
      inputRequirements: model.inputRequirements,
      inputModalities: model.inputModalities,
      outputModalities: model.outputModalities,
      featureFlags: model.featureFlags,
      apiContractVerified: model.apiContractVerified,
      modelName: model.modelName,
      vendorModelId: model.providerModelId,
    },
    inputPlan,
    status: model.status,
    providerModelId: model.modelId,
    workflowPresetId: undefined,
    enabled: !disabledReason,
    disabledReason,
    normalizedParams: {
      sourceType: 'provider_model',
      providerId: model.providerId,
      providerKind: model.providerKind,
      providerModelId: model.modelId,
    },
  }
}

function workflowPresetToExecutableOption(preset: WorkflowPresetDto): ExecutableMediaOptionDto {
  const provider = providerConfigs.find((item) => item.providerId === preset.providerId)
  const disabledReason = getWorkflowPresetDisabledReason(preset, provider)
  const capability = String(preset.abilityTypes[0] ?? 'workflow_execution')
  const inputPlan = createMockInputPlan(capability, 'workflow', {}, preset.limits, preset.paramSchema, preset.defaultParams, readStringConfig(preset.config, 'imageKind', 'image_kind'), readStringConfig(preset.config, 'assetKind', 'asset_kind'))
  return {
    optionId: `workflow_preset:${preset.workflowPresetId}`,
    sourceType: 'workflow_preset',
    sourceId: preset.workflowPresetId,
    label: preset.displayName,
    providerId: preset.providerId,
    providerKind: 'workflow',
    vendor: preset.vendor,
    kind: 'workflow_preset',
    capability,
    capabilities: preset.abilityTypes.map(String),
    constraints: {
      limits: preset.limits,
      paramSchema: preset.paramSchema,
      nodeMap: preset.nodeMap,
      outputMap: preset.outputMap,
      workflowKey: preset.workflowKey,
      workflowId: preset.workflowId,
      workflowVersion: preset.workflowVersion,
      inputModalities: preset.inputModalities,
      outputModalities: preset.outputModalities,
    },
    inputPlan,
    status: preset.status,
    providerModelId: undefined,
    workflowPresetId: preset.workflowPresetId,
    enabled: !disabledReason,
    disabledReason,
    normalizedParams: {
      sourceType: 'workflow_preset',
      providerId: preset.providerId,
      providerKind: 'workflow',
      workflowPresetId: preset.workflowPresetId,
      workflowVendor: preset.vendor,
      defaultParams: preset.defaultParams,
    },
  }
}

function getProviderModelDisabledReason(model: ProviderModelDto, provider?: ProviderConfigDto) {
  if (!provider) return 'provider.not_found'
  if (!provider.isEnabled || provider.status === 'disabled') return 'provider.disabled'
  if (provider.status !== 'ready') return `provider.status.${provider.status}`
  if (!model.isEnabled || model.status === 'disabled') return 'provider.model_disabled'
  if (model.status !== 'ready') return `provider.model_status.${model.status}`
  return undefined
}

function getWorkflowPresetDisabledReason(preset: WorkflowPresetDto, provider?: ProviderConfigDto) {
  if (!provider) return 'provider.not_found'
  if (!provider.isEnabled || provider.status === 'disabled') return 'provider.disabled'
  if (provider.status !== 'ready') return `provider.status.${provider.status}`
  if (provider.providerKind !== 'workflow') return 'workflow.provider_kind_mismatch'
  if (provider.vendor !== preset.vendor) return 'workflow.provider_unavailable'
  if (!preset.isEnabled || preset.status === 'disabled') return 'workflow.preset_disabled'
  if (preset.status !== 'ready') return `workflow.preset_status.${preset.status}`
  if (Object.keys(preset.nodeMap).length === 0) return 'workflow.invalid_node_map'
  if (Object.keys(preset.outputMap).length === 0) return 'workflow.output_missing'
  return undefined
}

function createMockInputPlan(
  abilityType: string,
  providerKind: string,
  inputRequirements: Record<string, unknown> | unknown[],
  limits: Record<string, unknown>,
  paramSchema: Record<string, unknown> = {},
  defaultParams: Record<string, unknown> = {},
  imageKind?: string,
  assetKind?: string,
): ExecutableMediaOptionDto['inputPlan'] {
  const items: ExecutableMediaOptionDto['inputPlan']['items'] = []
  const requirements = Array.isArray(inputRequirements) ? {} : inputRequirements
  const requirementFor = (inputKey: string, fallback: 'required' | 'optional' | 'unused') => {
    const required = Array.isArray(requirements.requiredInputs) ? requirements.requiredInputs : []
    const optional = Array.isArray(requirements.optionalInputs) ? requirements.optionalInputs : []
    const unused = Array.isArray(requirements.unusedInputs) ? requirements.unusedInputs : []
    if (required.includes(inputKey)) return 'required'
    if (optional.includes(inputKey)) return 'optional'
    if (unused.includes(inputKey)) return 'unused'
    return fallback
  }
  const pushItem = (
    inputKey: string,
    inputGroup: string,
    requirement: 'required' | 'optional' | 'unused',
    constraints: Record<string, unknown> = {},
  ) => {
    items.push({
      inputKey,
      inputGroup,
      ownerType: inputGroup === 'image' ? 'storyboard_item' : 'project',
      requirement,
      sourceOptions: inputGroup === 'image' ? ['generate', 'upload', 'select_existing'] : ['generate'],
      missingReason: requirement === 'required' ? `${inputKey}.required` : undefined,
      uiSchema: { component: inputGroup === 'image' ? 'asset-picker' : inputGroup === 'workflow_param' ? 'number-input' : 'textarea' },
      constraints,
      normalizedParams: { field: inputKey },
    })
  }

  if (abilityType.includes('video') || abilityType.includes('i2v')) {
    if (abilityType !== 'text_to_video') pushItem('startFrame', 'image', requirementFor('startFrame', 'required'), limits)
    if (abilityType === 'start_end_frame_i2v') pushItem('endFrame', 'image', requirementFor('endFrame', 'required'), limits)
    pushItem('videoPrompt', 'text', requirementFor('videoPrompt', 'required'))
    pushItem('durationSeconds', 'workflow_param', requirementFor('durationSeconds', 'optional'), limits)
    pushItem('fps', 'workflow_param', requirementFor('fps', 'optional'), limits)
  } else {
    pushItem('prompt', 'text', requirementFor('prompt', 'required'))
    if (abilityType === 'image_to_image') pushItem('referenceAsset', 'image', requirementFor('referenceAsset', 'required'), limits)
  }
  pushItem('aspectRatio', 'workflow_param', requirementFor('aspectRatio', 'optional'), limits)
  pushItem('resolution', 'workflow_param', requirementFor('resolution', 'optional'), limits)

  const characterKeys = [
    'character_front_view',
    'character_side_view',
    'character_back_view',
    'character_full_body',
    'character_face_closeup',
    'character_expression_sheet',
    'character_outfit',
    'character_pose',
    'character_mood',
  ]
  for (const key of characterKeys) {
    pushItem(key, 'image', requirementFor(key, key === 'character_front_view' ? 'optional' : 'unused'), { assetKind: key, maxCount: 1 })
  }

  for (const [key, schema] of Object.entries(paramSchema)) {
    if (key === 'required' || key === 'properties') continue
    const schemaRecord = schema && typeof schema === 'object' && !Array.isArray(schema) ? schema as Record<string, unknown> : {}
    const inputGroup = schemaRecord.type === 'image' || schemaRecord.type === 'asset_path' ? 'image' : schemaRecord.type === 'video' ? 'video' : schemaRecord.type === 'audio' ? 'audio' : schemaRecord.type === 'string' ? 'text' : 'workflow_param'
    items.push({
      inputKey: `workflowParams.${key}`,
      inputGroup,
      ownerType: 'project',
      requirement: schemaRecord.required === true ? 'required' : 'optional',
      sourceOptions: inputGroup === 'image' || inputGroup === 'video' || inputGroup === 'audio' ? ['upload', 'select_existing'] : ['generate'],
      missingReason: schemaRecord.required === true ? `workflow_param.${key}.missing` : undefined,
      uiSchema: { component: inputGroup === 'image' ? 'asset-picker' : 'input', schema: schemaRecord },
      constraints: schemaRecord,
      normalizedParams: { paramKey: key, defaultValue: defaultParams[key] },
    })
  }

  const planKind = abilityType.includes('video') || abilityType.includes('i2v') || providerKind === 'video' ? 'video' : 'image'
  const resolvedImageKind = imageKind || (planKind === 'image' ? 'storyboard_image' : undefined)
  const resolvedAssetKind = assetKind || (resolvedImageKind ? defaultAssetKindForImageKind(resolvedImageKind) : undefined)
  return {
    planKind,
    abilityType,
    imageKind: resolvedImageKind,
    assetKind: resolvedAssetKind,
    items,
    requiredCount: items.filter((item) => item.requirement === 'required').length,
    optionalCount: items.filter((item) => item.requirement === 'optional').length,
    unusedCount: items.filter((item) => item.requirement === 'unused').length,
  }
}

function readStringConfig(config: Record<string, unknown>, ...keys: string[]) {
  for (const key of keys) {
    const value = config[key]
    if (typeof value === 'string' && value.trim()) return value
  }
  return undefined
}

function defaultAssetKindForImageKind(imageKind: string) {
  if (imageKind === 'character_reference') return 'character_reference_image'
  if (imageKind === 'scene_reference') return 'scene_reference_image'
  if (imageKind === 'style_reference') return 'style_reference_image'
  if (imageKind === 'cover_image') return 'cover_source'
  return 'generated_output'
}

async function ensureMockStyleBible(projectId: string): Promise<StyleBibleDto> {
  const existing = styleBiblesByProjectId.get(projectId)
  if (existing) return existing
  const { getProjectDetail } = await import('@/entities/project/api')
  const detail = await getProjectDetail(projectId)
  const stylePrompt = detail.project.stylePrompt ?? ''
  const style: StyleBibleDto = {
    styleBibleId: `style_${projectId}`,
    projectId,
    name: '默认画风',
    stylePrompt,
    colorPalette: [],
    lighting: '',
    composition: '',
    negativePrompt: '',
    referenceImagePath: null,
    referenceImages: [],
    data: {
      style_prompt: stylePrompt,
      color_palette: [],
      lighting: '',
      composition: '',
      negative_prompt: '',
      reference_image_path: null,
      reference_images_json: [],
    },
  }
  styleBiblesByProjectId.set(projectId, style)
  return style
}

async function ensureMockCharacterBibles(projectId: string): Promise<CharacterBibleDto[]> {
  const existing = characterBiblesByProjectId.get(projectId)
  if (existing) return existing
  const items: CharacterBibleDto[] = []
  characterBiblesByProjectId.set(projectId, items)
  return items
}

async function ensureMockLocationBibles(projectId: string): Promise<LocationBibleDto[]> {
  const existing = locationBiblesByProjectId.get(projectId)
  if (existing) return existing
  const items: LocationBibleDto[] = []
  locationBiblesByProjectId.set(projectId, items)
  return items
}

function builtinStylePresets(): StylePresetDto[] {
  return [
    {
      presetId: 'builtin.clean_realistic',
      sourceType: 'builtin',
      name: '干净真实短视频',
      stylePrompt: 'clean realistic short-video frame, natural human-scale detail, sharp subject, subtle background depth',
      colorPalette: ['neutral white', 'soft gray', 'muted teal accent'],
      lighting: 'soft natural light, gentle highlights, no harsh studio glare',
      composition: 'vertical composition, clear subject, balanced negative space',
      negativePrompt: 'low resolution, distorted face, extra fingers, watermark, heavy filter',
      referenceImagePath: null,
    },
    {
      presetId: 'builtin.minimal_line_art',
      sourceType: 'builtin',
      name: '极简线稿',
      stylePrompt: 'minimal black and white line art, clean hand-drawn strokes, simple shapes, white background',
      colorPalette: ['black', 'white', 'light gray'],
      lighting: 'flat clean lighting',
      composition: 'centered composition, clear empty space',
      negativePrompt: 'photorealistic, colorful, 3d render, complex background',
      referenceImagePath: null,
    },
    {
      presetId: 'builtin.cinematic_warm',
      sourceType: 'builtin',
      name: '暖调电影感',
      stylePrompt: 'cinematic realistic frame, warm restrained color grading, tactile detail, shallow depth of field',
      colorPalette: ['warm amber', 'deep green', 'charcoal'],
      lighting: 'warm side light, soft shadow, practical indoor glow',
      composition: 'rule-of-thirds vertical frame, foreground depth, stable camera feel',
      negativePrompt: 'oversaturated, plastic skin, blurry subject, text watermark, noisy artifacts',
      referenceImagePath: null,
    },
  ]
}

function mergeNegativePrompts(prompts: string[]) {
  const seen = new Set<string>()
  const parts: string[] = []
  for (const prompt of prompts) {
    for (const part of prompt.split(/[,，;；\n]/).map((item) => item.trim()).filter(Boolean)) {
      const key = part.toLowerCase()
      if (!seen.has(key)) {
        seen.add(key)
        parts.push(part)
      }
    }
  }
  const value = parts.join(', ')
  return value.length > 800 ? value.slice(0, 800).replace(/[,，;；\s]+$/, '') : value
}

const assets: AssetDto[] = []
const assetReferences: AssetReferenceDto[] = []

export async function importAsset(request: ImportAssetRequest): Promise<AssetDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AssetDto, { request: ImportAssetRequest }>(tauriCommands.importAsset, { request })
  }

  const asset: AssetDto = {
    assetId: `asset_${Date.now()}`,
    kind: request.kind,
    relativePath: `assets/${request.kind}/${request.displayName || 'mock_asset'}`,
    sourceKind: 'user_import',
    mimeType: request.mimeType,
    sizeBytes: 1024,
    checksum: undefined,
    isBuiltin: false,
    lifecycle: 'active',
    metadata: {
      ...(request.metadata ?? {}),
      displayName: request.displayName || 'mock_asset',
      mediaKind: mediaKindFromAsset(request.kind, request.mimeType),
      sourceType: 'user_import',
      sizeBytes: 1024,
    },
  }
  assets.unshift(asset)
  return asset
}

export async function readAssetPreview(request: AssetPreviewRequest): Promise<AssetPreviewDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AssetPreviewDto, { request: AssetPreviewRequest }>(tauriCommands.readAssetPreview, { request })
  }

  const asset = assets.find((item) => item.assetId === request.assetId)
  if (!asset) throw new Error(`Asset not found: ${request.assetId}`)
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="320" height="180" viewBox="0 0 320 180"><rect width="320" height="180" fill="#151a20"/><rect x="18" y="18" width="284" height="144" rx="8" fill="#25313d"/><text x="160" y="86" fill="#dfe7ee" font-family="sans-serif" font-size="18" text-anchor="middle">${asset.kind}</text><text x="160" y="112" fill="#91a0ad" font-family="sans-serif" font-size="12" text-anchor="middle">${asset.relativePath}</text></svg>`
  return {
    assetId: asset.assetId,
    relativePath: asset.relativePath,
    mediaKind: String(asset.metadata.mediaKind ?? mediaKindFromAsset(asset.kind, asset.mimeType)),
    mimeType: 'image/svg+xml',
    previewKind: String(asset.metadata.mediaKind) === 'video' ? 'video_frame' : 'image',
    bytes: Array.from(new TextEncoder().encode(svg)),
  }
}

export async function listAssets(request: ListAssetsRequest = {}): Promise<AssetDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AssetDto[], { request: ListAssetsRequest }>(tauriCommands.listAssets, { request })
  }

  return assets.filter((asset) => {
    if (request.kind && asset.kind !== request.kind) return false
    if (!request.includeDeleted && asset.lifecycle === 'deleted') return false
    return true
  })
}

export async function deleteAsset(request: DeleteAssetRequest): Promise<AssetDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AssetDto, { request: DeleteAssetRequest }>(tauriCommands.deleteAsset, { request })
  }

  const asset = assets.find((item) => item.assetId === request.assetId)
  if (!asset) throw new Error(`Asset not found: ${request.assetId}`)
  const references = assetReferences.filter((reference) => reference.assetId === request.assetId)
  if (references.length > 0) {
    const summary = references.map((reference) => `${reference.ownerKind}/${reference.ownerId}/${reference.usageKind}`).join(', ')
    throw new Error(`asset.in_use: ${summary}`)
  }
  asset.lifecycle = 'deleted'
  return asset
}

export async function createAssetReference(
  request: CreateAssetReferenceRequest,
): Promise<AssetReferenceDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AssetReferenceDto, { request: CreateAssetReferenceRequest }>(
      tauriCommands.createAssetReference,
      { request },
    )
  }

  const reference: AssetReferenceDto = {
    referenceId: `asset_ref_${Date.now()}`,
    assetId: request.assetId,
    ownerKind: request.ownerKind,
    ownerId: request.ownerId,
    usageKind: request.usageKind,
  }
  assetReferences.push(reference)
  return reference
}

export async function listAssetReferences(assetId: string): Promise<AssetReferenceDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AssetReferenceDto[], { assetId: string }>(tauriCommands.listAssetReferences, { assetId })
  }

  return assetReferences.filter((reference) => reference.assetId === assetId)
}

export async function deleteAssetReference(
  request: DeleteAssetReferenceRequest,
): Promise<AssetReferenceDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<AssetReferenceDto, { request: DeleteAssetReferenceRequest }>(
      tauriCommands.deleteAssetReference,
      { request },
    )
  }

  const existingIndex = assetReferences.findIndex((reference) => reference.referenceId === request.referenceId)
  if (existingIndex < 0) throw new Error(`asset_reference.not_found: ${request.referenceId}`)
  const [deleted] = assetReferences.splice(existingIndex, 1)
  return deleted
}

export async function collectProjectAssetPaths(projectId: string): Promise<string[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<string[], { projectId: string }>(tauriCommands.collectProjectAssetPaths, { projectId })
  }

  const referencedAssetIds = new Set(
    assetReferences
      .filter((reference) => reference.ownerId === projectId || (reference.ownerKind === 'video_pack' && reference.ownerId.startsWith(`${projectId}:`)))
      .map((reference) => reference.assetId),
  )
  return assets
    .filter((asset) => referencedAssetIds.has(asset.assetId))
    .map((asset) => asset.relativePath)
}

function mediaKindFromAsset(kind: string, mimeType?: string) {
  if (mimeType?.startsWith('image/')) return 'image'
  if (mimeType?.startsWith('video/')) return 'video'
  if (mimeType?.startsWith('audio/')) return 'audio'
  if (kind.includes('video')) return 'video'
  if (kind.includes('audio') || kind === 'bgm') return 'audio'
  if (kind === 'font') return 'font'
  if (kind === 'template_resource') return 'template'
  return 'image'
}

function normalizeStringList(values: string[]) {
  const seen = new Set<string>()
  const output: string[] = []
  for (const value of values) {
    const trimmed = value.trim()
    if (!trimmed || seen.has(trimmed)) continue
    seen.add(trimmed)
    output.push(trimmed)
  }
  return output
}

function stringFromRecord(value: Record<string, unknown> | undefined, keys: string[]) {
  if (!value) return undefined
  for (const key of keys) {
    const item = value[key]
    if (typeof item === 'string' && item.trim()) return item.trim()
  }
  return undefined
}

export async function listCreativeRules(
  request: ListCreativeRulesRequest = {},
): Promise<CreativeRuleDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CreativeRuleDto[], { request: ListCreativeRulesRequest }>(
      tauriCommands.listCreativeRules,
      { request },
    )
  }

  ensureMockCreativeRules()
  if (videoPacks.length > 0) refreshMockCreativeRuleReferenceCounts()
  return creativeRules.filter((rule) => {
    if (request.sourceType && rule.sourceType !== request.sourceType) return false
    if (request.module && rule.module !== request.module) return false
    return true
  })
}

export async function getCreativeRule(request: CreativeRuleIdRequest): Promise<CreativeRuleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CreativeRuleDto, { request: CreativeRuleIdRequest }>(tauriCommands.getCreativeRule, { request })
  }

  ensureMockCreativeRules()
  const rule = creativeRules.find((item) => item.ruleId === request.ruleId)
  if (!rule) throw new Error(`creative_rule.not_found: ${request.ruleId}`)
  return rule
}

export async function cloneCreativeRuleToUser(request: CreativeRuleIdRequest): Promise<CreativeRuleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CreativeRuleDto, { request: CreativeRuleIdRequest }>(tauriCommands.cloneCreativeRuleToUser, { request })
  }

  ensureMockCreativeRules()
  const source = creativeRules.find((rule) => rule.ruleId === request.ruleId)
  if (!source) throw new Error(`creative_rule.not_found: ${request.ruleId}`)
  if (source.sourceType !== 'builtin') throw new Error('only builtin creative rules can be cloned')
  const key = `user.${source.key}`
  const rule: CreativeRuleDto = {
    ...source,
    ruleId: `user:${source.module}:${key.replaceAll('.', '_')}`,
    key,
    name: `${source.name} Copy`,
    version: source.version,
    contentHash: source.contentHash,
    schemaHash: source.schemaHash,
    sourceType: 'user',
    enabled: false,
    relativePath: `prompts/user/${source.module}/${key.replaceAll('.', '_')}.md`,
    referenceCounts: emptyCreativeRuleReferenceCounts(),
  }
  creativeRules.push(rule)
  return rule
}

export async function saveUserCreativeRule(request: SaveCreativeRuleRequest): Promise<CreativeRuleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CreativeRuleDto, { request: SaveCreativeRuleRequest }>(tauriCommands.saveUserCreativeRule, { request })
  }

  ensureMockCreativeRules()
  if (!request.body.trim()) throw new Error('creative rule body cannot be empty')
  const ruleId = `user:${request.module}:${request.key.replaceAll('.', '_')}`
  const rule: CreativeRuleDto = {
    ...request,
    ruleId,
    version: request.version ?? '1.0.0',
    sourceType: 'user',
    relativePath: `prompts/user/${request.module}/${request.key.replaceAll('.', '_')}.md`,
    paramsSchema: request.paramsSchema ?? {},
    contentHash: mockRuleHash([request.key, request.module, request.ruleType, request.providerKind, request.version ?? '1.0.0', request.body, JSON.stringify(request.paramsSchema ?? {})]),
    schemaHash: mockRuleHash([JSON.stringify(request.outputSchema), JSON.stringify(request.paramsSchema ?? {})]),
    referenceCounts: creativeRules.find((item) => item.ruleId === ruleId)?.referenceCounts ?? emptyCreativeRuleReferenceCounts(),
  }
  const existingIndex = creativeRules.findIndex((item) => item.ruleId === ruleId)
  if (existingIndex >= 0) {
    creativeRules.splice(existingIndex, 1, rule)
  } else {
    creativeRules.push(rule)
  }
  return rule
}

export async function setUserCreativeRuleEnabled(
  request: SetCreativeRuleEnabledRequest,
): Promise<CreativeRuleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CreativeRuleDto, { request: SetCreativeRuleEnabledRequest }>(tauriCommands.setUserCreativeRuleEnabled, { request })
  }

  ensureMockCreativeRules()
  const rule = creativeRules.find((item) => item.ruleId === request.ruleId)
  if (!rule) throw new Error(`creative_rule.not_found: ${request.ruleId}`)
  if (rule.sourceType !== 'user') throw new Error('builtin creative rules cannot be modified directly')
  if (!request.enabled && creativeRuleReferenceTotal(rule.referenceCounts) > 0) throw new Error('creative_rule.in_use')
  rule.enabled = request.enabled
  return rule
}

export async function deleteUserCreativeRule(request: CreativeRuleIdRequest): Promise<CreativeRuleDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<CreativeRuleDto, { request: CreativeRuleIdRequest }>(tauriCommands.deleteUserCreativeRule, { request })
  }

  ensureMockCreativeRules()
  const existingIndex = creativeRules.findIndex((rule) => rule.ruleId === request.ruleId)
  if (existingIndex < 0) throw new Error(`creative_rule.not_found: ${request.ruleId}`)
  if (creativeRules[existingIndex].sourceType !== 'user') throw new Error('builtin creative rules cannot be deleted')
  if (creativeRuleReferenceTotal(creativeRules[existingIndex].referenceCounts) > 0) throw new Error('creative_rule.in_use')
  const [deleted] = creativeRules.splice(existingIndex, 1)
  return deleted
}

export async function validateStructuredOutput(
  request: ValidateStructuredOutputRequest,
): Promise<StructuredOutputValidationResult> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<StructuredOutputValidationResult, { request: ValidateStructuredOutputRequest }>(
      tauriCommands.validateStructuredOutput,
      { request },
    )
  }

  const errors: string[] = []
  let parsedJson: unknown
  try {
    parsedJson = JSON.parse(stripFencedJson(request.rawOutput))
  } catch (error) {
    errors.push(`output.invalid_json: ${error instanceof Error ? error.message : String(error)}`)
  }
  if (parsedJson !== undefined) {
    validateMockSchema(parsedJson, request.outputSchema, '$', errors)
    const expectedCount = request.expectedCount
    if (typeof expectedCount === 'number') {
      const countable = getMockCountableArray(parsedJson)
      if (!countable) errors.push('$.expected_count: no countable array found')
      else if (countable.length !== expectedCount) errors.push(`$: expected ${expectedCount} items but received ${countable.length}`)
    }
  }
  const attemptCount = request.repairAttemptCount ?? 0
  const maxAttempts = Math.min(request.maxRepairAttempts ?? 2, 2)
  const valid = errors.length === 0
  return {
    valid,
    parsedJson,
    errors,
    repairNeeded: !valid && attemptCount < maxAttempts,
    attemptCount,
    maxAttempts,
  }
}

function getMockCountableArray(value: unknown): unknown[] | undefined {
  if (Array.isArray(value)) return value
  if (!value || typeof value !== 'object') return undefined
  const record = value as Record<string, unknown>
  if (Array.isArray(record.items)) return record.items
  if (Array.isArray(record.prompts)) return record.prompts
  if (Array.isArray(record.narrations)) return record.narrations
  return undefined
}

function stripFencedJson(raw: string) {
  const trimmed = raw.trim()
  if (!trimmed.startsWith('```')) return trimmed
  const withoutStart = trimmed.replace(/^```(?:json|JSON)?\n/, '')
  return withoutStart.replace(/\n```$/, '').trim()
}

function validateMockSchema(value: unknown, schema: Record<string, unknown>, path: string, errors: string[]) {
  const schemaType = typeof schema.type === 'string' ? schema.type : undefined
  if (schemaType && !matchesMockType(value, schemaType)) {
    errors.push(`${path}: expected type ${schemaType}`)
    return
  }
  const enumValues = Array.isArray(schema.enum) ? schema.enum : undefined
  if (enumValues && !enumValues.some((item) => item === value)) errors.push(`${path}: value is not in enum`)
  const required = Array.isArray(schema.required) ? schema.required.filter((item): item is string => typeof item === 'string') : []
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    const record = value as Record<string, unknown>
    for (const field of required) {
      if (!(field in record)) errors.push(`${path}.${field}: required field missing`)
    }
    const properties = schema.properties && typeof schema.properties === 'object' && !Array.isArray(schema.properties)
      ? schema.properties as Record<string, Record<string, unknown>>
      : {}
    for (const [field, childSchema] of Object.entries(properties)) {
      if (field in record) validateMockSchema(record[field], childSchema, `${path}.${field}`, errors)
    }
  }
  if (Array.isArray(value)) {
    const minItems = typeof schema.minItems === 'number' ? schema.minItems : undefined
    const maxItems = typeof schema.maxItems === 'number' ? schema.maxItems : undefined
    if (minItems !== undefined && value.length < minItems) errors.push(`${path}: array length is less than minItems`)
    if (maxItems !== undefined && value.length > maxItems) errors.push(`${path}: array length exceeds maxItems`)
    if (schema.items && typeof schema.items === 'object' && !Array.isArray(schema.items)) {
      value.forEach((item, index) => validateMockSchema(item, schema.items as Record<string, unknown>, `${path}[${index}]`, errors))
    }
  }
}

function matchesMockType(value: unknown, schemaType: string) {
  if (schemaType === 'object') return Boolean(value) && typeof value === 'object' && !Array.isArray(value)
  if (schemaType === 'array') return Array.isArray(value)
  if (schemaType === 'string') return typeof value === 'string'
  if (schemaType === 'number') return typeof value === 'number'
  if (schemaType === 'integer') return Number.isInteger(value)
  if (schemaType === 'boolean') return typeof value === 'boolean'
  if (schemaType === 'null') return value === null
  return true
}

function emptyCreativeRuleReferenceCounts() {
  return {
    videoPacks: 0,
    projects: 0,
    taskSteps: 0,
    generationContexts: 0,
  }
}

function creativeRuleReferenceTotal(counts: CreativeRuleDto['referenceCounts']) {
  return counts.videoPacks + counts.projects + counts.taskSteps + counts.generationContexts
}

function mockRuleRef(slot: string, ruleKey: string) {
  const rule = creativeRules.find((item) => item.key === ruleKey)
  if (!rule) throw new Error(`creative_rule.not_found: ${ruleKey}`)
  return {
    slot,
    ruleKey: rule.key,
    ruleId: rule.ruleId,
    sourceType: rule.sourceType,
    ruleType: rule.ruleType,
    module: rule.module,
    name: rule.name,
    version: rule.version,
    contentHash: rule.contentHash,
    schemaHash: rule.schemaHash,
    enabled: rule.enabled,
  }
}

function refreshMockCreativeRuleReferenceCounts() {
  for (const rule of creativeRules) {
    rule.referenceCounts = emptyCreativeRuleReferenceCounts()
    for (const pack of videoPacks) {
      if (Object.values(pack.ruleRefs).some((ref) => ref.ruleId === rule.ruleId || ref.ruleKey === rule.key)) {
        rule.referenceCounts.videoPacks += 1
      }
    }
  }
}

function ensureMockCreativeRules() {
  if (creativeRules.length > 0) return
  const seeds: Array<Pick<CreativeRuleDto, 'key' | 'name' | 'module' | 'ruleType' | 'description' | 'body'>> = [
    {
      key: 'script.topic_narration',
      name: 'Topic Narration',
      module: 'script',
      ruleType: 'script_rule',
      description: 'Generate a short-video narration draft from a topic.',
      body: 'Create a concise narration plan from {{project.topic}}. Return JSON only.',
    },
    {
      key: 'storyboard.default',
      name: 'Default Storyboard',
      module: 'storyboard',
      ruleType: 'storyboard_rule',
      description: 'Split narration into storyboard items.',
      body: 'Create storyboard items from {{script.narrations}}. Return JSON only.',
    },
    {
      key: 'character.default',
      name: 'Character Bible Rule',
      module: 'character',
      ruleType: 'character_rule',
      description: 'Extract editable character bible fields.',
      body: 'Create character bibles with stable IDs and editable descriptions. Return JSON only.',
    },
    {
      key: 'scene.default',
      name: 'Scene Bible Rule',
      module: 'scene',
      ruleType: 'scene_rule',
      description: 'Extract reusable scene bible fields.',
      body: 'Create location bibles with lighting, props, and variants. Return JSON only.',
    },
    {
      key: 'style.default',
      name: 'Style Bible Rule',
      module: 'style',
      ruleType: 'style_rule',
      description: 'Maintain structured project style rules.',
      body: 'Normalize style prompt, palette, lighting, composition, and negative prompt. Return JSON only.',
    },
    {
      key: 'image_prompt.shot_frame',
      name: 'Storyboard Image Prompt',
      module: 'image_prompt',
      ruleType: 'image_prompt_rule',
      description: 'Create image prompts from storyboard and bible context.',
      body: 'Write image prompts using style and character bibles. Return JSON only.',
    },
    {
      key: 'storyboard_image.default',
      name: 'Storyboard Image Generation Rule',
      module: 'storyboard_image',
      ruleType: 'storyboard_image_rule',
      description: 'Define image generation params for storyboard rows.',
      body: 'Prepare storyboard image generation parameters and validation notes. Return JSON only.',
    },
    {
      key: 'video_prompt.image_to_video',
      name: 'Image To Video Prompt',
      module: 'video_prompt',
      ruleType: 'video_prompt_rule',
      description: 'Create motion prompts for image-to-video generation.',
      body: 'Write video motion prompts from selected images and storyboard intent. Return JSON only.',
    },
    {
      key: 'review.safety',
      name: 'Safety Review',
      module: 'review',
      ruleType: 'review_rule',
      description: 'Review generated text for safety and missing inputs.',
      body: 'Review the generation result for missing fields and unsafe wording. Return JSON only.',
    },
  ]
  for (const seed of seeds) {
    creativeRules.push({
      ...seed,
      ruleId: `builtin:${seed.module}:${seed.key.replaceAll('.', '_')}`,
      providerKind: 'llm',
      version: '1.0.0',
      outputSchema: { type: 'object' },
      paramsSchema: { type: 'object' },
      sourceType: 'builtin',
      enabled: true,
      relativePath: `prompts/builtin/${seed.module}/${seed.key.replaceAll('.', '_')}.md`,
      contentHash: mockRuleHash([seed.key, seed.module, seed.ruleType, 'llm', '1.0.0', seed.body, JSON.stringify({ type: 'object' })]),
      schemaHash: mockRuleHash([JSON.stringify({ type: 'object' }), JSON.stringify({ type: 'object' })]),
      referenceCounts: emptyCreativeRuleReferenceCounts(),
    })
  }
}

function mockRuleHash(parts: string[]) {
  let hash = 0
  for (const char of parts.join('\n')) {
    hash = (hash * 31 + char.charCodeAt(0)) >>> 0
  }
  return `hash_${hash.toString(16).padStart(8, '0')}`
}

function ensureMockVideoPacks() {
  if (videoPacks.length > 0) return
  ensureMockCreativeRules()
  videoPacks.push(
    {
      packId: 'pack_knowledge_short',
      sourceType: 'builtin',
      name: '知识科普短视频',
      description: '适合 30-90 秒竖屏知识内容，强调结构清楚、信息密度和镜头可生成性。',
      applicableInputTypes: ['topic', 'paste', 'article'],
      contentCategory: 'knowledge',
      defaultTone: '清楚、克制、有信息量',
      defaultAspectRatio: '9:16',
      defaultDurationSeconds: 60,
      defaultSceneCount: 8,
      ruleRefs: {
        script: mockRuleRef('script', 'script.topic_narration'),
        storyboard: mockRuleRef('storyboard', 'storyboard.default'),
        character: mockRuleRef('character', 'character.default'),
        scene: mockRuleRef('scene', 'scene.default'),
        style: mockRuleRef('style', 'style.default'),
        image_prompt: mockRuleRef('image_prompt', 'image_prompt.shot_frame'),
        storyboard_image: mockRuleRef('storyboard_image', 'storyboard_image.default'),
        video_prompt: mockRuleRef('video_prompt', 'video_prompt.image_to_video'),
        review: mockRuleRef('review', 'review.safety'),
      },
      recommendedExecutableRefs: {
        llm: {},
        image: {},
        video: {},
      },
      assetRefs: [],
      isEnabled: true,
      projectReferenceCount: 0,
    },
    {
      packId: 'pack_story_short',
      sourceType: 'builtin',
      name: '故事叙事短视频',
      description: '适合人物故事、经历复盘和情绪叙事，保留角色、场景和镜头提示词的连续性。',
      applicableInputTypes: ['topic', 'paste', 'article'],
      contentCategory: 'story',
      defaultTone: '真实、克制、带一点悬念',
      defaultAspectRatio: '9:16',
      defaultDurationSeconds: 45,
      defaultSceneCount: 7,
      ruleRefs: {
        script: mockRuleRef('script', 'script.topic_narration'),
        storyboard: mockRuleRef('storyboard', 'storyboard.default'),
        character: mockRuleRef('character', 'character.default'),
        scene: mockRuleRef('scene', 'scene.default'),
        style: mockRuleRef('style', 'style.default'),
        image_prompt: mockRuleRef('image_prompt', 'image_prompt.shot_frame'),
        storyboard_image: mockRuleRef('storyboard_image', 'storyboard_image.default'),
        video_prompt: mockRuleRef('video_prompt', 'video_prompt.image_to_video'),
        review: mockRuleRef('review', 'review.safety'),
      },
      recommendedExecutableRefs: {
        llm: {},
        image: {},
        video: {},
      },
      assetRefs: [],
      isEnabled: true,
      projectReferenceCount: 0,
    },
  )
  refreshMockCreativeRuleReferenceCounts()
}
