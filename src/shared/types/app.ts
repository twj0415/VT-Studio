export interface AppInfo {
  name: string;
  version: string;
  platform: NodeJS.Platform;
  isDev: boolean;
  userDataPath: string;
}

export interface MenuModule {
  id: string;
  title: string;
  routeName: string;
  scope: 'global' | 'project';
  description: string;
  status: 'ready' | 'planned';
}
