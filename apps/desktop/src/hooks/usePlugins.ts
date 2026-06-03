/**
 * plugin/list、plugin/install、plugin/uninstall（experimental API）
 */
import { useCallback, useEffect, useState } from 'react';
import type { PluginAvailability } from '@/generated/app-server/v2/PluginAvailability';
import type { PluginInstallPolicy } from '@/generated/app-server/v2/PluginInstallPolicy';
import type { PluginInstallResponse } from '@/generated/app-server/v2/PluginInstallResponse';
import type { PluginListResponse } from '@/generated/app-server/v2/PluginListResponse';
import type { PluginSummary } from '@/generated/app-server/v2/PluginSummary';
import type { PluginUninstallResponse } from '@/generated/app-server/v2/PluginUninstallResponse';
import type { MarketplaceLoadErrorInfo } from '@/generated/app-server/v2/MarketplaceLoadErrorInfo';
import { getAppServerClient } from '@/lib/appServerClient';

export interface PluginRow {
  id: string;
  name: string;
  displayName: string;
  description: string;
  marketplaceName: string;
  marketplacePath: string | null;
  remoteMarketplaceName: string | null;
  installed: boolean;
  availability: PluginAvailability;
  installPolicy: PluginInstallPolicy;
  brandColor: string | null;
  canInstall: boolean;
}

function toPluginRow(
  plugin: PluginSummary,
  marketplaceName: string,
  marketplacePath: string | null,
): PluginRow {
  const iface = plugin.interface;
  const displayName = iface?.displayName ?? plugin.name;
  const description =
    iface?.shortDescription ?? iface?.longDescription ?? '';
  const canInstall =
    plugin.availability === 'AVAILABLE' &&
    plugin.installPolicy !== 'NOT_AVAILABLE' &&
    !plugin.installed;
  return {
    id: plugin.id,
    name: plugin.name,
    displayName,
    description,
    marketplaceName,
    marketplacePath,
    remoteMarketplaceName: marketplacePath ? null : marketplaceName,
    installed: plugin.installed,
    availability: plugin.availability,
    installPolicy: plugin.installPolicy,
    brandColor: iface?.brandColor ?? null,
    canInstall,
  };
}

function flattenPlugins(response: PluginListResponse): {
  plugins: PluginRow[];
  errors: MarketplaceLoadErrorInfo[];
} {
  const plugins: PluginRow[] = [];
  const errors = [...response.marketplaceLoadErrors];
  for (const marketplace of response.marketplaces) {
    for (const plugin of marketplace.plugins) {
      plugins.push(
        toPluginRow(plugin, marketplace.name, marketplace.path),
      );
    }
  }
  return { plugins, errors };
}

export function usePlugins(rpcReady: boolean, workspaceCwd?: string | null) {
  const [plugins, setPlugins] = useState<PluginRow[]>([]);
  const [loadErrors, setLoadErrors] = useState<MarketplaceLoadErrorInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!rpcReady) {
      return;
    }
    const client = getAppServerClient();
    if (!client.isInitialized()) {
      setError('请先连接 app-server');
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const res = await client.request<PluginListResponse>('plugin/list', {
        cwds: workspaceCwd ? [workspaceCwd] : null,
        marketplaceKinds: null,
      });
      const flat = flattenPlugins(res);
      setPlugins(flat.plugins);
      setLoadErrors(flat.errors);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [rpcReady, workspaceCwd]);

  const installPlugin = useCallback(
    async (row: PluginRow) => {
      if (!rpcReady) {
        return;
      }
      setBusyId(row.id);
      setError(null);
      try {
        await getAppServerClient().request<PluginInstallResponse>(
          'plugin/install',
          {
            marketplacePath: row.marketplacePath,
            remoteMarketplaceName: row.remoteMarketplaceName,
            pluginName: row.name,
          },
        );
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setBusyId(null);
      }
    },
    [rpcReady, refresh],
  );

  const uninstallPlugin = useCallback(
    async (row: PluginRow) => {
      if (!rpcReady) {
        return;
      }
      setBusyId(row.id);
      setError(null);
      try {
        await getAppServerClient().request<PluginUninstallResponse>(
          'plugin/uninstall',
          { pluginId: row.id },
        );
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setBusyId(null);
      }
    },
    [rpcReady, refresh],
  );

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- data fetching hook
    void refresh();
  }, [refresh]);

  return {
    plugins,
    loadErrors,
    loading,
    error,
    busyId,
    refresh,
    installPlugin,
    uninstallPlugin,
  };
}
