/**
 * ThreadListDrawer —— Agent 面板左侧可折叠会话列表（按项目过滤）
 */
import { useAppStore } from '@/hooks/useAppStore';
import { formatRelativeTime } from '@/lib/threadListFormat';
import { cn } from '@/lib/utils';
import {
  Archive,
  ChevronLeft,
  ChevronRight,
  MessageSquare,
  Plus,
  Trash2,
} from 'lucide-react';
import type { Thread } from '@/types';

const THREAD_LIST_WIDTH = 260;

/** 单个线程行 */
function ThreadRow({
  thread,
  isSelected,
  onClick,
  onArchive,
  onDelete,
}: {
  thread: Thread;
  isSelected: boolean;
  onClick: () => void;
  onArchive: () => void;
  onDelete: () => void;
}) {
  return (
    <div
      className={cn(
        'group relative flex min-h-12 w-full items-start gap-2.5 px-3 py-2',
        'border-b border-[var(--border-default)] transition-colors duration-150',
        isSelected ? 'bg-[var(--bg-active)]' : 'hover:bg-[var(--bg-hover)]',
      )}
    >
      {isSelected && (
        <div className="absolute left-0 top-1/2 h-6 w-[3px] -translate-y-1/2 rounded-r-full bg-[var(--thread-selected)]" />
      )}

      <button type="button" onClick={onClick} className="flex min-w-0 flex-1 items-center gap-2.5 text-left">
        <span
          className={cn(
            'mt-1.5 h-2 w-2 shrink-0 rounded-full',
            thread.status === 'active'
              ? 'bg-[var(--accent-blue)]'
              : 'bg-[var(--text-tertiary)]',
          )}
        />

        <div className="flex min-w-0 flex-1 flex-col justify-center">
          <div
            className={cn(
              'truncate text-[13px] leading-tight',
              isSelected
                ? 'font-semibold text-[var(--text-primary)]'
                : 'font-medium text-[var(--text-primary)]',
            )}
          >
            {thread.title}
          </div>
          <div className="mt-0.5 truncate text-xs leading-tight text-[var(--text-tertiary)] line-clamp-1">
            {thread.preview}
          </div>
        </div>

        <span className="ml-1 shrink-0 text-[11px] text-[var(--text-tertiary)] group-hover:hidden">
          {formatRelativeTime(thread.timestamp)}
        </span>
      </button>

      <div className="hidden shrink-0 items-center gap-0.5 group-hover:flex">
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onArchive();
          }}
          className={cn(
            'flex h-6 w-6 items-center justify-center rounded-md',
            'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
          )}
          title="存档"
        >
          <Archive size={12} />
        </button>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onDelete();
          }}
          className={cn(
            'flex h-6 w-6 items-center justify-center rounded-md',
            'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-red-500',
          )}
          title="删除"
        >
          <Trash2 size={12} />
        </button>
      </div>
    </div>
  );
}

interface ThreadListDrawerProps {
  visible: boolean;
  useRpc?: boolean;
  onSelectThread?: (threadId: string) => void;
  onNewThread?: () => void;
  onArchiveThread?: (threadId: string) => void;
  onDeleteThread?: (threadId: string) => void;
}

export default function ThreadListDrawer({
  visible,
  useRpc = false,
  onSelectThread,
  onNewThread,
  onArchiveThread,
  onDeleteThread,
}: ThreadListDrawerProps) {
  const { state, dispatch } = useAppStore();
  const expanded = state.threadListOpen;

  if (!visible) {
    return null;
  }

  const handleToggle = () => {
    dispatch({ type: 'TOGGLE_THREAD_LIST' });
  };

  const handleNewThread = () => {
    if (useRpc && onNewThread) {
      void onNewThread();
      return;
    }
    const newThread: Thread = {
      id: `thread-${Date.now()}`,
      title: '新会话',
      preview: '',
      timestamp: Date.now(),
      status: 'active' as const,
    };
    dispatch({ type: 'SET_THREADS', payload: [...state.threads, newThread] });
    dispatch({ type: 'SET_CURRENT_THREAD', payload: newThread.id });
  };

  const handleSelectThread = (threadId: string) => {
    if (useRpc && onSelectThread) {
      void onSelectThread(threadId);
      return;
    }
    dispatch({ type: 'SET_CURRENT_THREAD', payload: threadId });
  };

  const handleArchiveThread = (threadId: string) => {
    if (useRpc && onArchiveThread) {
      void onArchiveThread(threadId);
      return;
    }
    dispatch({ type: 'REMOVE_THREAD', payload: threadId });
  };

  const handleDeleteThread = (threadId: string) => {
    if (useRpc && onDeleteThread) {
      void onDeleteThread(threadId);
      return;
    }
    if (window.confirm('确定删除此会话？')) {
      dispatch({ type: 'REMOVE_THREAD', payload: threadId });
    }
  };

  const projectLabel = state.workspaceCwd
    ? (state.workspaceCwd.split('/').filter(Boolean).pop() ?? '项目')
    : '未选择项目';

  return (
    <div
      className={cn(
        'flex h-full shrink-0 flex-col border-r border-[var(--border-default)]',
        'bg-[var(--bg-sidebar-solid)]',
        expanded ? undefined : 'w-9',
      )}
      style={expanded ? { width: THREAD_LIST_WIDTH } : undefined}
    >
      <div
        className={cn(
          'flex h-8 shrink-0 items-center border-b border-[var(--border-default)]',
          expanded ? 'justify-between px-2' : 'justify-center',
        )}
      >
        {expanded ? (
          <>
            <div className="flex min-w-0 flex-col">
              <div className="flex items-center gap-1.5">
                <MessageSquare size={12} className="text-[var(--text-tertiary)]" />
                <span className="text-2xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                  会话
                </span>
              </div>
              <span
                className="truncate pl-[18px] text-[10px] text-[var(--text-tertiary)]"
                title={state.workspaceCwd ?? undefined}
              >
                {projectLabel}
              </span>
            </div>
            <div className="flex items-center gap-0.5">
              <button
                type="button"
                onClick={handleNewThread}
                className={cn(
                  'flex h-6 w-6 items-center justify-center rounded-md',
                  'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
                )}
                title="新建会话"
              >
                <Plus size={14} />
              </button>
              <button
                type="button"
                onClick={handleToggle}
                className={cn(
                  'flex h-6 w-6 items-center justify-center rounded-md',
                  'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
                )}
                title="折叠会话栏"
              >
                <ChevronLeft size={14} />
              </button>
            </div>
          </>
        ) : (
          <button
            type="button"
            onClick={handleToggle}
            className={cn(
              'flex h-7 w-7 items-center justify-center rounded-md',
              'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
            )}
            title="展开会话栏"
          >
            <ChevronRight size={14} />
          </button>
        )}
      </div>

      {expanded && (
        <div className="min-h-0 flex-1 overflow-y-auto custom-scrollbar">
          {state.threads.length === 0 && (
            <div className="p-4 text-center text-2xs text-[var(--text-tertiary)]">
              {state.workspaceCwd ? '暂无会话，点击 + 新建' : '请先选择项目'}
            </div>
          )}
          {state.threads.map((thread) => (
            <ThreadRow
              key={thread.id}
              thread={thread}
              isSelected={thread.id === state.currentThreadId}
              onClick={() => handleSelectThread(thread.id)}
              onArchive={() => handleArchiveThread(thread.id)}
              onDelete={() => handleDeleteThread(thread.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
