/**
 * AgentFooter —— Agent 面板底部
 * 高度 32px
 * 左侧：ConnectionStatus（图标 + 简短文字）
 * 右侧：ModelBadge（点击跳转设置）
 */
import { cn } from '@/lib/utils';
import {
  CheckCircle2,
  Loader2,
  XCircle,
  AlertCircle,
} from 'lucide-react';
import type { ConnectionStatus } from '@/types';

interface AgentFooterProps {
  connectionStatus?: ConnectionStatus;
  modelName?: string;
  errorHint?: string | null;
  onRetry?: () => void;
  onOpenSettings?: () => void;
}

const statusConfig = {
  connected: {
    icon: CheckCircle2,
    text: '已连接',
    color: 'text-[var(--status-success)]',
    dotColor: 'bg-[var(--status-success)]',
  },
  connecting: {
    icon: Loader2,
    text: '连接中...',
    color: 'text-[var(--status-warning)]',
    dotColor: 'bg-[var(--status-warning)]',
  },
  disconnected: {
    icon: XCircle,
    text: '未连接',
    color: 'text-[var(--text-tertiary)]',
    dotColor: 'bg-[var(--text-tertiary)]',
  },
  error: {
    icon: AlertCircle,
    text: '连接失败',
    color: 'text-[var(--status-error)]',
    dotColor: 'bg-[var(--status-error)]',
  },
};

export default function AgentFooter({
  connectionStatus = 'connected',
  modelName = 'GPT-5.5 High',
  errorHint,
  onRetry,
  onOpenSettings,
}: AgentFooterProps) {
  const config = statusConfig[connectionStatus];
  const Icon = config.icon;
  const isConnecting = connectionStatus === 'connecting';

  return (
    <footer
      className={cn(
        'chat-column h-8 shrink-0 flex items-center justify-between',
        'bg-[var(--bg-surface)]/50',
        'border-t border-[var(--border-default)]',
        'text-2xs'
      )}
    >
      {/* 左侧：连接状态 */}
      <div
        className={cn(
          'flex items-center gap-1.5',
          'text-[var(--text-secondary)]'
        )}
      >
        {/* 状态圆点 / 图标 */}
          {isConnecting ? (
          <Icon
            size={12}
            className={cn('animate-spin', config.color)}
          />
        ) : (
          <span
            className={cn(
              'w-1.5 h-1.5 rounded-full',
              connectionStatus === 'connected' && 'animate-pulse',
              config.dotColor
            )}
          />
        )}
        <span className={config.color}>{config.text}</span>
        {errorHint && (
          <span
            className="ml-1 max-w-[200px] truncate text-[var(--status-error)]"
            title={errorHint}
          >
            {errorHint}
          </span>
        )}
        {onRetry &&
          (connectionStatus === 'disconnected' ||
            connectionStatus === 'error' ||
            errorHint) && (
          <button
            type="button"
            onClick={onRetry}
            className="ml-1 text-[var(--text-link)] hover:underline shrink-0"
          >
            重试
          </button>
        )}
      </div>

      {/* 右侧：模型徽章 */}
      <button
        type="button"
        onClick={onOpenSettings}
        className={cn(
          'px-2 py-0.5 rounded-full',
          'bg-[var(--bg-elevated)]',
          'text-[var(--text-secondary)] hover:text-[var(--text-primary)]',
          'border border-[var(--border-default)]',
          'hover:border-[var(--border-hover)]',
          'transition-all duration-normal'
        )}
      >
        {modelName}
      </button>
    </footer>
  );
}
