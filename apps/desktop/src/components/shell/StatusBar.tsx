/**
 * StatusBar —— 底部状态栏
 *
 * 精确设计参数：
 * - 高度：40px (h-statusbar)
 * - 背景：bg-surface/80 backdrop-blur-md
 * - 顶部边框：border-t border-border-subtle
 * - 左侧 ConnectionChip：状态圆点 6px + 状态文字
 *   - 已连接：bg-conn-connected 圆点 + "已连接 · 共用终端配置"
 *   - 连接中：bg-conn-connecting 圆点 + 旋转动画 + "连接中..."
 *   - 断开：bg-conn-disconnected 圆点 + "已断开"
 *   - 字体：text-2xs text-secondary
 * - 中间 UsageMeter：格式 "费用 ████░░ $2.35/50"
 *   - 进度条：h-1.5 w-24 rounded-full bg-border-subtle overflow-hidden
 *   - 已用部分：bg-brand-500 rounded-full
 *   - 文字：text-2xs text-secondary
 * - 右侧 ModelChip：pill 形状 + 主题切换按钮
 */
import { useAppStore } from '@/hooks/useAppStore';
import { cn } from '@/lib/utils';
import { Sun, Moon, Wifi, WifiOff, Loader2, Terminal, MoreHorizontal } from 'lucide-react';
import type { ConnectionStatus } from '@/types';

// ========================================
// ConnectionChip —— 连接状态芯片
// ========================================

/** 连接状态配置 */
const connectionConfig: Record<
  ConnectionStatus,
  {
    dotColor: string;
    label: string;
    description: string;
    icon: React.ReactNode;
  }
> = {
  connected: {
    dotColor: 'bg-conn-connected',
    label: '已连接',
    description: '共用终端配置',
    icon: <Wifi size={12} className="text-conn-connected" />,
  },
  connecting: {
    dotColor: 'bg-conn-connecting',
    label: '连接中',
    description: '...',
    icon: (
      <Loader2 size={12} className="text-conn-connecting animate-spin" />
    ),
  },
  disconnected: {
    dotColor: 'bg-conn-disconnected',
    label: '已断开',
    description: '点击重连',
    icon: <WifiOff size={12} className="text-conn-disconnected" />,
  },
  error: {
    dotColor: 'bg-conn-disconnected',
    label: '已断开',
    description: '连接失败',
    icon: <WifiOff size={12} className="text-conn-disconnected" />,
  },
};

/** 连接状态芯片 */
function ConnectionChip({
  status,
  transport,
  bcipSource,
}: {
  status: ConnectionStatus;
  transport: string | null;
  bcipSource: 'path' | 'sidecar' | 'workspace' | null;
}) {
  const config = connectionConfig[status];
  const isConnecting = status === 'connecting';
  const transportLabel =
    transport === 'proxy'
      ? 'proxy'
      : transport === 'stdio'
        ? 'stdio'
        : null;
  const bcipLabel =
    bcipSource === 'sidecar'
      ? 'bcip·内置'
      : bcipSource === 'path'
        ? 'bcip·PATH'
        : null;

  return (
    <div
      className={cn(
        'flex items-center gap-1.5 px-2 py-1 rounded-full',
        'hover:bg-[var(--bg-hover)]',
        'transition-colors duration-normal cursor-pointer'
      )}
    >
      {/* 状态圆点 8px */}
      <span
        className={cn(
          'h-2 w-2 rounded-full shrink-0',
          config.dotColor,
          isConnecting && 'animate-pulse'
        )}
      />

      {/* 图标 */}
      {config.icon}

      {/* 状态文字 + 描述 */}
      <span className="text-2xs text-[var(--text-secondary)] whitespace-nowrap">
        <span className="font-medium">{config.label}</span>
        {status === 'connected' && (
          <span className="text-[var(--text-tertiary)]">
            {' · '}
            {config.description}
            {transportLabel ? ` · ${transportLabel}` : ''}
            {bcipLabel ? ` · ${bcipLabel}` : ''}
          </span>
        )}
      </span>
    </div>
  );
}

// ========================================
// UsageMeter —— 用量条
// ========================================

function formatCompact(n: number): string {
  if (n >= 1_000_000) {
    return `${(n / 1_000_000).toFixed(1)}M`;
  }
  if (n >= 1000) {
    return `${(n / 1000).toFixed(1)}k`;
  }
  return String(Math.round(n));
}

function UsageMeter({
  label,
  used,
  max,
  hint,
}: {
  label: string;
  used: number;
  max: number;
  hint?: string;
}) {
  const percentage = Math.min((used / max) * 100, 100);
  const isPercentQuota = max === 100 && label === '额度';

  // 根据用量百分比确定填充色
  const fillColorClass =
    percentage > 95
      ? 'bg-status-error'
      : percentage > 80
        ? 'bg-status-warning'
        : 'bg-brand-500';

  return (
    <div className="flex items-center gap-2">
      {/* 标签 */}
      <span className="text-2xs text-[var(--text-secondary)] whitespace-nowrap">
        {label}
      </span>

      {/* 进度条容器 */}
      <div className="h-1.5 w-[60px] rounded-full bg-[var(--bg-hover)] overflow-hidden">
        {/* 已用部分 */}
        <div
          className={cn('h-full rounded-full transition-all duration-normal', fillColorClass)}
          style={{ width: `${percentage}%` }}
        />
      </div>

      {/* 数值 */}
      <span className="text-2xs text-[var(--text-secondary)] font-mono whitespace-nowrap">
        {isPercentQuota
          ? `${used.toFixed(0)}%`
          : hint && label === '余额'
            ? hint
            : `${formatCompact(used)}/${formatCompact(max)}`}
      </span>
    </div>
  );
}

