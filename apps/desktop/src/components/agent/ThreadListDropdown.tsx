/**
 * ThreadListDropdown —— 设计规范 §9.2.2（ThreadSelector 下拉）
 */
import { useEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { Plus } from 'lucide-react';
import { cn } from '@/lib/utils';
import {
  formatRelativeTime,
  truncateThreadPreview,
} from '@/lib/threadListFormat';
import type { Thread } from '@/types';

type ThreadTab = 'recent' | 'all';

interface ThreadListDropdownProps {
  open: boolean;
  anchorRef: React.RefObject<HTMLElement | null>;
  threads: Thread[];
  currentThreadId: string | null;
  onClose: () => void;
  onSelectThread: (threadId: string) => void;
  onNewThread: () => void;
}

function DropdownThreadRow({
  thread,
  isSelected,
  onClick,
}: {
  thread: Thread;
  isSelected: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'relative flex w-full min-h-12 items-start gap-2.5 px-3 py-2 text-left',
        'transition-colors duration-150',
        isSelected ? 'bg-[var(--bg-active)]' : 'hover:bg-[var(--bg-hover)]',
      )}
    >
      {isSelected && (
        <div className="absolute left-0 top-1/2 h-6 w-[3px] -translate-y-1/2 rounded-r-full bg-[var(--thread-selected)]" />
      )}
      <span
        className={cn(
          'mt-1.5 h-2 w-2 shrink-0 rounded-full',
          thread.status === 'active'
            ? 'bg-[var(--accent-blue)]'
            : 'bg-[var(--text-tertiary)]',
        )}
      />
      <div className="min-w-0 flex-1 pr-10">
        <div
          className={cn(
            'truncate text-[13px] leading-tight text-[var(--text-primary)]',
            isSelected ? 'font-semibold' : 'font-medium',
          )}
        >
          {thread.title}
        </div>
        {thread.preview && (
          <div className="mt-0.5 truncate text-xs leading-tight text-[var(--text-tertiary)]">
            {truncateThreadPreview(thread.preview)}
          </div>
        )}
      </div>
      <span className="absolute right-3 top-2 text-[11px] text-[var(--text-tertiary)]">
        {formatRelativeTime(thread.timestamp)}
      </span>
    </button>
  );
}

export default function ThreadListDropdown({
  open,
  anchorRef,
  threads,
  currentThreadId,
  onClose,
  onSelectThread,
  onNewThread,
}: ThreadListDropdownProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const [tab, setTab] = useState<ThreadTab>('recent');
  const [position, setPosition] = useState({ top: 0, left: 0, width: 280 });

  useEffect(() => {
    if (!open || !anchorRef.current) {
      return;
    }
    const rect = anchorRef.current.getBoundingClientRect();
    setPosition({
      top: rect.bottom + 4,
      left: rect.left,
      width: 280,
    });
  }, [open, anchorRef]);

  useEffect(() => {
    if (!open) {
      return;
    }
    const onPointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (
        panelRef.current?.contains(target) ||
        anchorRef.current?.contains(target)
      ) {
        return;
      }
      onClose();
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };
    document.addEventListener('mousedown', onPointerDown);
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('mousedown', onPointerDown);
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [open, onClose, anchorRef]);

  const sortedThreads = useMemo(
    () => [...threads].sort((a, b) => b.timestamp - a.timestamp),
    [threads],
  );

  const visibleThreads = useMemo(() => {
    if (tab === 'all') {
      return sortedThreads;
    }
    const weekAgo = Date.now() - 7 * 24 * 60 * 60 * 1000;
    return sortedThreads.filter((t) => t.timestamp >= weekAgo);
  }, [sortedThreads, tab]);

  if (!open) {
    return null;
  }

  return createPortal(
    <div
      ref={panelRef}
      className={cn(
        'fixed z-[70] flex max-h-[400px] flex-col overflow-hidden',
        'rounded-lg border border-[var(--border-default)]',
        'bg-[var(--bg-elevated)] shadow-lg',
      )}
      style={{
        top: position.top,
        left: position.left,
        width: position.width,
      }}
    >
      <div className="flex h-8 shrink-0 border-b border-[var(--border-default)] px-2">
        {(
          [
            ['recent', '最近'],
            ['all', '全部'],
          ] as const
        ).map(([id, label]) => (
          <button
            key={id}
            type="button"
            onClick={() => setTab(id)}
            className={cn(
              'relative flex h-full flex-1 items-center justify-center text-xs font-medium',
              tab === id
                ? 'text-[var(--text-primary)]'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]',
            )}
          >
            {label}
            {tab === id && (
              <span className="absolute bottom-0 left-2 right-2 h-0.5 rounded-full bg-[var(--accent-primary)]" />
            )}
          </button>
        ))}
      </div>

      <div className="custom-scrollbar min-h-0 flex-1 overflow-y-auto p-1">
        {visibleThreads.length === 0 ? (
          <p className="px-3 py-6 text-center text-xs text-[var(--text-tertiary)]">
            暂无会话
          </p>
        ) : (
          visibleThreads.map((thread) => (
            <DropdownThreadRow
              key={thread.id}
              thread={thread}
              isSelected={thread.id === currentThreadId}
              onClick={() => {
                onSelectThread(thread.id);
                onClose();
              }}
            />
          ))
        )}
      </div>

      <div className="shrink-0 border-t border-[var(--border-default)]">
        <button
          type="button"
          onClick={() => {
            onNewThread();
            onClose();
          }}
          className={cn(
            'flex h-9 w-full items-center justify-center gap-1.5',
            'text-xs font-medium text-[var(--accent-primary)]',
            'hover:bg-[var(--accent-primary-muted)] transition-colors duration-150',
          )}
        >
          <Plus size={14} />
          新建线程
        </button>
      </div>
    </div>,
    document.body,
  );
}
