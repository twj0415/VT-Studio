export type BusinessCanvasWheelMode = 'zoom' | 'scroll';

export interface BusinessSettingsConfig {
  chapterReg: string;
  requestTimeoutMs: number;
  canvasWheelMode: BusinessCanvasWheelMode;
  showInteractionState: boolean;
  assetsBatchGenerateSize: number;
  scriptEpisodeLength: number;
}

export interface BusinessSettingsResult {
  config: BusinessSettingsConfig;
}

export interface BusinessSettingsSavePayload extends BusinessSettingsConfig {}

export interface BusinessSettingsSaveResult extends BusinessSettingsResult {}

export interface BusinessSettingsRestoreDefaultChapterRegResult extends BusinessSettingsResult {}
