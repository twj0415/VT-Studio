import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-core-012-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-core-012-bundle.cjs');
const entryPath = join(tempRoot, 'verify-core-012-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['src/main/app/server.ts', 'handleMediaRequest'],
  ['src/main/ipc/index.ts', 'registerMediaIpc'],
  ['src/preload/index.ts', 'media:'],
  ['src/shared/contracts/preload.ts', 'createThumbnailUrl'],
  ['src/main/services/media/security.ts', 'createHmac'],
  ['src/main/services/media/request-handler.ts', 'content-range'],
  ['src/main/services/media/thumbnail.ts', 'getRuntimeDirectories().thumbnails'],
];

for (const [relativePath, needle] of staticChecks) {
  const content = await import('node:fs').then(({ readFileSync }) => readFileSync(join(workspaceRoot, relativePath), 'utf-8'));
  if (!content.includes(needle)) {
    throw new Error(`${relativePath} 缺少 ${needle}`);
  }
}

const entrySource = `
  import { existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs';
  import { join } from 'node:path';
  import { app } from 'electron';
  import { configureGpu } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/gpu.ts')))};
  import { configureRuntime } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/runtime.ts')))};
  import { startLocalServer, stopLocalServer } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/server.ts')))};
  import { initializeFileSystem, getRuntimeDirectories } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/file-system/index.ts')))};
  import { createMediaUrl, createThumbnailMediaUrl, resolveMediaUrlToPath, getOriginalMediaUrl } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/media/index.ts')))};

  function png1x1() {
    return Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR42mP8z8AABQMBgAb4iT0AAAAASUVORK5CYII=', 'base64');
  }

  async function expectStatus(url, status, init = {}) {
    const response = await fetch(url, init);
    if (response.status !== status) {
      throw new Error(\`Expected \${status}, got \${response.status} for \${url}\`);
    }
    return response;
  }

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureGpu();
    configureRuntime();
    initializeFileSystem();
    const directories = getRuntimeDirectories();

    const projectFile = join(directories.projects, 'demo', 'assets', 'image.png');
    mkdirSync(join(directories.projects, 'demo', 'assets'), { recursive: true });
    writeFileSync(projectFile, png1x1());
    writeFileSync(join(directories.projects, 'demo', 'assets', 'thumb.svg'), '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="red"/></svg>');

    const audioFile = join(directories.temp, 'tone.mp3');
    writeFileSync(audioFile, Buffer.from('0123456789abcdef'));

    const serverInfo = await startLocalServer();
    if (!serverInfo.url.startsWith('http://127.0.0.1:')) throw new Error('本地服务不是 127.0.0.1');

    try {
      const image = createMediaUrl({ root: 'project', relativePath: 'demo/assets/image.png' });
      if (!image.url.startsWith('http://127.0.0.1:')) throw new Error('media URL 未使用 127.0.0.1');
      if (image.url.includes('D:') || image.url.includes('\\\\')) throw new Error('media URL 泄露绝对路径');

      const imageResponse = await expectStatus(image.url, 200);
      if (imageResponse.headers.get('content-type') !== 'image/png') throw new Error('图片 Content-Type 不正确');

      const unsigned = image.url.replace(/&token=[^&]+/, '');
      await expectStatus(unsigned, 401);

      const tampered = image.url.replace('mode=original', 'mode=thumbnail');
      await expectStatus(tampered, 401);

      let pathEscapeBlocked = false;
      try {
        createMediaUrl({ root: 'project', relativePath: '../database/app.db' });
      } catch {
        pathEscapeBlocked = true;
      }
      if (!pathEscapeBlocked) throw new Error('路径越界未被拦截');

      const audio = createMediaUrl({ root: 'temp', relativePath: 'tone.mp3' });
      const rangeResponse = await expectStatus(audio.url, 206, { headers: { range: 'bytes=2-5' } });
      if (rangeResponse.headers.get('content-range') !== 'bytes 2-5/16') throw new Error('Range Content-Range 不正确');
      const rangeText = await rangeResponse.text();
      if (rangeText !== '2345') throw new Error('Range 内容不正确');
      await expectStatus(audio.url, 416, { headers: { range: 'bytes=999-1000' } });

      const thumbnail = createThumbnailMediaUrl({ root: 'project', relativePath: 'demo/assets/thumb.svg', size: 'small' });
      await expectStatus(thumbnail.url, 200);
      const thumbnailFiles = readdirSync(directories.thumbnails).filter((name) => name.endsWith('.webp'));
      if (thumbnailFiles.length === 0) throw new Error('缩略图缓存未生成');

      const resolved = resolveMediaUrlToPath({ url: image.url });
      if (resolved.root !== 'project' || resolved.relativePath !== 'demo/assets/image.png' || resolved.mode !== 'original') {
        throw new Error('media URL 反解结果不正确');
      }

      const original = getOriginalMediaUrl({ url: thumbnail.url });
      if (!original.url.includes('mode=original')) throw new Error('缩略图未能转原图 URL');

      const legacy = resolveMediaUrlToPath({ url: '/oss/demo/assets/image.png?size=20' });
      if (legacy.root !== 'project' || legacy.relativePath !== 'demo/assets/image.png') throw new Error('旧 /oss URL 反解失败');

      if (!existsSync(directories.thumbnails)) throw new Error('thumbnails 目录不存在');
    } finally {
      await stopLocalServer();
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
    external: ['electron', 'sharp'],
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
      VT_STUDIO_VERIFY_CORE_012: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`CORE-012 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('CORE-012 local media service verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
