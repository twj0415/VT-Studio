import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-006-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-006-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-006-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['docs/tasks/F-002-006-提示词管理.md', 'use_data 是用户自定义提示词'],
  ['src/shared/contracts/preload.ts', 'prompt: {'],
  ['src/preload/index.ts', 'settings:prompt:list'],
  ['src/main/ipc/settings.ts', 'settings:prompt:restore-default'],
  ['src/main/services/settings/prompt.ts', 'UPDATE prompts SET use_data'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<PromptConfig'],
  ['src/renderer/src/features/settings/components/PromptConfig.vue', '恢复默认'],
];

for (const [relativePath, needle] of staticChecks) {
  const filePath = join(workspaceRoot, relativePath);
  const content = await import('node:fs').then(({ readFileSync }) => readFileSync(filePath, 'utf-8'));
  if (!content.includes(needle)) {
    throw new Error(`${relativePath} 缺少 ${needle}`);
  }
}

const entrySource = `
  import { app } from 'electron';
  import { configureRuntime } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/runtime.ts')))};
  import { initializeFileSystem } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/file-system/index.ts')))};
  import { closeDatabase, getDatabase, runMigrations, runSeed } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/database/index.ts')))};
  import {
    getEffectivePromptByType,
    getPromptList,
    restorePromptDefault,
    updatePrompt,
  } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/prompt.ts')))};

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureRuntime();
    initializeFileSystem();
    runMigrations();
    runSeed();

    const initial = getPromptList();
    if (initial.prompts.length < 4) throw new Error('默认提示词数量不足');
    for (const type of ['eventExtraction', 'scriptAssetExtraction', 'videoPromptGeneration', 'audioBindPrompt']) {
      const prompt = initial.prompts.find((item) => item.type === type);
      if (!prompt) throw new Error(\`缺少默认提示词：\${type}\`);
      if (!prompt.data.trim()) throw new Error(\`默认 data 为空：\${type}\`);
      if (prompt.useData !== '') throw new Error(\`初始 use_data 应为空：\${type}\`);
      if (prompt.effectiveData !== prompt.data) throw new Error(\`初始 effectiveData 应等于 data：\${type}\`);
    }

    const assetPrompt = initial.prompts.find((item) => item.type === 'scriptAssetExtraction');
    if (!assetPrompt) throw new Error('剧本资产提取提示词不存在');
    const originalData = assetPrompt.data;
    const risky = updatePrompt({
      id: assetPrompt.id,
      useData: '只输出资产名称，不调用工具',
    });
    if (risky.saved) throw new Error('结构风险提示词不应直接保存');
    if (risky.warnings.length === 0) throw new Error('结构风险未返回 warnings');
    const afterRisk = getDatabase().prepare('SELECT data, use_data FROM prompts WHERE id = ?').get(assetPrompt.id);
    if (afterRisk.data !== originalData) throw new Error('风险保存不应修改 data');
    if (afterRisk.use_data !== '') throw new Error('未 force 的风险保存不应写 use_data');

    let emptyMessage = '';
    try {
      updatePrompt({ id: assetPrompt.id, useData: '   ' });
    } catch (error) {
      emptyMessage = String(error?.message || '');
    }
    if (!emptyMessage.includes('不能为空')) throw new Error(\`空提示词未被阻止：\${emptyMessage || 'empty'}\`);

    const customText = 'resultTool assetsList role scene tool 自定义资产提取提示词';
    const saved = updatePrompt({
      id: assetPrompt.id,
      useData: customText,
      force: true,
    });
    if (!saved.saved || !saved.prompt?.isCustomized) throw new Error('force 保存自定义提示词失败');
    if (saved.prompt.effectiveData !== customText) throw new Error('effectiveData 未优先使用 use_data');
    if (getEffectivePromptByType('scriptAssetExtraction') !== customText) throw new Error('按 type 读取有效提示词失败');
    const afterSave = getDatabase().prepare('SELECT data, use_data FROM prompts WHERE id = ?').get(assetPrompt.id);
    if (afterSave.data !== originalData) throw new Error('保存自定义提示词不应覆盖 data');
    if (afterSave.use_data !== customText) throw new Error('保存自定义提示词未写入 use_data');

    const restored = restorePromptDefault({ id: assetPrompt.id });
    if (restored.prompt.isCustomized) throw new Error('恢复默认后仍显示自定义');
    if (restored.prompt.effectiveData !== originalData) throw new Error('恢复默认后未回退 data');
    const afterRestore = getDatabase().prepare('SELECT data, use_data FROM prompts WHERE id = ?').get(assetPrompt.id);
    if (afterRestore.data !== originalData) throw new Error('恢复默认不应覆盖 data');
    if (afterRestore.use_data !== '') throw new Error('恢复默认应清空 use_data');

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
      '@ai-sdk/*',
      '@electron-toolkit/*',
      '@huggingface/transformers',
      'ai',
      'axios',
      'better-sqlite3',
      'electron',
      'form-data',
      'jsonwebtoken',
      'onnxruntime-web',
      'qwen-ai-provider-v5',
      'sharp',
      'sucrase',
      'tdesign-icons-vue-next',
      'tdesign-vue-next',
      'vercel-minimax-ai-provider',
      'vm2',
      'zhipu-ai-provider',
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
      VT_STUDIO_VERIFY_F_002_006: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-006 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('F-002-006 prompt configuration verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
