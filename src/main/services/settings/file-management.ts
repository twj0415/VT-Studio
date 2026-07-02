import { statSync } from 'node:fs';
import { shell } from 'electron';
import { VT_STATUS } from '@shared/constants/status';
import type {
  FileManagementDirectoryGroup,
  FileManagementDirectoryItem,
  FileManagementListResult,
  FileManagementOpenPayload,
  FileManagementOpenResult,
} from '@shared/types/file-management';
import { ensureDirectory, getRuntimeDirectories, pathExists } from '../file-system';
import { createError } from '../result';

interface DirectoryDefinition {
  key: string;
  name: string;
  description: string;
  group: FileManagementDirectoryGroup;
  autoCreate: boolean;
  resolvePath: () => string;
}

const DIRECTORY_DEFINITIONS: DirectoryDefinition[] = [
  {
    key: 'projects',
    name: '项目目录',
    description: '项目业务文件和后续素材产出会从这里分项目落位。',
    group: 'common',
    autoCreate: true,
    resolvePath: () => getRuntimeDirectories().projects,
  },
  {
    key: 'exports',
    name: '导出目录',
    description: '数据库备份、导出结果和后续交付文件统一放这里。',
    group: 'common',
    autoCreate: true,
    resolvePath: () => getRuntimeDirectories().exports,
  },
  {
    key: 'logs',
    name: '日志目录',
    description: 'main 日志和排错信息在这里查看。',
    group: 'diagnostic',
    autoCreate: true,
    resolvePath: () => getRuntimeDirectories().logs,
  },
  {
    key: 'cache',
    name: '缓存目录',
    description: '缩略图、模型测试和其它受控缓存文件放这里。',
    group: 'diagnostic',
    autoCreate: true,
    resolvePath: () => getRuntimeDirectories().cache,
  },
  {
    key: 'temp',
    name: '临时目录',
    description: '运行过程中的临时文件和中间产物放这里。',
    group: 'diagnostic',
    autoCreate: true,
    resolvePath: () => getRuntimeDirectories().temp,
  },
  {
    key: 'models',
    name: '本地模型目录',
    description: '本地 ONNX 模型和后续离线模型文件在这里维护。',
    group: 'advanced',
    autoCreate: true,
    resolvePath: () => getRuntimeDirectories().models,
  },
  {
    key: 'skills',
    name: 'Skill 目录',
    description: 'Skill 文件和运行时技能资源目录。',
    group: 'advanced',
    autoCreate: true,
    resolvePath: () => getRuntimeDirectories().skills,
  },
];

function toDirectoryItem(definition: DirectoryDefinition): FileManagementDirectoryItem {
  const directoryPath = definition.resolvePath();

  return {
    key: definition.key,
    name: definition.name,
    description: definition.description,
    path: directoryPath,
    exists: pathExists(directoryPath),
    group: definition.group,
    autoCreate: definition.autoCreate,
  };
}

function getDirectoryDefinition(key: string): DirectoryDefinition {
  const normalizedKey = key.trim();
  const definition = DIRECTORY_DEFINITIONS.find((item) => item.key === normalizedKey);
  if (!definition) {
    throw createError(VT_STATUS.INVALID_PARAMS, '目录 key 无效');
  }

  return definition;
}

function ensureDirectoryReady(definition: DirectoryDefinition): { path: string; created: boolean } {
  const directoryPath = definition.resolvePath();
  const exists = pathExists(directoryPath);

  if (!exists && !definition.autoCreate) {
    throw createError(VT_STATUS.FILE_NOT_FOUND, '目录不存在');
  }

  if (!exists) {
    ensureDirectory(directoryPath);
  }

  if (!statSync(directoryPath).isDirectory()) {
    throw createError(VT_STATUS.FILE_ERROR, '目标路径不是目录');
  }

  return {
    path: directoryPath,
    created: !exists,
  };
}

export function listOpenableDirectories(): FileManagementListResult {
  return {
    directories: DIRECTORY_DEFINITIONS.map(toDirectoryItem),
  };
}

export async function openDirectory(payload: FileManagementOpenPayload): Promise<FileManagementOpenResult> {
  const definition = getDirectoryDefinition(payload.key);
  const { created } = ensureDirectoryReady(definition);
  const directory = toDirectoryItem(definition);
  const openResult = await shell.openPath(directory.path);

  if (openResult) {
    throw createError(VT_STATUS.FILE_ERROR, `打开目录失败：${openResult}`);
  }

  return {
    directory,
    created,
  };
}
