/**
 * 合并同 turn 内连续的 agent 消息，减少时间线垂直占用。
 */
import type { Message, MessageStatus, ToolCall } from '@/types';

function joinContent(parts: string[]): string {
  return parts
    .map((part) => part.trim())
    .filter(Boolean)
    .join('\n\n');
}

function joinReasoning(parts: string[]): string | undefined {
  const merged = joinContent(parts);
  return merged || undefined;
}

function mergeToolCalls(items: ToolCall[][]): ToolCall[] | undefined {
  const merged = items.flat();
  return merged.length > 0 ? merged : undefined;
}

function pickStatus(statuses: (MessageStatus | undefined)[]): MessageStatus | undefined {
  if (statuses.some((s) => s === 'streaming')) {
    return 'streaming';
  }
  if (statuses.some((s) => s === 'error')) {
    return 'error';
  }
  if (statuses.some((s) => s === 'sending')) {
    return 'sending';
  }
  return statuses.at(-1) ?? 'complete';
}

function mergeAgentMessages(messages: Message[]): Message {
  const first = messages[0];
  return {
    id: first.id,
    role: 'agent',
    content: joinContent(messages.map((m) => m.content)),
    timestamp: messages.at(-1)?.timestamp ?? first.timestamp,
    status: pickStatus(messages.map((m) => m.status)),
    reasoning: joinReasoning(
      messages.map((m) => m.reasoning ?? '').filter(Boolean),
    ),
    toolCalls: mergeToolCalls(messages.map((m) => m.toolCalls ?? [])),
    itemKind: messages.some((m) => m.itemKind === 'tool') ? 'tool' : first.itemKind,
  };
}

/** 合并 turn 内连续的 agent 条目 */
export function compactAgentItems(turn: Message[]): Message[] {
  const result: Message[] = [];
  let agentBatch: Message[] = [];

  const flushAgentBatch = () => {
    if (agentBatch.length === 0) {
      return;
    }
    result.push(
      agentBatch.length === 1 ? agentBatch[0] : mergeAgentMessages(agentBatch),
    );
    agentBatch = [];
  };

  for (const message of turn) {
    if (message.role === 'agent') {
      agentBatch.push(message);
    } else {
      flushAgentBatch();
      result.push(message);
    }
  }

  flushAgentBatch();
  return result;
}

/** 以 user 消息为界分组为 turn */
export function groupIntoTurns(messages: Message[]): Message[][] {
  const turns: Message[][] = [];
  let current: Message[] = [];

  for (const msg of messages) {
    if (msg.role === 'user' && current.length > 0) {
      turns.push(current);
      current = [msg];
    } else {
      current.push(msg);
    }
  }

  if (current.length > 0) {
    turns.push(current);
  }

  return turns;
}

/** 分组并 compact 每条 turn */
export function compactTimelineMessages(messages: Message[]): Message[] {
  return groupIntoTurns(messages).flatMap(compactAgentItems);
}
