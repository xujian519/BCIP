import type { FileEntry } from '@/lib/fileSystem';

const TTL_MS = 60_000;
const cache = new Map<string, { entries: FileEntry[]; at: number }>();

export function getCachedDirectory(path: string): FileEntry[] | null {
  const hit = cache.get(path);
  if (!hit) return null;
  if (Date.now() - hit.at > TTL_MS) {
    cache.delete(path);
    return null;
  }
  return hit.entries;
}

export function setCachedDirectory(path: string, entries: FileEntry[]): void {
  cache.set(path, { entries, at: Date.now() });
}

export function invalidateDirectoryTree(rootPath: string): void {
  for (const key of cache.keys()) {
    if (key === rootPath || key.startsWith(`${rootPath}/`)) {
      cache.delete(key);
    }
  }
}
