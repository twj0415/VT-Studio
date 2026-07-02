import { mkdirSync } from 'node:fs';
import { getRuntimeDirectories, RuntimeDirectories } from './paths';

export function initializeFileSystem(): RuntimeDirectories {
  const directories = getRuntimeDirectories();

  mkdirSync(directories.projects, { recursive: true });
  mkdirSync(directories.cache, { recursive: true });
  mkdirSync(directories.thumbnails, { recursive: true });
  mkdirSync(directories.modelTest, { recursive: true });
  mkdirSync(directories.models, { recursive: true });
  mkdirSync(directories.skills, { recursive: true });
  mkdirSync(directories.vendors, { recursive: true });
  mkdirSync(directories.temp, { recursive: true });
  mkdirSync(directories.exports, { recursive: true });
  mkdirSync(directories.logs, { recursive: true });

  return directories;
}
