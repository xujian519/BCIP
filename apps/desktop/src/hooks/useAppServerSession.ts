/**
 * app-server 会话：thread/start、turn/start 与 item/* 通知 → store
 */
import { useCallback, useEffect, useRef, useState } from 'react';
import type { AgentMessageDeltaNotification } from '@/generated/app-server/v2/AgentMessageDeltaNotification';
import type { ItemCompletedNotification } from '@/generated/app-server/v2/ItemCompletedNotification';
import type { ItemStartedNotification } from '@/generated/app-server/v2/ItemStartedNotification';
import type { ThreadListResponse } from '@/generated/app-server/v2/ThreadListResponse';
import type { ThreadStartResponse } from '@/generated/app-server/v2/ThreadStartResponse';
import type { TurnStartParams } from '@/generated/app-server/v2/TurnStartParams';
import type { TurnStartResponse } from '@/generated/app-server/v2/TurnStartResponse';
import type { UserInput } from '@/generated/app-server/v2/UserInput';
import type { ErrorNotification } from '@/generated/app-server/v2/ErrorNotification';
import type { TurnCompletedNotification } from '@/generated/app-server/v2/TurnCompletedNotification';
import type { TurnPlanUpdatedNotification } from '@/generated/app-server/v2/TurnPlanUpdatedNotification';
import type { ThreadTokenUsageUpdatedNotification } from '@/generated/app-server/v2/ThreadTokenUsageUpdatedNotification';
import type { AccountRateLimitsUpdatedNotification } from '@/generated/app-server/v2/AccountRateLimitsUpdatedNotification';
import type { FileChangePatchUpdatedNotification } from '@/generated/app-server/v2/FileChangePatchUpdatedNotification';
import type { McpServerOauthLoginCompletedNotification } from '@/generated/app-server/v2/McpServerOauthLoginCompletedNotification';
import type { ThreadArchiveResponse } from '@/generated/app-server/v2/ThreadArchiveResponse';
import type { ThreadReadResponse } from '@/generated/app-server/v2/ThreadReadResponse';
import { handleAppServerRequest } from '@/lib/appServerServerRequest';
import { messagesFromThreadHistory } from '@/lib/loadThreadHistory';
import {
  getAppServerClient,
  type ConnectionStatus,
  type JsonRpcNotification,
} from '@/lib/appServerClient';
import {
  markMessageToolsRunning,
  threadItemToMessage,
} from '@/lib/threadItemMapper';
import { warmupAppServerSession } from '@/lib/appServerSessionWarmup';
import { tryResumeThread } from '@/lib/threadResume';
import { todosFromPlanSteps, todosFromPlanText } from '@/lib/patentTodos';
import {
  dispatchActivateWorkStage,
  inferWorkStageFromPlanSteps,
  inferWorkStageFromText,
} from '@/lib/patentWorkflow';
import {
  refreshFileTree,
  reloadPreview,
} from '@/lib/desktopEvents';
import { fetchAccountUsageMeter } from '@/lib/fetchAccountUsage';
import { pathsFromFileChanges } from '@/lib/workspacePaths';
import { toast } from 'sonner';
import {
  usageMeterFromRateLimits,
  usageMeterFromTokenUsage,
} from '@/lib/usageMeter';
import { isTauri } from '@/api/tauri';
import type { Thread as ApiThread } from '@/generated/app-server/v2/Thread';
import type { AppAction, BootPhase, Thread as UiThread } from '@/types';

function threadFromApi(t: ApiThread): UiThread {
  return {
    id: t.id,
    title: t.preview?.slice(0, 48) || '未命名线程',
    preview: t.preview ?? '',
    timestamp: t.updatedAt * 1000,
    status: 'active',
  };
}

/** 通知方法 → params 类型映射 */
type NotificationParamsByMethod = {
  'item/started': ItemStartedNotification;
  'item/agentMessage/delta': AgentMessageDeltaNotification;
  'turn/planUpdated': TurnPlanUpdatedNotification;
  'item/completed': ItemCompletedNotification;
  'thread/tokenUsage/updated': ThreadTokenUsageUpdatedNotification;
  'account/rateLimits/updated': AccountRateLimitsUpdatedNotification;
  'item/fileChange/patchUpdated': FileChangePatchUpdatedNotification;
  'turn/completed': TurnCompletedNotification;
  'error': ErrorNotification;
  'mcpServer/oauthLogin/completed': McpServerOauthLoginCompletedNotification;
};

