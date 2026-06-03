import { useCallback, useState, type FC } from 'react';
import { X, FileText } from 'lucide-react';
import type { WorkspaceTab as WTab } from '@/types';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/hooks/useAppStore';
import {
  TAB_DRAG_MIME,
  canSplitPane,
  decodeTabDragPayload,
  encodeTabDragPayload,
} from '@/lib/workspaceLayout';
import { useWorkspaceDrag } from './WorkspaceDragContext';
import WorkspaceTabContextMenu from './WorkspaceTabContextMenu';

interface WorkspaceTabsProps {
  paneId: string;
  tabs: WTab[];
  activeTabId: string | null;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
}

const TAB_BAR_EDGE_PX = 44;

function resolveInsertIndex(
  clientX: number,
  tabElement: HTMLElement,
  tabIndex: number,
): number {
  const rect = tabElement.getBoundingClientRect();
  const mid = rect.left + rect.width / 2;
  return clientX < mid ? tabIndex : tabIndex + 1;
}

const WorkspaceTabs: FC<WorkspaceTabsProps> = ({
  paneId,
  tabs,
  activeTabId,
  onSelect,
  onClose,
}) => {
  const { state, dispatch } = useAppStore();
  const { dragging, setDragging } = useWorkspaceDrag();
  const [insertIndex, setInsertIndex] = useState<number | null>(null);
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    tabId: string;
  } | null>(null);

  const handleDragStart = useCallback(
    (event: React.DragEvent, tab: WTab) => {
      const payload = { tabId: tab.id, paneId };
      event.dataTransfer.setData(TAB_DRAG_MIME, encodeTabDragPayload(payload));
      event.dataTransfer.effectAllowed = 'move';
      setDragging(payload);
    },
    [paneId, setDragging],
  );

  const handleDragEnd = useCallback(() => {
    setDragging(null);
    setInsertIndex(null);
  }, [setDragging]);

  const readDragPayload = useCallback(
    (event: React.DragEvent) => {
      const raw = event.dataTransfer.getData(TAB_DRAG_MIME);
      return decodeTabDragPayload(raw) ?? dragging;
    },
    [dragging],
  );

  const dispatchSplit = useCallback(
    (tabId: string, side: 'left' | 'right') => {
      if (!state.workspaceRoot || !canSplitPane(state.workspaceRoot, paneId)) {
        return;
      }
      dispatch({
        type: 'SPLIT_TAB',
        payload: { paneId, tabId, side },
      });
    },
    [dispatch, paneId, state.workspaceRoot],
  );

  const dispatchInsert = useCallback(
    (payload: { tabId: string; paneId: string }, toIndex: number) => {
      if (payload.paneId === paneId) {
        dispatch({
          type: 'REORDER_TAB',
          payload: { paneId, tabId: payload.tabId, toIndex },
        });
        return;
      }
      dispatch({
        type: 'MOVE_TAB',
        payload: {
          tabId: payload.tabId,
          targetPaneId: paneId,
          insertIndex: toIndex,
        },
      });
    },
    [dispatch, paneId],
  );

  const handleTabBarDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const handleTabBarDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      setInsertIndex(null);
      const payload = readDragPayload(event);
      if (!payload) {
        return;
      }

      const rect = event.currentTarget.getBoundingClientRect();
      const x = event.clientX - rect.left;

      if (x < TAB_BAR_EDGE_PX) {
        dispatchSplit(payload.tabId, 'left');
        return;
      }
      if (x > rect.width - TAB_BAR_EDGE_PX) {
        dispatchSplit(payload.tabId, 'right');
        return;
      }

      dispatchInsert(payload, tabs.length);
    },
    [dispatchInsert, dispatchSplit, readDragPayload, tabs.length],
  );

  const handleTabDragOver = useCallback(
    (event: React.DragEvent, tabIndex: number) => {
      event.preventDefault();
      event.stopPropagation();
      const nextIndex = resolveInsertIndex(
        event.clientX,
        event.currentTarget as HTMLElement,
        tabIndex,
      );
      setInsertIndex(nextIndex);
    },
    [],
  );

  const handleTabDrop = useCallback(
    (event: React.DragEvent, tabIndex: number) => {
      event.preventDefault();
      event.stopPropagation();
      setInsertIndex(null);
      const payload = readDragPayload(event);
      if (!payload) {
        return;
      }

      const barRect = event.currentTarget.parentElement?.getBoundingClientRect();
      if (barRect) {
        const x = event.clientX - barRect.left;
        if (x < TAB_BAR_EDGE_PX) {
          dispatchSplit(payload.tabId, 'left');
          return;
        }
        if (x > barRect.width - TAB_BAR_EDGE_PX) {
          dispatchSplit(payload.tabId, 'right');
          return;
        }
      }

      const toIndex = resolveInsertIndex(
        event.clientX,
        event.currentTarget as HTMLElement,
        tabIndex,
      );
      dispatchInsert(payload, toIndex);
    },
    [dispatchInsert, dispatchSplit, readDragPayload],
  );

  const tabBarStyle = {
    height: 36,
    backgroundColor: 'var(--bg-elevated)',
    borderBottom: '1px solid var(--border-primary)',
    userSelect: 'none' as const,
  } as const;

  if (tabs.length === 0 && !dragging) {
    return (
      <div
        className="flex shrink-0 items-center border-b border-[var(--border-primary)] px-3 text-[11px] text-[var(--text-tertiary)]"
        style={tabBarStyle}
        onDragOver={handleTabBarDragOver}
        onDrop={handleTabBarDrop}
      >
        拖拽标签到此处以合并 pane
      </div>
    );
  }

  if (tabs.length === 0) {
    return (
      <div
        className="relative flex shrink-0 overflow-x-auto"
        style={tabBarStyle}
        onDragOver={handleTabBarDragOver}
        onDrop={handleTabBarDrop}
      />
    );
  }

  return (
    <>
      <div
        className="relative flex shrink-0 overflow-x-auto"
        style={tabBarStyle}
        onDragOver={handleTabBarDragOver}
        onDrop={handleTabBarDrop}
        onDragLeave={() => setInsertIndex(null)}
      >
      {tabs.map((tab, index) => {
        const isActive = tab.id === activeTabId;
        const showInsertBefore = insertIndex === index;
        return (
          <div key={tab.id} className="relative flex shrink-0">
            {showInsertBefore && (
              <div
                className="pointer-events-none absolute bottom-0 top-0 z-10 w-0.5 bg-[var(--accent-primary)]"
                style={{ left: 0 }}
              />
            )}
            <div
              draggable
              onDragStart={(event) => handleDragStart(event, tab)}
              onDragEnd={handleDragEnd}
              onDragOver={(event) => handleTabDragOver(event, index)}
              onDrop={(event) => handleTabDrop(event, index)}
              className={cn(
                'group flex shrink-0 cursor-grab select-none items-center gap-1.5 active:cursor-grabbing',
                'transition-colors duration-150',
              )}
              style={{
                padding: '0 12px',
                height: 36,
                fontSize: 12,
                fontWeight: isActive ? 500 : 400,
                color: isActive ? 'var(--text-primary)' : 'var(--text-tertiary)',
                backgroundColor: isActive ? 'var(--bg-surface)' : 'transparent',
                borderRight: '1px solid var(--border-primary)',
                maxWidth: 180,
              }}
              onClick={() => onSelect(tab.id)}
              onContextMenu={(event) => {
                event.preventDefault();
                setContextMenu({
                  x: event.clientX,
                  y: event.clientY,
                  tabId: tab.id,
                });
              }}
            >
              <FileText size={12} style={{ flexShrink: 0 }} />
              <span className="flex-1 truncate">{tab.title}</span>
              <button
                onClick={(event) => {
                  event.stopPropagation();
                  onClose(tab.id);
                }}
                className={cn(
                  'shrink-0 rounded-md p-0.5 opacity-0',
                  'transition-all duration-150',
                  'hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
                  'group-hover:opacity-100',
                )}
                style={{ color: 'var(--text-tertiary)' }}
                type="button"
                aria-label={`关闭 ${tab.title}`}
              >
                <X size={12} />
              </button>
            </div>
          </div>
        );
      })}
      {insertIndex === tabs.length && (
        <div className="pointer-events-none relative w-0 shrink-0">
          <div className="absolute bottom-0 top-0 w-0.5 bg-[var(--accent-primary)]" />
        </div>
      )}
      </div>
      {contextMenu && (
        <WorkspaceTabContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          paneId={paneId}
          tabId={contextMenu.tabId}
          onClose={() => setContextMenu(null)}
        />
      )}
    </>
  );
};

export default WorkspaceTabs;
