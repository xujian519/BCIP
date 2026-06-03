import type { FileEntry, ProjectInfo, AppServerStatus, WriteFileParams, FileInfo } from './types';

const mockFiles: Record<string, FileEntry[]> = {
  '/Users/xujian': [
    { name: 'Documents', path: '/Users/xujian/Documents', isDirectory: true, size: 0, modifiedAt: Date.now(), children: [] },
    { name: 'Downloads', path: '/Users/xujian/Downloads', isDirectory: true, size: 0, modifiedAt: Date.now(), children: [] },
    { name: 'Projects', path: '/Users/xujian/Projects', isDirectory: true, size: 0, modifiedAt: Date.now(), children: [
      { name: 'BCIP', path: '/Users/xujian/Projects/BCIP', isDirectory: true, size: 0, modifiedAt: Date.now(), children: [
        { name: 'README.md', path: '/Users/xujian/Projects/BCIP/README.md', isDirectory: false, size: 1024, modifiedAt: Date.now() },
        { name: 'package.json', path: '/Users/xujian/Projects/BCIP/package.json', isDirectory: false, size: 2048, modifiedAt: Date.now() },
      ]},
    ]},
    { name: '.zshrc', path: '/Users/xujian/.zshrc', isDirectory: false, size: 512, modifiedAt: Date.now() },
    { name: '.gitconfig', path: '/Users/xujian/.gitconfig', isDirectory: false, size: 256, modifiedAt: Date.now() },
  ],
};

export async function readDir(path: string): Promise<FileEntry[]> {
  await delay(50);
  return mockFiles[path] ?? [];
}

export async function readFile(path: string): Promise<string> {
  await delay(50);
  return `// Mock content for ${path}\n\nThis is a placeholder. In the Tauri environment, real file content would be loaded here.`;
}

export async function readFileBinary(_path: string): Promise<number[]> {
  await delay(50);
  return Array.from({ length: 100 }, (_, i) => i);
}

export async function writeFile(_params: WriteFileParams): Promise<void> {
  await delay(30);
}

export async function writeFileBinary(_params: import('./types').WriteFileBinaryParams): Promise<void> {
  await delay(30);
}

export async function createDir(_path: string): Promise<void> {
  await delay(30);
}

export async function deleteItem(_path: string, _recursive?: boolean): Promise<void> {
  await delay(30);
}

export async function fileExists(): Promise<boolean> {
  return true;
}

const mockProjects: ProjectInfo[] = [];

export async function getFileInfo(_path: string): Promise<FileInfo> {
  await delay(30);
  return { name: 'mock.md', extension: 'md', size: 1024, isDirectory: false };
}

export async function getProjects(): Promise<ProjectInfo[]> {
  await delay(50);
  return mockProjects;
}

export async function createProject(path: string): Promise<ProjectInfo> {
  await delay(100);
  return {
    id: `proj-${Date.now()}`,
    name: path.split('/').pop() || '未命名项目',
    path,
    createdAt: Date.now(),
  };
}

export async function getCodexHomeInfo(): Promise<import('./types').CodexHomeInfo> {
  await delay(20);
  return {
    codexHome: '/Users/mock/.bcip',
    configToml: '/Users/mock/.bcip/config.toml',
  };
}

export async function convertDocToDocx(_inputPath: string): Promise<import('./types').DocConvertResult> {
  throw new Error('需要 Tauri 桌面端与 LibreOffice 才能预览 .doc 文件');
}

export async function getLibreOfficeStatus(): Promise<import('./types').LibreOfficeStatus> {
  return { available: false, path: null };
}

export async function checkBcip(): Promise<import('./types').BcipCheckResult> {
  return {
    installed: true,
    version: 'bcip mock',
    path: '/usr/local/bin/bcip',
    source: 'path',
  };
}

export async function revealPathInFileManager(_path: string): Promise<void> {
  await delay(20);
}

export async function startAppServer(): Promise<void> {
  await delay(200);
}

export async function stopAppServer(): Promise<void> {
  await delay(100);
}

export async function getAppServerStatus(): Promise<AppServerStatus> {
  return { connected: false, transport: 'mock', error: null };
}

export async function getAppServerUrl(): Promise<string> {
  return '';
}

export function isTauri(): boolean {
  return false;
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
