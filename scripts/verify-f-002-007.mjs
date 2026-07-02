import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { build } from 'esbuild';

const workspaceRoot = process.cwd();
const tempRoot = mkdtempSync(join(tmpdir(), 'vt-studio-f-002-007-'));
const bundleDirectory = join(workspaceRoot, 'node_modules', '.cache', 'vt-studio');
const bundlePath = join(bundleDirectory, 'verify-f-002-007-bundle.cjs');
const entryPath = join(tempRoot, 'verify-f-002-007-entry.ts');

function importPath(filePath) {
  return filePath.replace(/\\/g, '/');
}

const staticChecks = [
  ['docs/tasks/F-002-007-Skill管理.md', '只按 skillId 查库'],
  ['src/main/ipc/settings.ts', 'settings:skill:list'],
  ['src/main/ipc/settings.ts', 'settings:skill:get-content'],
  ['src/main/ipc/settings.ts', 'settings:skill:save-content'],
  ['src/preload/index.ts', 'settings:skill:save-content'],
  ['src/shared/contracts/preload.ts', 'skill: {'],
  ['src/renderer/src/features/settings/SettingsHome.vue', '<SkillManagement'],
  ['src/renderer/src/features/settings/components/SkillManagement.vue', 'Skill 结构风险'],
  ['src/main/services/settings/skill-management.ts', 'safeJoin(getRuntimeDirectories().skills'],
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
  import { getSkillContent, getSkillManagementList, saveSkillContent } from ${JSON.stringify(importPath(join(workspaceRoot, 'src/main/services/settings/skill-management.ts')))};

  async function main() {
    process.env.VT_STUDIO_USER_DATA = ${JSON.stringify(tempRoot.replace(/\\/g, '\\\\'))};
    configureGpu();
    configureRuntime();
    initializeFileSystem();
    runMigrations();
    runSeed();

    const directories = getRuntimeDirectories();
    const initial = getSkillManagementList({});
    if (initial.skills.length < 20) throw new Error('Skill 列表数量不正确');

    const target = initial.skills.find((skill) => skill.path === 'references/pipeline.md');
    if (!target) throw new Error('pipeline Skill 不存在');
    if (target.fileStatus !== 'missing') throw new Error('默认缺失文件状态判断不正确');

    const filePath = join(directories.skills, target.path);
    mkdirSync(dirname(filePath), { recursive: true });
    writeFileSync(filePath, [
      '---',
      'name: pipeline',
      'description: 原始描述',
      '---',
      '',
      '# Pipeline',
      '',
      'content',
    ].join('\\n'));

    const searched = getSkillManagementList({ keyword: 'pipeline' });
    const ready = searched.skills.find((skill) => skill.id === target.id);
    if (!ready || ready.fileStatus !== 'ready') throw new Error('文件存在状态未刷新');

    const content = await getSkillContent({ id: target.id });
    if (!content.content.includes('# Pipeline')) throw new Error('Skill 内容读取失败');

    const risky = await saveSkillContent({ id: target.id, content: '# no frontmatter' });
    if (risky.saved || risky.warnings.length === 0) throw new Error('缺少 frontmatter 未触发风险提示');

    const saved = await saveSkillContent({
      id: target.id,
      content: [
        '---',
        'name: pipeline_updated',
        'description: 更新后的描述',
        '---',
        '',
        '# Pipeline Updated',
      ].join('\\n'),
    });
    if (!saved.saved || !saved.skill) throw new Error('Skill 保存失败');
    if (saved.skill.name !== 'pipeline_updated') throw new Error('Skill name 未从 frontmatter 更新');
    if (saved.skill.description !== '更新后的描述') throw new Error('Skill description 未从 frontmatter 更新');
    if (saved.skill.embeddingStatus !== 'expired' || saved.skill.state !== -1) throw new Error('保存后未标记 embedding 过期');
    if (saved.skill.md5 === target.md5) throw new Error('Skill md5 未更新');

    const savedContent = await getSkillContent({ id: target.id });
    if (!savedContent.content.includes('Pipeline Updated')) throw new Error('保存后内容未写入文件');

    getDatabase().prepare('UPDATE skill_list SET path = ? WHERE id = ?').run('../outside.md', target.id);
    let pathEscapeBlocked = false;
    try {
      await getSkillContent({ id: target.id });
    } catch {
      pathEscapeBlocked = true;
    }
    if (!pathEscapeBlocked) throw new Error('Skill 路径越界未被拦截');

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
      VT_STUDIO_VERIFY_F_002_007: '1',
    },
  });

  if (result.status !== 0) {
    if (result.error) {
      throw result.error;
    }
    throw new Error(`F-002-007 Electron verification failed with exit code ${result.status}, signal ${result.signal ?? 'none'}`);
  }

  console.log('F-002-007 skill management verification passed');
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
  rmSync(bundlePath, { force: true });
}
