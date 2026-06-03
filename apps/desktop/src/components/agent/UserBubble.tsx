/**
 * UserBubble —— 用户消息气泡
 */
import { cn } from '@/lib/utils';
import { useState, useEffect } from 'react';

interface UserBubbleProps {
  content: string;
  timestamp?: string;
  animate?: boolean;
}

function formatContent(content: string) {
  const parts = content.split(/(```[\s\S]*?```|`[^`]+`)/g);
  return parts.map((part, index) => {
    if (part.startsWith('```') && part.endsWith('```')) {
      const code = part.slice(3, -3).trim();
      return (
        <pre
          key={index}
          className={cn(
            'my-0.5 overflow-x-auto rounded-md bg-black/5 p-1.5 font-mono text-xs',
            'text-[var(--text-primary)]',
          )}
        >
          <code>{code}</code>
        </pre>
      );
    }
    if (part.startsWith('`') && part.endsWith('`') && part.length > 2) {
      const code = part.slice(1, -1);
      return (
        <code
          key={index}
          className={cn(
            'rounded bg-black/5 px-1 py-0.5 font-mono text-xs',
            'text-[var(--text-primary)]',
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
  });
}

export default function UserBubble({
  content,
  timestamp,
  animate = true,
}: UserBubbleProps) {
  const [visible, setVisible] = useState(!animate);

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
        'flex w-full justify-end',
        'transition-all duration-250',
        visible ? 'translate-x-0 opacity-100' : 'translate-x-4 opacity-0',
      )}
      style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
    >
      <div
        className={cn(
          'max-w-[85%] text-sm leading-normal text-[var(--text-primary)]',
          'bg-[var(--accent-primary-muted)]',
          'rounded-2xl rounded-br-md',
          'shadow-sm',
          'border border-[var(--accent-primary-muted)]',
          'hover:shadow-md',
          'transition-shadow duration-200',
        )}
        style={{
          padding: 'var(--chat-bubble-py) var(--chat-bubble-px)',
        }}
      >
        <div className="flex flex-wrap items-end justify-end gap-x-2 gap-y-0.5">
          <div className="min-w-0 flex-1 text-right">{formatContent(content)}</div>
          {timestamp && (
            <span className="shrink-0 text-[10px] text-[var(--text-tertiary)] opacity-50">
              {timestamp}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
