import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-010-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-010-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-010-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['docs/tasks/F-002-010-数据库管理.md', '只能导入 `userData/exports/database` 下的备份文件'],
  ['src/main/ipc/settings.ts', 'settings:database:info'],
  ['src/main/ipc/settings.ts', 'settings:database:clear-all'],
  ['src/preload/index.ts', 'settings:database:export'],
  ['src/shared/contracts/preload.ts', 'database: {'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<DatabaseManagement'],
  ['src/renderer/src/features/settings/components/DatabaseManagement.vue', '输入：清空全部数据'],
  ['src/main/services/settings/database-management.ts', 'assertNoRunningTasks'],
];

for (const [relativePath, needle] of staticChecks) {
  const content = await import('node:fs').then(({ readFileSync }) => readFileSync(join(workspaceRoot, relativePath), 'utf-8'));
  if (!content.includes(needle)) {
    throw new Error(`${relativePath} 缺少 ${needle}`);
  }
}

const entrySource = `
  import { existsSync, mkdirSync } from 'node:fs';
  import { join } from 'node:path';
  import { app } from 'electron';
  import { configureGpu } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/gpu.ts')))};
  import { configureRuntime } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/runtime.ts')))};
  import { initializeFileSystem, getRuntimeDirectories } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/file-system/index.ts')))};
  import { closeDatabase, getDatabase, runMigrations } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/database/index.ts')))};
  import {
    checkDatabaseRunningTasks,
    clearAllDatabaseData,
    clearDatabaseTable,
    exportDatabaseBackup,
    getDatabaseManagementInfo,
    importDatabaseBackup,
    listDatabaseBackups,
    listDatabaseTables,
  } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/database-management.ts')))};

  async function expectBlocked(label, handler) {
    let blocked = false;
    try {
      await handler();
    } catch {
      blocked = true;
    }
    if (!blocked) {
      throw new Error(label);
    }
  }

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureGpu();
    configureRuntime();
    initializeFileSystem();
    runMigrations();

    const directories = getRuntimeDirectories();
    const backupDirectory = join(directories.exports, 'database');
    mkdirSync(backupDirectory, { recursive: true });

    const initialInfo = getDatabaseManagementInfo().info;
    if (!initialInfo.exists || initialInfo.tableCount <= 0 || initialInfo.migrationCount <= 0) {
      throw new Error('数据库信息无效');
    }

    const firstBackup = await exportDatabaseBackup();
    if (!firstBackup.backup.name.endsWith('.sqlite')) throw new Error('导出备份文件名无效');
    if (!existsSync(join(backupDirectory, firstBackup.backup.name))) throw new Error('备份文件未生成');
    if (listDatabaseBackups().backups.length !== 1) throw new Error('备份列表无效');

    await expectBlocked('非法备份路径未阻止', () => importDatabaseBackup({ backupName: '../outside.sqlite', confirmText: '导入数据库' }));
    await expectBlocked('导入缺少确认短语未阻止', () => importDatabaseBackup({ backupName: firstBackup.backup.name, confirmText: 'wrong' }));

    getDatabase().prepare("INSERT INTO tasks (category, status, started_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?)").run('verify', 'running', Date.now(), Date.now(), Date.now());
    if (checkDatabaseRunningTasks().runningTaskCount !== 1) throw new Error('running 任务统计无效');
    await expectBlocked('running 任务未阻止导入', () => importDatabaseBackup({ backupName: firstBackup.backup.name, confirmText: '导入数据库' }));
    await expectBlocked('running 任务未阻止清表', () => clearDatabaseTable({ tableName: 'prompts', confirmText: 'prompts' }));
    await expectBlocked('running 任务未阻止清空全部', () => clearAllDatabaseData({ confirmText: '清空全部数据' }));
    getDatabase().prepare("UPDATE tasks SET status = 'succeeded', finished_at = ?, updated_at = ? WHERE status = 'running'").run(Date.now(), Date.now());

    await expectBlocked('受保护表未阻止清空', () => clearDatabaseTable({ tableName: 'schema_migrations', confirmText: 'schema_migrations' }));
    await expectBlocked('清表缺少表名确认未阻止', () => clearDatabaseTable({ tableName: 'prompts', confirmText: 'wrong' }));

    const beforeClearPrompts = getDatabase().prepare("SELECT COUNT(*) as count FROM prompts").get().count;
    if (beforeClearPrompts <= 0) throw new Error('默认 prompts 未初始化');
    const clearPrompts = await clearDatabaseTable({ tableName: 'prompts', confirmText: 'prompts' });
    if (clearPrompts.deleted !== beforeClearPrompts) throw new Error('清表删除数量不正确');
    if (getDatabase().prepare("SELECT COUNT(*) as count FROM prompts").get().count !== 0) throw new Error('清表未生效');
    if (listDatabaseBackups().backups.length < 2) throw new Error('清表前未自动备份');

    const importResult = await importDatabaseBackup({ backupName: firstBackup.backup.name, confirmText: '导入数据库' });
    if (importResult.importedBackupName !== firstBackup.backup.name) throw new Error('导入结果无效');
    if (getDatabase().prepare("SELECT COUNT(*) as count FROM prompts").get().count <= 0) throw new Error('导入后数据未恢复');

    await expectBlocked('清空全部缺少确认短语未阻止', () => clearAllDatabaseData({ confirmText: 'wrong' }));
    const clearAll = await clearAllDatabaseData({ confirmText: '清空全部数据' });
    if (!clearAll.autoBackupName.endsWith('.sqlite')) throw new Error('清空全部前未自动备份');
    if (getDatabase().prepare("SELECT COUNT(*) as count FROM users").get().count <= 0) throw new Error('清空全部后默认用户未恢复');
    if (getDatabase().prepare("SELECT COUNT(*) as count FROM prompts").get().count <= 0) throw new Error('清空全部后默认提示词未恢复');
    if (!listDatabaseTables().tables.some((table) => table.name === 'schema_migrations' && table.protected)) throw new Error('受保护表标记缺失');

    closeDatabase();
    app.quit();
  }

  module.exports = main().catch((error) => {
    console.error(error);
    app.exit(1);
  });
`;

try {
  mkdirSync(bundleDirectory, { recursive: true });
  writeFileSync(entryPath, entrySource);
  await build({
    entryPoints: [entryPath],
    outfile: bundlePath,
    bundle: true,
    platform: 'node',
    format: 'cjs',
    target: 'node20',
    external: [
      '@huggingface/transformers',
      'better-sqlite3',
      'electron',
    ],
    alias: {
      '@shared': join(workspaceRoot, 'src/shared'),
    },
    logLevel: 'silent',
  });

  if (!existsSync(bundlePath)) {
    throw new Error('验证 bundle 未生成');
  }

  const electronBin = process.platform === 'win32'
    ? join(workspaceRoot, 'node_modules', '.bin', 'electron.CMD')
    : join(workspaceRoot, 'node_modules', '.bin', 'electron');
  const command = process.platform === 'win32' ? 'cmd.exe' : electronBin;
  const args = process.platform === 'win32' ? ['/c', electronBin, bundlePath] : [bundlePath];
  const result = spawnSync(command, args, {
    cwd: workspaceRoot,
    stdio: 'inherit',
    timeout: 30000,
    env: {
      ...process.env,
      VT_STUDIO_VERIFY_F_002_010: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-010 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('F-002-010 database management verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
