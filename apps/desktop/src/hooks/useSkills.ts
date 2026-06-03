/**
 * skills/list、skills/config/write、skills/changed 通知
 */
import { useCallback, useEffect, useState } from 'react';
import type { SkillErrorInfo } from '@/generated/app-server/v2/SkillErrorInfo';
import type { SkillMetadata } from '@/generated/app-server/v2/SkillMetadata';
import type { SkillsConfigWriteResponse } from '@/generated/app-server/v2/SkillsConfigWriteResponse';
import type { SkillsListResponse } from '@/generated/app-server/v2/SkillsListResponse';
import type { JsonRpcNotification } from '@/lib/appServerClient';
import { getAppServerClient } from '@/lib/appServerClient';

export interface SkillRow {
  name: string;
  description: string;
  path: string;
  enabled: boolean;
  displayName: string;
}

function toSkillRow(skill: SkillMetadata): SkillRow {
  const displayName =
    skill.interface?.displayName ??
    skill.shortDescription ??
    skill.name;
  const description =
    skill.interface?.shortDescription ?? skill.description ?? '';
  return {
    name: skill.name,
    description,
    path: skill.path,
    enabled: skill.enabled,
    displayName,
  };
}

function flattenSkills(response: SkillsListResponse): {
  skills: SkillRow[];
  errors: SkillErrorInfo[];
  cwd: string | null;
} {
  const skills: SkillRow[] = [];
  const errors: SkillErrorInfo[] = [];
  let cwd: string | null = null;
  for (const entry of response.data) {
    cwd = entry.cwd;
    for (const skill of entry.skills) {
      skills.push(toSkillRow(skill));
    }
    errors.push(...entry.errors);
  }
  return { skills, errors, cwd };
}

export function useSkills(rpcReady: boolean, workspaceCwd?: string | null) {
  const [skills, setSkills] = useState<SkillRow[]>([]);
  const [errors, setErrors] = useState<SkillErrorInfo[]>([]);
  const [scopeCwd, setScopeCwd] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState<string | null>(null);

  const refresh = useCallback(
    async (forceReload = false) => {
      if (!rpcReady) {
        return;
      }
      const client = getAppServerClient();
      if (!client.isInitialized()) {
        setError('请先连接 app-server');
        return;
      }
      setLoading(true);
      setError(null);
      try {
        const res = await client.request<SkillsListResponse>('skills/list', {
          cwds: workspaceCwd ? [workspaceCwd] : [],
          forceReload,
        });
        const flat = flattenSkills(res);
        setSkills(flat.skills);
        setErrors(flat.errors);
        setScopeCwd(flat.cwd);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    },
    [rpcReady, workspaceCwd],
  );

  const setSkillEnabled = useCallback(
    async (skill: SkillRow, nextEnabled: boolean) => {
      if (!rpcReady) {
        return;
      }
      const client = getAppServerClient();
      setSaving(skill.name);
      setError(null);
      try {
        await client.request<SkillsConfigWriteResponse>('skills/config/write', {
          name: skill.name,
          path: null,
          enabled: nextEnabled,
        });
        setSkills((prev) =>
          prev.map((s) =>
            s.name === skill.name ? { ...s, enabled: nextEnabled } : s,
          ),
        );
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        await refresh();
      } finally {
        setSaving(null);
      }
    },
    [rpcReady, refresh],
  );

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- data fetching hook
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!rpcReady) {
      return;
    }
    const client = getAppServerClient();
    const onNotification = (notification: JsonRpcNotification) => {
      if (notification.method === 'skills/changed') {
        void refresh();
      }
    };
    client.mergeHandlers({ onNotification });
  }, [rpcReady, refresh]);

  return {
    skills,
    listErrors: errors,
    scopeCwd,
    loading,
    error,
    saving,
    refresh,
    setSkillEnabled,
  };
}
