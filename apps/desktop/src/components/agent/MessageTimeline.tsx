/**
 * MessageTimeline —— 消息时间线（RPC delta 直出）
 * 同一 turn 内（用户 → 计划/助手）紧凑排列，turn 之间留适当间距
 */
import { cn } from '@/lib/utils';
import {
  compactTimelineMessages,
  groupIntoTurns,
} from '@/lib/compactTimelineMessages';
import { useEffect, useMemo, useRef } from 'react';
import UserBubble from './UserBubble';
import AgentBlock from './AgentBlock';
import PlanMessageBlock from './PlanMessageBlock';
import SystemNotice from './SystemNotice';
import type { Message } from '@/types';

interface MessageTimelineProps {
  messages: Message[];
  isStreaming?: boolean;
  streamingMessageId?: string | null;
}

function MessageItem({ message }: { message: Message }) {
  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}`;
  };

  if (message.role === 'user') {
    return (
      <UserBubble
        content={message.content}
        timestamp={formatTime(message.timestamp)}
        animate={false}
      />
    );
  }

  if (message.role === 'system' && message.itemKind === 'plan') {
    return (
      <PlanMessageBlock
        content={message.content}
        timestamp={formatTime(message.timestamp)}
      />
    );
  }

  if (message.role === 'system') {
    return (
      <SystemNotice
        content={message.content}
        timestamp={formatTime(message.timestamp)}
      />
    );
  }

  return (
    <AgentBlock
      content={message.content}
      status={message.status ?? 'complete'}
      timestamp={formatTime(message.timestamp)}
      reasoning={message.reasoning}
      toolCalls={message.toolCalls}
      animate={false}
    />
  );
}

export default function MessageTimeline({
  messages,
  isStreaming = false,
  streamingMessageId = null,
}: MessageTimelineProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  const displayMessages = useMemo(() => {
    const mapped = messages.map((msg) => {
      if (
        isStreaming &&
        streamingMessageId &&
        msg.id === streamingMessageId &&
        msg.role === 'agent'
      ) {
        return { ...msg, status: 'streaming' as const };
      }
      return msg;
    });
    return compactTimelineMessages(mapped);
  }, [messages, isStreaming, streamingMessageId]);

  const turns = useMemo(
    () => groupIntoTurns(displayMessages),
    [displayMessages],
  );
  const lastContentLen = displayMessages.at(-1)?.content.length ?? 0;

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [displayMessages.length, lastContentLen, isStreaming]);

  return (
    <div
      ref={scrollRef}
      className={cn(
        'chat-column flex-1 overflow-y-auto custom-scrollbar',
        'flex flex-col bg-[var(--bg-base)]',
      )}
      style={{
        gap: 'var(--chat-turn-gap)',
        paddingTop: 'var(--chat-timeline-py)',
        paddingBottom: 'var(--chat-timeline-py)',
      }}
    >
      {displayMessages.length === 0 ? (
        <div className="flex flex-1 flex-col items-center justify-center gap-3 text-[var(--text-tertiary)] message-enter">
          <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-[var(--bg-elevated)] border border-[var(--border-default)] shadow-sm">
            <span className="text-xl">💬</span>
          </div>
          <p className="text-sm text-[var(--text-secondary)] font-medium">开始一个新的对话</p>
          <p className="max-w-[220px] text-center text-2xs leading-relaxed">
            输入消息开始与云熙智能助手对话
          </p>
        </div>
      ) : (
        turns.map((turn, turnIndex) => (
          <div
            key={turn[0]?.id ?? `turn-${turnIndex}`}
            className="flex flex-col message-enter"
            style={{ 
              gap: 'var(--chat-item-gap)',
              animationDelay: `${turnIndex * 0.05}s`,
            }}
          >
            {turn.map((msg) => (
              <MessageItem key={msg.id} message={msg} />
            ))}
          </div>
        ))
      )}
    </div>
  );
}
