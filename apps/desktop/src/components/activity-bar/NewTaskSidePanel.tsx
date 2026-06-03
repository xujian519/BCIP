import { Plus } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import { requestNewThread } from '@/lib/desktopEvents';
import { formatRelativeTime } from '@/lib/threadListFormat';
import { cn } from '@/lib/utils';

export default function NewTaskSidePanel() {
  const { state, dispatch } = useAppStore();

  const handleSelectThread = (threadId: string) => {
    dispatch({ type: 'SET_CURRENT_THREAD', payload: threadId });
    if (!state.agentPanelOpen) {
      dispatch({ type: 'SET_AGENT_PANEL_OPEN', payload: true });
    }
    if (state.layoutMode === 'document') {
      dispatch({ type: 'SET_LAYOUT_MODE', payload: 'three-column' });
    }
  };

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="flex shrink-0 items-center justify-between px-3 py-2">
        <span className="text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
          会话列表
        </span>
        <button
          type="button"
          onClick={() => requestNewThread()}
          className="flex h-7 items-center gap-1 rounded-md px-2 text-[11px] font-medium transition-colors duration-fast"
          style={{
            backgroundColor: 'var(--accent-primary)',
            color: 'var(--text-inverse)',
          }}
        >
          <Plus size={12} />
          新建会话
        </button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {state.threads.length === 0 ? (
          <p className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
            暂无会话，点击「新建」开始
          </p>
        ) : (
          state.threads.map((thread) => {
            const selected = state.currentThreadId === thread.id;
            return (
              <button
                key={thread.id}
                type="button"
                onClick={() => handleSelectThread(thread.id)}
                className={cn(
                  'flex w-full flex-col gap-0.5 border-b px-3 py-2 text-left transition-colors duration-fast',
                  selected ? 'bg-[var(--bg-active)]' : 'hover:bg-[var(--bg-hover)]',
                )}
                style={{ borderColor: 'var(--border-default)' }}
              >
                <span className="truncate text-xs font-medium text-[var(--text-primary)]">
                  {thread.title}
                </span>
                <span className="truncate text-[11px] text-[var(--text-tertiary)]">
                  {thread.preview}
                </span>
                <span className="text-[10px] text-[var(--text-tertiary)]">
                  {formatRelativeTime(thread.timestamp)}
                </span>
              </button>
            );
          })
        )}
      </div>
    </div>
  );
}
