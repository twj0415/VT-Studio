export type FileManagementDirectoryGroup = 'common' | 'diagnostic' | 'advanced';

export interface FileManagementDirectoryItem {
  key: string;
  name: string;
  description: string;
  path: string;
  exists: boolean;
  group: FileManagementDirectoryGroup;
  autoCreate: boolean;
}

export interface FileManagementListResult {
  directories: FileManagementDirectoryItem[];
}

export interface FileManagementOpenPayload {
  key: string;
}

export interface FileManagementOpenResult {
  directory: FileManagementDirectoryItem;
  created: boolean;
}
