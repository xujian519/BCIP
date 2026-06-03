import { useState, useEffect, useCallback } from 'react';
import { api } from '@/api';
import type { ProjectInfo } from '@/api/types';

export interface UseProjectsReturn {
  projects: ProjectInfo[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  createProject: (path: string) => Promise<ProjectInfo | null>;
}

export function useProjects(): UseProjectsReturn {
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const list = await api.getProjects();
      setProjects(list);
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载项目失败');
    } finally {
      setLoading(false);
    }
  }, []);

  const createProject = useCallback(async (path: string): Promise<ProjectInfo | null> => {
    try {
      const info = await api.createProject(path);
      await refresh();
      return info;
    } catch (err) {
      setError(err instanceof Error ? err.message : '创建项目失败');
      return null;
    }
  }, [refresh]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- data fetching hook
    refresh();
  }, [refresh]);

  return { projects, loading, error, refresh, createProject };
}
