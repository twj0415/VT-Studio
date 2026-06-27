import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'
import { getApiAdapter } from '@/shared/api/invoke'

import type { BackupWorkspaceDto, BackupWorkspaceRequest, ExportDiagnosticPackageDto, ExportDiagnosticPackageRequest, ExportFinalVideoRequest, ExportProjectPackageRequest, ExportRecordDto, ImportProjectPackageDto, ImportProjectPackageRequest, ListExportRecordsRequest, OpenExportDirectoryDto, OpenExportDirectoryRequest, RestoreWorkspaceDto, RestoreWorkspaceRequest } from './types'

const MOCK_NOW = '2026-06-22 10:00'
let exportRecords: ExportRecordDto[] = []

export async function exportFinalVideo(request: ExportFinalVideoRequest): Promise<ExportRecordDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ExportRecordDto, { request: ExportFinalVideoRequest }>(tauriCommands.exportFinalVideo, { request })
  }

  const index = exportRecords.filter((record) => record.projectId === request.projectId).length
  const suffix = index === 0 || request.overwrite ? '' : `_${String(index).padStart(2, '0')}`
  const record: ExportRecordDto = {
    exportId: `mock_export_${Date.now()}`,
    projectId: request.projectId,
    compositionTaskId: 'mock_composition_task',
    exportKind: 'final_video',
    sourceRelativePath: `outputs/exports/${request.projectId}/final.mp4`,
    targetRelativePath: `outputs/user_exports/${request.projectId}/final${suffix}.mp4`,
    status: 'succeeded',
    startedAt: MOCK_NOW,
    finishedAt: MOCK_NOW,
    errorJson: null,
    metadataJson: {
      mock: true,
      overwrite: Boolean(request.overwrite),
      controlledDirectory: 'outputs/user_exports',
    },
    createdAt: MOCK_NOW,
    updatedAt: MOCK_NOW,
  }
  exportRecords = [record, ...exportRecords]
  return record
}

export async function listExportRecords(request: ListExportRecordsRequest): Promise<ExportRecordDto[]> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ExportRecordDto[], { request: ListExportRecordsRequest }>(tauriCommands.listExportRecords, { request })
  }

  return exportRecords.filter((record) => record.projectId === request.projectId)
}

export async function exportProjectPackage(request: ExportProjectPackageRequest): Promise<ExportRecordDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ExportRecordDto, { request: ExportProjectPackageRequest }>(tauriCommands.exportProjectPackage, { request })
  }

  const index = exportRecords.filter((record) => record.projectId === request.projectId && record.exportKind === 'project_package').length
  const suffix = index === 0 || request.overwrite ? '' : `_${String(index).padStart(2, '0')}`
  const record: ExportRecordDto = {
    exportId: `mock_export_package_${Date.now()}`,
    projectId: request.projectId,
    compositionTaskId: null,
    exportKind: 'project_package',
    sourceRelativePath: null,
    targetRelativePath: `outputs/project_packages/${request.projectId}/${request.projectId}_project_package${suffix}.zip`,
    status: 'succeeded',
    startedAt: MOCK_NOW,
    finishedAt: MOCK_NOW,
    errorJson: null,
    metadataJson: {
      mock: true,
      packageVersion: 1,
      containsSecrets: false,
      controlledDirectory: 'outputs/project_packages',
    },
    createdAt: MOCK_NOW,
    updatedAt: MOCK_NOW,
  }
  exportRecords = [record, ...exportRecords]
  return record
}

export async function openExportDirectory(request: OpenExportDirectoryRequest): Promise<OpenExportDirectoryDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<OpenExportDirectoryDto, { request: OpenExportDirectoryRequest }>(tauriCommands.openExportDirectory, { request })
  }

  const record = exportRecords.find((entry) => entry.exportId === request.exportId)
  if (!record?.targetRelativePath) throw new Error(`Export record not found: ${request.exportId}`)
  return {
    exportId: request.exportId,
    directoryRelativePath: record.targetRelativePath.split('/').slice(0, -1).join('/'),
  }
}

export async function importProjectPackage(request: ImportProjectPackageRequest): Promise<ImportProjectPackageDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ImportProjectPackageDto, { request: ImportProjectPackageRequest }>(tauriCommands.importProjectPackage, { request })
  }

  return {
    projectId: `mock_imported_${Date.now()}`,
    sourceProjectId: request.packageRelativePath.split('/').at(-2) ?? 'mock_source',
    title: 'Imported project',
    importedAssetCount: 0,
  }
}

export async function backupWorkspace(request: BackupWorkspaceRequest = {}): Promise<BackupWorkspaceDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<BackupWorkspaceDto, { request: BackupWorkspaceRequest }>(tauriCommands.backupWorkspace, { request })
  }

  return {
    backupId: `mock_backup_${Date.now()}`,
    targetRelativePath: `outputs/backups/mock_workspace_backup_${Date.now()}.backup.zip`,
    projectCount: 1,
    assetCount: 0,
    containsSecrets: false,
    requiresSecretReentry: true,
  }
}

export async function restoreWorkspace(request: RestoreWorkspaceRequest): Promise<RestoreWorkspaceDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<RestoreWorkspaceDto, { request: RestoreWorkspaceRequest }>(tauriCommands.restoreWorkspace, { request })
  }

  return {
    backupId: request.backupRelativePath.split('/').at(-1) ?? 'mock_backup',
    restoredProjects: [],
    restoredAssetCount: 0,
    restoredTemplateFileCount: 0,
    requiresSecretReentry: true,
  }
}

export async function exportDiagnosticPackage(request: ExportDiagnosticPackageRequest = {}): Promise<ExportDiagnosticPackageDto> {
  if (getApiAdapter() === 'tauri') {
    return callCommand<ExportDiagnosticPackageDto, { request: ExportDiagnosticPackageRequest }>(tauriCommands.exportDiagnosticPackage, { request })
  }

  return {
    diagnosticId: `mock_diagnostic_${Date.now()}`,
    targetRelativePath: `outputs/diagnostics/mock_diagnostic_${Date.now()}.diagnostic.zip`,
    containsSecrets: false,
    includesMedia: Boolean(request.includeMedia),
    logFileCount: 0,
  }
}
