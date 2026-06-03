/**
 * Composer —— 输入区（像素级对标 Codex）
 * - 底部固定
 * - 背景：bg-elevated rounded-xl border border-border-subtle shadow-md
 * - 内边距：p-3
 * - 顶部行：附件按钮 + 权限指示器
 * - 输入区：auto-resize textarea
 * - SlashCommandPalette：输入 / 时弹出
 * - 底部行：模型选择器 + 发送按钮
 */
import { cn } from '@/lib/utils';
import {
  Paperclip,
  Shield,
  ArrowUp,
  Command,
  HelpCircle,
  BarChart3,
  Coins,
  PanelTopClose,
  Search,
  Microscope,
  FileEdit,
  ListTodo,
} from 'lucide-react';
import { useState, useRef, useEffect, useCallback, useMemo } from 'react';
import { BCIP_FOCUS_COMPOSER } from '@/lib/desktopEvents';
import { isPlanTodoId } from '@/lib/patentTodos';
import { patentSlashAction } from '@/lib/patentSlashCommands';
import { dispatchActivateWorkStage } from '@/lib/patentWorkflow';
import { useAppStore } from '@/hooks/useAppStore';

// ========================================
// Slash Command Palette 数据
// ========================================
interface SlashCommand {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
}

const slashCommands: SlashCommand[] = [
  {
    id: 'help',
    name: '/help',
    description: '显示帮助信息',
    icon: <HelpCircle size={16} />,
  },
  {
    id: 'status',
    name: '/status',
    description: '显示当前线程状态和用量',
    icon: <BarChart3 size={16} />,
  },
  {
    id: 'cost',
    name: '/cost',
    description: '显示费用统计',
    icon: <Coins size={16} />,
  },
  {
    id: 'compact',
    name: '/compact',
    description: '压缩对话上下文',
    icon: <PanelTopClose size={16} />,
  },
  {
    id: 'search',
    name: '/search',
    description: '搜索知识库',
    icon: <Search size={16} />,
  },
  {
    id: 'analyze',
    name: '/analyze',
    description: '分析当前文件',
    icon: <Microscope size={16} />,
  },
  {
    id: 'draft',
    name: '/draft',
    description: '起草专利文稿',
    icon: <FileEdit size={16} />,
  },
];

// ========================================
// SlashCommandPalette 组件
// ========================================
interface SlashCommandPaletteProps {
  query: string;
  onSelect: (command: SlashCommand) => void;
  onClose: () => void;
}

