/**
 * MCP 服务器状态：mcpServerStatus/list、reload、startup 通知
 */
import { useCallback, useEffect, useMemo, useState } from 'react';
import type { ListMcpServerStatusResponse } from '@/generated/app-server/v2/ListMcpServerStatusResponse';
import type { McpAuthStatus } from '@/generated/app-server/v2/McpAuthStatus';
import type { McpServerStatus } from '@/generated/app-server/v2/McpServerStatus';
import type { McpServerStartupState } from '@/generated/app-server/v2/McpServerStartupState';
import type { McpServerStatusUpdatedNotification } from '@/generated/app-server/v2/McpServerStatusUpdatedNotification';
import type { JsonRpcNotification } from '@/lib/appServerClient';
import { getAppServerClient } from '@/lib/appServerClient';

export interface McpServerRow {
  name: string;
  toolCount: number;
  authStatus: McpAuthStatus;
  startupStatus: McpServerStartupState;
  startupError: string | null;
}

function toRow(
  server: McpServerStatus,
  startup: Map<string, { status: McpServerStartupState; error: string | null }>,
): McpServerRow {
  const boot = startup.get(server.name);
  return {
    name: server.name,
    toolCount: Object.keys(server.tools).length,
    authStatus: server.authStatus,
    startupStatus: boot?.status ?? 'ready',
    startupError: boot?.error ?? null,
  };
}

export function useMcpServers(enabled: boolean) {
  const [listData, setListData] = useState<McpServerStatus[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [startupMap, setStartupMap] = useState(
    () => new Map<string, { status: McpServerStartupState; error: string | null }>(),
  );

  const servers = useMemo(
    () => listData.map((s) => toRow(s, startupMap)),
    [listData, startupMap],
  );

  const refresh = useCallback(async () => {
    if (!enabled) {
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
      const res = await client.request<ListMcpServerStatusResponse>(
        'mcpServerStatus/list',
        { detail: 'toolsAndAuthOnly', limit: 100 },
      );
      setListData(res.data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [enabled]);

  const reloadConfig = useCallback(async () => {
    if (!enabled) {
      return;
    }
    const client = getAppServerClient();
    await client.request('config/mcpServer/reload', {});
    await refresh();
  }, [enabled, refresh]);

  useEffect(() => {
    if (!enabled) {
      return;
    }
    // eslint-disable-next-line react-hooks/set-state-in-effect -- data fetching hook
    void refresh();
  }, [enabled, refresh]);

  useEffect(() => {
    if (!enabled) {
      return;
    }
    const client = getAppServerClient();
    const onNotification = (notification: JsonRpcNotification) => {
      if (notification.method !== 'mcpServer/startupStatus/updated') {
        return;
      }
      const p = notification.params as McpServerStatusUpdatedNotification;
      setStartupMap((prev) => {
        const next = new Map(prev);
        next.set(p.name, { status: p.status, error: p.error });
        return next;
      });
    };
    client.mergeHandlers({ onNotification });
  }, [enabled]);

  return { servers, loading, error, refresh, reloadConfig };
}