/** 类型守卫：用 method 串区分 params 类型 */
function isNotificationParams<T extends keyof NotificationParamsByMethod>(
  _method: T,
  params: unknown,
): params is NotificationParamsByMethod[T] {
  return typeof params === 'object' && params !== null;
}

function handleAppServerNotification(
  notification: JsonRpcNotification,
  dispatch: React.Dispatch<AppAction>,
  itemToMessageId: Map<string, string>,
  onSessionError: (message: string | null) => void,
  workspaceCwd: string | null,
): void {
  const { method, params } = notification;
  if (!params || typeof params !== 'object') {
    return;
  }

  switch (method) {
    case 'item/started': {
      if (!isNotificationParams('item/started', params)) break;
      const p = params;
      if (p.item.type === 'plan') {
        const todos = todosFromPlanText(p.item.text);
        if (todos.length > 0) {
          dispatch({ type: 'SET_TODOS', payload: todos });
        }
        const stage = inferWorkStageFromPlanSteps(
          todos.map((t) => t.text),
        );
        if (stage) {
          dispatchActivateWorkStage(dispatch, stage);
        }
      }
      const mapped = threadItemToMessage(p.item, p.startedAtMs);
      if (!mapped) {
        return;
      }
      if (itemToMessageId.has(p.item.id)) {
        return;
      }
      if (p.item.type === 'agentMessage') {
        mapped.status = 'streaming';
        mapped.content = p.item.text || '';
      }
      if (
        p.item.type === 'mcpToolCall' ||
        p.item.type === 'commandExecution' ||
        p.item.type === 'fileChange'
      ) {
        Object.assign(mapped, markMessageToolsRunning(mapped));
      }
      itemToMessageId.set(p.item.id, mapped.id);
      dispatch({ type: 'ADD_MESSAGE', payload: mapped });
      break;
    }
    case 'item/agentMessage/delta': {
      if (!isNotificationParams('item/agentMessage/delta', params)) break;
      const p = params;
      const messageId = itemToMessageId.get(p.itemId) ?? p.itemId;
      itemToMessageId.set(p.itemId, messageId);
      dispatch({
        type: 'APPEND_MESSAGE_DELTA',
        payload: { id: messageId, delta: p.delta },
      });
      break;
    }
    case 'turn/planUpdated': {
      if (!isNotificationParams('turn/planUpdated', params)) break;
      const p = params;
      if (p.plan.length > 0) {
        dispatch({ type: 'SET_TODOS', payload: todosFromPlanSteps(p.plan) });
        const stage = inferWorkStageFromPlanSteps(p.plan.map((s) => s.step));
        if (stage) {
          dispatchActivateWorkStage(dispatch, stage);
        }
      }
      break;
    }
    case 'item/completed': {
      if (!isNotificationParams('item/completed', params)) break;
      const p = params;
      if (p.item.type === 'plan') {
        const todos = todosFromPlanText(p.item.text);
        if (todos.length > 0) {
          dispatch({ type: 'SET_TODOS', payload: todos });
        }
      }
      const messageId = itemToMessageId.get(p.item.id) ?? p.item.id;
      const mapped = threadItemToMessage(p.item, p.completedAtMs);
      if (mapped) {
        dispatch({
          type: 'UPDATE_MESSAGE',
          payload: { id: messageId, updates: { ...mapped, status: 'complete' } },
        });
      } else {
        dispatch({
          type: 'UPDATE_MESSAGE',
          payload: { id: messageId, updates: { status: 'complete' } },
        });
      }
      break;
    }
    case 'thread/tokenUsage/updated': {
      if (!isNotificationParams('thread/tokenUsage/updated', params)) break;
      const p = params;
      dispatch({
        type: 'SET_USAGE_METER',
        payload: usageMeterFromTokenUsage(p.tokenUsage),
      });
      break;
    }
    case 'account/rateLimits/updated': {
      if (!isNotificationParams('account/rateLimits/updated', params)) break;
      const p = params;
      const meter = usageMeterFromRateLimits(p.rateLimits);
      if (meter) {
        dispatch({ type: 'SET_USAGE_METER', payload: meter });
      }
      break;
    }
    case 'item/fileChange/patchUpdated': {
      if (!isNotificationParams('item/fileChange/patchUpdated', params)) break;
      const p = params;
      const paths = pathsFromFileChanges(p.changes, workspaceCwd);
      refreshFileTree();
      reloadPreview(paths);
      toast.info('工作区文件已更新', {
        description:
          paths.length === 1
            ? paths[0].split('/').pop() ?? paths[0]
            : `${paths.length} 个文件`,
      });
      break;
    }
    case 'turn/completed': {
      if (!isNotificationParams('turn/completed', params)) break;
      const p = params;
      dispatch({ type: 'SET_STREAMING', payload: false });
      if (p.turn.status === 'failed' && p.turn.error) {
        const detail = p.turn.error.message;
        onSessionError(detail);
        dispatch({
          type: 'ADD_MESSAGE',
          payload: {
            id: `turn-err-${p.turn.id}`,
            role: 'system',
            content: `回合失败：${detail}`,
            timestamp: Date.now(),
            status: 'error',
          },
        });
        toast.error('对话回合失败', { description: detail });
      } else {
        onSessionError(null);
      }
      break;
    }
    case 'error': {
      if (!isNotificationParams('error', params)) break;
      const p = params;
      const detail = p.error.additionalDetails
        ? `${p.error.message} — ${p.error.additionalDetails}`
        : p.error.message;
      onSessionError(p.willRetry ? `${detail}（将重试）` : detail);
      dispatch({ type: 'SET_STREAMING', payload: false });
      break;
    }
    case 'mcpServer/oauthLogin/completed': {
      if (!isNotificationParams('mcpServer/oauthLogin/completed', params)) break;
      const p = params;
      if (p.success) {
        dispatch({
          type: 'SET_OAUTH_WAITING',
          payload: {
            serverName: p.name,
            phase: 'completed',
          },
        });
      } else {
        dispatch({
          type: 'SET_OAUTH_WAITING',
          payload: {
            serverName: p.name,
            phase: 'failed',
            error: p.error ?? '认证失败',
          },
        });
      }
      break;
    }
    default:
      break;
  }
}

