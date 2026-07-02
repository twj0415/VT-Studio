import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-005-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-005-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-005-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['src/shared/contracts/preload.ts', 'agentConfig: {'],
  ['src/preload/index.ts', 'settings:agent-config:get'],
  ['src/main/ipc/settings.ts', 'settings:agent-config:save'],
  ['src/main/services/settings/agent-config.ts', 'agent_model_configs'],
  ['src/main/services/settings/agent-config.ts', 'TEXT_AGENT_KEYS'],
  ['src/main/services/model/resolver.ts', 'resolveTextAgentModel'],
  ['src/main/services/model/text.ts', 'agentConfig?.temperature !== null'],
  ['src/main/services/model/text.ts', 'agentConfig.maxOutputTokens > 0'],
  ['src/renderer/src/features/settings/components/ModelServiceConfig.vue', '<AgentConfig'],
  ['src/renderer/src/features/settings/components/AgentConfig.vue', 'Agent 高级设置'],
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
  import { closeDatabase, runMigrations, runSeed } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/database/index.ts')))};
  import { saveApiConnection } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/model-config.ts')))};
  import { getAgentConfig, saveAgentConfig } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/agent-config.ts')))};
  import { resolveModelKey } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/model/resolver.ts')))};

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureRuntime();
    initializeFileSystem();
    runMigrations();
    runSeed();

    const saved = saveApiConnection({
      connection: {
        name: 'Agent 验证 OpenAI 中转',
        serviceType: 'openai-gateway',
        baseUrl: 'http://127.0.0.1:9000/v1',
        apiKey: 'test-key',
        capabilities: [],
        models: [
          { id: 'gpt-5.5', displayName: 'GPT-5.5', modelName: 'gpt-5.5', type: 'text', think: true },
          { id: 'gpt-5.4', displayName: 'GPT-5.4', modelName: 'gpt-5.4', type: 'text', think: false },
          { id: 'gpt-image-2', displayName: 'GPT Image 2', modelName: 'gpt-image-2', type: 'image' },
        ],
      },
    });

    const initial = getAgentConfig();
    if (initial.agents.some((agent) => agent.key === 'ttsDubbing')) throw new Error('F-002-005 不应返回 ttsDubbing');
    if (!initial.defaultTextModel || initial.defaultTextModel.modelName !== 'gpt-5.5') throw new Error('默认文本模型未读取成功');
    const scriptAgent = initial.agents.find((agent) => agent.key === 'scriptAgent');
    if (!scriptAgent) throw new Error('scriptAgent 未返回');
    if (scriptAgent.status !== 'inherited') throw new Error('scriptAgent 初始状态应继承默认文本模型');

    let resolved = resolveModelKey('scriptAgent');
    if (resolved.modelId !== \`\${saved.connection.id}:gpt-5.5\`) throw new Error(\`scriptAgent 未继承默认文本模型：\${resolved.modelId}\`);
    if (resolved.agentConfig?.temperature !== 0.7) throw new Error('默认全局 temperature 未生效');
    if (resolved.agentConfig?.maxOutputTokens !== 0) throw new Error('默认全局 maxOutputTokens 未保持自动');

    saveAgentConfig({
      globalSettings: { temperature: 0, maxOutputTokens: 0 },
      agents: [
        {
          key: 'scriptAgent',
          modelId: \`\${saved.connection.id}:gpt-5.4\`,
          temperature: 0,
          maxOutputTokens: 2048,
        },
      ],
    });

    resolved = resolveModelKey('scriptAgent');
    if (resolved.modelId !== \`\${saved.connection.id}:gpt-5.4\`) throw new Error('高级覆盖模型未生效');
    if (resolved.agentConfig?.temperature !== 0) throw new Error('temperature=0 被忽略');
    if (resolved.agentConfig?.maxOutputTokens !== 2048) throw new Error('Agent maxOutputTokens 覆盖未生效');

    const productionResolved = resolveModelKey('productionAgent');
    if (productionResolved.modelId !== \`\${saved.connection.id}:gpt-5.5\`) throw new Error('未覆盖 Agent 应继续继承默认文本模型');
    if (productionResolved.agentConfig?.temperature !== 0) throw new Error('全局 temperature=0 未继承');

    saveApiConnection({
      connection: {
        id: saved.connection.id,
        name: saved.connection.name,
        serviceType: saved.connection.serviceType,
        baseUrl: saved.connection.baseUrl,
        apiKey: saved.connection.apiKey,
        capabilities: [],
        models: [
          { id: 'gpt-5.5', displayName: 'GPT-5.5', modelName: 'gpt-5.5', type: 'text', think: true },
        ],
      },
    });

    let invalidOverrideMessage = '';
    try {
      resolveModelKey('scriptAgent');
    } catch (error) {
      invalidOverrideMessage = String(error?.message || '');
    }
    if (!invalidOverrideMessage.includes('覆盖模型已失效')) throw new Error(\`覆盖失效未返回明确错误：\${invalidOverrideMessage || 'empty'}\`);

    saveAgentConfig({
      globalSettings: { temperature: 0, maxOutputTokens: 0 },
      agents: [
        {
          key: 'scriptAgent',
          modelId: null,
          temperature: null,
          maxOutputTokens: null,
        },
      ],
    });

    resolved = resolveModelKey('scriptAgent');
    if (resolved.modelId !== \`\${saved.connection.id}:gpt-5.5\`) throw new Error('关闭覆盖后未恢复继承默认文本模型');

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
      VT_STUDIO_VERIFY_F_002_005: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-005 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('F-002-005 agent model configuration verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
