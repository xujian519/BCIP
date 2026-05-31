import { useCallback, useState } from 'react';
import { getAppServerClient } from '@/lib/appServerClient';

export interface AgentSpawnResponse {
  agentId: string;
  status: string;
}

export interface AgentStatusResponse {
  agentId: string;
  name: string;
  status: string;
  model: string | null;
  outputFile: string | null;
  error: string | null;
}

export interface AgentListResponse {
  agents: AgentStatusResponse[];
}

export interface AgentCancelResponse {
  cancelled: boolean;
  agentId: string;
}

export interface UseAgentRuntimeOptions {
  connectionReady: boolean;
}

export function useAgentRuntime({ connectionReady }: UseAgentRuntimeOptions) {
  const [agents, setAgents] = useState<AgentStatusResponse[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const spawnAgent = useCallback(
    async (
      description: string,
      prompt: string,
      opts?: {
        subagentType?: string;
        name?: string;
        model?: string;
      },
    ) => {
      if (!connectionReady) return null;
      setLoading(true);
      setError(null);
      try {
        const client = getAppServerClient();
        const res = await client.request<AgentSpawnResponse>('agent/spawn', {
          description,
          prompt,
          subagentType: opts?.subagentType ?? null,
          name: opts?.name ?? null,
          model: opts?.model ?? null,
        });
        return res;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        return null;
      } finally {
        setLoading(false);
      }
    },
    [connectionReady],
  );

  const getAgentStatus = useCallback(
    async (agentId: string) => {
      if (!connectionReady) return null;
      try {
        const client = getAppServerClient();
        const res = await client.request<AgentStatusResponse>(
          'agent/status',
          { agentId },
        );
        setAgents((prev) =>
          prev.map((a) =>
            a.agentId === agentId
              ? res
              : a,
          ),
        );
        return res;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        return null;
      }
    },
    [connectionReady],
  );

  const listAgents = useCallback(async () => {
    if (!connectionReady) return;
    try {
      const client = getAppServerClient();
      const res = await client.request<AgentListResponse>('agent/list');
      setAgents(res.agents);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    }
  }, [connectionReady]);

  const cancelAgent = useCallback(
    async (agentId: string) => {
      if (!connectionReady) return null;
      try {
        const client = getAppServerClient();
        const res = await client.request<AgentCancelResponse>(
          'agent/cancel',
          { agentId },
        );
        return res;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        return null;
      }
    },
    [connectionReady],
  );

  return {
    agents,
    loading,
    error,
    spawnAgent,
    getAgentStatus,
    listAgents,
    cancelAgent,
  };
}
