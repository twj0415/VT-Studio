<template>
  <n-config-provider :theme-overrides="themeOverrides">
    <n-message-provider>
      <n-dialog-provider>
        <AppShell>
          <router-view />
        </AppShell>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, onMounted, watchEffect } from 'vue'
import { NConfigProvider, NDialogProvider, NMessageProvider } from 'naive-ui'

import { useConfigStore } from '@/entities/config/store'
import AppShell from '@/widgets/app-shell/AppShell.vue'
import { i18n } from '@/shared/i18n'
import { useAppStore } from '@/shared/stores/appStore'
import { themeMap } from '@/shared/theme'
import { applyThemeVars } from '@/shared/theme/cssVars'

const appStore = useAppStore()
const configStore = useConfigStore()
const currentTheme = computed(() => themeMap[appStore.themePreset])
const themeOverrides = computed(() => currentTheme.value.naiveThemeOverrides)

onMounted(async () => {
  const config = await configStore.loadConfig()
  appStore.setThemePreset(config.themePreset)
  appStore.setAppLocale(config.appLocale)
  appStore.setLayoutDensity(config.layoutDensity)
})

watchEffect(() => {
  const theme = currentTheme.value
  document.body.dataset.preset = appStore.themePreset
  document.body.dataset.density = appStore.layoutDensity
  document.documentElement.lang = appStore.appLocale
  i18n.global.locale.value = appStore.appLocale
  applyThemeVars(theme.cssVars)
})
</script>
