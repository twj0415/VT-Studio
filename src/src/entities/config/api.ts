import { callCommand } from '@/shared/api/client'
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

export function getAppConfig(): Promise<AppConfigDto> {
  return callCommand<AppConfigDto>(tauriCommands.getAppConfig)
}

export async function updateAppConfig(patch: Partial<AppConfigDto>): Promise<AppConfigDto> {
  const current = await getAppConfig()
  return callCommand<AppConfigDto, { config: AppConfigDto }>(tauriCommands.updateAppConfig, { config: { ...current, ...patch } })
}

export function saveProviderSecret(request: SaveProviderSecretRequest): Promise<ProviderSecretHandleDto> {
  return callCommand<ProviderSecretHandleDto, { request: SaveProviderSecretRequest }>(tauriCommands.saveProviderSecret, { request })
}

export function deleteProviderSecret(keyAlias: string): Promise<ProviderSecretStatusDto> {
  return callCommand<ProviderSecretStatusDto, { request: { keyAlias: string } }>(tauriCommands.deleteProviderSecret, { request: { keyAlias } })
}

export function hasProviderSecret(keyAlias: string): Promise<ProviderSecretStatusDto> {
  return callCommand<ProviderSecretStatusDto, { request: { keyAlias: string } }>(tauriCommands.hasProviderSecret, { request: { keyAlias } })
}

export function checkFfmpegSidecars(): Promise<FfmpegSidecarStatusDto> {
  return callCommand<FfmpegSidecarStatusDto>(tauriCommands.checkFfmpegSidecars)
}

export function getAppReleaseInfo(): Promise<AppReleaseInfoDto> {
  return callCommand<AppReleaseInfoDto>(tauriCommands.getAppReleaseInfo)
}

export function runRuntimeSelfCheck(): Promise<RuntimeSelfCheckDto> {
  return callCommand<RuntimeSelfCheckDto>(tauriCommands.runRuntimeSelfCheck)
}

export function probeMedia(request: ProbeMediaRequest): Promise<MediaProbeDto> {
  return callCommand<MediaProbeDto, { request: ProbeMediaRequest }>(tauriCommands.probeMedia, { request })
}

export function listProviderConfigs(request: ListProviderConfigsRequest = {}): Promise<ProviderConfigDto[]> {
  return callCommand<ProviderConfigDto[], { request: ListProviderConfigsRequest }>(tauriCommands.listProviderConfigs, { request })
}

export function upsertProviderConfig(config: ProviderConfigDto): Promise<ProviderConfigDto> {
  return callCommand<ProviderConfigDto, { config: ProviderConfigDto }>(tauriCommands.upsertProviderConfig, { config })
}

export function deleteProviderConfig(request: DeleteProviderConfigRequest): Promise<ProviderConfigDto> {
  return callCommand<ProviderConfigDto, { request: DeleteProviderConfigRequest }>(tauriCommands.deleteProviderConfig, { request })
}

export function listProviderModels(request: ListProviderModelsRequest = {}): Promise<ProviderModelDto[]> {
  return callCommand<ProviderModelDto[], { request: ListProviderModelsRequest }>(tauriCommands.listProviderModels, { request })
}

export function upsertProviderModel(model: ProviderModelDto): Promise<ProviderModelDto> {
  return callCommand<ProviderModelDto, { model: ProviderModelDto }>(tauriCommands.upsertProviderModel, { model })
}

export function deleteProviderModel(request: DeleteProviderModelRequest): Promise<ProviderModelDto> {
  return callCommand<ProviderModelDto, { request: DeleteProviderModelRequest }>(tauriCommands.deleteProviderModel, { request })
}

export function listWorkflowPresets(request: ListWorkflowPresetsRequest = {}): Promise<WorkflowPresetDto[]> {
  return callCommand<WorkflowPresetDto[], { request: ListWorkflowPresetsRequest }>(tauriCommands.listWorkflowPresets, { request })
}

export function upsertWorkflowPreset(preset: WorkflowPresetDto): Promise<WorkflowPresetDto> {
  return callCommand<WorkflowPresetDto, { preset: WorkflowPresetDto }>(tauriCommands.upsertWorkflowPreset, { preset })
}

export function deleteWorkflowPreset(request: DeleteWorkflowPresetRequest): Promise<WorkflowPresetDto> {
  return callCommand<WorkflowPresetDto, { request: DeleteWorkflowPresetRequest }>(tauriCommands.deleteWorkflowPreset, { request })
}

export function listExecutableMediaOptions(): Promise<ExecutableMediaOptionDto[]> {
  return callCommand<ExecutableMediaOptionDto[]>(tauriCommands.listExecutableMediaOptions)
}

export function listVideoPacks(request: ListVideoPacksRequest = {}): Promise<VideoPackDto[]> {
  return callCommand<VideoPackDto[], { request: ListVideoPacksRequest }>(tauriCommands.listVideoPacks, { request })
}

