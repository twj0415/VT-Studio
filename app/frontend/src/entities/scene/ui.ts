const scenePreviewToneCount = 5

export function getScenePreviewToneClass(index: number) {
  return `scene-preview-tone-${(index - 1) % scenePreviewToneCount}`
}
