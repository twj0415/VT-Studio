import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';
import LoginHome from '@renderer/features/auth/LoginHome.vue';
import WorkbenchLayout from '@renderer/layouts/WorkbenchLayout.vue';
import ProjectHome from '@renderer/features/project/ProjectHome.vue';
import TaskCenter from '@renderer/features/task-center/TaskCenter.vue';
import SettingsHome from '@renderer/features/settings/SettingsHome.vue';
import NovelHome from '@renderer/features/novel/NovelHome.vue';
import ScriptAgentHome from '@renderer/features/script-agent/ScriptAgentHome.vue';
import ScriptHome from '@renderer/features/script/ScriptHome.vue';
import CornerScapeHome from '@renderer/features/corner-scape/CornerScapeHome.vue';
import ProductionHome from '@renderer/features/production/ProductionHome.vue';
import AssetsHome from '@renderer/features/assets/AssetsHome.vue';
import ExportHome from '@renderer/features/export/ExportHome.vue';
import { useAuthStore } from '@renderer/stores/auth';

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'login',
    component: LoginHome,
    meta: { title: '登录', public: true },
  },
  {
    path: '/',
    component: WorkbenchLayout,
    redirect: '/projects',
    meta: { requiresAuth: true },
    children: [
      {
        path: 'projects',
        name: 'projects',
        component: ProjectHome,
        meta: { title: '项目管理' },
      },
      {
        path: 'tasks',
        name: 'tasks',
        component: TaskCenter,
        meta: { title: '任务中心' },
      },
      {
        path: 'settings',
        name: 'settings',
        component: SettingsHome,
        meta: { title: '设置' },
      },
      {
        path: 'novel',
        name: 'novel',
        component: NovelHome,
        meta: { title: '小说/原文' },
      },
      {
        path: 'script-agent',
        name: 'script-agent',
        component: ScriptAgentHome,
        meta: { title: '剧本 Agent' },
      },
      {
        path: 'script',
        name: 'script',
        component: ScriptHome,
        meta: { title: '剧本' },
      },
      {
        path: 'corner-scape',
        name: 'corner-scape',
        component: CornerScapeHome,
        meta: { title: '角景音频绑定' },
      },
      {
        path: 'production',
        name: 'production',
        component: ProductionHome,
        meta: { title: '生产工作台' },
      },
      {
        path: 'assets',
        name: 'assets',
        component: AssetsHome,
        meta: { title: '资产中心' },
      },
      {
        path: 'export',
        name: 'export',
        component: ExportHome,
        meta: { title: '导出' },
      },
    ],
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

router.beforeEach(async (to) => {
  const authStore = useAuthStore();

  if (to.meta.public) {
    if (!authStore.restored) {
      await authStore.restoreSession();
    }

    return authStore.isLoggedIn ? { name: 'projects' } : true;
  }

  const ok = await authStore.restoreSession();
  if (!ok) {
    return {
      name: 'login',
      query: { redirect: to.fullPath },
    };
  }

  return true;
});
