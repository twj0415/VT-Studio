<template>
  <n-config-provider :theme="naiveTheme" :theme-overrides="themeOverrides">
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
import { computed, watchEffect } from 'vue'
import { darkTheme, NConfigProvider, NDialogProvider, NMessageProvider } from 'naive-ui'

import AppShell from '@/widgets/app-shell/AppShell.vue'
import { useAppStore } from '@/shared/stores/appStore'
import { themeMap } from '@/shared/theme'
import { applyThemeVars } from '@/shared/theme/cssVars'

const appStore = useAppStore()
const currentTheme = computed(() => themeMap[appStore.themePreset])
const naiveTheme = computed(() => (currentTheme.value.mode === 'dark' ? darkTheme : null))
const themeOverrides = computed(() => currentTheme.value.naiveThemeOverrides)

watchEffect(() => {
  const theme = currentTheme.value
  document.body.dataset.preset = appStore.themePreset
  document.body.dataset.density = appStore.layoutDensity
  document.body.dataset.themeMode = theme.mode
  applyThemeVars(theme.cssVars)
})
</script>
