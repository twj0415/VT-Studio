import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-002-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-002-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-002-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['docs/tasks/F-002-002-语言设置.md', '先只开放 `zh-CN / en`'],
  ['src/renderer/src/i18n/index.ts', 'SUPPORTED_LOCALES'],
  ['src/renderer/src/stores/language.ts', "defineStore('language'"],
  ['src/renderer/src/App.vue', 't-config-provider'],
  ['src/renderer/src/features/settings/components/LanguageConfig.vue', 'language.saved'],
  ['src/renderer/src/features/auth/LoginHome.vue', 'login-language-select'],
];

for (const [relativePath, needle] of staticChecks) {
  const content = await import('node:fs').then(({ readFileSync }) => readFileSync(join(workspaceRoot, relativePath), 'utf-8'));
  if (!content.includes(needle)) {
    throw new Error(`${relativePath} 缺少 ${needle}`);
  }
}

const entrySource = `
  import {
    APP_LOCALE_INITIALIZED_KEY,
    APP_LOCALE_KEY,
    LANGUAGE_OPTIONS,
    detectSystemLocale,
    getStoredLocale,
    hasLocaleInitialized,
    normalizeLocale,
    persistLocale,
    resolveInitialLocale,
    resolveTDesignGlobalConfig,
    setI18nLocale,
  } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/renderer/src/i18n/index.ts')))};

  const storage = new Map();
  globalThis.window = {
    localStorage: {
      getItem(key) {
        return storage.has(key) ? storage.get(key) : null;
      },
      setItem(key, value) {
        storage.set(key, String(value));
      },
      removeItem(key) {
        storage.delete(key);
      },
    },
  };
  globalThis.document = {
    documentElement: {
      lang: 'zh-CN',
    },
  };
  globalThis.navigator = {
    language: 'en-US',
  };

  if (LANGUAGE_OPTIONS.length !== 2) throw new Error('第一版语言数量不正确');
  if (normalizeLocale('en-US') !== 'en' || normalizeLocale('zh-TW') !== 'zh-CN') throw new Error('locale 归一化失败');
  if (detectSystemLocale() !== 'en') throw new Error('系统语言检测失败');

  const initialLocale = resolveInitialLocale();
  if (initialLocale !== 'en') throw new Error('首次启动未按系统语言初始化');
  if (!hasLocaleInitialized() || getStoredLocale() !== 'en') throw new Error('首次启动未写入本地 locale');

  persistLocale('zh-CN');
  if (resolveInitialLocale() !== 'zh-CN') throw new Error('已选择语言仍被系统语言覆盖');

  setI18nLocale('zh-CN');
  if (document.documentElement.lang !== 'zh-CN') throw new Error('document lang 未同步');

  const zhConfig = resolveTDesignGlobalConfig('zh-CN');
  const enConfig = resolveTDesignGlobalConfig('en');
  if (!zhConfig || !enConfig || zhConfig === enConfig) throw new Error('TDesign locale 映射无效');

  persistLocale('en');
  setI18nLocale('en');
  if (storage.get(APP_LOCALE_KEY) !== 'en' || storage.get(APP_LOCALE_INITIALIZED_KEY) !== '1') throw new Error('locale 持久化失败');
  if (document.documentElement.lang !== 'en') throw new Error('locale 未同步 document lang');

  console.log('F-002-002 language verification passed');
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
    absWorkingDir: workspaceRoot,
    alias: {
      '@renderer': join(workspaceRoot, 'src/renderer/src'),
    },
    logLevel: 'silent',
  });

  if (!existsSync(bundlePath)) {
    throw new Error('验证 bundle 未生成');
  }

  const result = spawnSync(process.execPath, [bundlePath], {
    cwd: workspaceRoot,
    stdio: 'inherit',
    timeout: 30000,
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-002 verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
