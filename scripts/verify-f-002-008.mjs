import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-008-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-008-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-008-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['docs/tasks/F-002-008-记忆配置.md', '全部清空必须输入确认短语'],
  ['src/main/ipc/settings.ts', 'settings:memory:get'],
  ['src/main/ipc/settings.ts', 'settings:memory:clear'],
  ['src/preload/index.ts', 'settings:memory:save'],
  ['src/shared/contracts/preload.ts', 'memory: {'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<MemoryConfig'],
  ['src/renderer/src/features/settings/components/MemoryConfig.vue', '清空全部记忆'],
  ['src/main/services/settings/memory-settings.ts', 'disposeEmbedding'],
];

for (const [relativePath, needle] of staticChecks) {
  const content = await import('node:fs').then(({ readFileSync }) => readFileSync(join(workspaceRoot, relativePath), 'utf-8'));
  if (!content.includes(needle)) {
    throw new Error(`${relativePath} 缺少 ${needle}`);
  }
}

const entrySource = `
  import { mkdirSync, writeFileSync } from 'node:fs';
  import { dirname, join } from 'node:path';
  import { app } from 'electron';
  import { configureGpu } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/gpu.ts')))};
  import { configureRuntime } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/runtime.ts')))};
  import { initializeFileSystem, getRuntimeDirectories } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/file-system/index.ts')))};
  import { closeDatabase, getDatabase, runMigrations, runSeed } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/database/index.ts')))};
  import {
    clearMemoryBySettings,
    getMemorySettings,
    restoreDefaultMemorySettings,
    saveMemorySettings,
    validateMemoryModelPath,
  } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/memory-settings.ts')))};

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureGpu();
    configureRuntime();
    initializeFileSystem();
    runMigrations();
    runSeed();

    const directories = getRuntimeDirectories();
    const initial = getMemorySettings();
    if (initial.config.messagesPerSummary !== 10 || initial.config.ragLimit !== 3) throw new Error('默认记忆配置不正确');
    if (initial.modelStatus.available) throw new Error('测试环境不应默认存在 ONNX 模型');

    const modelPath = join(directories.models, 'demo-model', 'onnx', 'model.onnx');
    mkdirSync(dirname(modelPath), { recursive: true });
    writeFileSync(modelPath, 'fake onnx');
    const modelCheck = validateMemoryModelPath({ modelOnnxFile: ['demo-model', 'onnx', 'model.onnx'] });
    if (!modelCheck.available || modelCheck.relativePath !== 'demo-model/onnx/model.onnx') throw new Error('模型路径校验失败');

    let pathEscapeBlocked = false;
    try {
      validateMemoryModelPath({ modelOnnxFile: ['..', 'outside.onnx'] });
    } catch {
      pathEscapeBlocked = true;
    }
    if (!pathEscapeBlocked) throw new Error('ONNX 路径越界未阻止');

    const saved = await saveMemorySettings({
      modelOnnxFile: ['missing-model', 'onnx', 'model.onnx'],
      modelDtype: 'fp32',
      messagesPerSummary: 12,
      shortTermLimit: 6,
      summaryMaxLength: 300,
      summaryLimit: 7,
      ragLimit: 4,
      deepRetrieveSummaryLimit: 8,
    });
    if (saved.config.modelDtype !== 'fp32' || saved.config.messagesPerSummary !== 12) throw new Error('记忆配置保存失败');
    if (saved.modelStatus.available) throw new Error('不存在模型不应显示可用');

    let rangeBlocked = false;
    try {
      await saveMemorySettings({
        ...saved.config,
        ragLimit: 999,
      });
    } catch {
      rangeBlocked = true;
    }
    if (!rangeBlocked) throw new Error('数字越界未阻止');

    const restored = await restoreDefaultMemorySettings();
    if (restored.config.modelDtype !== 'fp16' || restored.config.messagesPerSummary !== 10) throw new Error('恢复默认失败');

    const now = Date.now();
    const insert = getDatabase().prepare(
      'INSERT INTO memories (id, isolation_key, type, role, name, content, embedding, metadata, related_message_ids, summarized, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)',
    );
    insert.run('m1', '1:scriptAgent', 'message', 'user', null, 'hello', '[]', null, null, 0, now);
    insert.run('m2', '1:scriptAgent', 'summary', null, null, 'summary', '[]', null, JSON.stringify(['m1']), 0, now + 1);
    insert.run('m3', '2:scriptAgent', 'message', 'user', null, 'other', '[]', null, null, 0, now + 2);

    const stats = getMemorySettings().stats;
    if (stats.total !== 3 || stats.messages !== 2 || stats.summaries !== 1) throw new Error('记忆统计不正确');

    const isolated = clearMemoryBySettings({ scope: 'isolation', projectId: 1, agentType: 'scriptAgent', type: 'summary' });
    if (isolated.deleted !== 1 || isolated.updated !== 0) throw new Error('指定范围清空摘要失败');
    if (isolated.stats.total !== 2) throw new Error('指定范围清空后统计不正确');

    let confirmBlocked = false;
    try {
      clearMemoryBySettings({ scope: 'all', confirmText: 'wrong' });
    } catch {
      confirmBlocked = true;
    }
    if (!confirmBlocked) throw new Error('全部清空缺少确认短语未阻止');

    const all = clearMemoryBySettings({ scope: 'all', confirmText: '清空全部记忆' });
    if (all.deleted !== 2 || all.stats.total !== 0) throw new Error('全部清空失败');

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
      VT_STUDIO_VERIFY_F_002_008: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-008 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('F-002-008 memory settings verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
