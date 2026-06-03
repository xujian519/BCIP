/**
 * 待办/Plan 步骤行 —— 对齐设计规范 §8.3.3（32px 行高、16px 方角复选框）
 */
import { cn } from '@/lib/utils';
import { Check, Trash2 } from 'lucide-react';

export interface TodoRowProps {
  text: string;
  completed: boolean;
  /** 当前执行中的 plan 步骤 */
  active?: boolean;
  readOnly?: boolean;
  onToggle?: () => void;
  onDelete?: () => void;
  showDelete?: boolean;
}

export default function TodoRow({
  text,
  completed,
  active = false,
  readOnly = false,
  onToggle,
  onDelete,
  showDelete = false,
}: TodoRowProps) {
  return (
    <div
      className={cn(
        'group flex h-8 items-center gap-2 rounded-md px-2',
        'transition-colors duration-150',
        !readOnly && 'hover:bg-[var(--bg-hover)]',
        active && 'bg-[var(--plan-step-active)]',
      )}
    >
      <button
        type="button"
        disabled={readOnly || !onToggle}
        onClick={onToggle}
        className={cn(
          'flex h-4 w-4 shrink-0 items-center justify-center rounded-sm border transition-all duration-150',
          completed
            ? 'border-[var(--accent-primary)] bg-[var(--accent-primary)]'
            : active
              ? 'border-[var(--plan-accent)] bg-[var(--bg-elevated)]'
              : 'border-[var(--border-default)] bg-transparent hover:border-[var(--plan-accent)]',
          (readOnly || !onToggle) && 'cursor-default',
        )}
        aria-checked={completed}
        role="checkbox"
      >
        {completed && (
          <Check size={10} className="text-[var(--text-inverse)]" strokeWidth={3} />
        )}
      </button>

      <span
        className={cn(
          'min-w-0 flex-1 truncate text-[13px] leading-none',
          completed
            ? 'text-[var(--text-tertiary)] line-through'
            : active
              ? 'font-medium text-[var(--text-primary)]'
              : 'text-[var(--text-primary)]',
        )}
      >
        {text}
      </span>

      {showDelete && onDelete && (
        <button
          type="button"
          onClick={onDelete}
          className={cn(
            'shrink-0 text-[var(--text-tertiary)] opacity-0 transition-opacity duration-150',
            'hover:text-[var(--status-error)] group-hover:opacity-100',
          )}
          title="删除"
        >
          <Trash2 size={12} />
        </button>
      )}
    </div>
  );
}
