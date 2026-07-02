export const MODEL_TYPES = {
  TEXT: 'text',
  IMAGE: 'image',
  VIDEO: 'video',
  TTS: 'tts',
  ALL: 'all',
} as const;

export const AGENT_USE_MODE = {
  SIMPLE: '0',
  ADVANCED: '1',
} as const;

export const MODEL_SETTING_KEYS = {
  agentUseMode: 'agentUseMode',
  switchAiDevTool: 'switchAiDevTool',
} as const;

export const MODEL_TEST_FILE_NAMES = {
  image: 'test-image.txt',
  video: 'test-video.txt',
  tts: 'test-audio.txt',
} as const;

export const AGENT_MODEL_KEYS = [
  'scriptAgent',
  'productionAgent',
  'universalAi',
  'ttsDubbing',
  'scriptAgent:decisionAgent',
  'scriptAgent:supervisionAgent',
  'scriptAgent:storySkeletonAgent',
  'scriptAgent:adaptationStrategyAgent',
  'scriptAgent:scriptAgent',
  'productionAgent:decisionAgent',
  'productionAgent:supervisionAgent',
  'productionAgent:deriveAssetsAgent',
  'productionAgent:generateAssetsAgent',
  'productionAgent:directorPlanAgent',
  'productionAgent:storyboardGenAgent',
  'productionAgent:storyboardPanelAgent',
  'productionAgent:storyboardTableAgent',
] as const;

export type AgentModelKey = (typeof AGENT_MODEL_KEYS)[number];
export type ModelType = (typeof MODEL_TYPES)[keyof typeof MODEL_TYPES];
