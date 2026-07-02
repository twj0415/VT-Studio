import { app, BrowserWindow } from 'electron';
import { configureGpu } from './app/gpu';
import { configureRuntime } from './app/runtime';
import { startLocalServer, stopLocalServer } from './app/server';
import { createMainWindow } from './app/window';
import { registerIpc } from './ipc';
import { closeDatabase, getDatabaseInfo, runMigrations } from './services/database';
import { getRuntimeDirectories, initializeFileSystem } from './services/file-system';
import { logger } from './services/logger';
import { startSocketService, stopSocketService } from './services/socket';
import { recoverRunningTasks } from './services/task';

configureRuntime();
configureGpu();

app.whenReady().then(async () => {
  logger.section('VT Studio 启动');
  logger.info('启动', 'VT Studio 正在启动...');

  const directories = initializeFileSystem();
  logger.info('运行目录', `已就绪：${directories.userData}`);
  logger.detail('运行目录', '运行目录详情', getRuntimeDirectories());

  runMigrations();
  const recoveredTaskCount = recoverRunningTasks();
  const databaseInfo = getDatabaseInfo();
  logger.info('数据库', `已连接：${databaseInfo.tableCount} 张表，${databaseInfo.migrationCount} 个迁移记录`);
  logger.detail('数据库', '数据库详情', databaseInfo);
  logger.info('任务中心', `已恢复 ${recoveredTaskCount} 个运行中任务`);

  const localServerInfo = await startLocalServer();
  startSocketService(localServerInfo.server, localServerInfo.url);

  registerIpc();
  createMainWindow();
  logger.info('桌面窗口', '已打开');
  logger.info('启动', 'VT Studio 启动完成');
  logger.section('启动完成');

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createMainWindow();
    }
  });
});

app.on('before-quit', () => {
  void stopSocketService()
    .then(() => stopLocalServer())
    .catch((error) => logger.error('退出', '关闭本地服务失败', error))
    .finally(() => closeDatabase());
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
