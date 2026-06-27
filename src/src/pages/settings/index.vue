<template>
  <section class="view">
    <div class="gpage">
      <div class="wrap flex min-h-0 flex-1 flex-col">
        <div class="phead"><div><h1>{{ t('settings.title') }}</h1><div class="desc">{{ t('settings.desc') }}</div></div></div>
        <div class="set-layout">
          <nav class="set-nav">
            <div class="sn active">{{ t('settings.nav.appearance') }}</div>
            <div class="sn">{{ t('settings.nav.provider') }}</div>
            <div class="sn">{{ t('settings.nav.capability') }}</div>
            <div class="sn">{{ t('settings.nav.secrets') }}</div>
            <div class="sn">{{ t('settings.nav.path') }}</div>
            <div class="sn">{{ t('settings.nav.defaults') }}</div>
            <div class="sn">{{ t('settings.nav.importExport') }}</div>
          </nav>
          <div class="set-form">
            <div class="setcard">
              <h4>{{ t('settings.themeTitle') }}</h4>
              <div class="mb-vt-4 text-[12.5px] text-muted">{{ t('settings.themeHint') }}</div>
              <div class="preset-row">
                <div v-for="option in themePresetOptions" :key="option.value" class="preset" :class="{ sel: appStore.themePreset === option.value }" @click="setTheme(option.value)">
                  <div class="swatch" :style="presetStyle(option.value)"><span class="sw-card" :style="presetCardStyle(option.value)"></span></div>
                  <div class="pn">{{ option.label }}</div>
                </div>
              </div>
            </div>

            <div class="setcard">
              <h4>{{ t('settings.interfaceTitle') }}</h4>
              <div class="setrow"><span class="k">{{ t('settings.appLocale') }}</span><div class="opt-row"><div v-for="option in appLocaleOptions" :key="option.value" class="opt" :class="{ sel: appStore.appLocale === option.value }" @click="setLocale(option.value)">{{ option.label }}</div></div></div>
              <div class="setrow"><span class="k">{{ t('settings.density') }}</span><div class="opt-row"><div v-for="option in layoutDensityOptions" :key="option.value" class="opt" :class="{ sel: appStore.layoutDensity === option.value }" @click="setDensity(option.value)">{{ option.label }}</div></div></div>
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

            <div class="setcard">
              <h4>{{ t('settings.futureTitle') }}</h4>
              <div class="setrow"><span class="k">{{ t('settings.nav.provider') }}</span><span class="v">{{ t('settings.providerLater') }}</span></div>
              <div class="setrow"><span class="k">{{ t('settings.nav.secrets') }}</span><span class="v">{{ t('settings.secretsOnlyKeyring') }}</span></div>
              <div class="setrow"><span class="k">{{ t('settings.nav.importExport') }}</span><span class="v">{{ t('settings.exportNoSecrets') }}</span></div>
            </div>
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

import { getAppReleaseInfo, runRuntimeSelfCheck } from '@/entities/config/api'
import { useConfigStore } from '@/entities/config/store'
import type { AppReleaseInfoDto, RuntimeCheckItemDto, RuntimeSelfCheckDto, SidecarBinaryStatusDto } from '@/entities/config/types'
import type { TemplateSidecarBinaryStatusDto, TemplateSidecarStatusDto } from '@/entities/template/types'
import { useDictOptions } from '@/shared/dict/useDictOptions'
import type { AppLocale, LayoutDensity, ThemePreset } from '@/shared/stores/appStore'
import { useAppStore } from '@/shared/stores/appStore'
import { getThemePreviewCardStyle, getThemePreviewStyle } from '@/shared/theme'

const appStore = useAppStore()
const configStore = useConfigStore()
const message = useMessage()
const { t } = useI18n()
const themePresetOptions = useDictOptions('themePreset')
const appLocaleOptions = useDictOptions('appLocale')
const layoutDensityOptions = useDictOptions('layoutDensity')
const releaseInfo = ref<AppReleaseInfoDto | null>(null)
const runtimeSelfCheck = ref<RuntimeSelfCheckDto | null>(null)
const sidecarLoading = ref(false)
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
  const [config, release] = await Promise.all([
    configStore.loadConfig(),
    getAppReleaseInfo(),
  ])
  releaseInfo.value = release
  appStore.setThemePreset(config.themePreset)
  appStore.setAppLocale(config.appLocale)
  appStore.setLayoutDensity(config.layoutDensity)
  await loadSidecars()
})

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
    const reason = error instanceof Error ? error.message : String(error)
    message.error(t('settings.sidecar.checkFailed', { reason }))
  } finally {
    sidecarLoading.value = false
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
</script>
