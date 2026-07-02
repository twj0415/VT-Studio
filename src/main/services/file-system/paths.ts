import { join } from 'node:path';
import { getUserDataPath } from '../../app/runtime';

export const FILE_SYSTEM_DIRECTORY_NAMES = {
  projects: 'projects',
  cache: 'cache',
  thumbnails: 'thumbnails',
  modelTest: 'model-test',
  models: 'models',
  skills: 'skills',
  vendors: 'vendors',
  temp: 'temp',
  exports: 'exports',
  logs: 'logs',
} as const;

export interface RuntimeDirectories {
  userData: string;
  projects: string;
  cache: string;
  thumbnails: string;
  modelTest: string;
  models: string;
  skills: string;
  vendors: string;
  temp: string;
  exports: string;
  logs: string;
}

export function getUserDataRoot(): string {
  return getUserDataPath();
}

export function getRuntimeDirectories(): RuntimeDirectories {
  const userData = getUserDataRoot();
  const cache = join(userData, FILE_SYSTEM_DIRECTORY_NAMES.cache);

  return {
    userData,
    projects: join(userData, FILE_SYSTEM_DIRECTORY_NAMES.projects),
    cache,
    thumbnails: join(cache, FILE_SYSTEM_DIRECTORY_NAMES.thumbnails),
    modelTest: join(cache, FILE_SYSTEM_DIRECTORY_NAMES.modelTest),
    models: join(userData, FILE_SYSTEM_DIRECTORY_NAMES.models),
    skills: join(userData, FILE_SYSTEM_DIRECTORY_NAMES.skills),
    vendors: join(userData, FILE_SYSTEM_DIRECTORY_NAMES.vendors),
    temp: join(userData, FILE_SYSTEM_DIRECTORY_NAMES.temp),
    exports: join(userData, FILE_SYSTEM_DIRECTORY_NAMES.exports),
    logs: join(userData, FILE_SYSTEM_DIRECTORY_NAMES.logs),
  };
}
