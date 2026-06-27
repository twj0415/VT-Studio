<template>
  <section class="view">
    <div class="gpage">
      <div class="wrap flex min-h-0 flex-1 flex-col">
        <div class="phead">
          <div>
            <h1>{{ t('settings.title') }}</h1>
            <div class="desc">{{ t('settings.desc') }}</div>
          </div>
        </div>

        <div class="set-layout">
          <nav class="set-nav">
            <button v-for="item in navItems" :key="item.key" type="button" class="sn text-left" :class="{ active: activeSection === item.key }" @click="activeSection = item.key">
              {{ item.label }}
            </button>
          </nav>

          <div class="set-form">
            <template v-if="activeSection === 'appearance'">
              <div class="setcard">
                <h4>{{ t('settings.themeTitle') }}</h4>
                <div class="mb-vt-4 text-[12.5px] text-muted">{{ t('settings.themeHint') }}</div>
                <div class="preset-row">
                  <button v-for="option in themePresetOptions" :key="option.value" type="button" class="preset text-left" :class="{ sel: appStore.themePreset === option.value }" @click="setTheme(option.value)">
                    <div class="swatch" :style="presetStyle(option.value)"><span class="sw-card" :style="presetCardStyle(option.value)"></span></div>
                    <div class="pn">{{ option.label }}</div>
                  </button>
                </div>
              </div>

              <div class="setcard">
                <h4>{{ t('settings.interfaceTitle') }}</h4>
                <div class="setrow">
                  <span class="k">{{ t('settings.appLocale') }}</span>
                  <div class="opt-row">
                    <button v-for="option in appLocaleOptions" :key="option.value" type="button" class="opt" :class="{ sel: appStore.appLocale === option.value }" @click="setLocale(option.value)">{{ option.label }}</button>
                  </div>
                </div>
                <div class="setrow">
                  <span class="k">{{ t('settings.density') }}</span>
                  <div class="opt-row">
                    <button v-for="option in layoutDensityOptions" :key="option.value" type="button" class="opt" :class="{ sel: appStore.layoutDensity === option.value }" @click="setDensity(option.value)">{{ option.label }}</button>
                  </div>
                </div>
              </div>

              <div class="setcard">
                <h4>{{ t('settings.release.title') }}</h4>
                <div class="mb-vt-3 text-[12.5px] text-muted">{{ t('settings.release.desc') }}</div>
                <div class="setrow"><span class="k">{{ t('settings.release.version') }}</span><span class="v">{{ releaseInfo?.version || '-' }}</span></div>
                <div class="setrow"><span class="k">{{ t('settings.release.targetPlatform') }}</span><span class="v">{{ releaseInfo?.targetPlatform || '-' }}</span></div>
                <div class="setrow"><span class="k">{{ t('settings.release.updateChannel') }}</span><span class="v">{{ releaseInfo ? releaseChannelLabel : '-' }}</span></div>
                <div class="setrow"><span class="k">{{ t('settings.release.autoUpdate') }}</span><span class="v">{{ releaseInfo ? booleanLabel(releaseInfo.autoUpdateEnabled) : '-' }}</span></div>
                <div class="setrow"><span class="k">{{ t('settings.release.installerSignature') }}</span><span class="v">{{ releaseInfo ? signatureLabel : '-' }}</span></div>
                <div class="setrow"><span class="k">{{ t('settings.release.updateSecurity') }}</span><span class="v">{{ releaseInfo ? updateSecurityLabel : '-' }}</span></div>
              </div>
            </template>

            <template v-else-if="activeSection === 'provider'">
              <div class="setcard">
                <div class="flex items-start justify-between gap-vt-3">
                  <div>
                    <h4>{{ t('settings.nav.provider') }}</h4>
                    <div class="mt-vt-1 text-[12.5px] text-muted">{{ t('settings.providerSummary', { count: providers.length }) }}</div>
                  </div>
                  <button type="button" class="inline-flex h-8 items-center rounded-vt-sm border border-border-strong px-vt-3 text-xs font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="settingsLoading" @click="loadSettingsData">
                    {{ settingsLoading ? t('common.loading') : t('common.refresh') }}
                  </button>
                </div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <div v-if="providers.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">{{ t('settings.emptyProviders') }}</div>
                  <div v-for="provider in providers" :key="provider.providerId" class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-start justify-between gap-vt-2">
                      <div class="min-w-0">
                        <div class="font-semibold text-primary">{{ provider.displayName }}</div>
                        <div class="mt-vt-1 break-all font-mono text-[11px] text-muted">{{ provider.providerId }} · {{ provider.vendor }} · {{ provider.providerKind }}</div>
                        <div class="mt-vt-1 break-all text-xs text-muted">{{ provider.baseUrl || '-' }}</div>
                      </div>
                      <span class="rounded-vt-sm border px-vt-2 py-1 text-xs" :class="provider.isEnabled ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-card text-muted'">{{ provider.status }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </template>

            <template v-else-if="activeSection === 'capability'">
              <div class="setcard">
                <h4>{{ t('settings.nav.capability') }}</h4>
                <div class="mb-vt-3 text-[12.5px] text-muted">{{ t('settings.capabilitySummary', { models: providerModels.length, workflows: workflowPresets.length }) }}</div>
                <div class="grid gap-vt-2">
                  <div v-if="providerModels.length === 0 && workflowPresets.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">{{ t('settings.emptyCapabilities') }}</div>
                  <div v-for="model in providerModels" :key="model.modelId" class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-start justify-between gap-vt-2">
                      <div class="min-w-0">
                        <div class="font-semibold text-primary">{{ model.displayName }}</div>
                        <div class="mt-vt-1 break-all font-mono text-[11px] text-muted">{{ model.providerId }} · {{ model.providerModelId }}</div>
                        <div class="mt-vt-1 text-xs text-muted">{{ model.abilityTypes.join(' / ') }}</div>
                      </div>
                      <span class="rounded-vt-sm border px-vt-2 py-1 text-xs" :class="model.isEnabled ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-card text-muted'">{{ model.status }}</span>
                    </div>
                  </div>
                  <div v-for="preset in workflowPresets" :key="preset.workflowPresetId" class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-start justify-between gap-vt-2">
                      <div class="min-w-0">
                        <div class="font-semibold text-primary">{{ preset.displayName }}</div>
                        <div class="mt-vt-1 break-all font-mono text-[11px] text-muted">{{ preset.providerId }} · {{ preset.workflowKey }} · {{ preset.workflowVersion }}</div>
                        <div class="mt-vt-1 text-xs text-muted">{{ preset.abilityTypes.join(' / ') }}</div>
                      </div>
                      <span class="rounded-vt-sm border px-vt-2 py-1 text-xs" :class="preset.isEnabled ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-card text-muted'">{{ preset.status }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </template>

            <template v-else-if="activeSection === 'secrets'">
              <div class="setcard">
                <h4>{{ t('settings.nav.secrets') }}</h4>
                <div class="mb-vt-3 text-[12.5px] text-muted">{{ t('settings.secretsOnlyKeyring') }}</div>
                <div class="grid gap-vt-2">
                  <div v-if="providers.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">{{ t('settings.emptyProviders') }}</div>
                  <div v-for="provider in providers" :key="provider.providerId" class="setrow">
                    <span class="k">{{ provider.displayName }}</span>
                    <span class="v">{{ provider.authType }} / {{ provider.keyAlias || '-' }}</span>
                  </div>
                </div>
              </div>
            </template>

            <template v-else-if="activeSection === 'path'">
              <div class="setcard">
                <div class="flex items-start justify-between gap-vt-3">
                  <div>
                    <h4>{{ t('settings.sidecar.title') }}</h4>
                    <div class="mt-vt-1 text-[12.5px] text-muted">{{ t('settings.sidecar.desc') }}</div>
                  </div>
                  <button type="button" class="inline-flex h-8 flex-none items-center rounded-vt-sm border border-border-strong px-vt-3 text-xs font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="sidecarLoading" @click="loadSidecars">
                    {{ sidecarLoading ? t('settings.sidecar.checking') : t('settings.sidecar.refresh') }}
                  </button>
                </div>
                <div class="mt-vt-3 flex flex-wrap items-center gap-vt-2 text-xs">
                  <span class="rounded-vt-sm border px-vt-2 py-1" :class="sidecarStatus?.ready ? 'border-accent-line bg-accent-soft text-accent' : 'border-status-failed/50 bg-status-failed/10 text-status-failed'">{{ sidecarReadyLabel }}</span>
                  <span class="rounded-vt-sm border px-vt-2 py-1" :class="templateSidecarStatus?.ready ? 'border-accent-line bg-accent-soft text-accent' : 'border-status-failed/50 bg-status-failed/10 text-status-failed'">{{ templateSidecarReadyLabel }}</span>
                  <span v-if="sidecarStatus?.checkedAt" class="text-muted">{{ t('settings.sidecar.checkedAt', { time: sidecarCheckedAt }) }}</span>
                </div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <div class="text-xs font-semibold text-secondary">{{ t('settings.sidecar.runtimeGroup') }}</div>
                  <div class="grid gap-vt-2 sm:grid-cols-2">
                    <div v-for="item in runtimeCheckItems" :key="item.key" class="rounded-vt-sm border border-border bg-page p-vt-3">
                      <div class="flex items-start justify-between gap-vt-2">
                        <div class="min-w-0">
                          <div class="text-xs font-semibold text-primary">{{ runtimeCheckLabel(item.key) }}</div>
                          <div class="mt-vt-1 text-xs text-muted">{{ item.message || runtimeCheckMessage(item) }}</div>
                        </div>
                        <span class="rounded-vt-sm border px-vt-2 py-1 text-xs" :class="item.ready ? 'border-accent-line bg-accent-soft text-accent' : item.skipped ? 'border-border-strong bg-card text-muted' : 'border-status-failed/50 bg-status-failed/10 text-status-failed'">{{ runtimeCheckStatusLabel(item) }}</span>
                      </div>
                      <div v-if="item.errorCode" class="mt-vt-2 font-mono text-[11px] text-status-failed">{{ item.errorCode }}</div>
                    </div>
                  </div>
                  <div class="text-xs font-semibold text-secondary">{{ t('settings.sidecar.mediaGroup') }}</div>
                  <div v-for="binary in sidecarBinaries" :key="binary.name" class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-start justify-between gap-vt-2">
                      <div class="min-w-0">
                        <div class="font-mono text-xs font-semibold text-primary">{{ binary.relativePath }}</div>
                        <div class="mt-vt-1 text-xs text-muted">{{ binary.version || binary.message || t('settings.sidecar.noVersion') }}</div>
                      </div>
                      <span class="rounded-vt-sm border px-vt-2 py-1 text-xs" :class="binary.executable ? 'border-accent-line bg-accent-soft text-accent' : 'border-status-failed/50 bg-status-failed/10 text-status-failed'">{{ binaryLabel(binary) }}</span>
                    </div>
                    <div v-if="binary.errorCode" class="mt-vt-2 font-mono text-[11px] text-status-failed">{{ binary.errorCode }}</div>
                  </div>
                  <div class="mt-vt-2 text-xs font-semibold text-secondary">{{ t('settings.sidecar.templateGroup') }}</div>
                  <div v-for="binary in templateSidecarBinaries" :key="binary.name" class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-start justify-between gap-vt-2">
                      <div class="min-w-0">
                        <div class="font-mono text-xs font-semibold text-primary">{{ binary.relativePath }}</div>
                        <div class="mt-vt-1 text-xs text-muted">{{ binary.version || binary.message || t('settings.sidecar.noVersion') }}</div>
                      </div>
                      <span class="rounded-vt-sm border px-vt-2 py-1 text-xs" :class="binary.executable ? 'border-accent-line bg-accent-soft text-accent' : 'border-status-failed/50 bg-status-failed/10 text-status-failed'">{{ binaryLabel(binary) }}</span>
                    </div>
                    <div v-if="binary.errorCode" class="mt-vt-2 font-mono text-[11px] text-status-failed">{{ binary.errorCode }}</div>
                  </div>
                </div>
              </div>
            </template>

            <template v-else-if="activeSection === 'defaults'">
              <div class="setcard">
                <h4>{{ t('settings.nav.defaults') }}</h4>
                <div class="grid gap-vt-2">
                  <div class="setrow"><span class="k">{{ t('settings.defaultTheme') }}</span><span class="v">{{ appStore.themePreset }}</span></div>
                  <div class="setrow"><span class="k">{{ t('settings.defaultLocale') }}</span><span class="v">{{ appStore.appLocale }}</span></div>
                  <div class="setrow"><span class="k">{{ t('settings.defaultDensity') }}</span><span class="v">{{ appStore.layoutDensity }}</span></div>
                </div>
              </div>
            </template>

            <template v-else>
              <div class="setcard">
                <div class="flex items-start justify-between gap-vt-3">
                  <div>
                    <h4>{{ t('settings.nav.importExport') }}</h4>
                    <div class="mt-vt-1 text-[12.5px] text-muted">{{ t('settings.exportNoSecrets') }}</div>
                  </div>
                </div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <button type="button" class="inline-flex h-9 w-fit items-center rounded-vt-sm border border-border-strong px-vt-3 text-xs font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isBackingUpWorkspace" @click="handleBackupWorkspace">
                    {{ isBackingUpWorkspace ? t('settings.backup.running') : t('settings.backup.workspace') }}
                  </button>
                  <button type="button" class="inline-flex h-9 w-fit items-center rounded-vt-sm border border-border-strong px-vt-3 text-xs font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isExportingDiagnostic" @click="handleExportDiagnostic">
                    {{ isExportingDiagnostic ? t('settings.backup.running') : t('settings.backup.diagnostic') }}
                  </button>
                  <div v-if="lastBackupPath" class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs">
                    <div class="text-muted">{{ t('settings.backup.lastWorkspace') }}</div>
                    <div class="mt-vt-1 break-all font-mono text-primary">{{ lastBackupPath }}</div>
                  </div>
                  <div v-if="lastDiagnosticPath" class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs">
                    <div class="text-muted">{{ t('settings.backup.lastDiagnostic') }}</div>
                    <div class="mt-vt-1 break-all font-mono text-primary">{{ lastDiagnosticPath }}</div>
                  </div>
                </div>
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { getAppReleaseInfo, listProviderConfigs, listProviderModels, listWorkflowPresets, runRuntimeSelfCheck } from '@/entities/config/api'
import { useConfigStore } from '@/entities/config/store'
import type { AppReleaseInfoDto, ProviderConfigDto, ProviderModelDto, RuntimeCheckItemDto, RuntimeSelfCheckDto, SidecarBinaryStatusDto, WorkflowPresetDto } from '@/entities/config/types'
import { backupWorkspace, exportDiagnosticPackage } from '@/entities/export/api'
import type { TemplateSidecarBinaryStatusDto, TemplateSidecarStatusDto } from '@/entities/template/types'
import { useDictOptions } from '@/shared/dict/useDictOptions'
import type { AppLocale, LayoutDensity, ThemePreset } from '@/shared/stores/appStore'
import { useAppStore } from '@/shared/stores/appStore'
import { getThemePreviewCardStyle, getThemePreviewStyle } from '@/shared/theme'

type SettingsSection = 'appearance' | 'provider' | 'capability' | 'secrets' | 'path' | 'defaults' | 'importExport'

const appStore = useAppStore()
const configStore = useConfigStore()
const message = useMessage()
const { t } = useI18n()
const themePresetOptions = useDictOptions('themePreset')
const appLocaleOptions = useDictOptions('appLocale')
const layoutDensityOptions = useDictOptions('layoutDensity')
const activeSection = ref<SettingsSection>('appearance')
const releaseInfo = ref<AppReleaseInfoDto | null>(null)
const runtimeSelfCheck = ref<RuntimeSelfCheckDto | null>(null)
const providers = ref<ProviderConfigDto[]>([])
const providerModels = ref<ProviderModelDto[]>([])
const workflowPresets = ref<WorkflowPresetDto[]>([])
const sidecarLoading = ref(false)
const settingsLoading = ref(false)
const isBackingUpWorkspace = ref(false)
const isExportingDiagnostic = ref(false)
const lastBackupPath = ref('')
const lastDiagnosticPath = ref('')

const navItems = computed(() => [
  { key: 'appearance' as const, label: t('settings.nav.appearance') },
  { key: 'provider' as const, label: t('settings.nav.provider') },
  { key: 'capability' as const, label: t('settings.nav.capability') },
  { key: 'secrets' as const, label: t('settings.nav.secrets') },
  { key: 'path' as const, label: t('settings.nav.path') },
  { key: 'defaults' as const, label: t('settings.nav.defaults') },
  { key: 'importExport' as const, label: t('settings.nav.importExport') },
])
const sidecarStatus = computed(() => runtimeSelfCheck.value?.ffmpeg ?? null)
const templateSidecarStatus = computed<TemplateSidecarStatusDto | null>(() => runtimeSelfCheck.value?.templateSidecar ?? null)
const runtimeCheckItems = computed(() => runtimeSelfCheck.value ? [
  runtimeSelfCheck.value.workspace,
  runtimeSelfCheck.value.sqlite,
  runtimeSelfCheck.value.templateManifest,
  runtimeSelfCheck.value.templatePreview,
] : [])
const sidecarBinaries = computed(() => (sidecarStatus.value ? [sidecarStatus.value.ffmpeg, sidecarStatus.value.ffprobe] : []))
const templateSidecarBinaries = computed(() => (templateSidecarStatus.value ? [templateSidecarStatus.value.node, templateSidecarStatus.value.chromium, templateSidecarStatus.value.playwrightDriver] : []))
const sidecarReadyLabel = computed(() => (sidecarStatus.value?.ready ? t('settings.sidecar.ready') : t('settings.sidecar.notReady')))
const templateSidecarReadyLabel = computed(() => (templateSidecarStatus.value?.ready ? t('settings.sidecar.templateReady') : t('settings.sidecar.templateNotReady')))
const sidecarCheckedAt = computed(() => formatCheckedAt(sidecarStatus.value?.checkedAt))
const releaseChannelLabel = computed(() => (releaseInfo.value?.updateChannel === 'manual' ? t('settings.release.manualChannel') : releaseInfo.value?.updateChannel ?? '-'))
const signatureLabel = computed(() => {
  if (!releaseInfo.value) return '-'
  if (!releaseInfo.value.installerSignatureRequired) return t('settings.release.signatureNotRequired')
  return releaseInfo.value.signedInstallerVerified ? t('settings.release.signatureVerified') : t('settings.release.signaturePending')
})
const updateSecurityLabel = computed(() => {
  if (!releaseInfo.value) return '-'
  const parts = [
    releaseInfo.value.updateRequiresHttps ? t('settings.release.httpsRequired') : t('settings.release.httpsMissing'),
    releaseInfo.value.updateSignatureRequired ? t('settings.release.updateSignatureRequired') : t('settings.release.updateSignatureMissing'),
  ]
  return parts.join(' / ')
})

onMounted(async () => {
  await Promise.all([loadAppConfig(), loadSettingsData(), loadSidecars()])
})

async function loadAppConfig() {
  const [config, release] = await Promise.all([
    configStore.loadConfig(),
    getAppReleaseInfo(),
  ])
  releaseInfo.value = release
  appStore.setThemePreset(config.themePreset)
  appStore.setAppLocale(config.appLocale)
  appStore.setLayoutDensity(config.layoutDensity)
}

async function loadSettingsData() {
  settingsLoading.value = true
  try {
    const [providerList, modelList, workflowList] = await Promise.all([
      listProviderConfigs({}),
      listProviderModels({}),
      listWorkflowPresets({}),
    ])
    providers.value = providerList
    providerModels.value = modelList
    workflowPresets.value = workflowList
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    settingsLoading.value = false
  }
}

function presetStyle(theme: ThemePreset) {
  return getThemePreviewStyle(theme)
}

function presetCardStyle(theme: ThemePreset) {
  return getThemePreviewCardStyle(theme)
}

function syncConfig() {
  void configStore.updateConfig({
    themePreset: appStore.themePreset,
    appLocale: appStore.appLocale,
    layoutDensity: appStore.layoutDensity,
  })
}

function setTheme(theme: ThemePreset) {
  appStore.setThemePreset(theme)
  syncConfig()
}

function setLocale(locale: AppLocale) {
  appStore.setAppLocale(locale)
  syncConfig()
}

function setDensity(density: LayoutDensity) {
  appStore.setLayoutDensity(density)
  syncConfig()
}

async function loadSidecars() {
  sidecarLoading.value = true
  try {
    runtimeSelfCheck.value = await runRuntimeSelfCheck()
  } catch (error) {
    message.error(t('settings.sidecar.checkFailed', { reason: errorMessage(error) }))
  } finally {
    sidecarLoading.value = false
  }
}

async function handleBackupWorkspace() {
  isBackingUpWorkspace.value = true
  try {
    const result = await backupWorkspace()
    lastBackupPath.value = result.targetRelativePath
    message.success(t('settings.backup.workspaceSuccess', { path: result.targetRelativePath }))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isBackingUpWorkspace.value = false
  }
}

async function handleExportDiagnostic() {
  isExportingDiagnostic.value = true
  try {
    const result = await exportDiagnosticPackage({ includeMedia: false })
    lastDiagnosticPath.value = result.targetRelativePath
    message.success(t('settings.backup.diagnosticSuccess', { path: result.targetRelativePath }))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isExportingDiagnostic.value = false
  }
}

function binaryLabel(binary: SidecarBinaryStatusDto | TemplateSidecarBinaryStatusDto) {
  if (binary.executable) return t('settings.sidecar.executable')
  if (binary.exists) return t('settings.sidecar.notExecutable')
  return t('settings.sidecar.missing')
}

function runtimeCheckLabel(key: string) {
  return t(`settings.sidecar.runtime.${key}`)
}

function runtimeCheckMessage(item: RuntimeCheckItemDto) {
  if (item.ready) return t('settings.sidecar.runtimeReady')
  if (item.skipped) return t('settings.sidecar.runtimeSkipped')
  return t('settings.sidecar.runtimeFailed')
}

function runtimeCheckStatusLabel(item: RuntimeCheckItemDto) {
  if (item.ready) return t('settings.sidecar.readyShort')
  if (item.skipped) return t('settings.sidecar.skipped')
  return t('settings.sidecar.failed')
}

function booleanLabel(value: boolean) {
  return value ? t('settings.release.enabled') : t('settings.release.disabled')
}

function formatCheckedAt(value?: string) {
  if (!value) return '-'
  const seconds = Number(value)
  if (Number.isFinite(seconds) && seconds > 0) {
    return new Date(seconds * 1000).toLocaleString()
  }
  return value
}

function errorMessage(error: unknown) {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return String(error)
}
</script>
