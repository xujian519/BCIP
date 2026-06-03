/**
 * 启动后预热 app-server 会话：thread/list + thread/start（60s 超时）
 */
import type { Dispatch } from 'react';
import type { ThreadListResponse } from '@/generated/app-server/v2/ThreadListResponse';
import type { ThreadReadResponse } from '@/generated/app-server/v2/ThreadReadResponse';
import type { ThreadStartResponse } from '@/generated/app-server/v2/ThreadStartResponse';
import type { Thread as ApiThread } from '@/generated/app-server/v2/Thread';
import { getAppServerClient } from '@/lib/appServerClient';
import { messagesFromThreadHistory } from '@/lib/loadThreadHistory';
import { tryResumeThread } from '@/lib/threadResume';
import type { AppAction, Thread as UiThread } from '@/types';

const WARMUP_TIMEOUT_MS = 60_000;

function threadFromApi(t: ApiThread): UiThread {
  return {
    id: t.id,
    title: t.preview?.slice(0, 48) || '未命名线程',
    preview: t.preview ?? '',
    timestamp: t.updatedAt * 1000,
    status: 'active',
  };
}

function withTimeout<T>(promise: Promise<T>, label: string): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`${label}超时（${WARMUP_TIMEOUT_MS / 1000}s）`));
    }, WARMUP_TIMEOUT_MS);
    promise
      .then((value) => {
        clearTimeout(timer);
        resolve(value);
      })
      .catch((err) => {
        clearTimeout(timer);
        reject(err);
      });
  });
}

export interface SessionWarmupResult {
  threadId: string | null;
  elapsedMs: number;
}

/** Boot 与 Agent 面板共用：确保 60s 内完成线程列表 + 活动线程 */
export async function warmupAppServerSession(
  dispatch: Dispatch<AppAction>,
  projectCwd: string | null,
): Promise<SessionWarmupResult> {
  const started = Date.now();
  const client = getAppServerClient();
  if (!client.isInitialized()) {
    return { threadId: null, elapsedMs: 0 };
  }

  const list = await withTimeout(
    client.request<ThreadListResponse>('thread/list', {
      limit: 30,
      archived: false,
      cwd: projectCwd ?? null,
      useStateDbOnly: true,
    }),
    '线程列表',
  );

  dispatch({
    type: 'SET_THREADS',
    payload: list.data.map(threadFromApi),
  });

  let threadId: string | null = null;

  if (list.data.length > 0) {
    threadId = list.data[0].id;
    dispatch({ type: 'SET_CURRENT_THREAD', payload: threadId });
    const resumed = await withTimeout(
      tryResumeThread(threadId, projectCwd),
      '恢复线程',
    );
    if (resumed) {
      const read = await withTimeout(
        client.request<ThreadReadResponse>('thread/read', {
          threadId,
          includeTurns: true,
        }),
        '读取线程',
      );
      dispatch({
        type: 'SET_MESSAGES',
        payload: messagesFromThreadHistory(read.thread),
      });
    } else {
      dispatch({ type: 'SET_MESSAGES', payload: [] });
    }
  } else {
    const res = await withTimeout(
      client.request<ThreadStartResponse>('thread/start', {
        cwd: projectCwd ?? null,
      }),
      '创建线程',
    );
    threadId = res.thread.id;
    dispatch({ type: 'SET_CURRENT_THREAD', payload: threadId });
    if (res.model) {
      dispatch({ type: 'SET_CURRENT_MODEL', payload: res.model });
    }
    const thread: UiThread = {
      id: res.thread.id,
      title: res.thread.preview?.slice(0, 48) || '新线程',
      preview: res.thread.preview ?? '',
      timestamp: res.thread.createdAt * 1000,
      status: 'active',
    };
    dispatch({ type: 'SET_THREADS', payload: [thread] });
  }

  return { threadId, elapsedMs: Date.now() - started };
}
