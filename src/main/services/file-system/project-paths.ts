import { join } from 'node:path';
import { getRuntimeDirectories } from './paths';
import { safeJoin } from './safe-path';

export const PROJECT_ASSET_TYPES = ['images', 'audio', 'video', 'clips', 'documents'] as const;
export const PROJECT_GENERATED_TYPES = ['images', 'audio', 'video', 'storyboard'] as const;

export type ProjectAssetType = (typeof PROJECT_ASSET_TYPES)[number];
export type ProjectGeneratedType = (typeof PROJECT_GENERATED_TYPES)[number];

function assertKnownValue<T extends string>(value: string, allowedValues: readonly T[], label: string): asserts value is T {
  if (!allowedValues.includes(value as T)) {
    throw new Error(`Unknown ${label}: ${value}`);
  }
}

export function resolveProjectRoot(projectId: string): string {
  return safeJoin(getRuntimeDirectories().projects, projectId);
}

export function resolveProjectSourcePath(projectId: string, relativePath: string): string {
  return safeJoin(join(resolveProjectRoot(projectId), 'source'), relativePath);
}

export function resolveProjectAssetPath(projectId: string, assetType: string, relativePath: string): string {
  assertKnownValue(assetType, PROJECT_ASSET_TYPES, 'project asset type');

  return safeJoin(join(resolveProjectRoot(projectId), 'assets', assetType), relativePath);
}

export function resolveProjectGeneratedPath(projectId: string, generatedType: string, relativePath: string): string {
  assertKnownValue(generatedType, PROJECT_GENERATED_TYPES, 'project generated type');

  return safeJoin(join(resolveProjectRoot(projectId), 'generated', generatedType), relativePath);
}

export function resolveProjectTempPath(projectId: string, relativePath: string): string {
  return safeJoin(join(resolveProjectRoot(projectId), 'temp'), relativePath);
}

export function resolveProjectExportPath(projectId: string, relativePath: string): string {
  return safeJoin(join(resolveProjectRoot(projectId), 'exports'), relativePath);
}
