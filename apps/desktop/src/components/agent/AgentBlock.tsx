/**
 * AgentBlock —— 助手消息（以纯文字为主，工具调用为紧凑行）
 */
import { cn } from '@/lib/utils';
import { useState, useEffect } from 'react';
import ReasoningBlock from './ReasoningBlock';
import ToolCallCard from './ToolCallCard';
import type { MessageStatus, ToolCall as ToolCallType } from '@/types';

interface AgentBlockProps {
  content: string;
  status?: MessageStatus;
  timestamp?: string;
  reasoning?: string;
  toolCalls?: ToolCallType[];
  animate?: boolean;
}

function formatContent(content: string, isStreaming: boolean) {
  if (!content) {
    return isStreaming ? <StreamingCursor /> : null;
  }

  const parts = content.split(/(```[\s\S]*?```|`[^`]+`)/g);
  return (
    <>
      {parts.map((part, index) => {
        if (part.startsWith('```') && part.endsWith('```')) {
          const lines = part.slice(3, -3).split('\n');
          const lang = lines[0].trim();
          const code = lines.slice(1).join('\n').trim();
          return (
          <div key={index} className="my-2 relative group">
            {lang && (
              <div
                className="absolute top-0 right-0 font-mono text-2xs text-[var(--text-tertiary)]
                           bg-[var(--bg-elevated)] border-b border-l border-[var(--border-default)]
                           rounded-bl-lg px-2 py-0.5 z-10 opacity-70 group-hover:opacity-100 transition-opacity duration-200"
              >
                {lang}
              </div>
            )}
            <pre
              className={cn(
                'overflow-x-auto rounded-xl border border-[var(--border-default)]',
                'bg-[var(--bg-surface)] p-3 pt-6 font-mono text-xs text-[var(--text-primary)]',
                'shadow-sm hover:shadow-md transition-shadow duration-200',
              )}
            >
              <code>{code}</code>
            </pre>
          </div>
          );
        }
        if (part.startsWith('`') && part.endsWith('`') && part.length > 2) {
          const code = part.slice(1, -1);
          return (
          <code
            key={index}
            className={cn(
              'rounded-md border border-[var(--border-default)] bg-[var(--bg-surface)]',
              'px-1.5 py-0.5 font-mono text-xs text-[var(--text-primary)]',
            )}
          >
            {code}
          </code>
          );
        }
        return (
          <span key={index} className="whitespace-pre-wrap break-words">
            {part}
          </span>
        );
      })}
      {isStreaming && <StreamingCursor />}
    </>
  );
}

function StreamingCursor() {
  return (
    <span
      className={cn(
        'ml-0.5 inline-block h-[1.2em] w-[2px] align-text-bottom',
        'bg-[var(--accent-cyan)] animate-cursor-blink',
      )}
    />
  );
}

function getBorderClass(status?: MessageStatus): string {
  switch (status) {
    case 'streaming':
      return cn('border-l-2 pl-2 agent-streaming-border');
    case 'sending':
      return 'border-l-2 border-l-[var(--status-warning)] pl-2';
    case 'error':
      return 'border-l-2 border-l-[var(--status-error)] pl-2';
    case 'complete':
    default:
      return 'border-l-2 border-l-transparent pl-2 transition-[border-color] duration-300 ease-out';
  }
}

function InlineTimestamp({ timestamp }: { timestamp?: string }) {
  if (!timestamp) {
    return null;
  }
  return (
    <span className="ml-2 shrink-0 text-[10px] text-[var(--text-tertiary)] opacity-60">
      {timestamp}
    </span>
  );
}

export default function AgentBlock({
  content,
  status = 'complete',
  timestamp,
  reasoning,
  toolCalls,
  animate = true,
}: AgentBlockProps) {
  const [visible, setVisible] = useState(!animate);
  const isStreaming = status === 'streaming';
  const showText = Boolean(content?.trim()) || isStreaming;
  const hasToolCalls = Boolean(toolCalls?.length);

  useEffect(() => {
    if (animate) {
      const timer = requestAnimationFrame(() => {
        setVisible(true);
      });
      return () => cancelAnimationFrame(timer);
    }
  }, [animate]);

  return (
    <div
      className={cn(
        'w-full',
        'transition-all duration-250',
        visible ? 'translate-y-0 opacity-100' : 'translate-y-3 opacity-0',
      )}
      style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
    >
      {showText && (
        <div
          className={cn(
            'text-sm leading-relaxed text-[var(--text-primary)]',
            getBorderClass(status),
          )}
        >
          <div className="flex flex-wrap items-end gap-x-2 gap-y-0.5">
            <div className="min-w-0 flex-1">{formatContent(content, isStreaming)}</div>
            {!hasToolCalls && !reasoning && (
              <InlineTimestamp timestamp={timestamp} />
            )}
          </div>
        </div>
      )}

      {reasoning && <ReasoningBlock content={reasoning} />}

      {hasToolCalls && (
        <div className={cn(showText ? 'mt-1.5 space-y-0.5' : 'space-y-0.5')}>
          {toolCalls!.map((tc, index) => (
            <ToolCallCard
              key={tc.id}
              toolCall={tc}
              variant="inline"
              trailing={
                index === 0 && !showText && !reasoning ? (
                  <InlineTimestamp timestamp={timestamp} />
                ) : undefined
              }
            />
          ))}
        </div>
      )}

      {(hasToolCalls || reasoning) && showText && (
        <div className="mt-1 flex justify-end">
          <InlineTimestamp timestamp={timestamp} />
        </div>
      )}
    </div>
  );
}
