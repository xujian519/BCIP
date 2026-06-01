import { invoke, isTauri as coreIsTauri } from '@tauri-apps/api/core';
import type {
  FileEntry,
  ProjectInfo,
  AppServerStatus,
  WriteFileParams,
  WriteFileBinaryParams,
  FileInfo,
  CodexHomeInfo,
  BcipCheckResult,
  DocConvertResult,
  LibreOfficeStatus,
} from './types';

export async function readDir(path: string): Promise<FileEntry[]> {
  return invoke<FileEntry[]>('read_dir', { path });
}

export async function readFile(path: string): Promise<string> {
  return invoke<string>('read_file', { path });
}

export async function readFileBinary(path: string): Promise<number[]> {
  return invoke<number[]>('read_file_binary', { path });
}

export async function writeFile(params: WriteFileParams): Promise<void> {
  return invoke('write_file', { params });
}

export async function writeFileBinary(params: WriteFileBinaryParams): Promise<void> {
  return invoke('write_file_binary', { params });
}

export async function createDir(path: string): Promise<void> {
  return invoke('create_dir', { path });
}

export async function deleteItem(path: string, recursive?: boolean): Promise<void> {
  return invoke('delete_file', { path, recursive });
}

export async function fileExists(path: string): Promise<boolean> {
  return invoke<boolean>('file_exists', { path });
}

export async function getFileInfo(path: string): Promise<FileInfo> {
  return invoke<FileInfo>('get_file_info', { path });
}

export async function getProjects(): Promise<ProjectInfo[]> {
  return invoke<ProjectInfo[]>('project_list');
}

export async function createProject(path: string): Promise<ProjectInfo> {
  return invoke<ProjectInfo>('project_create', { path });
}

export async function checkBcip(): Promise<BcipCheckResult> {
  return invoke<BcipCheckResult>('check_bcip_installed');
}

export async function revealPathInFileManager(path: string): Promise<void> {
  return invoke('reveal_path_in_file_manager', { path });
}

export async function startAppServer(): Promise<AppServerStatus> {
  return invoke<AppServerStatus>('start_app_server');
}

export async function stopAppServer(): Promise<void> {
  return invoke('stop_app_server');
}

export async function getAppServerStatus(): Promise<AppServerStatus> {
  return invoke<AppServerStatus>('get_app_server_status');
}

export async function appServerConnect(): Promise<AppServerStatus> {
  return invoke<AppServerStatus>('app_server_connect');
}

export async function appServerDisconnect(): Promise<void> {
  return invoke('app_server_disconnect');
}

export async function appServerSend(line: string): Promise<void> {
  return invoke('app_server_send', { line });
}

/** @deprecated 使用 stdio 传输，无 WebSocket URL */
export async function getAppServerUrl(): Promise<string> {
  return invoke<string>('get_app_server_url');
}

export async function getCodexHomeInfo(): Promise<CodexHomeInfo> {
  return invoke<CodexHomeInfo>('get_codex_home_info');
}

export async function convertDocToDocx(inputPath: string): Promise<DocConvertResult> {
  return invoke<DocConvertResult>('convert_doc_to_docx', { inputPath });
}

export async function getLibreOfficeStatus(): Promise<LibreOfficeStatus> {
  return invoke<LibreOfficeStatus>('libreoffice_status');
}

/** Tauri 2 默认不注入 __TAURI__，优先用官方 API 检测 */
export function isTauri(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }
  try {
    if (coreIsTauri()) {
      return true;
    }
  } catch {
    // 忽略
  }
  const w = window as unknown as Record<string, unknown>;
  return '__TAURI_INTERNALS__' in w || '__TAURI__' in w;
}
