export type ExportStatus = 'running' | 'succeeded' | 'failed'
export type ExportKind = 'final_video' | 'project_package'

export interface ExportRecordDto {
  exportId: string
  projectId: string
  compositionTaskId: string | null
  exportKind: ExportKind
  sourceRelativePath: string | null
  targetRelativePath: string | null
  status: ExportStatus
  startedAt: string
  finishedAt: string | null
  errorJson: Record<string, unknown> | null
  metadataJson: Record<string, unknown>
  createdAt: string
  updatedAt: string
}

export interface ExportFinalVideoRequest {
  projectId: string
  overwrite?: boolean
}

export interface ExportProjectPackageRequest {
  projectId: string
  overwrite?: boolean
}

export interface ImportProjectPackageRequest {
  packageRelativePath: string
}

export interface ImportProjectPackageDto {
  projectId: string
  sourceProjectId: string
  title: string
  importedAssetCount: number
}

export interface BackupWorkspaceRequest {
  overwrite?: boolean
}

export interface BackupWorkspaceDto {
  backupId: string
  targetRelativePath: string
  projectCount: number
  assetCount: number
  containsSecrets: boolean
  requiresSecretReentry: boolean
}

export interface RestoreWorkspaceRequest {
  backupRelativePath: string
}

export interface RestoredProjectDto {
  projectId: string
  sourceProjectId: string
  title: string
}

export interface RestoreWorkspaceDto {
  backupId: string
  restoredProjects: RestoredProjectDto[]
  restoredAssetCount: number
  restoredTemplateFileCount: number
  requiresSecretReentry: boolean
}

export interface ExportDiagnosticPackageRequest {
  includeMedia?: boolean
}

export interface ExportDiagnosticPackageDto {
  diagnosticId: string
  targetRelativePath: string
  containsSecrets: boolean
  includesMedia: boolean
  logFileCount: number
}

export interface ListExportRecordsRequest {
  projectId: string
}

export interface OpenExportDirectoryRequest {
  exportId: string
}

export interface OpenExportDirectoryDto {
  exportId: string
  directoryRelativePath: string
}
