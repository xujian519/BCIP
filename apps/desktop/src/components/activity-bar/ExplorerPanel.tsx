import { useCallback } from 'react';
import { ChevronLeft, FolderOpen, PanelLeftClose, PanelLeftOpen, Plus } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '@/hooks/useAppStore';
import { useFileSystem } from '@/hooks/useFileSystem';
import { useProjects } from '@/hooks/useProjects';
import { addRecentProjectPath } from '@/lib/recentProjects';
import { tabIdForPath } from '@/lib/workspaceLayout';
import FileTree from '@/components/sidebar/FileTree';
import ProjectRail from './ProjectRail';

export default function ExplorerPanel() {
  const { state, dispatch } = useAppStore();
  const { projects, createProject } = useProjects();
  const defaultRoot = state.workspaceCwd ?? projects[0]?.path ?? '';
  const {
    files,
    loading: filesLoading,
    selectedPath,
    expandedPaths,
    selectFile,
    toggleExpanded,
    loadChildren,
  } = useFileSystem(defaultRoot);

  const handleSelect = (path: string) => {
    selectFile(path);
    dispatch({ type: 'SET_CURRENT_FILE', payload: path });
    const fileName = path.split('/').pop() ?? path;
    dispatch({
      type: 'OPEN_TAB',
      payload: {
        id: tabIdForPath(path),
        filePath: path,
        title: fileName,
      },
    });
  };

  const handleOpenDirectory = useCallback(async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择工作区目录',
      });
      if (selected && typeof selected === 'string') {
        addRecentProjectPath(selected);
        await createProject(selected).catch(() => null);
        dispatch({ type: 'SWITCH_PROJECT', payload: selected });
      }
    } catch {
      // user cancelled
    }
  }, [createProject, dispatch]);

  const workspaceName = defaultRoot
    ? defaultRoot.split('/').filter(Boolean).pop() ?? defaultRoot
    : null;

  return (
    <div className="flex h-full flex-col">
      <div
        className="flex items-center justify-between px-3 py-2"
        style={{ borderBottom: '1px solid var(--border-primary)' }}
      >
        <span className="text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
          资源管理器
        </span>
        <div className="flex items-center gap-0.5">
          <button
            type="button"
            onClick={() => dispatch({ type: 'TOGGLE_PROJECT_RAIL' })}
            className="flex h-6 w-6 items-center justify-center rounded transition-colors hover:bg-[var(--bg-hover)]"
            style={{ color: 'var(--text-tertiary)' }}
            title={state.projectRailOpen ? '收起项目栏' : '展开项目栏'}
          >
            {state.projectRailOpen ? <PanelLeftClose size={14} /> : <PanelLeftOpen size={14} />}
          </button>
          <button
            type="button"
            onClick={() => void handleOpenDirectory()}
            className="flex h-6 w-6 items-center justify-center rounded transition-colors hover:bg-[var(--bg-hover)]"
            style={{ color: 'var(--text-tertiary)' }}
            title="打开文件夹"
          >
            <Plus size={14} />
          </button>
          <button
            type="button"
            onClick={() => dispatch({ type: 'SET_ACTIVITY_BAR_TAB', payload: null })}
            className="flex h-6 w-6 items-center justify-center rounded transition-colors hover:bg-[var(--bg-hover)]"
            style={{ color: 'var(--text-tertiary)' }}
            title="向左折叠"
          >
            <ChevronLeft size={14} />
          </button>
        </div>
      </div>

      <div className="flex min-h-0 flex-1">
        {state.projectRailOpen && <ProjectRail />}

        <div className="flex min-w-0 flex-1 flex-col">
          {workspaceName && (
            <div
              className="truncate px-3 py-1 font-mono text-[11px]"
              style={{ color: 'var(--text-tertiary)' }}
              title={defaultRoot}
            >
              {workspaceName}
            </div>
          )}

          <div className="flex-1 overflow-auto">
            {defaultRoot ? (
              <>
                {filesLoading && files.length === 0 && (
                  <div className="px-3 py-2 text-[11px]" style={{ color: 'var(--text-tertiary)' }}>
                    加载文件树…
                  </div>
                )}
                <FileTree
                rootPath={defaultRoot}
                entries={files}
                selectedPath={selectedPath}
                expandedPaths={expandedPaths}
                onSelect={handleSelect}
                onToggleExpand={toggleExpanded}
                onLoadChildren={loadChildren}
              />
              </>
            ) : (
              <div className="flex flex-col items-center justify-center gap-3 p-4">
                <FolderOpen size={28} style={{ color: 'var(--text-tertiary)', opacity: 0.5 }} />
                <span className="text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
                  从左侧项目栏选择或添加项目
                </span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