export interface UseAppServerSessionOptions {
  connectionStatus: ConnectionStatus;
  bootPhase: BootPhase;
  dispatch: React.Dispatch<AppAction>;
  /** 当前项目根路径，传给 thread/start.cwd */
  projectCwd?: string | null;
}

export function useAppServerSession({
  connectionStatus,
  bootPhase,
  dispatch,
  projectCwd,
}: UseAppServerSessionOptions) {
  const [threadId, setThreadId] = useState<string | null>(null);
  const [sessionReady, setSessionReady] = useState(false);
  const [sessionError, setSessionError] = useState<string | null>(null);
  const itemToMessageIdRef = useRef(new Map<string, string>());

  const useRpc = isTauri() && connectionStatus === 'connected';
  const prevCwdRef = useRef<string | null>(null);

  const refreshThreads = useCallback(async () => {
    const client = getAppServerClient();
    const res = await client.request<ThreadListResponse>('thread/list', {
      limit: 30,
      archived: false,
      cwd: projectCwd ?? null,
      useStateDbOnly: true,
    });
    dispatch({
      type: 'SET_THREADS',
      payload: res.data.map(threadFromApi),
    });
    if (res.data.length > 0) {
      setThreadId((current) => {
        const next =
          current && res.data.some((t) => t.id === current)
            ? current
            : res.data[0].id;
        dispatch({ type: 'SET_CURRENT_THREAD', payload: next });
        return next;
      });
    } else {
      setThreadId(null);
      dispatch({ type: 'SET_CURRENT_THREAD', payload: null });
      dispatch({ type: 'SET_MESSAGES', payload: [] });
    }
  }, [dispatch, projectCwd]);

  const ensureThread = useCallback(async (): Promise<string> => {
    const client = getAppServerClient();
    if (threadId) {
      await tryResumeThread(threadId, projectCwd ?? null);
      return threadId;
    }
    const res = await client.request<ThreadStartResponse>('thread/start', {
      cwd: projectCwd ?? null,
    });
    setThreadId(res.thread.id);
    dispatch({ type: 'SET_CURRENT_THREAD', payload: res.thread.id });
    dispatch({ type: 'SET_CURRENT_MODEL', payload: res.model });
    const thread: UiThread = {
      id: res.thread.id,
      title: res.thread.preview?.slice(0, 48) || '新线程',
      preview: res.thread.preview ?? '',
      timestamp: res.thread.createdAt * 1000,
      status: 'active',
    };
    dispatch({ type: 'SET_THREADS', payload: [thread] });
    return res.thread.id;
  }, [threadId, projectCwd, dispatch]);

  const sendMessage = useCallback(
    async (text: string) => {
      if (!useRpc) {
        return;
      }
      setSessionError(null);
      const client = getAppServerClient();
      let activeThreadId: string;
      try {
        activeThreadId = await ensureThread();
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setSessionError(message);
        toast.error('无法准备会话', { description: message });
        return;
      }

      // 用户消息由 app-server `item/started` (userMessage) 写入，避免与乐观更新重复
      const stage = inferWorkStageFromText(text);
      if (stage) {
        dispatchActivateWorkStage(dispatch, stage);
      }
      dispatch({ type: 'SET_STREAMING', payload: true });

      const input: UserInput[] = [
        { type: 'text', text, text_elements: [] },
      ];
      const params: TurnStartParams = {
        threadId: activeThreadId,
        input,
      };
      try {
        await client.request<TurnStartResponse>('turn/start', params);
      } catch (err) {
        dispatch({ type: 'SET_STREAMING', payload: false });
        const message = err instanceof Error ? err.message : String(err);
        setSessionError(message);
        dispatch({
          type: 'ADD_MESSAGE',
          payload: {
            id: `err-${Date.now()}`,
            role: 'system',
            content: `请求失败：${message}`,
            timestamp: Date.now(),
            status: 'error',
          },
        });
        toast.error('无法开始对话回合', { description: message });
      }
    },
    [useRpc, ensureThread, dispatch],
  );

  useEffect(() => {
    if (!useRpc) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- conditional init
      setSessionReady(false);
      setThreadId(null);
      return;
    }

    if (bootPhase !== 'ready' || connectionStatus !== 'connected') {
      setSessionReady(false);
      return;
    }

    const client = getAppServerClient();
    const itemMap = itemToMessageIdRef.current;

    client.mergeHandlers({
      onServerRequest: (request) => {
        handleAppServerRequest(request, dispatch, {
          workspaceCwd: projectCwd ?? null,
        });
      },
      onNotification: (notification) => {
        handleAppServerNotification(
          notification,
          dispatch,
          itemMap,
          setSessionError,
          projectCwd ?? null,
        );
      },
    });

    let cancelled = false;

    const cwdChanged = prevCwdRef.current !== projectCwd;
    prevCwdRef.current = projectCwd ?? null;

    if (cwdChanged) {
      setThreadId(null);
      dispatch({ type: 'SET_THREADS', payload: [] });
      dispatch({ type: 'SET_CURRENT_THREAD', payload: null });
      dispatch({ type: 'SET_MESSAGES', payload: [] });
      dispatch({ type: 'SET_TODOS', payload: [] });
      dispatch({ type: 'SET_STREAMING', payload: false });
    }

    (async () => {
      try {
        if (!client.isInitialized()) {
          setSessionReady(false);
          setSessionError('app-server 未初始化');
          return;
        }

        if (cwdChanged) {
          const list = await client.request<ThreadListResponse>('thread/list', {
            limit: 30,
            archived: false,
            cwd: projectCwd ?? null,
            useStateDbOnly: true,
          });
          if (cancelled) return;

          const projectThreads = list.data.map(threadFromApi);
          dispatch({ type: 'SET_THREADS', payload: projectThreads });

          if (list.data.length > 0) {
            const activeId = list.data[0].id;
            setThreadId(activeId);
            dispatch({ type: 'SET_CURRENT_THREAD', payload: activeId });
            dispatch({ type: 'SET_MESSAGES', payload: [] });

            void (async () => {
              try {
                const resumed = await tryResumeThread(
                  activeId,
                  projectCwd ?? null,
                );
                if (cancelled) return;
                if (!resumed) {
                  return;
                }
                const read = await client.request<ThreadReadResponse>('thread/read', {
                  threadId: activeId,
                  includeTurns: true,
                });
                if (cancelled) return;
                dispatch({
                  type: 'SET_MESSAGES',
                  payload: messagesFromThreadHistory(read.thread),
                });
              } catch (err) {
                if (!cancelled) {
                  setSessionError(err instanceof Error ? err.message : String(err));
                }
              }
            })();
          } else {
            const res = await client.request<ThreadStartResponse>('thread/start', {
              cwd: projectCwd ?? null,
            });
            if (cancelled) return;
            setThreadId(res.thread.id);
            dispatch({ type: 'SET_CURRENT_THREAD', payload: res.thread.id });
            dispatch({ type: 'SET_CURRENT_MODEL', payload: res.model });
            dispatch({ type: 'SET_MESSAGES', payload: [] });
            const thread: UiThread = {
              id: res.thread.id,
              title: res.thread.preview?.slice(0, 48) || '新线程',
              preview: res.thread.preview ?? '',
              timestamp: res.thread.createdAt * 1000,
              status: 'active',
            };
            dispatch({ type: 'SET_THREADS', payload: [thread] });
          }
        } else {
          const warmed = await warmupAppServerSession(dispatch, projectCwd ?? null);
          if (warmed.threadId) {
            setThreadId(warmed.threadId);
          }
        }

        if (!cancelled) {
          setSessionReady(true);
          setSessionError(null);
          const meter = await fetchAccountUsageMeter();
          if (meter) {
            dispatch({ type: 'SET_USAGE_METER', payload: meter });
          }
        }
      } catch (err) {
        if (!cancelled) {
          setSessionError(err instanceof Error ? err.message : String(err));
          setSessionReady(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [useRpc, projectCwd, dispatch, connectionStatus, bootPhase]);

  const selectThread = useCallback(
    async (id: string) => {
      dispatch({ type: 'SET_CURRENT_THREAD', payload: id });
      if (!useRpc) {
        return;
      }
      setSessionError(null);
      const client = getAppServerClient();
      const resumed = await tryResumeThread(id, projectCwd ?? null);
      setThreadId(id);
      if (resumed) {
        const read = await client.request<ThreadReadResponse>('thread/read', {
          threadId: id,
          includeTurns: true,
        });
        dispatch({
          type: 'SET_MESSAGES',
          payload: messagesFromThreadHistory(read.thread),
        });
      } else {
        dispatch({ type: 'SET_MESSAGES', payload: [] });
      }
      dispatch({ type: 'SET_STREAMING', payload: false });
    },
    [useRpc, projectCwd, dispatch],
  );

  const startNewThread = useCallback(async () => {
    if (!useRpc) {
      return null;
    }
    setSessionError(null);
    const client = getAppServerClient();
    const res = await client.request<ThreadStartResponse>('thread/start', {
      cwd: projectCwd ?? null,
    });
    setThreadId(res.thread.id);
    dispatch({ type: 'SET_CURRENT_THREAD', payload: res.thread.id });
    dispatch({ type: 'SET_CURRENT_MODEL', payload: res.model });
    dispatch({ type: 'SET_MESSAGES', payload: [] });
    dispatch({ type: 'SET_STREAMING', payload: false });
    await refreshThreads();
    dispatch({ type: 'SET_CURRENT_THREAD', payload: res.thread.id });
    return res.thread.id;
  }, [useRpc, projectCwd, dispatch, refreshThreads]);

  const archiveThread = useCallback(
    async (id: string) => {
      if (!useRpc) {
        dispatch({ type: 'REMOVE_THREAD', payload: id });
        return;
      }
      setSessionError(null);
      const client = getAppServerClient();
      await client.request<ThreadArchiveResponse>('thread/archive', { threadId: id });
      if (threadId === id) {
        setThreadId(null);
        dispatch({ type: 'SET_MESSAGES', payload: [] });
      }
      await refreshThreads();
    },
    [useRpc, dispatch, refreshThreads, threadId],
  );

  const deleteThread = useCallback(
    async (id: string) => {
      if (
        !window.confirm('确定删除此会话？删除后会从当前项目会话列表中移除。')
      ) {
        return;
      }
      await archiveThread(id);
    },
    [archiveThread],
  );

  return {
    useRpc,
    sessionReady,
    sessionError,
    threadId,
    sendMessage,
    refreshThreads,
    setThreadId,
    selectThread,
    startNewThread,
    archiveThread,
    deleteThread,
  };
}
