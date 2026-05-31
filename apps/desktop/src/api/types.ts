export interface FileEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  size: number;
  modifiedAt: number;
  children?: FileEntry[];
}

export interface ProjectInfo {
  id: string;
  name: string;
  path: string;
  createdAt: number;
}

export interface BcipStatus {
  running: boolean;
  websocketUrl: string | null;
  pid: number | null;
}

export interface AppServerStatus {
  connected: boolean;
  transport: string;
  error?: string | null;
}

export interface CodexHomeInfo {
  codexHome: string;
  configToml: string;
}

export interface BcipCheckResult {
  installed: boolean;
  version: string | null;
  path: string | null;
  source?: 'path' | 'sidecar' | 'workspace' | null;
}

export interface FileInfo {
  name: string;
  extension: string | null;
  size: number;
  isDirectory: boolean;
}

export interface WriteFileParams {
  path: string;
  content: string;
}

export interface WriteFileBinaryParams {
  path: string;
  content: number[];
}

