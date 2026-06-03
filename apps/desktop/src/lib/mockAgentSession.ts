/**
 * 浏览器 / VITE_DEV_MOCK 演示：无 app-server 时的本地对话反馈
 */
import type { Dispatch } from 'react';
import { dispatchActivateWorkStage, inferWorkStageFromText } from '@/lib/patentWorkflow';
import type { AppAction, Message } from '@/types';

function mockAgentReply(userText: string): string {
  const t = userText.trim();
  if (/你好|您好|hello/i.test(t)) {
    return '你好！当前为演示模式（未连接 app-server）。请在本机执行 npm run tauri:dev，并确保已安装 bcip，即可进行真实对话。';
  }
  if (/专利|检索|对比|审查|起草/.test(t)) {
    return `（演示）已收到：「${t.slice(0, 80)}」。连接 app-server 后，Agent 将调用与 CLI 相同的 MCP/技能与模型配置处理该请求。`;
  }
  return `（演示）已收到你的消息。真实回复需通过 Tauri 连接 app-server；输入「你好」可查看连接说明。`;
}

/** 模拟一轮用户发送 + Agent 回复（无 LLM） */
export function sendMockAgentMessage(
  dispatch: Dispatch<AppAction>,
  text: string,
): void {
  const userMessage: Message = {
    id: `user-mock-${Date.now()}`,
    role: 'user',
    content: text,
    timestamp: Date.now(),
    status: 'complete',
  };
  dispatch({ type: 'ADD_MESSAGE', payload: userMessage });

  const stage = inferWorkStageFromText(text);
  if (stage) {
    dispatchActivateWorkStage(dispatch, stage);
  }

  const agentId = `agent-mock-${Date.now()}`;
  dispatch({ type: 'SET_STREAMING', payload: true });
  dispatch({
    type: 'ADD_MESSAGE',
    payload: {
      id: agentId,
      role: 'agent',
      content: '',
      timestamp: Date.now(),
      status: 'streaming',
      itemKind: 'agent',
    },
  });

  const reply = mockAgentReply(text);
  let index = 0;
  const tick = () => {
    if (index >= reply.length) {
      dispatch({
        type: 'UPDATE_MESSAGE',
        payload: {
          id: agentId,
          updates: { content: reply, status: 'complete' },
        },
      });
      dispatch({ type: 'SET_STREAMING', payload: false });
      return;
    }
    const chunk = reply.slice(index, index + 8);
    index += chunk.length;
    dispatch({
      type: 'APPEND_MESSAGE_DELTA',
      payload: { id: agentId, delta: chunk },
    });
    window.setTimeout(tick, 24);
  };
  window.setTimeout(tick, 120);
}
