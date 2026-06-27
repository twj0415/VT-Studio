import { callCommand } from '@/shared/api/client'
import { tauriCommands } from '@/shared/api/commands'

import type { BackupWorkspaceDto, BackupWorkspaceRequest, ExportDiagnosticPackageDto, ExportDiagnosticPackageRequest, ExportFinalVideoRequest, ExportProjectPackageRequest, ExportRecordDto, ImportProjectPackageDto, ImportProjectPackageRequest, ListExportRecordsRequest, OpenExportDirectoryDto, OpenExportDirectoryRequest, RestoreWorkspaceDto, RestoreWorkspaceRequest } from './types'

export function exportFinalVideo(request: ExportFinalVideoRequest): Promise<ExportRecordDto> {
  return callCommand<ExportRecordDto, { request: ExportFinalVideoRequest }>(tauriCommands.exportFinalVideo, { request })
}

export function listExportRecords(request: ListExportRecordsRequest): Promise<ExportRecordDto[]> {
  return callCommand<ExportRecordDto[], { request: ListExportRecordsRequest }>(tauriCommands.listExportRecords, { request })
}

export function exportProjectPackage(request: ExportProjectPackageRequest): Promise<ExportRecordDto> {
  return callCommand<ExportRecordDto, { request: ExportProjectPackageRequest }>(tauriCommands.exportProjectPackage, { request })
}

export function openExportDirectory(request: OpenExportDirectoryRequest): Promise<OpenExportDirectoryDto> {
  return callCommand<OpenExportDirectoryDto, { request: OpenExportDirectoryRequest }>(tauriCommands.openExportDirectory, { request })
}

export function importProjectPackage(request: ImportProjectPackageRequest): Promise<ImportProjectPackageDto> {
  return callCommand<ImportProjectPackageDto, { request: ImportProjectPackageRequest }>(tauriCommands.importProjectPackage, { request })
}

export function backupWorkspace(request: BackupWorkspaceRequest = {}): Promise<BackupWorkspaceDto> {
  return callCommand<BackupWorkspaceDto, { request: BackupWorkspaceRequest }>(tauriCommands.backupWorkspace, { request })
}

export function restoreWorkspace(request: RestoreWorkspaceRequest): Promise<RestoreWorkspaceDto> {
  return callCommand<RestoreWorkspaceDto, { request: RestoreWorkspaceRequest }>(tauriCommands.restoreWorkspace, { request })
}

export function exportDiagnosticPackage(request: ExportDiagnosticPackageRequest = {}): Promise<ExportDiagnosticPackageDto> {
  return callCommand<ExportDiagnosticPackageDto, { request: ExportDiagnosticPackageRequest }>(tauriCommands.exportDiagnosticPackage, { request })
}
