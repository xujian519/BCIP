/**
 * ToolCallCard —— 工具调用（inline 紧凑行 / card 完整卡片）
 */
import { cn } from '@/lib/utils';
import {
  Terminal,
  FileText,
  Search,
  ChevronDown,
  Check,
  X,
  Loader2,
  Plug,
  Files,
} from 'lucide-react';
import { useState, useRef, useEffect, type ReactNode } from 'react';
import type { ToolCall } from '@/types';

interface ToolCallCardProps {
  toolCall: ToolCall;
  /** 行内附加内容（如时间戳） */
  trailing?: ReactNode;
  /** inline：单行文字；card：完整卡片（默认 inline） */
  variant?: 'inline' | 'card';
}

function ToolCallLeadingIcon({ toolCall }: { toolCall: ToolCall }) {
  const className = 'text-[var(--text-tertiary)] shrink-0';
  if (toolCall.kind === 'mcp') {
    return <Plug size={12} className={className} />;
  }
  if (toolCall.kind === 'patch') {
    return <Files size={12} className={className} />;
  }
  const lower = toolCall.name.toLowerCase();
  if (lower.includes('search') || lower.includes('find')) {
    return <Search size={12} className={className} />;
  }
  if (lower.includes('file') || lower.includes('read') || lower.includes('write')) {
    return <FileText size={12} className={className} />;
  }
  return <Terminal size={12} className={className} />;
}

function displayTitle(toolCall: ToolCall): string {
  if (toolCall.kind === 'mcp' && toolCall.detail) {
    return `${toolCall.detail} · ${toolCall.name}`;
  }
  if (toolCall.kind === 'shell' && toolCall.detail) {
    return toolCall.detail;
  }
  if (toolCall.kind === 'patch') {
    return '文件变更';
  }
  return toolCall.name;
}

function DetailPre({
  children,
  className,
  error,
}: {
  children: ReactNode;
  className?: string;
  error?: boolean;
}) {
  return (
    <pre
      className={cn(
        'font-mono text-2xs',
        error ? 'text-[var(--status-error)]' : 'text-[var(--text-secondary)]',
        className,
      )}
    >
      {children}
    </pre>
  );
}

function StatusIcon({ status }: { status: ToolCall['status'] }) {
  switch (status) {
    case 'running':
      return (
        <Loader2 size={11} className="animate-spin text-[var(--status-warning)]" />
      );
    case 'success':
      return <Check size={11} className="text-[var(--status-success)]" />;
    case 'error':
      return <X size={11} className="text-[var(--status-error)]" />;
    default:
      return null;
  }
}

function ToolCallDetails({
  toolCall,
  isExpanded,
  contentHeight,
  contentRef,
  measureRef,
}: {
  toolCall: ToolCall;
  isExpanded: boolean;
  contentHeight: number;
  contentRef: React.RefObject<HTMLDivElement | null>;
  measureRef: React.RefObject<HTMLDivElement | null>;
}) {
  return (
    <>
      <div
        ref={contentRef}
        className="overflow-hidden transition-[height] duration-250 ease-in-out"
        style={{ height: isExpanded ? contentHeight : 0 }}
      >
        <div
          className={cn(
            'space-y-1.5 transition-opacity duration-150',
            isExpanded ? 'opacity-100' : 'opacity-0',
          )}
          style={{
            padding: `0 var(--chat-tool-px) var(--chat-tool-py)`,
          }}
        >
          {toolCall.kind === 'patch' && toolCall.detail && (
            <DetailPre className="whitespace-pre-wrap break-all max-h-32 overflow-y-auto">
              {toolCall.detail}
            </DetailPre>
          )}
          {toolCall.args && (
            <div>
              <span className="text-2xs text-[var(--text-tertiary)]">参数</span>
              <DetailPre className="mt-0.5 whitespace-pre-wrap break-all max-h-32 overflow-y-auto bg-black/20 rounded p-1.5">
                {toolCall.args}
              </DetailPre>
            </div>
          )}
          {(toolCall.output || toolCall.error) && (
            <div>
              <span className="text-2xs text-[var(--text-tertiary)]">
                {toolCall.error ? '错误' : '输出'}
              </span>
              <DetailPre
                className="mt-0.5 whitespace-pre-wrap break-all max-h-40 overflow-y-auto bg-black/30 rounded p-1.5"
                error={!!toolCall.error}
              >
                {toolCall.error ?? toolCall.output}
              </DetailPre>
            </div>
          )}
        </div>
      </div>

      <div ref={measureRef} className="absolute invisible pointer-events-none w-full">
        <div
          className="space-y-1.5"
          style={{ padding: `0 var(--chat-tool-px) var(--chat-tool-py)` }}
        >
          {toolCall.kind === 'patch' && toolCall.detail && (
            <DetailPre>{toolCall.detail}</DetailPre>
          )}
          {toolCall.args && <DetailPre>{toolCall.args}</DetailPre>}
          {(toolCall.output || toolCall.error) && (
            <DetailPre error={!!toolCall.error}>
              {toolCall.error ?? toolCall.output}
            </DetailPre>
          )}
        </div>
      </div>
    </>
  );
}

