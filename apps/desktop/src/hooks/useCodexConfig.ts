/**
 * config/read 与 config/value/write（与 TUI 共用 config.toml）
 */
import { useCallback, useEffect, useState } from 'react';
import type { Config } from '@/generated/app-server/v2/Config';
import type { ConfigReadResponse } from '@/generated/app-server/v2/ConfigReadResponse';
import type { JsonValue } from '@/generated/app-server/serde_json/JsonValue';
import type { MergeStrategy } from '@/generated/app-server/v2/MergeStrategy';
import { configGet } from '@/lib/configAccess';
import { getAppServerClient } from '@/lib/appServerClient';

export function useCodexConfig(enabled: boolean, cwd?: string | null) {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

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
      const res = await client.request<ConfigReadResponse>('config/read', {
        cwd: cwd ?? null,
      });
      setConfig(res.config);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [enabled, cwd]);

  const writeValue = useCallback(
    async (
      keyPath: string,
      value: JsonValue,
      mergeStrategy: MergeStrategy = 'replace',
    ) => {
      if (!enabled) {
        return;
      }
      const client = getAppServerClient();
      setSaving(true);
      setError(null);
      try {
        await client.request('config/value/write', {
          keyPath,
          value,
          mergeStrategy,
        });
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        throw err;
      } finally {
        setSaving(false);
      }
    },
    [enabled, refresh],
  );

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- data fetching hook, refresh contains async setState
    void refresh();
  }, [refresh]);

  const get = useCallback(
    (keyPath: string) => configGet(config, keyPath),
    [config],
  );

  return {
    config,
    loading,
    error,
    saving,
    refresh,
    writeValue,
    get,
  };
}
