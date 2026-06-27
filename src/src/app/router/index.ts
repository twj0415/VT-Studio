import { createRouter, createWebHistory } from 'vue-router'

import CreateProjectPage from '@/pages/create-project/index.vue'
import AiToolsPage from '@/pages/ai-tools/index.vue'
import CreativeResourcesPage from '@/pages/creative-resources/index.vue'
import CompositionPage from '@/pages/composition/index.vue'
import HomePage from '@/pages/home/index.vue'
import ImageGenerationPage from '@/pages/image-generation/index.vue'
import ModelWorkflowPage from '@/pages/model-workflow/index.vue'
import ProjectWorkbenchPage from '@/pages/project-workbench/index.vue'
import SettingsPage from '@/pages/settings/index.vue'
import StoryboardEditorPage from '@/pages/storyboard-editor/index.vue'
import VideoGenerationPage from '@/pages/video-generation/index.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'home', component: HomePage },
    { path: '/create-project', name: 'create-project', component: CreateProjectPage },
    { path: '/ai-tools', name: 'ai-tools', component: AiToolsPage },
    { path: '/creative-resources', name: 'creative-resources', component: CreativeResourcesPage },
    { path: '/model-workflow', name: 'model-workflow', component: ModelWorkflowPage },
    { path: '/projects/:projectId', name: 'project-workbench', component: ProjectWorkbenchPage },
    { path: '/projects/:projectId/workspace/storyboard', name: 'storyboard-editor', component: StoryboardEditorPage },
    { path: '/projects/:projectId/workspace/image', name: 'image-generation', component: ImageGenerationPage },
    { path: '/projects/:projectId/workspace/video', name: 'video-generation', component: VideoGenerationPage },
    { path: '/projects/:projectId/workspace/compose', name: 'composition', component: CompositionPage },
    { path: '/projects/:projectId/storyboard', redirect: (to) => `/projects/${String(to.params.projectId)}/workspace/storyboard` },
    { path: '/projects/:projectId/image', redirect: (to) => `/projects/${String(to.params.projectId)}/workspace/image` },
    { path: '/projects/:projectId/video', redirect: (to) => `/projects/${String(to.params.projectId)}/workspace/video` },
    { path: '/projects/:projectId/compose', redirect: (to) => `/projects/${String(to.params.projectId)}/workspace/compose` },
    { path: '/settings', name: 'settings', component: SettingsPage },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
})

router.beforeEach((to) => {
  const lockedStagePaths = ['/subtitle', '/cover', '/export']

  if (lockedStagePaths.some((path) => to.path.includes(path))) {
    return '/'
  }

  return true
})
