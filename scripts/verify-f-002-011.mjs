import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-011-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-011-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-011-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['docs/tasks/F-002-011-文件管理.md', '只允许打开白名单 key，不接受任意 path'],
  ['src/main/ipc/settings.ts', 'settings:files:list-openable-dirs'],
  ['src/main/ipc/settings.ts', 'settings:files:open-dir'],
  ['src/preload/index.ts', 'settings:files:open-dir'],
  ['src/shared/contracts/preload.ts', 'files: {'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<FileManagement'],
  ['src/renderer/src/features/settings/components/FileManagement.vue', "t('files.hint')"],
  ['src/main/services/settings/file-management.ts', 'shell.openPath'],
];

for (const [relativePath, needle] of staticChecks) {
  const content = await import('node:fs').then(({ readFileSync }) => readFileSync(join(workspaceRoot, relativePath), 'utf-8'));
  if (!content.includes(needle)) {
    throw new Error(`${relativePath} 缺少 ${needle}`);
  }
}

const entrySource = `
  import { rmSync } from 'node:fs';
  import { app, shell } from 'electron';
  import { configureGpu } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/gpu.ts')))};
  import { configureRuntime } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/runtime.ts')))};
  import { initializeFileSystem, getRuntimeDirectories } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/file-system/index.ts')))};
  import {
    listOpenableDirectories,
    openDirectory,
  } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/file-management.ts')))};

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureGpu();
    configureRuntime();
    initializeFileSystem();

    const directories = getRuntimeDirectories();
    const list = listOpenableDirectories();
    if (list.directories.length !== 7) throw new Error('白名单目录数量不正确');
    if (list.directories.some((item) => item.key === 'vendors' || item.key === 'database')) throw new Error('危险目录不应暴露');

    const exportsEntry = list.directories.find((item) => item.key === 'exports');
    if (!exportsEntry || !exportsEntry.exists) throw new Error('exports 目录默认应存在');

    rmSync(directories.temp, { recursive: true, force: true });
    const afterRemove = listOpenableDirectories().directories.find((item) => item.key === 'temp');
    if (!afterRemove || afterRemove.exists) throw new Error('目录缺失状态未刷新');

    let openedPath = '';
    shell.openPath = async (targetPath) => {
      openedPath = targetPath;
      return '';
    };

    const opened = await openDirectory({ key: 'temp' });
    if (!opened.created) throw new Error('缺失目录未自动创建');
    if (openedPath !== directories.temp) throw new Error('打开路径不正确');

    let invalidBlocked = false;
    try {
      await openDirectory({ key: '../outside' });
    } catch {
      invalidBlocked = true;
    }
    if (!invalidBlocked) throw new Error('非法 key 未被阻止');

    shell.openPath = async () => 'mock error';
    let openFailureBlocked = false;
    try {
      await openDirectory({ key: 'logs' });
    } catch {
      openFailureBlocked = true;
    }
    if (!openFailureBlocked) throw new Error('系统打开失败未抛错');

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
      VT_STUDIO_VERIFY_F_002_011: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-011 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('F-002-011 file management verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
