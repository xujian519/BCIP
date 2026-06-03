const STORAGE_KEY = 'bcip-recent-project-paths';
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
  const existing = loadRecentProjectPaths().filter((p) => p !== trimmed);
  const next = [trimmed, ...existing].slice(0, MAX_RECENT);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
}

export function projectDisplayName(path: string): string {
  return path.split('/').filter(Boolean).pop() ?? path;
}
