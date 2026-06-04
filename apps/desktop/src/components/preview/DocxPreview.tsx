import { useState, useEffect } from 'react';
import DOMPurify from 'dompurify';
import mammoth from 'mammoth';
import { cn } from '@/lib/utils';
import { readFileBinary, isTauri } from '@/lib/fileSystem';
import { resolveDocxPreviewPath } from '@/lib/docConversion';
import { api } from '@/api';

interface DocxPreviewProps {
  filePath: string;
  /** 旧版 .doc 只读预览（经 LibreOffice 转换） */
  legacyDoc?: boolean;
}

export default function DocxPreview({ filePath, legacyDoc = false }: DocxPreviewProps) {
  const [html, setHtml] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [converted, setConverted] = useState(false);
  const [fromCache, setFromCache] = useState(false);

  useEffect(() => {
    const loadDocx = async () => {
      try {
        setLoading(true);
        setError(null);

        const resolved = legacyDoc || filePath.toLowerCase().endsWith('.doc')
          ? await resolveDocxPreviewPath(filePath)
          : { previewPath: filePath, converted: false, fromCache: false };

        setConverted(resolved.converted);
        setFromCache(resolved.fromCache);

        const data = await readFileBinary(resolved.previewPath);
        const arrayBuffer = data.buffer.slice(
          data.byteOffset,
          data.byteOffset + data.byteLength,
        ) as ArrayBuffer;

        const result = await mammoth.convertToHtml({ arrayBuffer });
        const sanitized = DOMPurify.sanitize(result.value, {
          ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'ul', 'ol', 'li', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'table', 'thead', 'tbody', 'tr', 'th', 'td', 'pre', 'code', 'blockquote', 'img', 'a'],
          ALLOWED_ATTR: ['class', 'src', 'alt', 'href', 'target'],
        });
        setHtml(sanitized);
      } catch (err) {
        const message =
          err instanceof Error ? err.message : '无法解析 Word 文档';
        setError(message);
      } finally {
        setLoading(false);
      }
    };

    void loadDocx();
  }, [filePath, legacyDoc]);

  const handleRevealSource = async () => {
    if (isTauri()) {
      await api.revealPathInFileManager(filePath);
    }
  };

  if (loading) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2">
        <div
          className="h-8 w-8 animate-spin rounded-full border-b-2"
          style={{ borderColor: 'var(--accent-primary)' }}
        />
        {legacyDoc && (
          <p className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
            正在通过 LibreOffice 转换 .doc …
          </p>
        )}
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full flex-col items-center justify-center px-6 text-center">
        <div className="mb-3 text-4xl">📄</div>
        <p className="font-medium" style={{ color: 'var(--status-error)' }}>
          {error}
        </p>
        <p className="mt-2 max-w-md text-sm" style={{ color: 'var(--text-secondary)' }}>
          {legacyDoc
            ? '请安装 LibreOffice（需包含 soffice 命令），或在 Word 中将文件另存为 .docx 后再预览。'
            : '该 Word 文档可能包含复杂格式或已损坏'}
        </p>
        {isTauri() && (
          <button
            type="button"
            onClick={() => void handleRevealSource()}
            className="mt-4 rounded-md px-3 py-1.5 text-sm transition-colors hover:bg-[var(--bg-hover)]"
            style={{
              color: 'var(--text-link)',
              border: '1px solid var(--border-default)',
            }}
          >
            在 Finder 中显示原文件
          </button>
        )}
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      {converted && (
        <div
          className={cn(
            'shrink-0 border-b px-4 py-2 text-xs',
            'bg-[var(--status-info-bg)] text-[var(--text-secondary)]',
          )}
          style={{ borderColor: 'var(--border-default)' }}
        >
          已从旧版 .doc 转换为 .docx 预览
          {fromCache ? '（使用缓存）' : ''}，仅只读。
        </div>
      )}
      <div className="flex-1 overflow-auto p-6">
        <div
          className="mx-auto max-w-4xl rounded-lg p-6 shadow-lg"
          style={{
            backgroundColor: 'var(--bg-elevated)',
            minHeight: '70vh',
          }}
        >
          <div
            className="prose prose-invert max-w-none"
            dangerouslySetInnerHTML={{ __html: html }}
            style={{ color: 'var(--text-primary)' }}
          />
        </div>
      </div>
    </div>
  );
}