export default function ToolCallCard({
  toolCall,
  trailing,
  variant = 'inline',
}: ToolCallCardProps) {
  const [isExpanded, setIsExpanded] = useState(toolCall.status === 'running');
  const [contentHeight, setContentHeight] = useState(0);
  const contentRef = useRef<HTMLDivElement>(null);
  const measureRef = useRef<HTMLDivElement>(null);

  const title = displayTitle(toolCall);
  const hasBody = Boolean(toolCall.detail || toolCall.args || toolCall.output || toolCall.error);

  useEffect(() => {
    if (measureRef.current) {
      setContentHeight(measureRef.current.scrollHeight);
    }
  }, [toolCall.output, toolCall.error, toolCall.args, toolCall.detail]);

  const toggleExpanded = () => {
    if (!isExpanded && contentRef.current && contentHeight > 0) {
      contentRef.current.style.height = `${contentHeight}px`;
    }
    setIsExpanded((prev) => !prev);
  };

  if (variant === 'inline') {
    return (
      <div className="w-full">
        <button
          type="button"
          onClick={hasBody ? toggleExpanded : undefined}
          disabled={!hasBody}
          className={cn(
            'flex w-full items-center gap-1.5 py-0.5 text-left',
            hasBody && 'cursor-pointer hover:opacity-80',
            !hasBody && 'cursor-default',
          )}
        >
          <ToolCallLeadingIcon toolCall={toolCall} />
          <span className="min-w-0 flex-1 truncate font-mono text-xs text-[var(--text-secondary)]">
            {title}
          </span>
          <div className="flex shrink-0 items-center gap-1">
            {trailing}
            <StatusIcon status={toolCall.status} />
            {hasBody && (
              <ChevronDown
                size={11}
                className={cn(
                  'text-[var(--text-tertiary)] transition-transform duration-150',
                  isExpanded ? 'rotate-180' : 'rotate-0',
                )}
              />
            )}
          </div>
        </button>
        {hasBody && isExpanded && (
          <div className="ml-4 border-l border-[var(--border-default)] py-1 pl-2"
            style={{
              animation: 'message-enter 0.15s ease-out forwards',
            }}
          >
            {toolCall.kind === 'patch' && toolCall.detail && (
              <DetailPre className="whitespace-pre-wrap break-all">
                {toolCall.detail}
              </DetailPre>
            )}
            {toolCall.args && (
              <DetailPre className="whitespace-pre-wrap break-all">
                {toolCall.args}
              </DetailPre>
            )}
            {(toolCall.output || toolCall.error) && (
              <DetailPre className="whitespace-pre-wrap break-all" error={!!toolCall.error}>
                {toolCall.error ?? toolCall.output}
              </DetailPre>
            )}
          </div>
        )}
      </div>
    );
  }

  return (
    <div
      className={cn(
        'bg-[var(--bg-surface)] rounded-md',
        'border border-[var(--border-default)]',
        'overflow-hidden',
      )}
    >
      <button
        type="button"
        onClick={toggleExpanded}
        disabled={!hasBody}
        className={cn(
          'w-full flex items-center gap-1.5',
          hasBody && 'hover:bg-[var(--bg-hover)]',
          'transition-colors duration-150 cursor-pointer select-none',
          !hasBody && 'cursor-default',
        )}
        style={{
          padding: 'var(--chat-tool-py) var(--chat-tool-px)',
        }}
      >
        <ToolCallLeadingIcon toolCall={toolCall} />
        <span className="text-xs font-medium text-[var(--text-primary)] flex-1 text-left truncate">
          {title}
        </span>
        <div className="flex items-center gap-1 shrink-0">
          {trailing}
          <StatusIcon status={toolCall.status} />
          {hasBody && (
            <div
              className={cn(
                'transition-transform duration-150',
                isExpanded ? 'rotate-180' : 'rotate-0',
              )}
            >
              <ChevronDown size={12} className="text-[var(--text-tertiary)]" />
            </div>
          )}
        </div>
      </button>

      {hasBody && (
        <ToolCallDetails
          toolCall={toolCall}
          isExpanded={isExpanded}
          contentHeight={contentHeight}
          contentRef={contentRef}
          measureRef={measureRef}
        />
      )}
    </div>
  );
}
