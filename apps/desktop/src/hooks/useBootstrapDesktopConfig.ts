/**
 * 连接 app-server 后应用 desktop.* 外观配置到 DOM / store
 */
import { useEffect } from 'react';
import type { ConfigReadResponse } from '@/generated/app-server/v2/ConfigReadResponse';
import { applyDesktopAppearance, applyThemeToDocument } from '@/lib/desktopAppearance';
import { configGet } from '@/lib/configAccess';
import { getAppServerClient } from '@/lib/appServerClient';
import type { AppAction } from '@/types';

export function useBootstrapDesktopConfig(
  rpcReady: boolean,
  workspaceCwd: string | null,
  dispatch: React.Dispatch<AppAction>,
): void {
  useEffect(() => {
    if (!rpcReady) {
      return;
    }
    const client = getAppServerClient();
    if (!client.isInitialized()) {
      return;
    }
    void (async () => {
      try {
        const res = await client.request<ConfigReadResponse>('config/read', {
          cwd: workspaceCwd,
        });
        const get = (path: string) => configGet(res.config, path);
        const theme = applyDesktopAppearance(get);
        if (theme) {
          applyThemeToDocument(theme);
          dispatch({ type: 'SET_THEME', payload: theme });
        }
      } catch {
        // 忽略启动时配置读取失败
      }
    })();
  }, [rpcReady, workspaceCwd, dispatch]);
}
