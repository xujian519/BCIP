/**
 * 统一 app-server 连接：写入全局 store，避免多处 useAppServer 状态分叉
 */
import type { Dispatch } from 'react';
import { isTauri } from '@/api/tauri';
import { getAppServerClient } from '@/lib/appServerClient';
import type { AppAction } from '@/types';

export async function connectAppServer(
  dispatch: Dispatch<AppAction>,
): Promise<void> {
  if (!isTauri()) {
    dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'error' });
    throw new Error('需要桌面端环境才能连接 app-server');
  }

  dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'connecting' });

  const client = getAppServerClient({
    onStatusChange: (status) => {
      dispatch({ type: 'SET_CONNECTION_STATUS', payload: status });
    },
    onTransportError: () => {
      dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'error' });
    },
  });

  await client.connect();
  console.log('[connectAppServer] connect done, initialized:', client.isInitialized());

  if (!client.isInitialized()) {
    dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'error' });
    throw new Error('app-server 未初始化');
  }

  dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'connected' });
}

export async function disconnectAppServer(
  dispatch: Dispatch<AppAction>,
): Promise<void> {
  if (!isTauri()) {
    return;
  }
  await getAppServerClient().disconnect();
  dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'disconnected' });
}
