/**
 * model/list — 动态模型目录（禁止硬编码模型表）
 */
import { useCallback, useEffect, useState } from 'react';
import type { Model } from '@/generated/app-server/v2/Model';
import type { ModelListResponse } from '@/generated/app-server/v2/ModelListResponse';
import { getAppServerClient } from '@/lib/appServerClient';

export function useModelCatalog(enabled: boolean) {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
      const all: Model[] = [];
      let cursor: string | null = null;
      for (;;) {
        const page: ModelListResponse = await client.request<ModelListResponse>(
          'model/list',
          {
          limit: 100,
          cursor,
            includeHidden: false,
          },
        );
        all.push(...page.data);
        cursor = page.nextCursor;
        if (!cursor) {
          break;
        }
      }
      setModels(all);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [enabled]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- data fetching hook
    void refresh();
  }, [refresh]);

  return { models, loading, error, refresh };
}

/** 按 config.model 字段匹配目录项 */
export function findModelByConfigValue(
  models: Model[],
  configModel: string | null | undefined,
): Model | undefined {
  if (!configModel) {
    return models.find((m) => m.isDefault) ?? models[0];
  }
  return (
    models.find((m) => m.model === configModel || m.id === configModel) ??
    models.find((m) => m.isDefault)
  );
}
