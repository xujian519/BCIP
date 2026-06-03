import { describe, expect, it } from 'vitest';
import {
  compactAgentItems,
  compactTimelineMessages,
  groupIntoTurns,
} from './compactTimelineMessages';
import type { Message } from '@/types';

function agentMessage(
  id: string,
  overrides: Partial<Message> = {},
): Message {
  return {
    id,
    role: 'agent',
    content: '',
    timestamp: 1000,
    status: 'complete',
    ...overrides,
  };
}

describe('compactAgentItems', () => {
  it('merges text and shell tool calls in one turn', () => {
    const turn: Message[] = [
      agentMessage('a1', { content: '正在检索', timestamp: 1000 }),
      agentMessage('a2', {
        timestamp: 2000,
        itemKind: 'tool',
        toolCalls: [
          {
            id: 't1',
            name: 'shell',
            kind: 'shell',
            detail: 'ls -la',
            status: 'success',
          },
        ],
      }),
      agentMessage('a3', {
        timestamp: 3000,
        itemKind: 'tool',
        toolCalls: [
          {
            id: 't2',
            name: 'shell',
            kind: 'shell',
            detail: 'pwd',
            status: 'success',
          },
        ],
      }),
    ];

    const compacted = compactAgentItems(turn);
    expect(compacted).toHaveLength(1);
    expect(compacted[0].id).toBe('a1');
    expect(compacted[0].content).toBe('正在检索');
    expect(compacted[0].timestamp).toBe(3000);
    expect(compacted[0].toolCalls).toHaveLength(2);
    expect(compacted[0].toolCalls?.[0].detail).toBe('ls -la');
    expect(compacted[0].toolCalls?.[1].detail).toBe('pwd');
  });

  it('merges tool-only chain', () => {
    const turn: Message[] = [
      agentMessage('a1', {
        itemKind: 'tool',
        toolCalls: [
          { id: 't1', name: 'shell', status: 'success', detail: 'cmd1' },
        ],
      }),
      agentMessage('a2', {
        status: 'streaming',
        itemKind: 'tool',
        toolCalls: [
          { id: 't2', name: 'shell', status: 'running', detail: 'cmd2' },
        ],
      }),
    ];

    const compacted = compactAgentItems(turn);
    expect(compacted).toHaveLength(1);
    expect(compacted[0].toolCalls).toHaveLength(2);
    expect(compacted[0].status).toBe('streaming');
  });

  it('does not merge across user boundaries within turn compact', () => {
    const turn: Message[] = [
      {
        id: 'u1',
        role: 'user',
        content: 'hello',
        timestamp: 500,
        status: 'complete',
      },
      agentMessage('a1', { content: 'reply' }),
    ];

    const compacted = compactAgentItems(turn);
    expect(compacted).toHaveLength(2);
    expect(compacted[0].role).toBe('user');
    expect(compacted[1].content).toBe('reply');
  });
});

describe('compactTimelineMessages', () => {
  it('does not merge agent messages across turns', () => {
    const messages: Message[] = [
      {
        id: 'u1',
        role: 'user',
        content: 'first',
        timestamp: 100,
        status: 'complete',
      },
      agentMessage('a1', { content: 'answer1', timestamp: 200 }),
      {
        id: 'u2',
        role: 'user',
        content: 'second',
        timestamp: 300,
        status: 'complete',
      },
      agentMessage('a2', { content: 'answer2', timestamp: 400 }),
    ];

    const turns = groupIntoTurns(messages);
    expect(turns).toHaveLength(2);

    const compacted = compactTimelineMessages(messages);
    expect(compacted).toHaveLength(4);
    expect(compacted.map((m) => m.id)).toEqual(['u1', 'a1', 'u2', 'a2']);
  });
});
