import { useCallback } from 'react';
import { RefreshCw, Loader2, LogIn } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import { useMcpServers } from '@/hooks/useMcpServers';
import { startMcpOauthLogin } from '@/lib/mcpOAuth';
import type { McpAuthStatus } from '@/generated/app-server/v2/McpAuthStatus';
import type { McpServerStartupState } from '@/generated/app-server/v2/McpServerStartupState';
import { mcpStatusColors, settingsTheme } from '../settingsTheme';

const statusColors: Record<McpServerStartupState, string> = mcpStatusColors;

const statusLabels: Record<McpServerStartupState, string> = {
  starting: '启动中',
  ready: '就绪',
  failed: '失败',
  cancelled: '已取消',
};

const authLabels: Record<McpAuthStatus, string> = {
  unsupported: '无需认证',
  notLoggedIn: '未登录',
  bearerToken: 'Token',
  oAuth: 'OAuth',
};

export default function McpServersSettings() {
  const { state, dispatch } = useAppStore();
  const rpcReady =
    state.connectionStatus === 'connected' &&
    typeof window !== 'undefined' &&
    !!(window as unknown as Record<string, unknown>).__TAURI__;

  const { servers, loading, error, refresh, reloadConfig } = useMcpServers(rpcReady);

  const handleOAuthLogin = useCallback(
    async (serverName: string) => {
      try {
        const waiting = await startMcpOauthLogin(serverName);
        dispatch({ type: 'SET_OAUTH_WAITING', payload: waiting });
      } catch (err) {
        dispatch({
          type: 'SET_OAUTH_WAITING',
          payload: {
            serverName,
            phase: 'failed',
            error: err instanceof Error ? err.message : String(err),
          },
        });
      }
    },
    [dispatch],
  );

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className={settingsTheme.pageTitle}>MCP 服务器</h1>
          <p className="text-xs text-[var(--text-secondary)] mt-1">
            与终端共用 config.toml，修改后请重新加载
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => void reloadConfig()}
            disabled={!rpcReady || loading}
            className="h-8 px-3 flex items-center gap-1.5 bg-[var(--bg-hover)] hover:bg-[var(--bg-active)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] text-xs rounded-lg transition-colors duration-150 disabled:opacity-50"
          >
            <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
            重新加载
          </button>
          <button
            type="button"
            onClick={() => void refresh()}
            disabled={!rpcReady || loading}
            className="h-8 px-3 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] disabled:opacity-50"
          >
            刷新列表
          </button>
        </div>
      </div>

      {!rpcReady && (
        <p className="text-sm text-[var(--status-warning)] mb-4">
          请在桌面端连接 app-server 后查看 MCP 状态（与 TUI 配置同步）。
        </p>
      )}

      {error && (
        <p className="text-sm text-[var(--status-error)] mb-4">{error}</p>
      )}

      <h3 className="text-[13px] font-semibold text-[var(--text-primary)] mb-2">已配置服务器</h3>
      <div className="space-y-2 mb-6">
        {loading && servers.length === 0 && (
          <div className="flex items-center gap-2 text-sm text-[var(--text-secondary)] py-4">
            <Loader2 size={16} className="animate-spin" />
            加载中…
          </div>
        )}
        {!loading && servers.length === 0 && (
          <p className="text-sm text-[var(--text-secondary)] py-4">
            暂无 MCP 服务器。请在 ~/.bcip/config.toml 中配置后重新加载。
          </p>
        )}
        {servers.map((server) => (
          <div
            key={server.name}
            className="flex items-center justify-between p-3 bg-[var(--bg-base)] rounded-lg border border-[var(--border-default)]"
          >
            <div className="flex items-center gap-3 min-w-0">
              {server.startupStatus === 'starting' ? (
                <Loader2
                  size={14}
                  className="animate-spin shrink-0"
                  style={{ color: statusColors.starting }}
                />
              ) : (
                <div
                  className="w-2.5 h-2.5 rounded-full shrink-0"
                  style={{ backgroundColor: statusColors[server.startupStatus] }}
                />
              )}
              <div className="min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-sm font-medium text-[var(--text-primary)] font-mono">
                    {server.name}
                  </span>
                  <span className="text-[11px] px-1.5 py-0.5 rounded-full bg-[var(--bg-hover)] text-[var(--text-secondary)]">
                    {statusLabels[server.startupStatus]}
                  </span>
                  <span className="text-[11px] text-[var(--text-secondary)]">
                    {authLabels[server.authStatus]}
                  </span>
                </div>
                {server.startupError && (
                  <p className="text-xs text-[var(--status-error)] mt-0.5 truncate">
                    {server.startupError}
                  </p>
                )}
              </div>
            </div>
            <div className="flex items-center gap-2 shrink-0">
              {server.toolCount > 0 && (
                <span className="text-[11px] text-[var(--text-secondary)] font-mono">
                  {server.toolCount} tools
                </span>
              )}
              {(server.authStatus === 'oAuth' || server.authStatus === 'notLoggedIn') && (
                <button
                  type="button"
                  onClick={() => void handleOAuthLogin(server.name)}
                  className="h-7 px-2 flex items-center gap-1 text-xs text-[var(--accent-primary)] hover:bg-[var(--accent-primary-muted)] rounded-md transition-colors"
                >
                  <LogIn size={12} />
                  登录
                </button>
              )}
            </div>
          </div>
        ))}
      </div>

      <p className="text-xs text-[var(--text-secondary)]">
        高级：直接编辑{' '}
        <span className="font-mono text-[var(--text-secondary)]">~/.bcip/config.toml</span>
        {' '}中的 MCP 段落后点击「重新加载」。
      </p>
    </div>
  );
}
