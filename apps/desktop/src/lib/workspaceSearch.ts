import type { FileEntry } from '@/lib/fileSystem';
import { readDirectory, readFile } from '@/lib/fileSystem';

export interface WorkspaceSearchHit {
  path: string;
  name: string;
  kind: 'filename' | 'content';
  snippet?: string;
}

export interface WorkspaceSearchOptions {
  maxResults?: number;
  maxDepth?: number;
  maxFileBytes?: number;
}

const DEFAULT_MAX_RESULTS = 50;
const DEFAULT_MAX_DEPTH = 8;
const DEFAULT_MAX_FILE_BYTES = 512 * 1024;

const SKIP_DIR_NAMES = new Set([
  '.git',
  'node_modules',
  'target',
  'dist',
  'build',
  '.next',
  '.turbo',
  'coverage',
]);

const TEXT_EXTENSIONS = new Set([
  '.md',
  '.txt',
  '.json',
  '.toml',
  '.yaml',
  '.yml',
  '.ts',
  '.tsx',
  '.js',
  '.jsx',
  '.rs',
  '.css',
  '.html',
  '.xml',
  '.csv',
  '.log',
  '.env',
  '.ini',
  '.cfg',
  '.conf',
]);

function isTextFile(name: string): boolean {
  const dot = name.lastIndexOf('.');
  if (dot < 0) {
    return false;
  }
  return TEXT_EXTENSIONS.has(name.slice(dot).toLowerCase());
}

function normalizeQuery(query: string): string {
  return query.trim().toLowerCase();
}

function snippetAround(content: string, index: number, queryLength: number): string {
  const start = Math.max(0, index - 40);
  const end = Math.min(content.length, index + queryLength + 60);
  const raw = content.slice(start, end).replace(/\s+/g, ' ').trim();
  return start > 0 ? `…${raw}` : raw;
}

export function rankFilenameMatch(name: string, query: string): boolean {
  return name.toLowerCase().includes(query);
}

export async function searchWorkspace(
  root: string,
  rawQuery: string,
  options: WorkspaceSearchOptions = {},
): Promise<WorkspaceSearchHit[]> {
  const query = normalizeQuery(rawQuery);
  if (!root || !query) {
    return [];
  }

  const maxResults = options.maxResults ?? DEFAULT_MAX_RESULTS;
  const maxDepth = options.maxDepth ?? DEFAULT_MAX_DEPTH;
  const maxFileBytes = options.maxFileBytes ?? DEFAULT_MAX_FILE_BYTES;
  const hits: WorkspaceSearchHit[] = [];

  async function walk(dir: string, depth: number): Promise<void> {
    if (hits.length >= maxResults || depth > maxDepth) {
      return;
    }

    let entries: FileEntry[];
    try {
      entries = await readDirectory(dir);
    } catch {
      return;
    }

    for (const entry of entries) {
      if (hits.length >= maxResults) {
        return;
      }

      if (entry.isDirectory) {
        if (SKIP_DIR_NAMES.has(entry.name)) {
          continue;
        }
        await walk(entry.path, depth + 1);
        continue;
      }

      if (rankFilenameMatch(entry.name, query)) {
        hits.push({
          path: entry.path,
          name: entry.name,
          kind: 'filename',
        });
        continue;
      }

      if (!isTextFile(entry.name) || entry.size > maxFileBytes) {
        continue;
      }

      try {
        const content = await readFile(entry.path);
        const lower = content.toLowerCase();
        const index = lower.indexOf(query);
        if (index >= 0) {
          hits.push({
            path: entry.path,
            name: entry.name,
            kind: 'content',
            snippet: snippetAround(content, index, query.length),
          });
        }
      } catch {
        // 跳过不可读文件
      }
    }
  }

  await walk(root, 0);
  return hits;
}
