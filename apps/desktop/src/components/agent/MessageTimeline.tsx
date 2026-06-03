/**
 * MessageTimeline —— 消息时间线（RPC delta 直出）
 * 同一 turn 内（用户 → 计划/助手）紧凑排列，turn 之间留适当间距
 */
import { cn } from '@/lib/utils';
import {
  compactTimelineMessages,
  groupIntoTurns,
} from '@/lib/compactTimelineMessages';
import { useEffect, useMemo, useRef, useState } from 'react';
import { Sparkles, MessageSquare, Zap } from 'lucide-react';
import UserBubble from './UserBubble';
import AgentBlock from './AgentBlock';
import PlanMessageBlock from './PlanMessageBlock';
import SystemNotice from './SystemNotice';
import type { Message } from '@/types';

const quickStarts = [
  { label: '帮我分析这份专利文件', icon: MessageSquare },
  { label: '搜索相关技术方案', icon: Sparkles },
  { label: '/draft 起草专利文稿', icon: Zap },
];

function EmptyConversation({ onQuickStart }: { onQuickStart?: (text: string) => void }) {
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const t = requestAnimationFrame(() => setVisible(true));
    return () => cancelAnimationFrame(t);
  }, []);

  return (
    <div
      className={cn(
        'flex flex-1 flex-col items-center justify-center gap-4 px-4',
        'transition-all duration-500',
        visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4',
      )}
      style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
    >
      {/* 品牌图标 */}
      <div
        className={cn(
          'relative flex h-14 w-14 items-center justify-center rounded-2xl',
          'bg-[var(--bg-elevated)] border border-[var(--border-default)]',
          'shadow-sm',
        )}
      >
        <Sparkles size={24} className="text-[var(--accent-primary)]" />
        <div
          className="absolute inset-0 rounded-2xl"
          style={{ boxShadow: '0 0 24px rgba(74, 124, 111, 0.08)' }}
        />
      </div>

      {/* 标题 */}
      <div className="flex flex-col items-center gap-1.5">
        <p className="text-sm font-semibold text-[var(--text-primary)]">开始新对话</p>
        <p className="max-w-[200px] text-center text-2xs text-[var(--text-tertiary)] leading-relaxed">
          输入消息或使用 / 命令开始与云熙智能助手对话
        </p>
      </div>

      {/* 快捷操作 */}
      <div className="flex flex-col gap-1.5 w-full max-w-[240px]">
        {quickStarts.map((item, i) => {
          const Icon = item.icon;
          return (
            <button
              key={item.label}
              type="button"
              onClick={() => onQuickStart?.(item.label)}
              className={cn(
                'flex items-center gap-2 rounded-lg px-3 py-2 w-full',
                'bg-[var(--bg-elevated)] border border-[var(--border-default)]',
                'text-xs text-[var(--text-secondary)]',
                'transition-all duration-300',
                'hover:bg-[var(--bg-hover)] hover:border-[var(--border-hover)]',
                'hover:text-[var(--text-primary)]',
                'cursor-pointer text-left',
              )}
              style={{
                transitionDelay: `${300 + i * 80}ms`,
                opacity: visible ? 1 : 0,
                transform: visible ? 'translateY(0)' : 'translateY(4px)',
                transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
              }}
            >
              <Icon size={13} className="text-[var(--text-tertiary)] shrink-0" />
              <span className="truncate">{item.label}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

interface MessageTimelineProps {
  messages: Message[];
  isStreaming?: boolean;
  streamingMessageId?: string | null;
  onQuickStart?: (text: string) => void;
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
  onQuickStart,
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
        <EmptyConversation onQuickStart={onQuickStart} />
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
