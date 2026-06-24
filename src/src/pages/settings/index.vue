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
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { useConfigStore } from '@/entities/config/store'
import { useDictOptions } from '@/shared/dict/useDictOptions'
import type { AppLocale, LayoutDensity, ThemePreset } from '@/shared/stores/appStore'
import { useAppStore } from '@/shared/stores/appStore'
import { getThemePreviewCardStyle, getThemePreviewStyle } from '@/shared/theme'

const appStore = useAppStore()
const configStore = useConfigStore()
const { t } = useI18n()
const themePresetOptions = useDictOptions('themePreset')
const appLocaleOptions = useDictOptions('appLocale')
const layoutDensityOptions = useDictOptions('layoutDensity')

onMounted(async () => {
  const config = await configStore.loadConfig()
  appStore.setThemePreset(config.themePreset)
  appStore.setAppLocale(config.appLocale)
  appStore.setLayoutDensity(config.layoutDensity)
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
</script>
