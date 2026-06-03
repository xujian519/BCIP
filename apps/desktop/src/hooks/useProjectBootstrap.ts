import { useEffect } from 'react';
import { useAppStore } from '@/hooks/useAppStore';
import { useProjects } from '@/hooks/useProjects';
import { addRecentProjectPath, loadRecentProjectPaths } from '@/lib/recentProjects';

/** 启动时自动选中最近项目或首个已注册项目 */
export function useProjectBootstrap(): void {
  const { state, dispatch } = useAppStore();
  const { projects, loading } = useProjects();

  useEffect(() => {
    if (loading || state.workspaceCwd) {
      return;
    }

    const recent = loadRecentProjectPaths();
    const fallback = recent[0] ?? projects[0]?.path;
    if (!fallback) {
      return;
    }

    addRecentProjectPath(fallback);
    dispatch({ type: 'SWITCH_PROJECT', payload: fallback });
  }, [loading, projects, state.workspaceCwd, dispatch]);
}
