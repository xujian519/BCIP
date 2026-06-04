import { useMemo, useCallback, useState } from 'react';
import { FolderOpen, Plus, X } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '@/hooks/useAppStore';
import { useProjects } from '@/hooks/useProjects';
import {
  addRecentProjectPath,
  loadHiddenProjectPaths,
  loadRecentProjectPaths,
  projectDisplayName,
  removeProjectFromRail,
} from '@/lib/recentProjects';
import { cn } from '@/lib/utils';
import ProjectRailContextMenu from './ProjectRailContextMenu';

interface ProjectEntry {
  path: string;
  name: string;
}

function mergeProjectEntries(
  apiPaths: string[],
  recentPaths: string[],
  hiddenPaths: string[],
): ProjectEntry[] {
  const hidden = new Set(hiddenPaths);
  const seen = new Set<string>();
  const entries: ProjectEntry[] = [];
  for (const path of [...recentPaths, ...apiPaths]) {
    if (hidden.has(path) || seen.has(path)) continue;
    seen.add(path);
    entries.push({ path, name: projectDisplayName(path) });
  }
  return entries;
}

export default function ProjectRail() {
  const { state, dispatch } = useAppStore();
  const { projects, createProject } = useProjects();
  const [recentPaths, setRecentPaths] = useState(loadRecentProjectPaths);
  const [hiddenPaths, setHiddenPaths] = useState(loadHiddenProjectPaths);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; path: string } | null>(null);

  const entries = useMemo(
    () => mergeProjectEntries(
      projects.map((p) => p.path),
      recentPaths,
      hiddenPaths,
    ),
    [projects, recentPaths, hiddenPaths],
  );

  const refreshProjectLists = useCallback(() => {
    setRecentPaths(loadRecentProjectPaths());
    setHiddenPaths(loadHiddenProjectPaths());
  }, []);

  const handleSelect = useCallback(
    (path: string) => {
      addRecentProjectPath(path);
      if (state.workspaceCwd !== path) {
        dispatch({ type: 'SWITCH_PROJECT', payload: path });
      }
    },
    [dispatch, state.workspaceCwd],
  );

  const handleAdd = useCallback(async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '添加项目目录',
      });
      if (selected && typeof selected === 'string') {
        addRecentProjectPath(selected);
        refreshProjectLists();
        dispatch({ type: 'SWITCH_PROJECT', payload: selected });
      }
    } catch {
      // user cancelled
    }
  }, [dispatch, refreshProjectLists]);

  const handleCreate = useCallback(async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '新建项目目录',
      });
      if (selected && typeof selected === 'string') {
        await createProject(selected);
        addRecentProjectPath(selected);
        refreshProjectLists();
        dispatch({ type: 'SWITCH_PROJECT', payload: selected });
      }
    } catch {
      // user cancelled
    }
  }, [createProject, dispatch, refreshProjectLists]);

  const handleRemove = useCallback(
    (path: string) => {
      const remaining = entries.filter((e) => e.path !== path);
      removeProjectFromRail(path);
      refreshProjectLists();
      if (state.workspaceCwd !== path) {
        return;
      }
      const next = remaining[0]?.path;
      if (next) {
        dispatch({ type: 'SWITCH_PROJECT', payload: next });
      } else {
        dispatch({ type: 'SET_WORKSPACE_CWD', payload: null });
      }
    },
    [dispatch, entries, refreshProjectLists, state.workspaceCwd],
  );

  return (
    <div
      className="flex h-full shrink-0 flex-col border-r"
      style={{
        width: 168,
        borderColor: 'var(--border-primary)',
        backgroundColor: 'var(--bg-sidebar)',
      }}
    >
      <div
        className="flex items-center justify-between px-2 py-2"
        style={{ borderBottom: '1px solid var(--border-primary)' }}
      >
        <span className="text-[11px] font-medium" style={{ color: 'var(--text-secondary)' }}>
          项目
        </span>
        <div className="flex items-center gap-0.5">
          <button
            type="button"
            onClick={() => void handleAdd()}
            className="flex h-6 w-6 items-center justify-center rounded transition-colors hover:bg-[var(--bg-hover)]"
            style={{ color: 'var(--text-tertiary)' }}
            title="添加目录"
          >
            <FolderOpen size={13} />
          </button>
          <button
            type="button"
            onClick={() => void handleCreate()}
            className="flex h-6 w-6 items-center justify-center rounded transition-colors hover:bg-[var(--bg-hover)]"
            style={{ color: 'var(--text-tertiary)' }}
            title="新建项目"
          >
            <Plus size={13} />
          </button>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto custom-scrollbar">
        {entries.length === 0 && (
          <div className="p-3 text-center text-[10px]" style={{ color: 'var(--text-tertiary)' }}>
            点击 + 添加项目
          </div>
        )}
        {entries.map((entry) => {
          const isActive = state.workspaceCwd === entry.path;
          return (
            <div
              key={entry.path}
              className={cn(
                'group flex w-full items-center gap-0.5 pr-1',
                isActive ? 'bg-[var(--bg-sidebar-active)]' : 'hover:bg-[var(--bg-hover)]',
              )}
            >
              <button
                type="button"
                onClick={() => handleSelect(entry.path)}
                onContextMenu={(e) => {
                  e.preventDefault();
                  setContextMenu({ x: e.clientX, y: e.clientY, path: entry.path });
                }}
                className="flex min-w-0 flex-1 items-center gap-1.5 px-2 py-1.5 text-left"
                title={entry.path}
              >
                <FolderOpen
                  size={12}
                  className="shrink-0"
                  style={{ color: isActive ? 'var(--accent-primary)' : 'var(--text-tertiary)' }}
                />
                <span
                  className="truncate text-[11px] font-medium"
                  style={{ color: isActive ? 'var(--text-primary)' : 'var(--text-secondary)' }}
                >
                  {entry.name}
                </span>
              </button>
              <button
                type="button"
                onClick={() => handleRemove(entry.path)}
                className="flex h-5 w-5 shrink-0 items-center justify-center rounded opacity-0 transition-opacity group-hover:opacity-100 hover:bg-[var(--bg-hover)]"
                style={{ color: 'var(--text-tertiary)' }}
                title="从列表移除"
                aria-label={`从列表移除 ${entry.name}`}
              >
                <X size={11} />
              </button>
            </div>
          );
        })}
      </div>
      {contextMenu && (
        <ProjectRailContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          projectName={projectDisplayName(contextMenu.path)}
          onRemove={() => handleRemove(contextMenu.path)}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  );
}
