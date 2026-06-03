import { useState, useEffect } from 'react';
import { Copy, Check } from 'lucide-react';

interface TextPreviewProps {
  filePath: string;
  fileType: 'text' | 'code';
}

export default function TextPreview({ filePath, fileType }: TextPreviewProps) {
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const loadText = async () => {
      try {
        if (window.__TAURI__) {
          const { readFile } = await import('@/lib/fileSystem');
          const text = await readFile(filePath);
          setContent(text);
        } else {
          const response = await fetch(filePath);
          const text = await response.text();
          setContent(text);
        }
      } catch (_err) {
        setError('无法加载文件');
      } finally {
        setLoading(false);
      }
    };

    loadText();
  }, [filePath]);

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(content);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (_err) {
      console.error('Failed to copy:', _err);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2" style={{ borderColor: 'var(--accent-primary)' }} />
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
      {/* 工具栏 */}
      <div
        className="flex items-center justify-between py-2 px-4"
        style={{
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        <span className="text-sm" style={{ color: 'var(--text-tertiary)' }}>
          {content.length} 字符
        </span>
        <button
          onClick={copyToClipboard}
          className="flex items-center gap-1 px-3 py-1 rounded text-sm transition-colors"
          style={{
            backgroundColor: copied ? 'var(--status-success)' : 'transparent',
            color: copied ? 'var(--text-inverse)' : 'var(--text-secondary)',
            border: copied ? 'none' : '1px solid var(--border-primary)',
          }}
        >
          {copied ? (
            <>
              <Check size={14} />
              已复制
            </>
          ) : (
            <>
              <Copy size={14} />
              复制
            </>
          )}
        </button>
      </div>

      {/* 内容 */}
      <div className="flex-1 overflow-auto">
        <pre
          className="p-4 text-sm leading-relaxed whitespace-pre-wrap font-mono"
          style={{
            color: 'var(--text-secondary)',
            backgroundColor: fileType === 'code' ? 'var(--bg-elevated)' : 'transparent',
          }}
        >
          {content}
        </pre>
      </div>
    </div>
  );
}
