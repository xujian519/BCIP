import type { FC } from 'react';
import { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  ChevronLeft,
  ChevronRight,
  FolderOpen,
  Plus,
  HelpCircle,
  Search as SearchIcon,
  X,
} from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import FileTree from './FileTree';
import { useFileSystem } from '@/hooks/useFileSystem';
import { useProjects } from '@/hooks/useProjects';
import { BCIP_FILE_TREE_REFRESH } from '@/lib/desktopEvents';

interface LeftSidebarProps {
  isExpanded: boolean;
  onToggleExpand: () => void;
  width: number;
  onSelectFile: (path: string) => void;
  onWorkspaceCwd?: (cwd: string) => void;
  workspaceCwd?: string | null;
}

function resolveWorkspaceCwd(
  filePath: string,
  projects: { path: string }[],
): string {
  const match = projects.find(
    (p) => filePath === p.path || filePath.startsWith(`${p.path}/`),
  );
  if (match) {
    return match.path;
  }
  const parts = filePath.split('/');
  parts.pop();
  return parts.join('/') || filePath;
}

const LeftSidebar: FC<LeftSidebarProps> = ({
  isExpanded,
  onToggleExpand,
  onSelectFile,
  onWorkspaceCwd,
  workspaceCwd,
}) => {
  const [searchQuery, setSearchQuery] = useState('');

  const {
    projects,
    createProject,
  } = useProjects();

  const defaultRoot = workspaceCwd ?? projects[0]?.path ?? '';

  const {
    files,
    loading,
    error,
    selectedPath,
    expandedPaths,
    selectFile,
    toggleExpanded,
    loadChildren,
    refresh,
    setRootPath,
  } = useFileSystem(defaultRoot);

  useEffect(() => {
    if (workspaceCwd) {
      setRootPath(workspaceCwd);
    }
  }, [workspaceCwd, setRootPath]);

  useEffect(() => {
    const handler = () => {
      void refresh();
    };
    window.addEventListener(BCIP_FILE_TREE_REFRESH, handler);
    return () => window.removeEventListener(BCIP_FILE_TREE_REFRESH, handler);
  }, [refresh]);

  const initialCwdSet = useRef(false);
  useEffect(() => {
    if (initialCwdSet.current || !onWorkspaceCwd || projects.length === 0) {
      return;
    }
    initialCwdSet.current = true;
    onWorkspaceCwd(projects[0].path);
  }, [projects, onWorkspaceCwd]);

  const handleSelectFile = (path: string) => {
    selectFile(path);
    onSelectFile(path);
    if (onWorkspaceCwd) {
      onWorkspaceCwd(resolveWorkspaceCwd(path, projects));
    }
  };

  const handleOpenDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择工作区目录',
      });
      if (selected && typeof selected === 'string') {
        await createProject(selected);
        if (onWorkspaceCwd) {
          onWorkspaceCwd(selected);
        }
      }
    } catch {
      // user cancelled
    }
  };

  const hasWorkspace = !!defaultRoot;

  return (
    <div
      className="flex h-full flex-col glass"
      style={{
        backgroundColor: 'var(--bg-sidebar)',
        borderRight: '1px solid var(--border-primary)',
      }}
    >
      {/* Toggle Button */}
      <div className="flex items-center justify-center" style={{ padding: '8px 0 4px' }}>
        <button
          onClick={onToggleExpand}
          className="flex items-center justify-center transition-colors duration-150"
          style={{
            width: 28,
            height: 28,
            borderRadius: 6,
            color: 'var(--text-tertiary)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-tertiary)';
          }}
          type="button"
          aria-label={isExpanded ? '折叠侧边栏' : '展开侧边栏'}
        >
          {isExpanded ? <ChevronLeft size={16} /> : <ChevronRight size={16} />}
        </button>
      </div>

      {/* Search & workspace info (expanded only) */}
      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.2 }}
            style={{ padding: '4px 12px 8px' }}
          >
            {workspaceCwd && (
              <p
                className="mb-2 truncate font-mono text-[10px]"
                style={{ color: 'var(--text-tertiary)' }}
                title={workspaceCwd}
              >
                {workspaceCwd.split('/').filter(Boolean).pop() ?? workspaceCwd}
              </p>
            )}
            {hasWorkspace && (
              <div className="relative">
                <SearchIcon
                  size={14}
                  className="pointer-events-none absolute"
                  style={{ left: 10, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-tertiary)' }}
                />
                <input
                  type="text"
                  placeholder="搜索文件..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full transition-all duration-200 focus:outline-none"
                  style={{
                    height: 30,
                    padding: '6px 10px 6px 32px',
                    fontSize: 12,
                    borderRadius: 8,
                    backgroundColor: 'var(--bg-elevated)',
                    border: '1px solid var(--border-primary)',
                    color: 'var(--text-primary)',
                    transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
                  }}
                  onFocus={(e) => {
                    e.currentTarget.style.borderColor = 'var(--border-focus)';
                    e.currentTarget.style.boxShadow = '0 0 0 3px var(--accent-primary-muted)';
                  }}
                  onBlur={(e) => {
                    e.currentTarget.style.borderColor = 'var(--border-primary)';
                    e.currentTarget.style.boxShadow = 'none';
                  }}
                />
                {searchQuery && (
                  <button
                    onClick={() => setSearchQuery('')}
                    className="absolute"
                    style={{ right: 8, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-tertiary)' }}
                    type="button"
                  >
                    <X size={12} />
                  </button>
                )}
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        {!hasWorkspace ? (
          <div
            className="flex flex-col items-center justify-center h-full gap-3 px-4"
            style={{ color: 'var(--text-tertiary)' }}
          >
            <FolderOpen size={28} style={{ opacity: 0.5 }} />
            <span className="text-xs text-center">
              选择一个目录作为工作区
            </span>
          <button
            onClick={handleOpenDirectory}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200"
            style={{
              backgroundColor: 'var(--accent-primary)',
              color: 'var(--text-inverse)',
              transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
            }}
            type="button"
          >
            <Plus size={14} />
            打开目录
          </button>
          </div>
        ) : (
          <>
            {loading && (
              <div className="p-4 text-center" style={{ color: 'var(--text-tertiary)', fontSize: 12 }}>
                加载中...
              </div>
            )}
            {error && (
              <div className="p-4 text-center" style={{ color: 'var(--status-error)', fontSize: 12 }}>
                {error}
              </div>
            )}
            {!loading && !error && (
              <FileTree
                rootPath={defaultRoot}
                entries={files}
                selectedPath={selectedPath}
                expandedPaths={expandedPaths}
                onSelect={handleSelectFile}
                onToggleExpand={toggleExpanded}
                onLoadChildren={loadChildren}
              />
            )}
          </>
        )}
      </div>

      {/* Bottom Actions */}
      <div
        className="flex items-center"
        style={{
          padding: isExpanded ? '8px 12px' : '8px 0',
          borderTop: '1px solid var(--border-primary)',
          justifyContent: isExpanded ? 'space-between' : 'center',
          gap: 4,
        }}
      >
        <button
          onClick={handleOpenDirectory}
          className="flex items-center justify-center transition-all duration-150"
          style={{
            width: isExpanded ? 'auto' : 32,
            height: 32,
            padding: isExpanded ? '0 12px' : '0',
            borderRadius: 9999,
            backgroundColor: 'var(--accent-primary)',
            color: 'var(--text-inverse)',
            gap: isExpanded ? 6 : 0,
            fontSize: 11,
            fontWeight: 500,
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = 'scale(1.05)';
            e.currentTarget.style.boxShadow = '0 4px 12px rgba(74, 124, 111, 0.3)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = 'scale(1)';
            e.currentTarget.style.boxShadow = 'none';
          }}
          type="button"
        >
          <Plus size={16} />
          <AnimatePresence>
            {isExpanded && (
              <motion.span
                initial={{ opacity: 0, width: 0 }}
                animate={{ opacity: 1, width: 'auto' }}
                exit={{ opacity: 0, width: 0 }}
                transition={{ duration: 0.2 }}
                className="overflow-hidden whitespace-nowrap"
              >
                打开目录
              </motion.span>
            )}
          </AnimatePresence>
        </button>
        <button
          className="flex items-center justify-center transition-colors duration-150"
          style={{
            width: 28,
            height: 28,
            borderRadius: 6,
            color: 'var(--text-tertiary)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-tertiary)';
          }}
          type="button"
        >
          <HelpCircle size={16} />
        </button>
      </div>
    </div>
  );
};

export default LeftSidebar;
