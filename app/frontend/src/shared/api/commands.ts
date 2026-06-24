export const tauriCommands = {
  createProject: 'create_project',
  listProjects: 'list_projects',
  getProjectDetail: 'get_project_detail',
  updateProject: 'update_project',
  getStoryboard: 'get_storyboard',
  updateStoryboardItem: 'update_storyboard_item',
  batchUpdateStoryboardItems: 'batch_update_storyboard_items',
  reorderStoryboardItems: 'reorder_storyboard_items',
  generateImagePrompts: 'generate_image_prompts',
  startImageGeneration: 'start_image_generation',
  selectImageCandidate: 'select_image_candidate',
  startVideoGeneration: 'start_video_generation',
  selectVideoSegment: 'select_video_segment',
  startComposition: 'start_composition',
  getTaskDetail: 'get_task_detail',
  approveTaskStep: 'approve_task_step',
  getAppConfig: 'get_app_config',
  updateAppConfig: 'update_app_config',
  listDictionaries: 'list_dictionaries',
  listExecutableMediaOptions: 'list_executable_media_options',
} as const

export type TauriCommandName = (typeof tauriCommands)[keyof typeof tauriCommands]
