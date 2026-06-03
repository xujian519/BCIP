import { useEffect, useRef } from 'react';
import {
  ArrowDownToLine,
  ArrowLeftToLine,
  ArrowRightToLine,
  ArrowUpToLine,
  Merge,
} from 'lucide-react';
import type { WorkspaceSplitSide } from '@/types';
import { useAppStore } from '@/hooks/useAppStore';
import { canMergePane, canSplitPane } from '@/lib/workspaceLayout';

interface WorkspaceTabContextMenuProps {
  x: number;
  y: number;
  paneId: string;
  tabId: string;
  onClose: () => void;
}

interface MenuItem {
  label: string;
  side?: WorkspaceSplitSide;
  merge?: boolean;
  icon: typeof ArrowRightToLine;
  disabled?: boolean;
}

export default function WorkspaceTabContextMenu({
  x,
  y,
  paneId,
  tabId,
  onClose,
}: WorkspaceTabContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const { state, dispatch } = useAppStore();

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [onClose]);

  const splitAllowed =
    !!state.workspaceRoot && canSplitPane(state.workspaceRoot, paneId);
  const mergeAllowed =
    !!state.workspaceRoot && canMergePane(state.workspaceRoot, paneId);

  const items: MenuItem[] = [
    {
      label: '在右侧分屏',
      side: 'right',
      icon: ArrowRightToLine,
      disabled: !splitAllowed,
    },
    {
      label: '在左侧分屏',
      side: 'left',
      icon: ArrowLeftToLine,
      disabled: !splitAllowed,
    },
    {
      label: '在下方分屏',
      side: 'bottom',
      icon: ArrowDownToLine,
      disabled: !splitAllowed,
    },
    {
      label: '在上方分屏',
      side: 'top',
      icon: ArrowUpToLine,
      disabled: !splitAllowed,
    },
    {
      label: '合并当前 pane',
      merge: true,
      icon: Merge,
      disabled: !mergeAllowed,
    },
  ];

  const handleSelect = (item: MenuItem) => {
    if (item.disabled) {
      return;
    }
    if (item.merge) {
      dispatch({ type: 'SET_FOCUSED_PANE', payload: paneId });
      dispatch({ type: 'MERGE_FOCUSED_PANE' });
    } else if (item.side) {
      dispatch({
        type: 'SPLIT_TAB',
        payload: { paneId, tabId, side: item.side },
      });
    }
    onClose();
  };

  return (
    <div
      ref={menuRef}
      className="fixed z-[100] rounded-lg border shadow-lg"
      style={{
        left: x,
        top: y,
        backgroundColor: 'var(--bg-elevated)',
        borderColor: 'var(--border-primary)',
        padding: 4,
        minWidth: 196,
      }}
    >
      {items.map((item) => {
        const Icon = item.icon;
        return (
          <button
            key={item.label}
            type="button"
            disabled={item.disabled}
            onClick={() => handleSelect(item)}
            className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-40"
            style={{ color: 'var(--text-primary)' }}
            onMouseEnter={(event) => {
              if (!item.disabled) {
                event.currentTarget.style.backgroundColor = 'var(--bg-hover)';
              }
            }}
            onMouseLeave={(event) => {
              event.currentTarget.style.backgroundColor = 'transparent';
            }}
          >
            <Icon size={14} />
            {item.label}
          </button>
        );
      })}
    </div>
  );
}
