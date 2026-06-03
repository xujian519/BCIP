/**
 * TodoDock —— 设计规范 §8.3：底部可折叠待办坞（Plan 模式同步 Agent turn/plan）
 */
import { useState, useRef, useEffect, useCallback } from 'react';
import { ChevronDown, ChevronRight, Plus, Send } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/hooks/useAppStore';
import { isPlanTodoId } from '@/lib/patentTodos';
import TodoRow from './TodoRow';
import type { TodoItem } from '@/types';

function TodoAddInline({
  visible,
  onAdd,
  onCancel,
}: {
  visible: boolean;
  onAdd: (text: string) => void;
  onCancel: () => void;
}) {
  const [text, setText] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (visible) {
      setText('');
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [visible]);

  if (!visible) {
    return null;
  }

  const submit = () => {
    const trimmed = text.trim();
    if (trimmed) {
      onAdd(trimmed);
      setText('');
    }
  };

  return (
    <div className="flex h-9 shrink-0 items-center gap-2 px-3">
      <div
        className={cn(
          'flex h-7 flex-1 items-center gap-2 rounded-md border px-3',
          'border-[var(--border-default)] bg-[var(--bg-elevated)]',
          'focus-within:border-[var(--plan-accent)]',
        )}
      >
        <input
          ref={inputRef}
          type="text"
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              submit();
            } else if (e.key === 'Escape') {
              onCancel();
            }
          }}
          placeholder="输入新待办..."
          className={cn(
            'min-w-0 flex-1 bg-transparent text-[13px] text-[var(--text-primary)]',
            'placeholder:text-[var(--text-tertiary)] outline-none',
          )}
        />
      </div>
      <button
        type="button"
        onClick={submit}
        className={cn(
          'flex h-7 w-7 shrink-0 items-center justify-center rounded-md',
          'bg-[var(--accent-primary)] text-[var(--text-inverse)]',
          'hover:bg-[var(--accent-primary-hover)] transition-colors duration-150',
        )}
        title="保存"
      >
        <Send size={12} />
      </button>
      <button
        type="button"
        onClick={onCancel}
        className={cn(
          'h-7 shrink-0 rounded-md px-2 text-xs text-[var(--text-secondary)]',
          'hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
        )}
      >
        取消
      </button>
    </div>
  );
}

