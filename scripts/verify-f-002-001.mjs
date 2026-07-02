import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-001-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-001-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-001-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['docs/tasks/F-002-001-外观设置.md', '普通模式取消任意 HEX 主色'],
  ['src/renderer/src/main.ts', 'useAppearanceStore(pinia).init()'],
  ['src/renderer/src/stores/appearance.ts', "defineStore('appearance'"],
  ['src/renderer/src/features/settings/components/AppearanceConfig.vue', 'appearance.presetLabel'],
  ['src/renderer/src/styles/tokens.scss', "data-theme-preset='warm'"],
  ['src/renderer/src/styles/tokens.scss', "data-theme-preset='work'"],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<AppearanceConfig'],
];

for (const [relativePath, needle] of staticChecks) {
  const content = await import('node:fs').then(({ readFileSync }) => readFileSync(join(workspaceRoot, relativePath), 'utf-8'));
  if (!content.includes(needle)) {
    throw new Error(`${relativePath} 缺少 ${needle}`);
  }
}

const entrySource = `
  import {
    APPEARANCE_PRESETS,
    applyAppearanceSettings,
    initAppearance,
    normalizeAppearanceSettings,
    persistAppearanceSettings,
    readStoredAppearanceSettings,
  } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/renderer/src/features/settings/appearance/theme.ts')))};

  const storage = new Map();
  let mediaListener = null;
  const classSet = new Set();
  const attributes = new Map();
  const style = {};
  const mediaQuery = {
    matches: false,
    addEventListener(event, listener) {
      if (event === 'change') {
        mediaListener = listener;
      }
    },
    addListener(listener) {
      mediaListener = listener;
    },
  };

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
    matchMedia() {
      return mediaQuery;
    },
  };

  globalThis.document = {
    documentElement: {
      style,
      setAttribute(name, value) {
        attributes.set(name, String(value));
      },
      getAttribute(name) {
        return attributes.get(name) ?? null;
      },
      classList: {
        toggle(name, force) {
          if (force) classSet.add(name);
          else classSet.delete(name);
        },
        contains(name) {
          return classSet.has(name);
        },
      },
    },
  };

  const normalized = normalizeAppearanceSettings({
    mode: 'invalid',
    themePresetId: 'unknown',
    fontSize: 999,
  });
  if (normalized.mode !== 'auto' || normalized.themePresetId !== 'studio' || normalized.fontSize !== 14) {
    throw new Error('外观设置归一化失败');
  }

  if (APPEARANCE_PRESETS.length !== 3) {
    throw new Error('主题预设数量不正确');
  }

  persistAppearanceSettings({
    mode: 'dark',
    themePresetId: 'work',
    fontSize: 16,
  });
  const stored = readStoredAppearanceSettings();
  if (stored.mode !== 'dark' || stored.themePresetId !== 'work' || stored.fontSize !== 16) {
    throw new Error('外观设置持久化读取失败');
  }

  const initSettings = initAppearance();
  if (initSettings.mode !== 'dark' || !classSet.has('dark')) {
    throw new Error('初始化主题未生效');
  }
  if (attributes.get('data-theme-preset') !== 'work' || style.fontSize !== '16px') {
    throw new Error('初始化主题属性不正确');
  }

  applyAppearanceSettings({
    mode: 'auto',
    themePresetId: 'warm',
    fontSize: 18,
  });
  if (attributes.get('theme-mode') !== 'auto' || attributes.get('data-theme-preset') !== 'warm' || style.fontSize !== '18px') {
    throw new Error('应用 auto/warm/18 失败');
  }

  mediaQuery.matches = true;
  mediaListener?.();
  if (!classSet.has('dark') || attributes.get('data-color-mode') !== 'dark') {
    throw new Error('系统主题监听未生效');
  }

  applyAppearanceSettings({
    mode: 'light',
    themePresetId: 'studio',
    fontSize: 20,
  });
  if (attributes.get('data-theme-preset') !== 'studio' || classSet.has('dark') || style.fontSize !== '20px') {
    throw new Error('重新应用 studio/light/20 失败');
  }

  console.log('F-002-001 appearance verification passed');
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
    throw new Error(`F-002-001 verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
