export interface DigitalHumanProjectStateDto {
  projectId: string
  ttsStatus: string
  videoStatus: string
  referenceImagePath: string | null
  referenceAudioPath: string | null
  outputVideoPath: string | null
}

export interface StartDigitalHumanVideoRequest {
  projectId: string
  referenceImagePath?: string | null
  prompt: string
}