export default function TodoDock() {
  const { state, dispatch } = useAppStore();
  const [adding, setAdding] = useState(false);
  const planMode = state.todos.some((t) => isPlanTodoId(t.id));
  const activeIndex = state.todos.findIndex((t) => !t.completed);

  const handleToggle = (todo: TodoItem) => {
    dispatch({
      type: 'UPDATE_TODO',
      payload: { id: todo.id, completed: !todo.completed },
    });
  };

  const handleAdd = (text: string) => {
    dispatch({
      type: 'ADD_TODO',
      payload: {
        id: `todo-${Date.now()}`,
        text,
        completed: false,
        createdAt: Date.now(),
      },
    });
    setAdding(false);
  };

  const handleDelete = (id: string) => {
    dispatch({ type: 'DELETE_TODO', payload: id });
  };

  const openDock = () => {
    if (!state.todoDockOpen) {
      dispatch({ type: 'TOGGLE_TODO_DOCK' });
    }
  };

  const dockHeight = state.todoDockOpen ? state.todoDockHeight : 36;

  const handleResizeStart = useCallback(
    (event: React.MouseEvent) => {
      event.preventDefault();
      event.stopPropagation();
      const startY = event.clientY;
      const startHeight = state.todoDockHeight;

      const onMove = (moveEvent: MouseEvent) => {
        const next = Math.max(
          36,
          Math.min(240, startHeight + (startY - moveEvent.clientY)),
        );
        dispatch({ type: 'SET_TODO_DOCK_HEIGHT', payload: next });
      };

      const onUp = () => {
        document.removeEventListener('mousemove', onMove);
        document.removeEventListener('mouseup', onUp);
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
      };

      document.body.style.cursor = 'ns-resize';
      document.body.style.userSelect = 'none';
      document.addEventListener('mousemove', onMove);
      document.addEventListener('mouseup', onUp);
    },
    [dispatch, state.todoDockHeight],
  );

  return (
    <div
      className={cn(
        'relative flex shrink-0 flex-col overflow-hidden border-t border-[var(--border-default)]',
        'bg-[var(--bg-surface)] transition-[height] duration-250 ease-in-out',
        planMode && 'border-t-[var(--plan-border)]',
      )}
      style={{ height: `${dockHeight}px` }}
    >
      {state.todoDockOpen && (
        <div
          role="separator"
          aria-orientation="horizontal"
          aria-label="调整 Todo 面板高度"
          className="group absolute inset-x-0 top-0 z-10 h-1 cursor-ns-resize"
          onMouseDown={handleResizeStart}
        >
          <div
            className={cn(
              'mx-auto mt-0 h-0.5 w-10 rounded-full transition-colors duration-150',
              'bg-transparent group-hover:bg-[var(--accent-primary)]',
            )}
          />
        </div>
      )}
      {/* §8.3.2 TodoHeader — 36px */}
      <div
        role="button"
        tabIndex={0}
        onClick={() => dispatch({ type: 'TOGGLE_TODO_DOCK' })}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            dispatch({ type: 'TOGGLE_TODO_DOCK' });
          }
        }}
        className={cn(
          'flex h-9 shrink-0 cursor-pointer items-center justify-between px-3',
          'border-b border-[var(--border-default)] transition-colors duration-150',
          'hover:bg-[var(--bg-hover)]',
          planMode && !state.todoDockOpen && 'bg-[var(--plan-bg)]',
        )}
      >
        <div className="flex min-w-0 items-center gap-1.5">
          {state.todoDockOpen ? (
            <ChevronDown size={12} className="shrink-0 text-[var(--text-secondary)]" />
          ) : (
            <ChevronRight size={12} className="shrink-0 text-[var(--text-secondary)]" />
          )}
          <span className="text-xs font-semibold text-[var(--text-primary)]">Todo</span>
          {planMode && (
            <span
              className={cn(
                'rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide',
                'bg-[var(--plan-bg)] text-[var(--plan-accent)]',
                'border border-[var(--plan-border)]',
              )}
            >
              Plan
            </span>
          )}
          <span className="text-[11px] font-medium text-[var(--text-tertiary)]">
            ({state.todos.filter((t) => !t.completed).length})
          </span>
        </div>

        <button
          type="button"
          title="添加待办"
          onClick={(e) => {
            e.stopPropagation();
            openDock();
            setAdding(true);
          }}
          className={cn(
            'flex h-6 items-center gap-0.5 rounded px-1.5',
            'text-xs font-medium text-[var(--accent-primary)] transition-colors duration-150',
            'hover:bg-[var(--bg-hover)]',
          )}
        >
          <Plus size={12} />
          <span>添加</span>
        </button>
      </div>

      {state.todoDockOpen && (
        <div className="flex min-h-0 flex-1 flex-col">
          <div className="custom-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-1">
            {state.todos.length === 0 && !adding && (
              <div className="flex h-8 items-center justify-center">
                <p className="text-[11px] text-[var(--text-tertiary)]">
                  暂无待办，点击 + 添加
                </p>
              </div>
            )}
            {state.todos.map((todo, index) => (
              <TodoRow
                key={todo.id}
                text={todo.text}
                completed={todo.completed}
                active={planMode && index === activeIndex && !todo.completed}
                onToggle={() => handleToggle(todo)}
                onDelete={() => handleDelete(todo.id)}
                showDelete
              />
            ))}
          </div>

          <TodoAddInline
            visible={adding}
            onAdd={handleAdd}
            onCancel={() => setAdding(false)}
          />
        </div>
      )}
    </div>
  );
}
