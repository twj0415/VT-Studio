export interface DatabaseManagementInfo {
  directory: string;
  filePath: string;
  exists: boolean;
  sizeBytes: number;
  tableCount: number;
  migrationCount: number;
  sqliteVersion: string;
  backupCount: number;
  latestBackupName: string | null;
  runningTaskCount: number;
}

export interface DatabaseBackupItem {
  name: string;
  sizeBytes: number;
  createdAt: number;
}

export interface DatabaseTableInfo {
  name: string;
  rowCount: number;
  protected: boolean;
  module: string;
}

export interface DatabaseManagementInfoResult {
  info: DatabaseManagementInfo;
}

export interface DatabaseBackupListResult {
  backups: DatabaseBackupItem[];
}

export interface DatabaseExportResult {
  backup: DatabaseBackupItem;
}

export interface DatabaseImportPayload {
  backupName: string;
  confirmText: string;
}

export interface DatabaseImportResult {
  importedBackupName: string;
  autoBackupName: string;
  info: DatabaseManagementInfo;
}

export interface DatabaseTableListResult {
  tables: DatabaseTableInfo[];
}

export interface DatabaseClearTablePayload {
  tableName: string;
  confirmText: string;
}

export interface DatabaseClearTableResult {
  tableName: string;
  deleted: number;
  autoBackupName: string;
  tables: DatabaseTableInfo[];
}

export interface DatabaseClearAllPayload {
  confirmText: string;
}

export interface DatabaseClearAllResult {
  autoBackupName: string;
  info: DatabaseManagementInfo;
  tables: DatabaseTableInfo[];
}

export interface DatabaseRunningTasksResult {
  runningTaskCount: number;
}
