import type { MenuModule } from '@shared/types/app';

export const globalMenus: MenuModule[] = [
  {
    id: 'M-003',
    title: '项目管理',
    routeName: 'projects',
    scope: 'global',
    description: '创建、编辑、删除、打开项目和项目模型配置。',
    status: 'planned',
  },
  {
    id: 'M-009',
    title: '任务中心',
    routeName: 'tasks',
    scope: 'global',
    description: '任务列表、状态、失败原因、刷新和筛选。',
    status: 'planned',
  },
];

export const projectMenus: MenuModule[] = [
  {
    id: 'M-004',
    title: '小说/原文',
    routeName: 'novel',
    scope: 'project',
    description: '小说文本、章节、事件分析和事件管理。',
    status: 'planned',
  },
  {
    id: 'M-011',
    title: '剧本 Agent',
    routeName: 'script-agent',
    scope: 'project',
    description: '基于原文生成故事骨架、改编策略和剧本。',
    status: 'planned',
  },
  {
    id: 'M-005',
    title: '剧本',
    routeName: 'script',
    scope: 'project',
    description: '剧本管理、批量操作、导出和资产提取。',
    status: 'planned',
  },
  {
    id: 'M-007',
    title: '角景音频绑定',
    routeName: 'corner-scape',
    scope: 'project',
    description: '角色、场景、音频资产绑定和批量处理。',
    status: 'planned',
  },
  {
    id: 'M-008',
    title: '生产工作台',
    routeName: 'production',
    scope: 'project',
    description: '流程节点、Agent、流程数据和视频工作台。',
    status: 'planned',
  },
  {
    id: 'M-006',
    title: '资产中心',
    routeName: 'assets',
    scope: 'project',
    description: '角色、场景、道具、素材、音频和生成能力。',
    status: 'planned',
  },
  {
    id: 'M-010',
    title: '导出',
    routeName: 'export',
    scope: 'project',
    description: '剪映草稿导出和导出前素材校验。',
    status: 'planned',
  },
];
