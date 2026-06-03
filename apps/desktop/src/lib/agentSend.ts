import { toast } from 'sonner';
import { isTauri } from '@/api/tauri';
import { sendMockAgentMessage } from '@/lib/mockAgentSession';
import type { Dispatch } from 'react';
import type { AppAction } from '@/types';

export interface AgentSendContext {
  useRpc: boolean;
  connectionStatus: string;
  bootPhase: string;
  sendRpc: (text: string) => Promise<void>;
  dispatch: Dispatch<AppAction>;
}

export function handleAgentSend(ctx: AgentSendContext, text: string): void {
  const trimmed = text.trim();
  if (!trimmed) {
    return;
  }

  if (ctx.useRpc) {
    void ctx.sendRpc(trimmed);
    return;
  }

  sendMockAgentMessage(ctx.dispatch, trimmed);

  if (isTauri() && ctx.connectionStatus !== 'connected') {
    toast.info('演示模式', {
      description: 'app-server 未连接，当前为本地演示回复。连接后可使用完整 Agent 功能。',
      duration: 4000,
    });
  }
}
