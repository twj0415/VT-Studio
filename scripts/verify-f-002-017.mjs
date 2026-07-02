import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-017-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-017-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-017-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['src/shared/contracts/preload.ts', 'api: {'],
  ['src/shared/contracts/preload.ts', 'resource: {'],
  ['src/preload/index.ts', 'settings:api:save'],
  ['src/preload/index.ts', 'settings:resource:save-binding'],
  ['src/main/ipc/settings.ts', 'settings:api:test'],
  ['src/main/ipc/settings.ts', 'settings:resource:test'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<ModelServiceConfig'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '模型服务'],
  ['src/renderer/src/features/settings/components/ModelServiceConfig.vue', '启用模型'],
  ['src/renderer/src/features/settings/components/ModelServiceConfig.vue', '默认模型'],
  ['src/main/services/settings/model-config.ts', 'gpt-image-2'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<template v-if="developerVisible">'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<VendorConfig'],
];

for (const [relativePath, needle] of staticChecks) {
  const filePath = join(workspaceRoot, relativePath);
  const content = await import('node:fs').then(({ readFileSync }) => readFileSync(filePath, 'utf-8'));
  if (!content.includes(needle)) {
    throw new Error(`${relativePath} 缺少 ${needle}`);
  }
}

const entrySource = `
  import { createServer } from 'node:http';
  import { app } from 'electron';
  import { configureRuntime } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/runtime.ts')))};
  import { initializeFileSystem, getRuntimeDirectories } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/file-system/index.ts')))};
  import { closeDatabase, runMigrations, runSeed } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/database/index.ts')))};
  import {
    getApiConnectionList,
    getConnectionTemplates,
    getResourceConfig,
    saveApiConnection,
    testResourceBinding,
  } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/model-config.ts')))};

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureRuntime();
    initializeFileSystem();
    runMigrations();
    runSeed();

    const directories = getRuntimeDirectories();
    if (!directories.userData || !directories.vendors) throw new Error('运行目录未初始化');

    let requestUrl = '';
    const server = createServer((req, res) => {
      requestUrl = req.url ?? '';
      if (req.method !== 'POST' || requestUrl !== '/v1/chat/completions') {
        res.writeHead(404, { 'content-type': 'text/html; charset=utf-8' });
        res.end('<html>not found</html>');
        return;
      }

      req.resume();
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(JSON.stringify({
        id: 'chatcmpl-f-002-017',
        object: 'chat.completion',
        created: Math.floor(Date.now() / 1000),
        model: 'gpt-5.5',
        choices: [
          {
            index: 0,
            message: { role: 'assistant', content: 'resource binding ok' },
            finish_reason: 'stop',
          },
        ],
        usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 },
      }));
    });

    await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
    const address = server.address();
    if (!address || typeof address === 'string') throw new Error('mock 服务端口获取失败');

    try {
      const templates = getConnectionTemplates();
      const gateway = templates.services.find((service) => service.serviceType === 'openai-gateway');
      if (!gateway) throw new Error('OpenAI 中转模板不存在');
      if (!gateway.models.some((model) => model.modelName === 'gpt-5.5' && model.type === 'text')) throw new Error('OpenAI 中转缺少 gpt-5.5 文本模型');
      if (!gateway.models.some((model) => model.modelName === 'gpt-5.4' && model.type === 'text')) throw new Error('OpenAI 中转缺少 gpt-5.4 文本模型');
      if (!gateway.models.some((model) => model.modelName === 'gpt-image-2' && model.type === 'image')) throw new Error('OpenAI 中转缺少 gpt-image-2 图片模型');

      const saved = saveApiConnection({
        connection: {
          name: '验证 OpenAI 中转',
          serviceType: 'openai-gateway',
          baseUrl: \`http://127.0.0.1:\${address.port}/v1\`,
          apiKey: 'test-key',
          capabilities: [],
          models: [
            { id: 'gpt-5.5', displayName: 'GPT-5.5', modelName: 'gpt-5.5', type: 'text', think: true },
            { id: 'gpt-image-2', displayName: 'GPT Image 2', modelName: 'gpt-image-2', type: 'image' },
          ],
        },
      });

      if (saved.connection.status !== 'ready') throw new Error('连接保存后未 ready');
      if (!saved.connection.capabilities.includes('text') || !saved.connection.capabilities.includes('image')) throw new Error('连接能力未从模型自动推导');
      const list = getApiConnectionList();
      if (!list.connections.some((connection) => connection.id === saved.connection.id)) throw new Error('连接列表未返回新连接');

      const resource = getResourceConfig();
      const text = resource.capabilities.find((capability) => capability.capability === 'text');
      if (!text || text.status !== 'configured' || text.binding?.modelName !== 'gpt-5.5') throw new Error('文本默认模型未自动绑定成功');
      const image = resource.capabilities.find((capability) => capability.capability === 'image');
      if (!image || image.status !== 'configured' || image.binding?.modelName !== 'gpt-image-2') throw new Error('图片默认模型未自动绑定成功');

      const result = await testResourceBinding({
        capability: 'text',
        prompt: 'hello',
      });
      if (result.content !== 'resource binding ok') throw new Error('文本能力测试结果不正确');
      if (requestUrl !== '/v1/chat/completions') throw new Error(\`连接 Base URL 未正确调用：\${requestUrl}\`);
    } finally {
      await new Promise((resolve) => server.close(resolve));
      closeDatabase();
      app.quit();
    }
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
      VT_STUDIO_VERIFY_F_002_017: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-017 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('F-002-017 model connection configuration verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