function SlashCommandPalette({ query, onSelect, onClose }: SlashCommandPaletteProps) {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);

  const filterText = query.slice(1).toLowerCase();
  const filtered = slashCommands.filter(
    (cmd) =>
      cmd.name.toLowerCase().includes(filterText) ||
      cmd.description.toLowerCase().includes(filterText)
  );

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1) % filtered.length);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 + filtered.length) % filtered.length);
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (filtered[selectedIndex]) {
          onSelect(filtered[selectedIndex]);
        }
      } else if (e.key === 'Escape') {
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [filtered, selectedIndex, onSelect, onClose]);

  // Scroll selected into view
  useEffect(() => {
    const el = itemRefs.current[selectedIndex];
    if (el) {
      el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  }, [selectedIndex]);

  if (filtered.length === 0) return null;

  return (
    <div
      ref={listRef}
      className={cn(
        'absolute bottom-full left-0 right-0 mb-2',
        'bg-[var(--bg-elevated)] rounded-lg',
        'border border-[var(--border-default)]',
        'shadow-lg',
        'max-h-[280px] overflow-y-auto',
        'z-50',
        'animate-slide-up'
      )}
      style={{
        animation: 'slideUp 150ms ease-out',
      }}
    >
      <div className="py-1">
        {filtered.map((cmd, index) => (
          <button
            key={cmd.id}
            ref={(el) => { itemRefs.current[index] = el; }}
            onClick={() => onSelect(cmd)}
            className={cn(
              'w-full flex items-center gap-3 px-3 py-2',
              'text-left transition-colors duration-fast',
              index === selectedIndex
                ? 'bg-[var(--bg-hover)]'
                : 'hover:bg-[var(--bg-hover)]',
              'cursor-pointer'
            )}
          >
            <span className="text-[var(--text-secondary)] shrink-0">{cmd.icon}</span>
            <div className="flex-1 min-w-0">
              <div className="text-xs font-medium text-[var(--text-primary)]">
                {cmd.name}
              </div>
              <div className="text-2xs text-[var(--text-tertiary)]">
                {cmd.description}
              </div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

// ========================================
// Composer 主组件
// ========================================
interface ComposerProps {
  onSend?: (message: string) => void;
  disabled?: boolean;
  /** 输入被禁用时向用户说明原因 */
  disabledReason?: string;
  placeholder?: string;
}

export default function Composer({
  onSend,
  disabled = false,
  disabledReason,
  placeholder = 'Ask Codex anything, @ to add files, / for commands',
}: ComposerProps) {
  const { state, dispatch } = useAppStore();
  const [text, setText] = useState('');
  const [isComposing, setIsComposing] = useState(false);
  const [slashClosedFor, setSlashClosedFor] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const showSlashPalette = useMemo(() => {
    if (isComposing) return false;
    if (!text.startsWith('/') || text.indexOf('\n') !== -1) return false;
    if (slashClosedFor === text) return false;
    return true;
  }, [text, isComposing, slashClosedFor]);

  useEffect(() => {
    const el = textareaRef.current;
    if (el) {
      el.style.height = 'auto';
      const newHeight = Math.min(Math.max(el.scrollHeight, 36), 200);
      el.style.height = `${newHeight}px`;
    }
  }, [text]);

  useEffect(() => {
    const onFocus = () => textareaRef.current?.focus();
    window.addEventListener(BCIP_FOCUS_COMPOSER, onFocus);
    return () => window.removeEventListener(BCIP_FOCUS_COMPOSER, onFocus);
  }, []);

  const handleSend = useCallback(() => {
    const trimmed = text.trim();
    if (!trimmed || disabled) return;
    onSend?.(trimmed);
    setText('');
    dispatch({ type: 'CLEAR_CHAT_MENTIONS' });
    setSlashClosedFor('');
    // Reset textarea height
    if (textareaRef.current) {
      textareaRef.current.style.height = '36px';
    }
  }, [text, disabled, onSend, dispatch]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (isComposing) return;

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (!showSlashPalette) {
        handleSend();
      }
    }
  };

  const handleSlashSelect = (command: SlashCommand) => {
    const patent = patentSlashAction(command.id);
    if (patent) {
      dispatchActivateWorkStage(dispatch, patent.stage);
      setText(patent.prompt);
    } else {
      setText(`${command.name} `);
    }
    setSlashClosedFor('');
    textareaRef.current?.focus();
  };

  const handleSlashClose = () => {
    setSlashClosedFor(text);
  };

  const hasContent = text.trim().length > 0;
  const planModeActive = state.todos.some((t) => isPlanTodoId(t.id));

  return (
    <div
      className={cn(
        'chat-column shrink-0 py-1.5',
        'relative',
        'border-t border-[var(--border-default)]',
        'bg-[var(--bg-surface)]',
      )}
    >
      <div
        className={cn(
          'bg-[var(--bg-composer)] rounded-2xl',
          'border border-[var(--border-default)]',
          'shadow-[var(--shadow-composer)]',
          'focus-within:border-[var(--border-focus)]',
          'focus-within:shadow-[var(--shadow-composer-focus)]',
          'focus-within:ring-2 focus-within:ring-[var(--accent-primary-muted)]',
          'transition-all duration-200',
          'relative',
        )}
        style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
      >
        {/* Slash Command Palette */}
        {showSlashPalette && (
          <SlashCommandPalette
            key={text}
            query={text}
            onSelect={handleSlashSelect}
            onClose={handleSlashClose}
          />
        )}

        {/* 顶部行：附件 + 权限 / Plan 模式 */}
        <div className="flex items-center gap-2 px-4 pt-3 pb-2">
          <button
            className={cn(
              'w-6 h-6 flex items-center justify-center rounded-md',
              'text-[var(--text-tertiary)] hover:text-[var(--text-primary)]',
              'hover:bg-[var(--bg-hover)]',
              'transition-colors duration-150'
            )}
            title="添加附件"
          >
            <Paperclip size={14} />
          </button>
          {planModeActive ? (
            <div
              className={cn(
                'flex items-center gap-1 rounded-full px-2 py-0.5',
                'text-[10px] font-semibold uppercase tracking-wide',
                'bg-[var(--plan-bg)] text-[var(--plan-accent)]',
                'border border-[var(--plan-border)]',
              )}
            >
              <ListTodo size={11} />
              <span>Plan mode</span>
            </div>
          ) : (
            <div
              className={cn(
                'flex items-center gap-1',
                'text-2xs text-[var(--status-warning)]'
              )}
            >
              <Shield size={12} />
              <span>Full access</span>
            </div>
          )}
        </div>

        {state.chatMentions.length > 0 && (
          <div className="flex flex-wrap gap-1 px-4 pb-2">
            {state.chatMentions.map((m, i) => (
              <span
                key={i}
                className="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-2xs"
                style={{
                  backgroundColor: 'var(--accent-primary-muted)',
                  color: 'var(--accent-primary)',
                  border: '1px solid rgba(var(--accent-primary-rgb, 74,124,111), 0.3)',
                }}
              >
                @{m.path.split('/').pop()}
              </span>
            ))}
          </div>
        )}

        {/* 输入区 */}
        <div className="px-4 py-1">
          <textarea
            ref={textareaRef}
            id="bcip-composer-input"
            aria-label="Agent 消息输入"
            value={text}
            onChange={(e) => setText(e.target.value)}
            onKeyDown={handleKeyDown}
            onCompositionStart={() => setIsComposing(true)}
            onCompositionEnd={() => setIsComposing(false)}
            placeholder={placeholder}
            rows={1}
            disabled={disabled}
            className={cn(
              'w-full bg-transparent outline-none ring-0',
              'text-sm text-[var(--text-primary)]',
              'placeholder:text-[var(--text-tertiary)]',
              'placeholder:transition-opacity placeholder:duration-200',
              'resize-none',
              'max-h-[200px]',
              'leading-relaxed',
              'disabled:opacity-50',
            )}
            style={{ minHeight: 'var(--chat-composer-min-h)' }}
          />
        </div>

        {disabled && disabledReason && (
          <p className="px-4 pb-2 text-2xs text-[var(--status-warning)]">
            {disabledReason}
          </p>
        )}

        {/* 底部行：模型选择器 + 发送按钮 */}
        <div className="flex items-center justify-between px-4 pb-3 pt-2">
          {/* 模型选择器 */}
          <button
            type="button"
            onClick={() => dispatch({ type: 'OPEN_SETTINGS', payload: 'model' })}
            className={cn(
              'flex items-center gap-1 px-2 py-0.5 rounded-full',
              'bg-[var(--bg-surface)]',
              'text-2xs text-[var(--text-secondary)]',
              'border border-[var(--border-default)]',
              'hover:border-[var(--border-hover)]',
              'hover:text-[var(--text-primary)]',
              'transition-all duration-150',
            )}
            title="切换模型"
            aria-label="当前模型"
          >
            <Command size={10} />
            <span className="max-w-[120px] truncate font-mono">
              {state.currentModel.split('/').pop() ?? state.currentModel}
            </span>
          </button>

          {/* 发送按钮 */}
          <button
            onClick={handleSend}
            disabled={!hasContent || disabled}
            className={cn(
              'w-7 h-7 flex items-center justify-center rounded-full',
              'transition-all duration-150',
              hasContent && !disabled
                ? 'bg-[var(--accent-primary)] text-white hover:bg-[var(--accent-primary-hover)]'
                : 'bg-[var(--border-default)] text-[var(--text-tertiary)]',
              'disabled:cursor-not-allowed',
              'active:scale-95'
            )}
          >
            <ArrowUp size={14} />
          </button>
        </div>
      </div>
    </div>
  );
}
