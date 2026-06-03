import { useState, useEffect, useCallback, useRef } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import 'highlight.js/styles/github-dark.css';
import { Bold, Italic, Heading1, Heading2, List, ListOrdered, Quote } from 'lucide-react';
import { api } from '@/api';

interface MarkdownPreviewProps {
  filePath: string;
}

type SaveStatus = 'saved' | 'editing' | 'saving';

export default function MarkdownPreview({ filePath }: MarkdownPreviewProps) {
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [saveStatus, setSaveStatus] = useState<SaveStatus>('saved');
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const data = await api.readFile(filePath);
        setContent(data);
      } catch {
        setError('无法加载 Markdown 文件');
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [filePath]);

  const autoSave = useCallback(
    (text: string) => {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
      setSaveStatus('editing');
      saveTimerRef.current = setTimeout(async () => {
        setSaveStatus('saving');
        try {
          await api.writeFile({ path: filePath, content: text });
          setSaveStatus('saved');
        } catch {
          setSaveStatus('saved');
        }
      }, 2000);
    },
    [filePath]
  );

  const handleChange = (value: string) => {
    setContent(value);
    autoSave(value);
  };

  const insertMarkdown = useCallback(
    (before: string, after = '') => {
      const el = textareaRef.current;
      if (!el) return;
      const start = el.selectionStart;
      const end = el.selectionEnd;
      const selected = content.substring(start, end);
      const newText =
        content.substring(0, start) + before + selected + after + content.substring(end);
      setContent(newText);
      autoSave(newText);
      setTimeout(() => {
        el.focus();
        el.setSelectionRange(start + before.length, start + before.length + selected.length);
      }, 0);
    },
    [content, autoSave]
  );

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div
          className="animate-spin rounded-full h-8 w-8 border-b-2"
          style={{ borderColor: 'var(--accent-primary)' }}
        />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <p style={{ color: 'var(--status-error)' }}>{error}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* 顶部操作栏 */}
      <div
        className="flex items-center justify-between"
        style={{
          height: 36,
          padding: '0 8px',
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        <div className="flex items-center gap-1">
          {isEditing && (
            <>
              <ToolBtn title="粗体" onClick={() => insertMarkdown('**', '**')}>
                <Bold size={14} />
              </ToolBtn>
              <ToolBtn title="斜体" onClick={() => insertMarkdown('*', '*')}>
                <Italic size={14} />
              </ToolBtn>
              <ToolBtn title="标题 1" onClick={() => insertMarkdown('# ')}>
                <Heading1 size={14} />
              </ToolBtn>
              <ToolBtn title="标题 2" onClick={() => insertMarkdown('## ')}>
                <Heading2 size={14} />
              </ToolBtn>
              <ToolBtn title="无序列表" onClick={() => insertMarkdown('- ')}>
                <List size={14} />
              </ToolBtn>
              <ToolBtn title="有序列表" onClick={() => insertMarkdown('1. ')}>
                <ListOrdered size={14} />
              </ToolBtn>
              <ToolBtn title="引用" onClick={() => insertMarkdown('> ')}>
                <Quote size={14} />
              </ToolBtn>
            </>
          )}
        </div>
        <div className="flex items-center gap-2">
          {isEditing && (
            <span
              className="text-xs flex items-center gap-1"
              style={{ color: saveStatus === 'saved' ? 'var(--status-success)' : 'var(--status-warning)' }}
            >
              <span
                className="inline-block rounded-full"
                style={{
                  width: 6,
                  height: 6,
                  backgroundColor:
                    saveStatus === 'saved' ? 'var(--status-success)' : 'var(--status-warning)',
                }}
              />
              {saveStatus === 'saved' ? '已保存' : saveStatus === 'saving' ? '保存中...' : '编辑中...'}
            </span>
          )}
          <button
            onClick={() => setIsEditing(!isEditing)}
            className="text-xs px-2 py-1 rounded transition-colors"
            style={{
              backgroundColor: isEditing ? 'var(--accent-primary)' : 'var(--bg-sidebar-active)',
              color: isEditing ? 'var(--text-inverse)' : 'var(--text-secondary)',
            }}
            type="button"
          >
            {isEditing ? '预览' : '编辑'}
          </button>
        </div>
      </div>

      {/* 编辑 / 预览区域 */}
      {isEditing ? (
        <div className="flex flex-1 overflow-hidden">
          {/* 编辑区 */}
          <textarea
            ref={textareaRef}
            value={content}
            onChange={(e) => handleChange(e.target.value)}
            className="flex-1 resize-none bg-transparent p-4 focus:outline-none"
            style={{
              fontFamily: "'JetBrains Mono', 'SF Mono', monospace",
              fontSize: 13,
              lineHeight: 1.7,
              color: 'var(--text-primary)',
              borderRight: '1px solid var(--border-primary)',
            }}
            placeholder="输入 Markdown 内容..."
          />
          {/* 预览区 */}
          <div
            className="flex-1 overflow-auto p-4"
            style={{ borderLeft: '1px solid var(--border-primary)' }}
          >
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight]}
              components={markdownComponents}
            >
              {content}
            </ReactMarkdown>
          </div>
        </div>
      ) : (
        <div className="flex-1 overflow-auto p-8">
          <div className="max-w-4xl mx-auto">
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight]}
              components={markdownComponents}
            >
              {content}
            </ReactMarkdown>
          </div>
        </div>
      )}
    </div>
  );
}