export function getVideoPack(request: VideoPackIdRequest): Promise<VideoPackDto> {
  return callCommand<VideoPackDto, { request: VideoPackIdRequest }>(tauriCommands.getVideoPack, { request })
}

export function cloneVideoPackToUser(request: VideoPackIdRequest): Promise<VideoPackDto> {
  return callCommand<VideoPackDto, { request: VideoPackIdRequest }>(tauriCommands.cloneVideoPackToUser, { request })
}

export function upsertUserVideoPack(request: UpsertUserVideoPackRequest): Promise<VideoPackDto> {
  return callCommand<VideoPackDto, { request: UpsertUserVideoPackRequest }>(tauriCommands.upsertUserVideoPack, { request })
}

export function setVideoPackEnabled(request: SetVideoPackEnabledRequest): Promise<VideoPackDto> {
  return callCommand<VideoPackDto, { request: SetVideoPackEnabledRequest }>(tauriCommands.setVideoPackEnabled, { request })
}

export function deleteUserVideoPack(request: VideoPackIdRequest): Promise<VideoPackDto> {
  return callCommand<VideoPackDto, { request: VideoPackIdRequest }>(tauriCommands.deleteUserVideoPack, { request })
}

export function saveProjectConfigAsVideoPack(request: SaveProjectConfigAsVideoPackRequest): Promise<VideoPackDto> {
  return callCommand<VideoPackDto, { request: SaveProjectConfigAsVideoPackRequest }>(tauriCommands.saveProjectConfigAsVideoPack, { request })
}

export function getProjectStyleBible(projectId: string): Promise<StyleBibleDto> {
  return callCommand<StyleBibleDto>(tauriCommands.getProjectStyleBible, { projectId })
}

export function listProjectCharacterBibles(projectId: string): Promise<CharacterBibleDto[]> {
  return callCommand<CharacterBibleDto[]>(tauriCommands.listProjectCharacterBibles, { projectId })
}

export function upsertProjectCharacterBible(request: UpsertProjectCharacterBibleRequest): Promise<CharacterBibleDto> {
  return callCommand<CharacterBibleDto, { request: UpsertProjectCharacterBibleRequest }>(tauriCommands.upsertProjectCharacterBible, { request })
}

export function deleteProjectCharacterBible(request: CharacterBibleIdRequest): Promise<CharacterBibleDto> {
  return callCommand<CharacterBibleDto, { request: CharacterBibleIdRequest }>(tauriCommands.deleteProjectCharacterBible, { request })
}

export function bindCharacterReferenceAsset(request: BindCharacterReferenceAssetRequest): Promise<BindCharacterReferenceAssetResponse> {
  return callCommand<BindCharacterReferenceAssetResponse, { request: BindCharacterReferenceAssetRequest }>(tauriCommands.bindCharacterReferenceAsset, { request })
}

export function listProjectLocationBibles(projectId: string): Promise<LocationBibleDto[]> {
  return callCommand<LocationBibleDto[]>(tauriCommands.listProjectLocationBibles, { projectId })
}

export function upsertProjectLocationBible(request: UpsertProjectLocationBibleRequest): Promise<LocationBibleDto> {
  return callCommand<LocationBibleDto, { request: UpsertProjectLocationBibleRequest }>(tauriCommands.upsertProjectLocationBible, { request })
}

export function deleteProjectLocationBible(request: LocationBibleIdRequest): Promise<LocationBibleDto> {
  return callCommand<LocationBibleDto, { request: LocationBibleIdRequest }>(tauriCommands.deleteProjectLocationBible, { request })
}

export function bindLocationReferenceAsset(request: BindLocationReferenceAssetRequest): Promise<BindLocationReferenceAssetResponse> {
  return callCommand<BindLocationReferenceAssetResponse, { request: BindLocationReferenceAssetRequest }>(tauriCommands.bindLocationReferenceAsset, { request })
}

export function analyzeStyleReferenceImage(request: AnalyzeStyleReferenceRequest): Promise<StyleReferenceAnalysisDto> {
  return callCommand<StyleReferenceAnalysisDto, { request: AnalyzeStyleReferenceRequest }>(tauriCommands.analyzeStyleReferenceImage, { request })
}

export function listStylePresets(): Promise<StylePresetDto[]> {
  return callCommand<StylePresetDto[]>(tauriCommands.listStylePresets)
}

export function upsertProjectStyleBible(request: UpsertProjectStyleBibleRequest): Promise<StyleBibleDto> {
  return callCommand<StyleBibleDto, { request: UpsertProjectStyleBibleRequest }>(tauriCommands.upsertProjectStyleBible, { request })
}

export function applyStylePreset(request: ApplyStylePresetRequest): Promise<StyleBibleDto> {
  return callCommand<StyleBibleDto, { request: ApplyStylePresetRequest }>(tauriCommands.applyStylePreset, { request })
}

