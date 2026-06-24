import { defineStore } from 'pinia'

import { getAppConfig, updateAppConfig } from './api'
import type { AppConfigDto } from './types'

export const useConfigStore = defineStore('config', {
  state: () => ({
    config: null as AppConfigDto | null,
  }),
  actions: {
    async loadConfig() {
      this.config = await getAppConfig()
      return this.config
    },
    async updateConfig(patch: Partial<AppConfigDto>) {
      this.config = await updateAppConfig(patch)
      return this.config
    },
  },
})