// ========================================
// ModelChip —— 模型芯片
// ========================================

function ModelChip({
  model,
  streaming,
  onOpenSettings,
}: {
  model: string;
  streaming?: boolean;
  onOpenSettings: () => void;
}) {
  // 简化模型名显示
  const displayName = model.includes('claude')
    ? 'claude-sonnet'
    : model.includes('gpt')
      ? 'gpt-5.x'
      : model.slice(0, 10);

  return (
    <button
      type="button"
      onClick={onOpenSettings}
      className={cn(
        'flex items-center gap-1.5 px-2 py-0.5 rounded-full',
        streaming
          ? 'bg-[rgba(58,139,140,0.15)] border border-[var(--plan-border)] text-[var(--accent-cyan)]'
          : 'bg-[var(--bg-hover)] border border-[var(--border-default)] text-[var(--text-secondary)]',
        'text-2xs',
        'cursor-pointer hover:bg-[var(--bg-active)] hover:text-[var(--text-primary)]',
        'transition-colors duration-normal',
      )}
      title="打开模型设置"
      aria-label="当前模型，点击打开设置"
    >
      <Terminal size={12} />
      <span className="font-mono font-medium whitespace-nowrap">{displayName}</span>
    </button>
  );
}

// ========================================
// StatusBarActions —— §7.4.5
// ========================================

function StatusBarActions({
  terminalOpen,
  onToggleTerminal,
  onOpenCommandPalette,
  onToggleTheme,
  isDark,
}: {
  terminalOpen: boolean;
  onToggleTerminal: () => void;
  onOpenCommandPalette: () => void;
  onToggleTheme: () => void;
  isDark: boolean;
}) {
  return (
    <div className="flex items-center gap-1">
      <button
        type="button"
        onClick={onToggleTerminal}
        className={cn(
          'flex h-7 w-7 items-center justify-center rounded-md font-mono text-xs',
          terminalOpen
            ? 'bg-[var(--bg-active)] text-[var(--accent-primary)]'
            : 'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
          'transition-all duration-normal',
        )}
        title="切换终端 (⌘⇧J)"
        aria-label="切换终端"
      >
        &gt;_
      </button>
      <button
        type="button"
        onClick={onOpenCommandPalette}
        className={cn(
          'flex h-7 w-7 items-center justify-center rounded-md',
          'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
          'transition-all duration-normal',
        )}
        title="更多命令 (⌘⇧P)"
        aria-label="打开命令面板"
      >
        <MoreHorizontal size={16} />
      </button>
      <button
        type="button"
        onClick={onToggleTheme}
        className={cn(
          'flex h-7 w-7 items-center justify-center rounded-md',
          'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
          'transition-all duration-normal',
        )}
        title={isDark ? '切换到浅色模式' : '切换到深色模式'}
      >
        {isDark ? <Sun size={16} /> : <Moon size={16} />}
      </button>
    </div>
  );
}

// ========================================
// 主状态栏
// ========================================

export default function StatusBar() {
  const { state, dispatch } = useAppStore();

  return (
    <footer
      className={cn(
        'h-statusbar flex items-center justify-between shrink-0 px-3',
        'bg-[var(--bg-surface)]/80 backdrop-blur-md',
        'border-t border-[var(--border-default)]',
        'select-none z-50'
      )}
    >
      {/* 左侧：连接状态 */}
      <div className="flex items-center">
        <ConnectionChip
          status={state.connectionStatus}
          transport={state.appServerTransport}
          bcipSource={state.bcipSource}
        />
      </div>

      {/* 中间：用量条 */}
      <div className="flex items-center min-w-0">
        {state.usageMeter ? (
          <UsageMeter
            label={state.usageMeter.label}
            used={state.usageMeter.used}
            max={state.usageMeter.max}
            hint={state.usageMeter.hint}
          />
        ) : state.connectionStatus === 'connected' ? (
          <span className="text-2xs text-[var(--text-tertiary)]">用量同步中…</span>
        ) : null}
      </div>

      {/* 右侧：模型 + 操作区 */}
      <div className="flex items-center gap-1">
        <ModelChip
          model={state.currentModel}
          streaming={state.isStreaming}
          onOpenSettings={() =>
            dispatch({ type: 'OPEN_SETTINGS', payload: 'model' })
          }
        />
        <StatusBarActions
          terminalOpen={state.terminalOverlayOpen}
          onToggleTerminal={() => dispatch({ type: 'TOGGLE_TERMINAL_OVERLAY' })}
          onOpenCommandPalette={() => dispatch({ type: 'TOGGLE_COMMAND_PALETTE' })}
          onToggleTheme={() => dispatch({ type: 'TOGGLE_DARK' })}
          isDark={state.isDark}
        />
      </div>
    </footer>
  );
}
