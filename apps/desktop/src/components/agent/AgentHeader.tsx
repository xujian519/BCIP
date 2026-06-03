/**
 * AgentHeader —— 设计规范 §9.1
 * ThreadSelector 下拉 + 新线程 + UsageStrip + 面板操作
 */
import { useRef, useState } from 'react';
import { useAppStore } from '@/hooks/useAppStore';
import { requestNewThread } from '@/lib/desktopEvents';
import { cn } from '@/lib/utils';
import {
  ChevronDown,
  PanelLeftClose,
  PanelLeftOpen,
  Plus,
  Settings,
} from 'lucide-react';
import ThreadListDropdown from './ThreadListDropdown';

interface AgentHeaderProps {
  threadTitle?: string;
  projectName?: string;
  usageText?: string;
  onSelectThread?: (threadId: string) => void;
  onNewThread?: () => void;
}

function usageToneClass(used: number, max: number): string {
  if (max <= 0) {
    return 'text-[var(--text-secondary)]';
  }
  const ratio = used / max;
  if (ratio > 0.95) {
    return 'text-[var(--status-error)]';
  }
  if (ratio > 0.8) {
    return 'text-[var(--status-warning)]';
  }
  if (ratio > 0.5) {
    return 'text-[#D4883A]';
  }
  return 'text-[var(--text-secondary)]';
}

export default function AgentHeader({
  threadTitle = 'New thread',
  projectName = '云熙专利助手',
  usageText,
  onSelectThread,
  onNewThread,
}: AgentHeaderProps) {
  const { state, dispatch } = useAppStore();
  const threadSelectorRef = useRef<HTMLButtonElement>(null);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const meter = state.usageMeter;
  const usageLabel =
    usageText ??
    (meter ? meter.label || `~${meter.used} / ${meter.max}` : null);
  const usageClass =
    meter && !usageText
      ? usageToneClass(meter.used, meter.max)
      : 'text-[var(--text-secondary)]';

  const handleNewThread = () => {
    if (onNewThread) {
      onNewThread();
    } else {
      requestNewThread();
    }
  };

  const handleSelectThread = (threadId: string) => {
    if (onSelectThread) {
      onSelectThread(threadId);
      return;
    }
    dispatch({ type: 'SET_CURRENT_THREAD', payload: threadId });
  };

  return (
    <header
      className={cn(
        'relative shrink-0 flex items-center justify-between px-3',
        'glass-strong',
        'border-b border-[var(--border-default)]',
        'z-10',
      )}
      style={{ height: 'var(--chat-header-h)' }}
    >
      <div className="flex min-w-0 flex-1 items-center gap-1">
        <button
          ref={threadSelectorRef}
          type="button"
          onClick={() => setDropdownOpen((open) => !open)}
          className={cn(
            'flex max-w-[160px] min-w-0 items-center gap-1.5 rounded-lg px-2.5 py-1.5',
            'text-[13px] font-medium text-[var(--text-primary)]',
            dropdownOpen
              ? 'bg-[var(--bg-active)]'
              : 'hover:bg-[var(--bg-hover)]',
            'transition-all duration-200',
          )}
          style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
          title="选择线程"
          aria-expanded={dropdownOpen}
        >
          <span
            className={cn(
              'h-2 w-2 shrink-0 rounded-full',
              state.isStreaming
                ? 'bg-[var(--accent-cyan)] animate-pulse'
                : 'bg-[var(--accent-blue)]',
            )}
          />
          <span className="truncate">{threadTitle}</span>
          <ChevronDown
            size={12}
            className={cn(
              'shrink-0 text-[var(--text-secondary)] transition-transform duration-150',
              dropdownOpen && 'rotate-180',
            )}
          />
        </button>

        <button
          type="button"
          onClick={handleNewThread}
          className={cn(
            'flex h-7 w-7 shrink-0 items-center justify-center rounded-lg',
            'text-[var(--text-secondary)] hover:text-[var(--text-primary)]',
            'hover:bg-[var(--bg-hover)] transition-all duration-200',
          )}
          style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
          title="新线程 (⌘N)"
        >
          <Plus size={16} />
        </button>

        <span
          className="hidden min-w-0 truncate text-2xs text-[var(--text-tertiary)] sm:inline"
          title={projectName}
        >
          {projectName}
        </span>
      </div>

      <div className="flex shrink-0 items-center gap-1">
        {usageLabel && (
          <button
            type="button"
            onClick={() => dispatch({ type: 'OPEN_SETTINGS', payload: 'model' })}
            className={cn(
              'hidden items-center rounded-full px-2.5 py-1 sm:flex',
              'bg-[var(--bg-hover)] font-mono text-[11px] font-medium',
              usageClass,
              'hover:bg-[var(--bg-active)] transition-all duration-200',
            )}
            style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
            title="用量详情"
          >
            {usageLabel}
          </button>
        )}

        <button
          type="button"
          onClick={() => dispatch({ type: 'TOGGLE_THREAD_LIST' })}
          className={cn(
            'flex h-7 w-7 items-center justify-center rounded-lg',
            'text-[var(--text-secondary)] hover:text-[var(--text-primary)]',
            'hover:bg-[var(--bg-hover)] transition-all duration-200',
          )}
          style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
          title={state.threadListOpen ? '折叠会话栏' : '展开会话栏'}
        >
          {state.threadListOpen ? (
            <PanelLeftClose size={16} />
          ) : (
            <PanelLeftOpen size={16} />
          )}
        </button>

        <button
          type="button"
          onClick={() => dispatch({ type: 'OPEN_SETTINGS', payload: 'general' })}
          className={cn(
            'flex h-7 w-7 items-center justify-center rounded-lg',
            'text-[var(--text-secondary)] hover:text-[var(--text-primary)]',
            'hover:bg-[var(--bg-hover)] transition-all duration-200',
          )}
          style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
          title="设置"
        >
          <Settings size={16} />
        </button>
      </div>

      <ThreadListDropdown
        open={dropdownOpen}
        anchorRef={threadSelectorRef}
        threads={state.threads}
        currentThreadId={state.currentThreadId}
        onClose={() => setDropdownOpen(false)}
        onSelectThread={handleSelectThread}
        onNewThread={handleNewThread}
      />
    </header>
  );
}
