import { useCallback, useEffect, useState } from 'react';
import {
  getAppServerClient,
  type ConnectionStatus,
  type JsonRpcNotification,
} from '@/lib/appServerClient';

function isTauri(): boolean {
  return (
    typeof window !== 'undefined' &&
    !!(window as unknown as Record<string, unknown>).__TAURI__
  );
}

export function useAppServer() {
  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [lastError, setLastError] = useState<string | null>(null);
  const [initialized, setInitialized] = useState(false);

  const connect = useCallback(async () => {
    if (!isTauri()) {
      setStatus('error');
      setLastError('需要桌面端环境才能连接 app-server');
      return;
    }
    setLastError(null);
    const client = getAppServerClient({
      onStatusChange: setStatus,
      onTransportError: (message) => setLastError(message),
    });
    try {
      await client.connect();
      setInitialized(client.isInitialized());
    } catch (err) {
      setLastError(err instanceof Error ? err.message : String(err));
      setInitialized(false);
    }
  }, []);

  const disconnect = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    await getAppServerClient().disconnect();
    setInitialized(false);
  }, []);

  useEffect(() => {
    return () => {
      if (isTauri()) {
        void getAppServerClient().disconnect();
      }
    };
  }, []);

  const client = getAppServerClient();

  return {
    status,
    lastError,
    initialized: initialized && client.isInitialized(),
    connect,
    disconnect,
    client,
  };
}

export function useAppServerNotifications(
  handler: (notification: JsonRpcNotification) => void,
) {
  useEffect(() => {
    const client = getAppServerClient({
      onNotification: handler,
    });
    return () => {
      void client;
    };
  }, [handler]);
}
