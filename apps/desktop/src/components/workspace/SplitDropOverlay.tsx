import { useCallback, useMemo, useState } from 'react';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/hooks/useAppStore';
import {
  MAX_WORKSPACE_SPLIT_DEPTH,
  canSplitPane,
} from '@/lib/workspaceLayout';
import { useWorkspaceDrag } from './WorkspaceDragContext';

interface SplitDropOverlayProps {
  paneId: string;
}

type DropSide = 'left' | 'right' | null;

export default function SplitDropOverlay({ paneId }: SplitDropOverlayProps) {
  const { dragging } = useWorkspaceDrag();
  const { state, dispatch } = useAppStore();
  const [hoverSide, setHoverSide] = useState<DropSide>(null);

  const splitAllowed = useMemo(() => {
    if (!state.workspaceRoot) {
      return false;
    }
    return canSplitPane(state.workspaceRoot, paneId);
  }, [paneId, state.workspaceRoot]);

  const handleDragOver = useCallback(
    (event: React.DragEvent, side: DropSide) => {
      event.preventDefault();
      event.dataTransfer.dropEffect = splitAllowed ? 'move' : 'none';
      setHoverSide(side);
    },
    [splitAllowed],
  );

  const handleDrop = useCallback(
    (event: React.DragEvent, side: 'left' | 'right') => {
      event.preventDefault();
      setHoverSide(null);
      if (!dragging) {
        return;
      }

      if (!splitAllowed) {
        if (dragging.paneId !== paneId) {
          dispatch({
            type: 'MOVE_TAB',
            payload: { tabId: dragging.tabId, targetPaneId: paneId },
          });
        }
        return;
      }

      if (dragging.paneId === paneId) {
        dispatch({
          type: 'SPLIT_TAB',
          payload: { paneId, tabId: dragging.tabId, side },
        });
        return;
      }

      dispatch({
        type: 'MOVE_TAB',
        payload: { tabId: dragging.tabId, targetPaneId: paneId },
      });
      dispatch({
        type: 'SPLIT_TAB',
        payload: { paneId, tabId: dragging.tabId, side },
      });
    },
    [dispatch, dragging, paneId, splitAllowed],
  );

  if (!dragging) {
    return null;
  }

  const hintLabel = splitAllowed
    ? null
    : `已达最大分屏层级 (${MAX_WORKSPACE_SPLIT_DEPTH})`;

  return (
    <div className="pointer-events-none absolute inset-0 z-20 flex flex-col">
      {hintLabel && (
        <div className="pointer-events-none absolute left-1/2 top-2 z-30 -translate-x-1/2 rounded-md bg-[var(--bg-elevated)] px-2 py-1 text-[11px] text-[var(--status-warning)] shadow-sm">
          {hintLabel}
          {dragging.paneId !== paneId ? '，将合并到当前 pane' : ''}
        </div>
      )}
      <div className="flex min-h-0 flex-1">
        <div
          className={cn(
            'pointer-events-auto flex h-full w-1/2 items-center justify-center border-r border-dashed transition-colors duration-150',
            hoverSide === 'left'
              ? splitAllowed
                ? 'border-[var(--accent-primary)] bg-[var(--accent-primary-muted)]'
                : 'border-[var(--status-warning)] bg-[var(--status-warning)]/10'
              : 'border-[var(--border-default)] bg-[var(--bg-hover)]/40',
          )}
          onDragOver={(event) => handleDragOver(event, 'left')}
          onDragLeave={() => setHoverSide(null)}
          onDrop={(event) => handleDrop(event, 'left')}
        >
          <span className="rounded-md bg-[var(--bg-elevated)] px-2 py-1 text-[11px] text-[var(--text-secondary)]">
            {splitAllowed ? '左侧分屏' : '合并到此处'}
          </span>
        </div>
        <div
          className={cn(
            'pointer-events-auto flex h-full w-1/2 items-center justify-center border-dashed transition-colors duration-150',
            hoverSide === 'right'
              ? splitAllowed
                ? 'border-[var(--accent-primary)] bg-[var(--accent-primary-muted)]'
                : 'border-[var(--status-warning)] bg-[var(--status-warning)]/10'
              : 'border-[var(--border-default)] bg-[var(--bg-hover)]/40',
          )}
          onDragOver={(event) => handleDragOver(event, 'right')}
          onDragLeave={() => setHoverSide(null)}
          onDrop={(event) => handleDrop(event, 'right')}
        >
          <span className="rounded-md bg-[var(--bg-elevated)] px-2 py-1 text-[11px] text-[var(--text-secondary)]">
            {splitAllowed ? '右侧分屏' : '合并到此处'}
          </span>
        </div>
      </div>
    </div>
  );
}