export function bindStyleReferenceAsset(request: BindStyleReferenceAssetRequest): Promise<BindStyleReferenceAssetResponse> {
  return callCommand<BindStyleReferenceAssetResponse, { request: BindStyleReferenceAssetRequest }>(tauriCommands.bindStyleReferenceAsset, { request })
}

export function buildImagePromptPreview(request: BuildImagePromptPreviewRequest): Promise<ImagePromptPreviewDto> {
  return callCommand<ImagePromptPreviewDto, { request: BuildImagePromptPreviewRequest }>(tauriCommands.buildImagePromptPreview, { request })
}

export function providerDryRun(request: ProviderDryRunRequest): Promise<ProviderDryRunResponse> {
  return callCommand<ProviderDryRunResponse, { request: ProviderDryRunRequest }>(tauriCommands.providerDryRun, { request })
}

export function providerGenerationTest(request: ProviderGenerationTestRequest): Promise<ProviderGenerationTestResponse> {
  return callCommand<ProviderGenerationTestResponse, { request: ProviderGenerationTestRequest }>(tauriCommands.providerGenerationTest, { request })
}

export function importAsset(request: ImportAssetRequest): Promise<AssetDto> {
  return callCommand<AssetDto, { request: ImportAssetRequest }>(tauriCommands.importAsset, { request })
}

export function readAssetPreview(request: AssetPreviewRequest): Promise<AssetPreviewDto> {
  return callCommand<AssetPreviewDto, { request: AssetPreviewRequest }>(tauriCommands.readAssetPreview, { request })
}

export function listAssets(request: ListAssetsRequest = {}): Promise<AssetDto[]> {
  return callCommand<AssetDto[], { request: ListAssetsRequest }>(tauriCommands.listAssets, { request })
}

export function deleteAsset(request: DeleteAssetRequest): Promise<AssetDto> {
  return callCommand<AssetDto, { request: DeleteAssetRequest }>(tauriCommands.deleteAsset, { request })
}

export function createAssetReference(request: CreateAssetReferenceRequest): Promise<AssetReferenceDto> {
  return callCommand<AssetReferenceDto, { request: CreateAssetReferenceRequest }>(tauriCommands.createAssetReference, { request })
}

export function listAssetReferences(assetId: string): Promise<AssetReferenceDto[]> {
  return callCommand<AssetReferenceDto[]>(tauriCommands.listAssetReferences, { assetId })
}

export function deleteAssetReference(request: DeleteAssetReferenceRequest): Promise<AssetReferenceDto> {
  return callCommand<AssetReferenceDto, { request: DeleteAssetReferenceRequest }>(tauriCommands.deleteAssetReference, { request })
}

export function collectProjectAssetPaths(projectId: string): Promise<string[]> {
  return callCommand<string[]>(tauriCommands.collectProjectAssetPaths, { projectId })
}

export function listCreativeRules(request: ListCreativeRulesRequest = {}): Promise<CreativeRuleDto[]> {
  return callCommand<CreativeRuleDto[], { request: ListCreativeRulesRequest }>(tauriCommands.listCreativeRules, { request })
}

export function getCreativeRule(request: CreativeRuleIdRequest): Promise<CreativeRuleDto> {
  return callCommand<CreativeRuleDto, { request: CreativeRuleIdRequest }>(tauriCommands.getCreativeRule, { request })
}

export function cloneCreativeRuleToUser(request: CreativeRuleIdRequest): Promise<CreativeRuleDto> {
  return callCommand<CreativeRuleDto, { request: CreativeRuleIdRequest }>(tauriCommands.cloneCreativeRuleToUser, { request })
}

export function saveUserCreativeRule(request: SaveCreativeRuleRequest): Promise<CreativeRuleDto> {
  return callCommand<CreativeRuleDto, { request: SaveCreativeRuleRequest }>(tauriCommands.saveUserCreativeRule, { request })
}

export function setUserCreativeRuleEnabled(request: SetCreativeRuleEnabledRequest): Promise<CreativeRuleDto> {
  return callCommand<CreativeRuleDto, { request: SetCreativeRuleEnabledRequest }>(tauriCommands.setUserCreativeRuleEnabled, { request })
}

export function deleteUserCreativeRule(request: CreativeRuleIdRequest): Promise<CreativeRuleDto> {
  return callCommand<CreativeRuleDto, { request: CreativeRuleIdRequest }>(tauriCommands.deleteUserCreativeRule, { request })
}

export function validateStructuredOutput(request: ValidateStructuredOutputRequest): Promise<StructuredOutputValidationResult> {
  return callCommand<StructuredOutputValidationResult, { request: ValidateStructuredOutputRequest }>(tauriCommands.validateStructuredOutput, { request })
}
