/**
 * dev mock 环境下预置演示线程，供新建任务侧面板与 Agent 面板使用。
 */
import { useEffect, useRef } from 'react';
import { mockThreads } from '@/data/desktopMockMessages';
import { isDevMockEnv } from '@/lib/devMock';
import { useAppStore } from '@/hooks/useAppStore';

export function useDevMockBootstrap(): void {
  const { state, dispatch } = useAppStore();
  const seededRef = useRef(false);

  useEffect(() => {
    if (!isDevMockEnv() || seededRef.current) {
      return;
    }
    if (state.threads.length > 0) {
      seededRef.current = true;
      return;
    }
    seededRef.current = true;
    dispatch({ type: 'SET_THREADS', payload: mockThreads });
    dispatch({ type: 'SET_CURRENT_THREAD', payload: mockThreads[0]?.id ?? null });
  }, [dispatch, state.threads.length]);
}
