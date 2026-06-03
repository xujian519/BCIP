/**
 * 将 app-server ThreadItem 映射为 UI Message（Codex parity 时间线）
 */
import type { ThreadItem } from '@/generated/app-server/v2/ThreadItem';
import type { Message, ToolCall } from '@/types';

function mapToolStatus(status: string): ToolCall['status'] {
  if (
    status === 'completed' ||
    status === 'succeeded' ||
    status === 'success'
  ) {
    return 'success';
  }
  if (status === 'failed' || status === 'error' || status === 'declined') {
    return 'error';
  }
  return 'running';
}

function jsonPreview(value: unknown): string | undefined {
  if (value === null || value === undefined) {
    return undefined;
  }
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

export function threadItemToMessage(
  item: ThreadItem,
  timestamp = Date.now(),
): Message | null {
  switch (item.type) {
    case 'userMessage':
      return {
        id: item.id,
        role: 'user',
        content: item.content
          .filter((part) => part.type === 'text')
          .map((part) => (part.type === 'text' ? part.text : ''))
          .join('\n'),
        timestamp,
        status: 'complete',
      };
    case 'agentMessage':
      return {
        id: item.id,
        role: 'agent',
        content: item.text,
        timestamp,
        status: 'complete',
        itemKind: 'agent',
      };
    case 'plan':
      return {
        id: item.id,
        role: 'system',
        content: item.text,
        timestamp,
        status: 'complete',
        itemKind: 'plan',
      };
    case 'reasoning':
      return {
        id: item.id,
        role: 'agent',
        content: item.summary.join('\n') || item.content.join('\n'),
        timestamp,
        status: 'complete',
        reasoning: item.content.join('\n'),
        itemKind: 'agent',
      };
    case 'commandExecution':
      return {
        id: item.id,
        role: 'agent',
        content: '',
        timestamp,
        status: 'complete',
        itemKind: 'tool',
        toolCalls: [
          {
            id: item.id,
            name: 'shell',
            kind: 'shell',
            detail: item.command,
            status: mapToolStatus(String(item.status)),
            output: item.aggregatedOutput ?? undefined,
          },
        ],
      };
    case 'mcpToolCall':
      return {
        id: item.id,
        role: 'agent',
        content: '',
        timestamp,
        status: 'complete',
        itemKind: 'tool',
        toolCalls: [
          {
            id: item.id,
            name: item.tool,
            kind: 'mcp',
            detail: item.server,
            args: jsonPreview(item.arguments),
            status: mapToolStatus(String(item.status)),
            output: jsonPreview(item.result),
            error: item.error ? jsonPreview(item.error) : undefined,
          },
        ],
      };
    case 'fileChange':
      return {
        id: item.id,
        role: 'agent',
        content: '',
        timestamp,
        status: 'complete',
        itemKind: 'tool',
        toolCalls: [
          {
            id: item.id,
            name: 'file_change',
            kind: 'patch',
            detail: item.changes.map((c) => `${c.kind}: ${c.path}`).join('\n'),
            status: mapToolStatus(String(item.status)),
          },
        ],
      };
    default:
      return null;
  }
}

/** item/started 时将工具类条目标为进行中 */
export function markMessageToolsRunning(message: Message): Message {
  if (!message.toolCalls?.length) {
    return message;
  }
  return {
    ...message,
    toolCalls: message.toolCalls.map((tc) => ({
      ...tc,
      status: 'running' as const,
    })),
  };
}
