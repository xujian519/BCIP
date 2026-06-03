/**
 * 阶段 5：桌面端无感接入 —— 检测 bcip、连接 app-server、更新 boot 状态
 */
import { useCallback, useEffect, useRef } from 'react';
import { toast } from 'sonner';
import { api } from '@/api';
import { isTauri } from '@/api/tauri';
import { useAppStore } from '@/hooks/useAppStore';
import { connectAppServer } from '@/lib/appServerConnect';
import { getAppServerClient } from '@/lib/appServerClient';
import { warmupAppServerSession } from '@/lib/appServerSessionWarmup';
import type { ConfigReadResponse } from '@/generated/app-server/v2/ConfigReadResponse';

export function useDesktopBoot() {
  const { state, dispatch } = useAppStore();
  const bootStarted = useRef(false);
  const lastConnectError = useRef<string | null>(null);

  const applyConfigAfterConnect = useCallback(
    async (workspaceCwd: string | null) => {
      if (!getAppServerClient().isInitialized()) {
        return;
      }
      try {
        const res = await getAppServerClient().request<ConfigReadResponse>(
          'config/read',
          { cwd: workspaceCwd },
        );
        // BCIP 桌面端忽略 desktop.auto_connect：桌面端本身即自动连接，
        // auto_connect 仅控制 CLI/TUI 是否自动附着桌面端 daemon。
        if (res.config.model) {
          const entry = res.config.model;
          dispatch({
            type: 'SET_CURRENT_MODEL',
            payload: typeof entry === 'string' ? entry : String(entry),
          });
        }
      } catch {
        // 使用默认连接行为
      }
    },
    [dispatch],
  );

  const runBoot = useCallback(async () => {
    dispatch({ type: 'SET_BOOT_PHASE', payload: 'checking' });
    dispatch({ type: 'SET_BOOT_ERROR', payload: null });
    dispatch({ type: 'CLEAR_BOOT_LOG' });

    const log = (line: string) => {
      dispatch({ type: 'APPEND_BOOT_LOG', payload: line });
      console.log(`[boot] ${line}`);
      try {
        const entry = `[${new Date().toISOString()}] ${line}\n`;
        const prev = localStorage.getItem('bcip-boot-log') ?? '';
        localStorage.setItem('bcip-boot-log', prev + entry);
      } catch { /* ignore */ }
    };

    try {
      log('检测 bcip 可执行文件…');
      const check = await api.checkBcip();
      console.log('[boot] check result:', check);
      dispatch({ type: 'SET_BCIP_INSTALLED', payload: check.installed });
      dispatch({
        type: 'SET_BCIP_RESOLUTION',
        payload: {
          path: check.path ?? null,
          version: check.version ?? null,
          source: check.source ?? null,
        },
      });

      if (check.installed && check.path) {
        const src =
          check.source === 'sidecar'
            ? '内置 sidecar'
            : check.source === 'workspace'
              ? '仓库内编译产物'
              : '系统 PATH';
        log(`bcip 已解析 (${src}): ${check.path}`);
        if (check.version) {
          log(`版本: ${check.version}`);
        }
      }

      if (!check.installed) {
        log('未找到 bcip（PATH 与 bundle sidecar 均无）');
        dispatch({ type: 'SET_BOOT_PHASE', payload: 'no_cli' });
        dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'disconnected' });
        return;
      }

      dispatch({ type: 'SET_BOOT_PHASE', payload: 'connecting' });
      log('连接 app-server…');
      dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'connecting' });

      lastConnectError.current = null;
      log('调用 connectAppServer…');
      await connectAppServer(dispatch);
      log('connectAppServer 完成');
      const status = await api.getAppServerStatus();
      const transport = status.transport || 'stdio';
      dispatch({
        type: 'SET_APP_SERVER_TRANSPORT',
        payload: transport,
      });
      log(
        transport === 'proxy'
          ? '已附着终端 daemon（proxy）'
          : '已启动本地 app-server（stdio）',
      );

      await applyConfigAfterConnect(state.workspaceCwd);
      log('applyConfig 完成');

      log(`isInitialized: ${getAppServerClient().isInitialized()}`);
      if (getAppServerClient().isInitialized()) {
        dispatch({ type: 'SET_BOOT_PHASE', payload: 'ready' });
        dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'connected' });

        try {
          const warmed = await warmupAppServerSession(
            dispatch,
            state.workspaceCwd,
          );
          if (warmed.threadId) {
            toast.success('已与终端配置同步', {
              description: `app-server 已就绪（${Math.round(warmed.elapsedMs / 1000)}s）`,
            });
          } else {
            toast.success('已与 BCIP 配置同步', {
              description: 'app-server 已连接，使用 ~/.bcip（与 Codex 桌面隔离）',
            });
          }
        } catch (warmErr) {
          toast.warning('app-server 已连接，会话预热未完成', {
            description:
              warmErr instanceof Error ? warmErr.message : String(warmErr),
          });
        }
      } else {
        throw new Error('app-server 未初始化');
      }
    } catch (err) {
      const message =
        err instanceof Error
          ? err.message
          : lastConnectError.current ?? String(err);
      console.error('[boot] FAILED:', message, err);
      log(`错误: ${message}`);
      dispatch({ type: 'APPEND_BOOT_LOG', payload: `错误: ${message}` });
      dispatch({ type: 'SET_BOOT_PHASE', payload: 'fault' });
      dispatch({ type: 'SET_BOOT_ERROR', payload: message });
      dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'error' });
    }
  }, [applyConfigAfterConnect, dispatch, state.workspaceCwd]);

  const retryBoot = useCallback(() => {
    bootStarted.current = true;
    void runBoot();
  }, [runBoot]);

  const continueFileMode = useCallback(() => {
    dispatch({ type: 'SET_BOOT_PHASE', payload: 'ready' });
    dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'disconnected' });
    dispatch({ type: 'SET_BOOT_ERROR', payload: null });
  }, [dispatch]);

  useEffect(() => {
    if (state.bootPhase !== 'ready' || !isTauri()) {
      return;
    }
    if (!getAppServerClient().isInitialized()) {
      return;
    }
    void applyConfigAfterConnect(state.workspaceCwd);
  }, [state.workspaceCwd, state.bootPhase, applyConfigAfterConnect]);



  useEffect(() => {
    if (!isTauri()) {
      dispatch({ type: 'SET_BOOT_PHASE', payload: 'ready' });
      dispatch({ type: 'SET_CONNECTION_STATUS', payload: 'disconnected' });
      return;
    }
    if (bootStarted.current) {
      return;
    }
    bootStarted.current = true;
    void runBoot();
  }, [dispatch, runBoot]);

  return { retryBoot, continueFileMode, bootPhase: state.bootPhase };
}
