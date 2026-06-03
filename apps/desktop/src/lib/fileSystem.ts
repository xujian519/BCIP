import { api } from '@/api';
import type { FileEntry as ApiFileEntry } from '@/api/types';

export interface FileEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  size: number;
  modifiedAt: number;
  createdAt: number;
}

function adaptEntry(e: ApiFileEntry): FileEntry {
  return {
    name: e.name,
    path: e.path,
    isDirectory: e.isDirectory,
    size: e.size,
    modifiedAt: e.modifiedAt,
    createdAt: e.modifiedAt,
  };
}

export const isTauri = () => api.isTauri();

export async function readDirectory(path: string): Promise<FileEntry[]> {
  const entries = await api.readDir(path);
  return entries.map(adaptEntry);
}

export async function readFile(path: string): Promise<string> {
  return api.readFile(path);
}

export async function readFileBinary(path: string): Promise<Uint8Array> {
  const data = await api.readFileBinary(path);
  return new Uint8Array(data);
}

export async function writeFile(path: string, content: string): Promise<void> {
  return api.writeFile({ path, content });
}

export async function writeFileBinary(path: string, content: Uint8Array): Promise<void> {
  return api.writeFileBinary({ path, content: Array.from(content) });
}

export async function createDirectory(path: string): Promise<void> {
  return api.createDir(path);
}

export async function deleteFile(path: string): Promise<void> {
  return api.deleteItem(path);
}

export async function getFileInfo(path: string): Promise<{
  name: string;
  extension: string | null;
  size: number;
  isDirectory: boolean;
}> {
  const info = await api.getFileInfo(path);
  return {
    name: info.name,
    extension: info.extension,
    size: info.size,
    isDirectory: info.isDirectory,
  };
}
