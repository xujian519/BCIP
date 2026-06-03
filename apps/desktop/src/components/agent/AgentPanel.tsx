/**
 * AgentPanel —— Codex 对齐右侧会话面板
 * Tauri + 已连接：真实 app-server RPC；Web/未连接：mock 或提示
 */
import { useEffect, useMemo } from 'react';
import { BCIP_NEW_THREAD } from '@/lib/desktopEvents';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/hooks/useAppStore';
import { useAppServerSession } from '@/hooks/useAppServerSession';
import { connectAppServer } from '@/lib/appServerConnect';
import { getAppServerClient } from '@/lib/appServerClient';
import { warmupAppServerSession } from '@/lib/appServerSessionWarmup';
import { handleAgentSend } from '@/lib/agentSend';
import AgentHeader from './AgentHeader';
import ThreadListDrawer from './ThreadListDrawer';
import MessageTimeline from './MessageTimeline';
import Composer from './Composer';
import AgentFooter from './AgentFooter';
import ApprovalPendingBanner from './ApprovalPendingBanner';

export default function AgentPanel({ width, fillWidth }: { width: number; fillWidth?: boolean }) {
  const { state, dispatch } = useAppStore();
  const session = useAppServerSession({
    connectionStatus: state.connectionStatus,
    bootPhase: state.bootPhase,
    dispatch,
    projectCwd: state.workspaceCwd,
  });

  const appInitialized = getAppServerClient().isInitialized();

  const currentThread = useMemo(
    () => state.threads.find((t) => t.id === state.currentThreadId),
    [state.threads, state.currentThreadId],
  );

  const streamingMessageId = useMemo(() => {
    if (!state.isStreaming) {
      return null;
    }
    for (let i = state.messages.length - 1; i >= 0; i--) {
      const msg = state.messages[i];
      if (msg.role === 'agent') {
        return msg.id;
      }
    }
    return null;
  }, [state.isStreaming, state.messages]);

  const { startNewThread } = session;
  useEffect(() => {
    const onNewThread = () => {
      void startNewThread();
    };
    window.addEventListener(BCIP_NEW_THREAD, onNewThread);
    return () => window.removeEventListener(BCIP_NEW_THREAD, onNewThread);
  }, [startNewThread]);

  useEffect(() => {
    // Mock data injection removed — use real RPC or empty state
  }, [session.useRpc, state.currentThreadId, state.threads.length, dispatch]);

  const handleReconnect = async () => {
    try {
      await connectAppServer(dispatch);
      dispatch({ type: 'SET_BOOT_PHASE', payload: 'ready' });
      dispatch({ type: 'SET_BOOT_ERROR', payload: null });
      await warmupAppServerSession(dispatch, state.workspaceCwd);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      dispatch({ type: 'SET_BOOT_ERROR', payload: message });
      dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'error' });
    }
  };

  const threadTitle = currentThread?.title ?? '新线程';
  const composerDisabled =
    (session.useRpc &&
      (state.bootPhase !== 'ready' || !appInitialized || state.isStreaming)) ||
    (!session.useRpc && state.isStreaming);

  const composerDisabledReason = !session.useRpc
    ? state.isStreaming
      ? '回复生成中…'
      : undefined
    : state.bootPhase !== 'ready'
      ? '正在启动…'
      : !appInitialized
        ? '正在连接 app-server…'
        : state.isStreaming
          ? '等待回复…'
          : undefined;

  const footerError = state.bootError ?? session.sessionError;

  return (
    <aside
      className={cn(
        'h-full flex flex-col',
        !fillWidth && 'shrink-0 border-l border-[var(--border-default)]',
        'bg-[var(--bg-surface)]',
      )}
      style={fillWidth ? { flex: 1 } : { width }}
    >
      <AgentHeader
        threadTitle={threadTitle}
        projectName={
          state.workspaceCwd
            ? state.workspaceCwd.split('/').filter(Boolean).pop() ?? '项目'
            : '云熙 · 专利助手'
        }
        onSelectThread={(id) => void session.selectThread(id)}
        onNewThread={() => void session.startNewThread()}
      />

      <div className="flex flex-1 min-h-0">
        <ThreadListDrawer
          visible
          useRpc={session.useRpc}
          onSelectThread={(id) => void session.selectThread(id)}
          onNewThread={() => void session.startNewThread()}
          onArchiveThread={(id) => void session.archiveThread(id)}
          onDeleteThread={(id) => void session.deleteThread(id)}
        />
        <div className="flex flex-col flex-1 min-w-0">
          <MessageTimeline
            messages={state.messages}
            isStreaming={state.isStreaming}
            streamingMessageId={streamingMessageId}
          />
          <ApprovalPendingBanner />
          <Composer
            onSend={(text) =>
              handleAgentSend(
                {
                  useRpc: session.useRpc,
                  connectionStatus: state.connectionStatus,
                  bootPhase: state.bootPhase,
                  sendRpc: session.sendMessage,
                  dispatch,
                },
                text,
              )
            }
            disabled={composerDisabled}
            disabledReason={composerDisabledReason}
            placeholder={
              session.useRpc && !session.sessionReady
                ? '会话预热中，可直接输入消息…'
                : '输入消息，Enter 发送（Shift+Enter 换行）'
            }
          />
          <AgentFooter
            connectionStatus={state.connectionStatus}
            modelName={state.currentModel}
            errorHint={footerError}
            onRetry={() => void handleReconnect()}
            onOpenSettings={() =>
              dispatch({ type: 'OPEN_SETTINGS', payload: 'model' })
            }
          />
        </div>
      </div>
    </aside>
  );
}
