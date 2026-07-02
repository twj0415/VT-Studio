import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-004-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-004-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-004-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['docs/tasks/F-002-004-模型与提示词映射.md', '新建 model_prompt_templates 和 model_prompt_mappings'],
  ['src/main/services/settings/model-prompt-migrations.ts', 'model_prompt_templates'],
  ['src/shared/contracts/preload.ts', 'modelPrompt: {'],
  ['src/preload/index.ts', 'settings:model-prompt:get'],
  ['src/main/ipc/settings.ts', 'settings:model-prompt:bind'],
  ['src/main/services/settings/model-prompt.ts', '模板类型和模型类型不匹配'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<ModelPromptConfig'],
  ['src/renderer/src/features/settings/components/ModelPromptConfig.vue', '模型专用模板'],
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
  import { saveApiConnection } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/model-config.ts')))};
  import {
    bindModelPromptTemplate,
    clearModelPromptBinding,
    deleteModelPromptTemplate,
    getModelPromptConfig,
    saveModelPromptTemplate,
  } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/model-prompt.ts')))};

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureRuntime();
    initializeFileSystem();
    runMigrations();
    runSeed();

    const tableRows = getDatabase()
      .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('model_prompt_templates', 'model_prompt_mappings')")
      .all();
    if (tableRows.length !== 2) throw new Error('模型提示词表未创建完整');

    const savedConnection = saveApiConnection({
      connection: {
        name: '模型模板验证连接',
        serviceType: 'openai-gateway',
        baseUrl: 'http://127.0.0.1:9000/v1',
        apiKey: 'test-key',
        capabilities: [],
        models: [
          { id: 'gpt-5.5', displayName: 'GPT-5.5', modelName: 'gpt-5.5', type: 'text', think: true },
          { id: 'gpt-image-2', displayName: 'GPT Image 2', modelName: 'gpt-image-2', type: 'image' },
          { id: 'video-test-1', displayName: 'Video Test 1', modelName: 'video-test-1', type: 'video' },
        ],
      },
    });

    let config = getModelPromptConfig();
    const group = config.connections.find((item) => item.connectionId === savedConnection.connection.id);
    if (!group) throw new Error('未返回模型连接分组');
    if (group.models.some((model) => model.modelType === 'text')) throw new Error('不应返回 text 模型');
    if (!group.models.some((model) => model.modelType === 'image' && model.modelName === 'gpt-image-2')) throw new Error('未返回图片模型');
    if (!group.models.some((model) => model.modelType === 'video' && model.modelName === 'video-test-1')) throw new Error('未返回视频模型');

    const imageTemplate = saveModelPromptTemplate({
      name: '通用图片模板',
      type: 'imagePrompt',
      content: 'Image prompt template with {{prompt}}',
    }).template;
    const videoTemplate = saveModelPromptTemplate({
      name: '通用视频模板',
      type: 'videoPrompt',
      content: 'Video prompt template with storyboardItem and videoDesc',
    }).template;

    let duplicateMessage = '';
    try {
      saveModelPromptTemplate({
        name: '通用图片模板',
        type: 'imagePrompt',
        content: 'duplicate',
      });
    } catch (error) {
      duplicateMessage = String(error?.message || '');
    }
    if (!duplicateMessage.includes('已存在')) throw new Error(\`同类型重名未阻止：\${duplicateMessage || 'empty'}\`);

    let mismatchMessage = '';
    try {
      bindModelPromptTemplate({
        connectionId: savedConnection.connection.id,
        modelName: 'gpt-image-2',
        modelType: 'image',
        templateId: videoTemplate.id,
      });
    } catch (error) {
      mismatchMessage = String(error?.message || '');
    }
    if (!mismatchMessage.includes('不匹配')) throw new Error(\`类型不匹配绑定未阻止：\${mismatchMessage || 'empty'}\`);

    const binding = bindModelPromptTemplate({
      connectionId: savedConnection.connection.id,
      modelName: 'gpt-image-2',
      modelType: 'image',
      templateId: imageTemplate.id,
    }).binding;
    if (binding.templateId !== imageTemplate.id) throw new Error('图片模板绑定失败');

    config = getModelPromptConfig();
    const imageModel = config.connections
      .flatMap((item) => item.models)
      .find((model) => model.connectionId === savedConnection.connection.id && model.modelName === 'gpt-image-2');
    if (!imageModel || imageModel.status !== 'bound' || imageModel.binding?.templateId !== imageTemplate.id) {
      throw new Error('绑定状态未在配置中体现');
    }

    let deleteReferencedMessage = '';
    try {
      deleteModelPromptTemplate({ id: imageTemplate.id });
    } catch (error) {
      deleteReferencedMessage = String(error?.message || '');
    }
    if (!deleteReferencedMessage.includes('引用')) throw new Error(\`被引用模板删除未阻止：\${deleteReferencedMessage || 'empty'}\`);

    let changeReferencedTypeMessage = '';
    try {
      saveModelPromptTemplate({
        id: imageTemplate.id,
        name: '通用图片模板',
        type: 'videoPrompt',
        content: 'changed type',
      });
    } catch (error) {
      changeReferencedTypeMessage = String(error?.message || '');
    }
    if (!changeReferencedTypeMessage.includes('不能修改类型')) throw new Error(\`被引用模板改类型未阻止：\${changeReferencedTypeMessage || 'empty'}\`);

    saveApiConnection({
      connection: {
        id: savedConnection.connection.id,
        name: savedConnection.connection.name,
        serviceType: savedConnection.connection.serviceType,
        baseUrl: savedConnection.connection.baseUrl,
        apiKey: savedConnection.connection.apiKey,
        capabilities: [],
        models: [
          { id: 'video-test-1', displayName: 'Video Test 1', modelName: 'video-test-1', type: 'video' },
        ],
      },
    });
    config = getModelPromptConfig();
    if (!config.invalidMappings.some((item) => item.modelName === 'gpt-image-2' && item.reason === 'model-missing')) {
      throw new Error('模型删除后失效映射不可见');
    }

    const cleared = clearModelPromptBinding({
      connectionId: savedConnection.connection.id,
      modelName: 'gpt-image-2',
      modelType: 'image',
    });
    if (!cleared.cleared) throw new Error('清除失效绑定失败');

    const deleted = deleteModelPromptTemplate({ id: imageTemplate.id });
    if (deleted.templateId !== imageTemplate.id) throw new Error('清除引用后删除模板失败');

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
      VT_STUDIO_VERIFY_F_002_004: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-004 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('F-002-004 model prompt template verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
