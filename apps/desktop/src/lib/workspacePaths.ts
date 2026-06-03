import type { FileUpdateChange } from '@/generated/app-server/v2/FileUpdateChange';

/** 将 patch 相对路径解析为绝对路径 */
export function resolveWorkspacePaths(
  paths: string[],
  workspaceCwd: string | null,
): string[] {
  return paths.map((p) => {
    if (p.startsWith('/')) {
      return p;
    }
    if (workspaceCwd) {
      return `${workspaceCwd.replace(/\/$/, '')}/${p}`;
    }
    return p;
  });
}

export function pathsFromFileChanges(
  changes: FileUpdateChange[],
  workspaceCwd: string | null,
): string[] {
  return resolveWorkspacePaths(
    changes.map((c) => c.path),
    workspaceCwd,
  );
}

export function pathMatchesAny(target: string, candidates: string[]): boolean {
  return candidates.some(
    (p) => target === p || target.startsWith(`${p}/`) || p.startsWith(`${target}/`),
  );
}
