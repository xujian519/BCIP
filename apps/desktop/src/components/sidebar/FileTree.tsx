import { useState, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Folder, FolderOpen, File, ChevronRight, ChevronDown } from 'lucide-react';
import type { FileEntry } from '@/lib/fileSystem';
import FileTreeContextMenu from './FileTreeContextMenu';
import { useAppStore } from '@/hooks/useAppStore';
import { tabIdForPath } from '@/lib/workspaceLayout';

interface FileTreeNodeProps {
  entry: FileEntry;
  depth: number;
  selectedPath: string | null;
  expandedPaths: Set<string>;
  onSelect: (path: string) => void;
  onToggleExpand: (path: string) => void;
  onLoadChildren: (path: string) => Promise<FileEntry[]>;
  onContextMenu: (e: React.MouseEvent, path: string) => void;
}

function FileTreeNode({
  entry,
  depth,
  selectedPath,
  expandedPaths,
  onSelect,
  onToggleExpand,
  onLoadChildren,
  onContextMenu,
}: FileTreeNodeProps) {
  const [children, setChildren] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const isExpanded = expandedPaths.has(entry.path);
  const isSelected = selectedPath === entry.path;
  const paddingLeft = 12 + depth * 16;

  const handleToggle = useCallback(async () => {
    if (entry.isDirectory) {
      if (!isExpanded && children.length === 0) {
        setLoading(true);
        try {
          const childEntries = await onLoadChildren(entry.path);
          setChildren(childEntries);
        } catch (error) {
          console.error('Failed to load children:', error);
        } finally {
          setLoading(false);
        }
      }
      onToggleExpand(entry.path);
    }
  }, [entry, isExpanded, children, onToggleExpand, onLoadChildren]);

  const handleSelect = useCallback(() => {
    onSelect(entry.path);
  }, [entry.path, onSelect]);

  return (
    <div>
      <button
        onClick={handleSelect}
        onDoubleClick={handleToggle}
        onContextMenu={(e) => onContextMenu(e, entry.path)}
        className="flex w-full items-center transition-colors duration-150"
        style={{
          height: 28,
          paddingLeft,
          paddingRight: 12,
          backgroundColor: isSelected ? 'var(--bg-sidebar-active)' : 'transparent',
          borderLeft: isSelected ? '2px solid var(--accent-primary)' : '2px solid transparent',
        }}
        onMouseEnter={(e) => {
          if (!isSelected) e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
        }}
        onMouseLeave={(e) => {
          if (!isSelected) e.currentTarget.style.backgroundColor = 'transparent';
        }}
        type="button"
      >
        {entry.isDirectory && (
          <span
            onClick={(e) => {
              e.stopPropagation();
              handleToggle();
            }}
            className="flex items-center justify-center"
            style={{ width: 16, height: 16, marginRight: 4, cursor: 'pointer' }}
          >
            {isExpanded ? (
              <ChevronDown size={14} style={{ color: 'var(--text-tertiary)' }} />
            ) : (
              <ChevronRight size={14} style={{ color: 'var(--text-tertiary)' }} />
            )}
          </span>
        )}
        {!entry.isDirectory && <span style={{ width: 16, marginRight: 4 }} />}
        
        {entry.isDirectory ? (
          isExpanded ? (
            <FolderOpen size={16} style={{ color: 'var(--accent-primary)', marginRight: 6, flexShrink: 0 }} />
          ) : (
            <Folder size={16} style={{ color: 'var(--text-tertiary)', marginRight: 6, flexShrink: 0 }} />
          )
        ) : (
          <File size={16} style={{ color: 'var(--text-tertiary)', marginRight: 6, flexShrink: 0 }} />
        )}
        
        <span
          className="truncate"
          style={{
            fontSize: 12,
            color: isSelected ? 'var(--text-primary)' : 'var(--text-secondary)',
            fontWeight: isSelected ? 500 : 400,
          }}
        >
          {entry.name}
        </span>
        
        {loading && (
          <span className="ml-2 animate-pulse" style={{ color: 'var(--text-tertiary)', fontSize: 10 }}>
            加载中...
          </span>
        )}
      </button>

      <AnimatePresence>
        {isExpanded && entry.isDirectory && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden"
          >
            {children.map((child) => (
              <FileTreeNode
                key={child.path}
                entry={child}
                depth={depth + 1}
                selectedPath={selectedPath}
                expandedPaths={expandedPaths}
                onSelect={onSelect}
                onToggleExpand={onToggleExpand}
                onLoadChildren={onLoadChildren}
                onContextMenu={onContextMenu}
              />
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

interface FileTreeProps {
  rootPath: string;
  entries: FileEntry[];
  selectedPath: string | null;
  expandedPaths: Set<string>;
  onSelect: (path: string) => void;
  onToggleExpand: (path: string) => void;
  onLoadChildren: (path: string) => Promise<FileEntry[]>;
}

export default function FileTree({
  rootPath: _rootPath,
  entries,
  selectedPath,
  expandedPaths,
  onSelect,
  onToggleExpand,
  onLoadChildren,
}: FileTreeProps) {
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; path: string } | null>(null);
  const { dispatch } = useAppStore();

  const handleContextMenu = (e: React.MouseEvent, path: string) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, path });
  };

  return (
    <div className="flex-1 overflow-y-auto custom-scrollbar">
      {entries.map((entry) => (
        <FileTreeNode
          key={entry.path}
          entry={entry}
          depth={0}
          selectedPath={selectedPath}
          expandedPaths={expandedPaths}
          onSelect={onSelect}
          onToggleExpand={onToggleExpand}
          onLoadChildren={onLoadChildren}
          onContextMenu={handleContextMenu}
        />
      ))}
      {contextMenu && (
        <FileTreeContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          filePath={contextMenu.path}
          onMention={(path) => {
            dispatch({ type: 'INSERT_CHAT_MENTION', payload: { path } });
            window.dispatchEvent(new CustomEvent('bcip:focus-composer'));
          }}
          onOpenSplitRight={(path) => {
            const fileName = path.split('/').pop() ?? path;
            dispatch({
              type: 'OPEN_TAB_SPLIT',
              payload: {
                tab: {
                  id: tabIdForPath(path),
                  filePath: path,
                  title: fileName,
                },
                side: 'right',
              },
            });
          }}
          onOpenSplitDown={(path) => {
            const fileName = path.split('/').pop() ?? path;
            dispatch({
              type: 'OPEN_TAB_SPLIT',
              payload: {
                tab: {
                  id: tabIdForPath(path),
                  filePath: path,
                  title: fileName,
                },
                side: 'bottom',
              },
            });
          }}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  );
}
