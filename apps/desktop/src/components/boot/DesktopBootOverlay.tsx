/**
 * 启动 / 接入浮层：检测 bcip、连接 app-server
 */
import { useState } from 'react';
import { Loader2, AlertCircle, Terminal, FileText, ChevronDown, ChevronUp } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import { cn } from '@/lib/utils';

interface DesktopBootOverlayProps {
  onRetry: () => void;
  onContinueFileMode: () => void;
}

export default function DesktopBootOverlay({
  onRetry,
  onContinueFileMode,
}: DesktopBootOverlayProps) {
  const { state } = useAppStore();
  const { bootPhase, bootError, bootLogLines } = state;
  const [logExpanded, setLogExpanded] = useState(false);

  if (bootPhase === 'ready' || bootPhase === 'idle') {
    return null;
  }

  const title =
    bootPhase === 'checking'
      ? '正在检测环境…'
      : bootPhase === 'connecting'
        ? '正在连接 app-server…'
        : bootPhase === 'no_cli'
          ? '未检测到 bcip CLI'
          : '无法连接 app-server';

  const subtitle =
    bootPhase === 'checking'
      ? '检查 PATH 与内置 sidecar 中的 bcip'
      : bootPhase === 'connecting'
        ? state.appServerTransport === 'proxy'
          ? '附着到已有终端 daemon（proxy）'
          : '启动本地 app-server（stdio）'
        : bootPhase === 'no_cli'
          ? '仍可浏览文件与预览；安装 bcip 或打包 sidecar 后可启用 Agent'
          : bootError ?? '请确认终端中已登录且 app-server 可用';

  const showSpinner = bootPhase === 'checking' || bootPhase === 'connecting';
  const showLogs =
    bootLogLines.length > 0 &&
    (bootPhase === 'connecting' || bootPhase === 'fault' || logExpanded);

  return (
    <div
      className={cn(
        'fixed inset-0 z-[180] flex items-center justify-center',
        'bg-[rgba(0,0,0,0.35)] backdrop-blur-sm',
      )}
      role="status"
      aria-live="polite"
    >
      <div
        className={cn(
          'mx-4 w-full max-w-md rounded-xl border p-6 shadow-lg',
          'border-[var(--border-default)] bg-[var(--bg-elevated)]',
        )}
      >
        <div className="flex items-start gap-4">
          <div
            className={cn(
              'flex h-10 w-10 shrink-0 items-center justify-center rounded-lg',
              bootPhase === 'no_cli'
                ? 'bg-[var(--status-warning)]/15 text-[var(--status-warning)]'
                : bootPhase === 'fault'
                  ? 'bg-[var(--status-error)]/15 text-[var(--status-error)]'
                  : 'bg-[var(--accent-primary-muted)] text-[var(--accent-primary)]',
            )}
          >
            {showSpinner ? (
              <Loader2 size={20} className="animate-spin" />
            ) : bootPhase === 'no_cli' ? (
              <FileText size={20} />
            ) : (
              <AlertCircle size={20} />
            )}
          </div>

          <div className="min-w-0 flex-1">
            <h2 className="text-base font-semibold text-[var(--text-primary)]">{title}</h2>
            <p className="mt-1 text-sm text-[var(--text-secondary)]">{subtitle}</p>

            {bootPhase === 'no_cli' && (
              <div className="mt-3 space-y-1 font-mono text-xs text-[var(--text-tertiary)]">
                <p>
                  安装后执行：
                  <span className="text-[var(--text-primary)]"> bcip --version</span>
                </p>
                <p>或将 bcip 放入应用 bundle：<span className="text-[var(--text-primary)]">Resources/bin/bcip</span></p>
              </div>
            )}

            {bootLogLines.length > 0 && (
              <div className="mt-3">
                <button
                  type="button"
                  onClick={() => setLogExpanded((v) => !v)}
                  className="flex items-center gap-1 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
                >
                  {logExpanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                  启动日志 ({bootLogLines.length})
                </button>
                {(showLogs || logExpanded) && (
                  <pre
                    className={cn(
                      'mt-2 max-h-32 overflow-auto rounded-md p-2 text-[10px] leading-relaxed',
                      'bg-[var(--bg-base)] text-[var(--text-tertiary)] border border-[var(--border-default)]',
                    )}
                  >
                    {bootLogLines.join('\n')}
                  </pre>
                )}
              </div>
            )}

            {bootPhase === 'no_cli' && (
              <div className="mt-4">
                <button
                  type="button"
                  onClick={onContinueFileMode}
                  className={cn(
                    'rounded-md border px-3 py-1.5 text-sm',
                    'border-[var(--border-default)] text-[var(--text-secondary)]',
                    'hover:bg-[var(--bg-hover)]',
                  )}
                >
                  仅文件模式
                </button>
              </div>
            )}

            {bootPhase === 'fault' && (
              <div className="mt-4 flex flex-wrap gap-2">
                <button
                  type="button"
                  onClick={onRetry}
                  className={cn(
                    'inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium',
                    'bg-[var(--accent-primary)] text-white hover:opacity-90',
                  )}
                >
                  <Terminal size={14} />
                  重试连接
                </button>
                <button
                  type="button"
                  onClick={onContinueFileMode}
                  className={cn(
                    'rounded-md border px-3 py-1.5 text-sm',
                    'border-[var(--border-default)] text-[var(--text-secondary)]',
                    'hover:bg-[var(--bg-hover)]',
                  )}
                >
                  仅文件模式
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
