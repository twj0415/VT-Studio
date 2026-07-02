import { existsSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-003-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-003-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-003-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const entrySource = `
  import { existsSync } from 'node:fs';
  import { createServer } from 'node:http';
  import { app } from 'electron';
  import { configureRuntime } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/runtime.ts')))};
  import { initializeFileSystem, getRuntimeDirectories } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/file-system/index.ts')))};
  import { getDatabase, closeDatabase, runMigrations, runSeed } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/database/index.ts')))};
  import {
    addVendorCode,
    deleteVendorModel,
    getVendorList,
    runVendorTextTest,
    saveVendorEnabled,
    saveVendorInputs,
    saveVendorModel,
  } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/vendor.ts')))};

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureRuntime();
    initializeFileSystem();
    runMigrations();
    runSeed();

    const directories = getRuntimeDirectories();
    if (!existsSync(directories.vendors)) throw new Error('vendors 运行目录未创建');

    const initial = getVendorList();
    const openai = initial.vendors.find((vendor) => vendor.id === 'openai');
    if (!openai) throw new Error('默认 OpenAI 供应商未初始化');
    if (openai.status !== 'ready') throw new Error('OpenAI 内置供应商未进入 ready 状态');
    if (openai.codeReady) throw new Error('OpenAI 缺用户 adapter 时应使用内置 adapter');
    if (!openai.models.some((model) => model.type === 'text')) throw new Error('OpenAI 缺默认文本模型');

    const anthropic = initial.vendors.find((vendor) => vendor.id === 'anthropic');
    const gemini = initial.vendors.find((vendor) => vendor.id === 'gemini');
    const comfyui = initial.vendors.find((vendor) => vendor.id === 'comfyui');
    if (!anthropic || !gemini || !comfyui) throw new Error('新增默认供应商未初始化完整');

    saveVendorInputs({ vendorId: 'openai', inputValues: { apiKey: 'test-key', baseUrl: 'https://example.test/v1', extra: 'ignored' } });
    saveVendorEnabled({ vendorId: 'openai', enabled: true });

    saveVendorModel({
      vendorId: 'openai',
      model: { name: '验证模型', modelName: 'verify-text-model', type: 'text', think: false },
    });

    const updated = getVendorList().vendors.find((vendor) => vendor.id === 'openai');
    if (!updated?.enabled) throw new Error('供应商启用状态未保存');
    if (updated.inputValues.extra) throw new Error('非法 input key 被保存');
    if (!updated.models.some((model) => model.modelName === 'verify-text-model')) throw new Error('新增模型未保存');

    const db = getDatabase();
    db.prepare('UPDATE agent_model_configs SET vendor_id = ?, model_id = ? WHERE key = ?').run('openai', 'openai:verify-text-model', 'universalAi');

    let blocked = false;
    try {
      deleteVendorModel({ vendorId: 'openai', modelName: 'verify-text-model' });
    } catch (error) {
      blocked = String(error?.message || '').includes('引用');
    }
    if (!blocked) throw new Error('被 Agent 引用的模型删除未被阻止');

    db.prepare('UPDATE agent_model_configs SET vendor_id = NULL, model_id = NULL WHERE key = ?').run('universalAi');
    deleteVendorModel({ vendorId: 'openai', modelName: 'verify-text-model' });
    const afterDelete = getVendorList().vendors.find((vendor) => vendor.id === 'openai');
    if (afterDelete?.models.some((model) => model.modelName === 'verify-text-model')) throw new Error('解除引用后模型未删除');

    let compatibleRequestUrl = '';
    const compatibleServer = createServer((req, res) => {
      compatibleRequestUrl = req.url ?? '';
      if (req.method !== 'POST' || compatibleRequestUrl !== '/v1/chat/completions') {
        res.writeHead(404, { 'content-type': 'text/html; charset=utf-8' });
        res.end('<html>not found</html>');
        return;
      }

      req.resume();
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(JSON.stringify({
        id: 'chatcmpl-verify',
        object: 'chat.completion',
        created: Math.floor(Date.now() / 1000),
        model: 'gpt-5.5',
        choices: [
          {
            index: 0,
            message: { role: 'assistant', content: 'compatible ok' },
            finish_reason: 'stop',
          },
        ],
        usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 },
      }));
    });

    await new Promise((resolve) => compatibleServer.listen(0, '127.0.0.1', resolve));
    const compatibleAddress = compatibleServer.address();
    if (!compatibleAddress || typeof compatibleAddress === 'string') throw new Error('OpenAI Compatible mock 服务端口获取失败');
    try {
      saveVendorInputs({ vendorId: 'openai', inputValues: { apiKey: 'test-key', baseUrl: \`http://127.0.0.1:\${compatibleAddress.port}/v1\` } });
      const compatibleResult = await runVendorTextTest({ vendorId: 'openai', modelName: 'gpt-5.5', prompt: 'hello' });
      if (compatibleResult.content !== 'compatible ok') throw new Error('OpenAI Compatible 文本测试结果不正确');
      if (compatibleRequestUrl !== '/v1/chat/completions') throw new Error(\`OpenAI 自定义 Base URL 未走 chat/completions：\${compatibleRequestUrl}\`);

      saveVendorInputs({ vendorId: 'openai', inputValues: { apiKey: 'test-key', baseUrl: \`http://127.0.0.1:\${compatibleAddress.port}/v1/chat/completions\` } });
      await runVendorTextTest({ vendorId: 'openai', modelName: 'gpt-5.5', prompt: 'hello' });
      if (compatibleRequestUrl !== '/v1/chat/completions') throw new Error(\`OpenAI 完整接口地址未被归一化：\${compatibleRequestUrl}\`);

      saveVendorInputs({ vendorId: 'openai', inputValues: { apiKey: 'test-key', baseUrl: \`http://127.0.0.1:\${compatibleAddress.port}\` } });
      await runVendorTextTest({ vendorId: 'openai', modelName: 'gpt-5.5', prompt: 'hello' });
      if (compatibleRequestUrl !== '/v1/chat/completions') throw new Error(\`OpenAI 根地址未自动补到 /v1：\${compatibleRequestUrl}\`);
    } finally {
      await new Promise((resolve) => compatibleServer.close(resolve));
    }

    addVendorCode({
      code: \`
        exports.vendor = {
          id: 'verify-error-vendor',
          name: '验证错误供应商',
          author: 'VT Studio',
          inputs: [],
          inputValues: {},
          models: [{ name: '错误模型', modelName: 'error-model', type: 'text', think: false }],
        };
        exports.textRequest = function () {
          return {
            specificationVersion: 'v3',
            provider: 'verify',
            modelId: 'error-model',
            supportedUrls: {},
            doGenerate: async function () {
              throw new Error('upstream refused test request');
            },
            doStream: async function () {
              throw new Error('stream should not be used');
            },
          };
        };
        exports.imageRequest = async function () { return ''; };
        exports.videoRequest = async function () { return ''; };
        exports.ttsRequest = async function () { return ''; };
      \`,
    });

    let testErrorMessage = '';
    try {
      await runVendorTextTest({
        vendorId: 'verify-error-vendor',
        modelName: 'error-model',
        prompt: 'hello',
      });
    } catch (error) {
      testErrorMessage = String(error?.message || '');
    }
    if (!testErrorMessage.includes('模型测试失败：upstream refused test request')) {
      throw new Error(\`模型测试错误未返回上游原因：\${testErrorMessage || 'empty'}\`);
    }

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
  await import('node:fs').then(({ writeFileSync }) => {
    writeFileSync(entryPath, entrySource);
  });
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
      VT_STUDIO_VERIFY_F_002_003: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-003 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('F-002-003 vendor configuration verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
