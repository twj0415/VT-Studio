import type { AppConfigDto } from './types'

let config: AppConfigDto = {
  appLocale: 'zh-CN',
  themePreset: 'graphite',
  layoutDensity: 'comfortable',
}

export async function getAppConfig(): Promise<AppConfigDto> {
  return config
}

export async function updateAppConfig(patch: Partial<AppConfigDto>): Promise<AppConfigDto> {
  config = { ...config, ...patch }
  return config
}