const ToolBtn: React.FC<{
  title: string;
  onClick?: () => void;
  children: React.ReactNode;
}> = ({ title, onClick, children }) => (
  <button
    onClick={onClick}
    className="p-1 rounded transition-colors"
    style={{ color: 'var(--text-tertiary)' }}
    onMouseEnter={(e) => {
      e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
      e.currentTarget.style.color = 'var(--text-secondary)';
    }}
    onMouseLeave={(e) => {
      e.currentTarget.style.backgroundColor = 'transparent';
      e.currentTarget.style.color = 'var(--text-tertiary)';
    }}
    title={title}
    type="button"
  >
    {children}
  </button>
);

const markdownComponents = {
  h1: ({ children }: { children?: React.ReactNode }) => (
    <h1 className="text-2xl font-bold mb-4 mt-6 pb-2 border-b"
      style={{ borderColor: 'var(--border-primary)', color: 'var(--text-primary)' }}
    >
      {children}
    </h1>
  ),
  h2: ({ children }: { children?: React.ReactNode }) => (
    <h2 className="text-xl font-bold mb-3 mt-5 pb-2 border-b"
      style={{ borderColor: 'var(--border-primary)', color: 'var(--text-primary)' }}
    >
      {children}
    </h2>
  ),
  h3: ({ children }: { children?: React.ReactNode }) => (
    <h3 className="text-lg font-semibold mb-2 mt-4"
      style={{ color: 'var(--text-primary)' }}
    >
      {children}
    </h3>
  ),
  p: ({ children }: { children?: React.ReactNode }) => (
    <p className="mb-4 leading-relaxed"
      style={{ color: 'var(--text-secondary)' }}
    >
      {children}
    </p>
  ),
  code: ({ children, className }: { children?: React.ReactNode; className?: string }) => {
    const isInline = !className;
    return isInline ? (
      <code className="px-1.5 py-0.5 rounded text-sm"
        style={{
          backgroundColor: 'var(--bg-elevated)',
          color: 'var(--accent-primary)',
        }}
      >
        {children}
      </code>
    ) : (
      <code className={className}>{children}</code>
    );
  },
  pre: ({ children }: { children?: React.ReactNode }) => (
    <pre className="p-4 rounded-lg mb-4 overflow-x-auto"
      style={{ backgroundColor: 'var(--bg-elevated)' }}
    >
      {children}
    </pre>
  ),
  blockquote: ({ children }: { children?: React.ReactNode }) => (
    <blockquote className="pl-4 border-l-4 italic my-4"
      style={{ borderColor: 'var(--accent-primary)', color: 'var(--text-tertiary)' }}
    >
      {children}
    </blockquote>
  ),
  ul: ({ children }: { children?: React.ReactNode }) => (
    <ul className="list-disc pl-6 mb-4" style={{ color: 'var(--text-secondary)' }}>
      {children}
    </ul>
  ),
  ol: ({ children }: { children?: React.ReactNode }) => (
    <ol className="list-decimal pl-6 mb-4" style={{ color: 'var(--text-secondary)' }}>
      {children}
    </ol>
  ),
  a: ({ children, href }: { children?: React.ReactNode; href?: string }) => (
    <a href={href} className="underline hover:no-underline transition-colors"
      style={{ color: 'var(--accent-primary)' }}
      target="_blank"
      rel="noopener noreferrer"
    >
      {children}
    </a>
  ),
  table: ({ children }: { children?: React.ReactNode }) => (
    <table className="w-full border-collapse mb-4"
      style={{ borderColor: 'var(--border-primary)' }}
    >
      {children}
    </table>
  ),
  th: ({ children }: { children?: React.ReactNode }) => (
    <th className="border px-4 py-2 text-left font-semibold"
      style={{
        borderColor: 'var(--border-primary)',
        backgroundColor: 'var(--bg-elevated)',
        color: 'var(--text-primary)',
      }}
    >
      {children}
    </th>
  ),
  td: ({ children }: { children?: React.ReactNode }) => (
    <td className="border px-4 py-2"
      style={{ borderColor: 'var(--border-primary)', color: 'var(--text-secondary)' }}
    >
      {children}
    </td>
  ),
};
