import { existsSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-core-011-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-core-011-bundle.cjs');
const entryPath = join(tempRoot, 'verify-core-011-entry.ts');
const transformersStubPath = join(tempRoot, 'transformers-stub.mjs');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const transformersStubSource = `
  export const env = {
    allowRemoteModels: false,
    allowLocalModels: true,
    localModelPath: '',
  };
  export async function pipeline() {
    throw new Error('transformers pipeline should not be initialized in missing-model verification');
  }
`;

const entrySource = `
  import { existsSync } from 'node:fs';
  import { app } from 'electron';
  import { configureRuntime } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/app/runtime.ts')))};
  import { initializeFileSystem, getRuntimeDirectories } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/file-system/index.ts')))};
  import { getDatabase, closeDatabase, runMigrations } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/database/index.ts')))};
  import { getEmbeddingModelStatus, embedText } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/embedding/index.ts')))};
  import { clearMemory, createMemoryIsolationKey, getMemoryContext } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/memory/index.ts')))};
  import { readSkillFile, resolveSkillsForAgent } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/skill-retrieval/index.ts')))};

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureRuntime();
    initializeFileSystem();
    runMigrations();

    const db = getDatabase();
    const memoriesTable = db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'memories'").get();
    if (!memoriesTable) throw new Error('memories 表未创建');

    const metadataColumn = db.prepare("PRAGMA table_info(memories)").all().find((column) => column.name === 'metadata');
    if (!metadataColumn) throw new Error('memories.metadata 字段未创建');

    const isolationKey = createMemoryIsolationKey({ projectId: 1, agentType: 'scriptAgent' });
    const context = await getMemoryContext({ isolationKey, query: '测试问题' });
    if (!Array.isArray(context.shortTerm) || !Array.isArray(context.summaries) || !Array.isArray(context.rag)) {
      throw new Error('memory.getContext 返回结构无效');
    }

    const clearResult = clearMemory({ projectId: 1, agentType: 'scriptAgent', type: 'summary' });
    if (typeof clearResult.deleted !== 'number' || typeof clearResult.updated !== 'number') {
      throw new Error('memory.clear 返回结构无效');
    }

    const modelStatus = getEmbeddingModelStatus();
    if (modelStatus.available) {
      const embedding = await embedText('VT Studio verification');
      if (!Array.isArray(embedding) || embedding.length === 0) throw new Error('embedText 未返回有效向量');
    } else {
      let failedAsExpected = false;
      try {
        await embedText('VT Studio verification');
      } catch (error) {
        failedAsExpected = String(error?.message || '').includes('本地向量模型文件不存在');
      }
      if (!failedAsExpected) throw new Error('缺少 ONNX 模型时 embedText 未返回清晰错误');
    }

    const resolvedSkills = await resolveSkillsForAgent({ attribution: 'script_agent_decision.md', query: '故事骨架', limit: 3 });
    if (!Array.isArray(resolvedSkills.mainSkills) || !Array.isArray(resolvedSkills.referenceSkills)) {
      throw new Error('skill.resolveForAgent 返回结构无效');
    }

    let pathEscapeBlocked = false;
    try {
      await readSkillFile('../outside.md');
    } catch {
      pathEscapeBlocked = true;
    }
    if (!pathEscapeBlocked) throw new Error('Skill 路径越界未被拦截');

    const directories = getRuntimeDirectories();
    if (!existsSync(directories.models) || !existsSync(directories.skills)) {
      throw new Error('runtime models/skills 目录未创建');
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
    writeFileSync(transformersStubPath, transformersStubSource);
    writeFileSync(entryPath, entrySource);
  });
  await build({
    entryPoints: [entryPath],
    outfile: bundlePath,
    bundle: true,
    platform: 'node',
    format: 'cjs',
    target: 'node20',
    external: ['better-sqlite3', 'electron'],
    alias: {
      '@shared': join(workspaceRoot, 'src/shared'),
      '@huggingface/transformers': transformersStubPath,
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
    env: {
      ...process.env,
      VT_STUDIO_VERIFY_CORE_011: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`CORE-011 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('CORE-011 service verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
