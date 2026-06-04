const STORAGE_KEY = 'bcip-recent-project-paths';
const HIDDEN_STORAGE_KEY = 'bcip-hidden-project-paths';
const MAX_RECENT = 20;

export function loadRecentProjectPaths(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((p): p is string => typeof p === 'string' && p.length > 0);
  } catch {
    return [];
  }
}

export function addRecentProjectPath(path: string): void {
  const trimmed = path.trim();
  if (!trimmed) return;
  unhideProjectPath(trimmed);
  const existing = loadRecentProjectPaths().filter((p) => p !== trimmed);
  const next = [trimmed, ...existing].slice(0, MAX_RECENT);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
}

export function loadHiddenProjectPaths(): string[] {
  try {
    const raw = localStorage.getItem(HIDDEN_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((p): p is string => typeof p === 'string' && p.length > 0);
  } catch {
    return [];
  }
}

function persistHiddenProjectPaths(paths: string[]): void {
  localStorage.setItem(HIDDEN_STORAGE_KEY, JSON.stringify(paths));
}

/** 从项目侧栏移除（不删除磁盘目录） */
export function removeProjectFromRail(path: string): void {
  const trimmed = path.trim();
  if (!trimmed) return;
  const recent = loadRecentProjectPaths().filter((p) => p !== trimmed);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(recent));
  const hidden = new Set(loadHiddenProjectPaths());
  hidden.add(trimmed);
  persistHiddenProjectPaths([...hidden]);
}

export function unhideProjectPath(path: string): void {
  const trimmed = path.trim();
  if (!trimmed) return;
  const next = loadHiddenProjectPaths().filter((p) => p !== trimmed);
  persistHiddenProjectPaths(next);
}

export function projectDisplayName(path: string): string {
  return path.split('/').filter(Boolean).pop() ?? path;
}
