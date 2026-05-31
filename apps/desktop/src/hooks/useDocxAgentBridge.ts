import { useCallback, useRef, useState } from 'react';
import { useAgentRuntime } from '@/hooks/useAgentRuntime';
import type { DocxEditorViewRef } from '@/components/preview/DocxEditorView';

export interface DocxAgentBridgeOptions {
  connectionReady: boolean;
  filePath: string | null;
  editorRef: React.RefObject<DocxEditorViewRef | null>;
}

export function useDocxAgentBridge({
  connectionReady,
  filePath,
  editorRef,
}: DocxAgentBridgeOptions) {
  const { spawnAgent, getAgentStatus, cancelAgent, agents, error: agentError } =
    useAgentRuntime({ connectionReady });
  const [drafting, setDrafting] = useState(false);
  const currentAgentId = useRef<string | null>(null);

  const startDraft = useCallback(
    async (task: string) => {
      if (!connectionReady || !filePath) return;
      setDrafting(true);
      const res = await spawnAgent(task, task, {
        subagentType: 'patent-drafter',
      });
      if (res) {
        currentAgentId.current = res.agentId;
      }
      return res;
    },
    [connectionReady, filePath, spawnAgent],
  );

  const startReview = useCallback(
    async (task: string) => {
      if (!connectionReady || !filePath) return;
      setDrafting(true);
      const res = await spawnAgent(task, task, {
        subagentType: 'patent-reviewer',
      });
      if (res) {
        currentAgentId.current = res.agentId;
      }
      return res;
    },
    [connectionReady, filePath, spawnAgent],
  );

  const checkAgentResult = useCallback(async () => {
    if (!currentAgentId.current) return null;
    const res = await getAgentStatus(currentAgentId.current);
    if (res?.status === 'completed' || res?.status === 'failed') {
      setDrafting(false);
    }
    return res;
  }, [getAgentStatus]);

  const cancelCurrentAgent = useCallback(async () => {
    if (!currentAgentId.current) return;
    await cancelAgent(currentAgentId.current);
    setDrafting(false);
    currentAgentId.current = null;
  }, [cancelAgent]);

  const saveAndReload = useCallback(async () => {
    if (editorRef.current) {
      await editorRef.current.save();
      await editorRef.current.load();
    }
  }, [editorRef]);

  return {
    drafting,
    agents,
    agentError,
    startDraft,
    startReview,
    checkAgentResult,
    cancelCurrentAgent,
    saveAndReload,
  };
}
