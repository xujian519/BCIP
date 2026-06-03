import { useEffect, useRef } from 'react';
import { useAppStore } from '@/hooks/useAppStore';
import { saveWorkspaceLayout } from '@/lib/workspaceLayoutStorage';

const SAVE_DEBOUNCE_MS = 400;

/** 按项目路径持久化工作区分屏布局 */
export function useWorkspaceLayoutPersistence(): void {
  const { state } = useAppStore();
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    if (!state.workspaceCwd) {
      return;
    }

    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
    }

    timerRef.current = window.setTimeout(() => {
      saveWorkspaceLayout(
        state.workspaceCwd!,
        state.workspaceRoot,
        state.focusedPaneId,
      );
      timerRef.current = null;
    }, SAVE_DEBOUNCE_MS);

    return () => {
      if (timerRef.current !== null) {
        window.clearTimeout(timerRef.current);
      }
    };
  }, [state.workspaceCwd, state.workspaceRoot, state.focusedPaneId]);
}
