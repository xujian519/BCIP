import { useMemo, useState } from 'react';
import { Search, Plus, Minus, Loader2, RefreshCw } from 'lucide-react';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useAppStore } from '@/hooks/useAppStore';
import { usePlugins } from '@/hooks/usePlugins';
import { isDesktopRpcReady } from '@/lib/configAccess';
import { SettingsRpcBanner } from '../SettingPrimitives';

type FilterMode = 'all' | 'installed' | 'available';

export default function PluginsSettings() {
  const { state } = useAppStore();
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const {
    plugins,
    loadErrors,
    loading,
    error,
    busyId,
    refresh,
    installPlugin,
    uninstallPlugin,
  } = usePlugins(rpcReady, state.workspaceCwd);

  const [search, setSearch] = useState('');
  const [filter, setFilter] = useState<FilterMode>('all');

  const filteredPlugins = useMemo(() => {
    const q = search.trim().toLowerCase();
    return plugins.filter((p) => {
      if (filter === 'installed' && !p.installed) {
        return false;
      }
      if (filter === 'available' && (p.installed || !p.canInstall)) {
        return false;
      }
      if (!q) {
        return true;
      }
      return (
        p.displayName.toLowerCase().includes(q) ||
        p.name.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q)
      );
    });
  }, [plugins, search, filter]);

  return (
    <div>
      <div className="flex items-center gap-3 mb-2">
        <h1 className="text-2xl font-semibold text-[var(--text-primary)]">插件</h1>
        <span className="px-1.5 py-0.5 text-[10px] font-semibold bg-[var(--status-warning)] text-[var(--text-primary)] rounded">
          Experimental
        </span>
        <button
          type="button"
          onClick={() => void refresh()}
          disabled={!rpcReady || loading}
          className="ml-auto h-8 px-3 flex items-center gap-1.5 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] disabled:opacity-50"
        >
          <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
          刷新
        </button>
      </div>

      <SettingsRpcBanner
        rpcReady={rpcReady}
        loading={loading}
        error={error}
        saving={busyId !== null}
      />

      <p className="text-xs text-[var(--text-secondary)] mb-4">
        数据来自 <span className="font-mono">plugin/list</span>。安装/卸载调用 experimental RPC，需 app-server 启用 plugins 特性。
      </p>

      {loadErrors.length > 0 && (
        <div className="mb-4 p-3 rounded-lg bg-[var(--status-error-bg)] border border-[var(--status-error)]/25">
          <p className="text-xs font-medium text-[var(--status-error)] mb-1">市场加载错误</p>
          {loadErrors.map((e) => (
            <p
              key={`${e.marketplacePath}-${e.message}`}
              className="text-[11px] text-[var(--text-secondary)] font-mono"
            >
              {e.marketplacePath}: {e.message}
            </p>
          ))}
        </div>
      )}

      <div className="flex items-center gap-2 mb-4">
        <div className="flex-1 relative">
          <Search
            size={16}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)]"
          />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="搜索插件..."
            className="w-full h-9 pl-9 pr-3 bg-[var(--bg-base)] rounded-lg border border-[var(--border-default)] text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)] focus:outline-none focus:border-[var(--border-focus)] transition-colors duration-150"
          />
        </div>
        <Select
          value={filter}
          onValueChange={(v) => setFilter(v as FilterMode)}
        >
          <SelectTrigger className="w-[100px] h-9 bg-[var(--bg-base)] border-[var(--border-default)] text-[var(--text-primary)] text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent className="bg-[var(--bg-base)] border-[var(--border-default)]">
            <SelectItem value="all" className="text-xs">
              全部
            </SelectItem>
            <SelectItem value="installed" className="text-xs">
              已安装
            </SelectItem>
            <SelectItem value="available" className="text-xs">
              可安装
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="grid grid-cols-2 gap-3">
        {!loading && filteredPlugins.length === 0 && (
          <p className="col-span-2 text-sm text-[var(--text-secondary)] py-8 text-center">
            未发现插件市场条目
          </p>
        )}
        {filteredPlugins.map((plugin) => {
          const busy = busyId === plugin.id;
          const iconColor = plugin.brandColor ?? 'var(--accent-primary)';
          return (
            <div
              key={`${plugin.marketplaceName}-${plugin.id}`}
              className="p-3 bg-[var(--bg-base)] rounded-lg border border-[var(--border-default)] flex items-start gap-3"
            >
              <div
                className="w-10 h-10 rounded-[10px] flex items-center justify-center shrink-0 text-sm font-semibold"
                style={{
                  backgroundColor: `${iconColor}22`,
                  color: iconColor,
                }}
              >
                {plugin.displayName.slice(0, 1).toUpperCase()}
              </div>
              <div className="flex-1 min-w-0">
                <span className="text-sm font-medium text-[var(--text-primary)]">
                  {plugin.displayName}
                </span>
                <p className="text-[11px] font-mono text-[var(--text-tertiary)]">
                  {plugin.name} · {plugin.marketplaceName}
                </p>
                {plugin.description && (
                  <p className="text-xs text-[var(--text-secondary)] mt-0.5 line-clamp-2">
                    {plugin.description}
                  </p>
                )}
                {plugin.availability === 'DISABLED_BY_ADMIN' && (
                  <p className="text-[11px] text-[var(--status-warning)] mt-1">
                    管理员已禁用
                  </p>
                )}
              </div>
              <button
                type="button"
                disabled={!rpcReady || busy}
                onClick={() =>
                  void (plugin.installed
                    ? uninstallPlugin(plugin)
                    : installPlugin(plugin))
                }
                className={`w-7 h-7 rounded-full flex items-center justify-center border transition-all duration-150 shrink-0 disabled:opacity-50 ${
                  plugin.installed
                    ? 'bg-[var(--accent-primary)] border-[var(--accent-primary)] text-white'
                    : 'border-[var(--border-hover)] text-[var(--text-secondary)] hover:border-[var(--border-focus)] hover:text-[var(--accent-primary)]'
                }`}
                title={plugin.installed ? '卸载' : '安装'}
              >
                {busy ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : plugin.installed ? (
                  <Minus size={14} />
                ) : (
                  <Plus size={14} />
                )}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
