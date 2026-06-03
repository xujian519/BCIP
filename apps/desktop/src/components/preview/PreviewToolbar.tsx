import { Download, ExternalLink, FileText, Image, FileCode, File } from 'lucide-react';
import type { FileType } from '@/lib/fileType';

interface PreviewToolbarProps {
  fileName: string;
  fileType: FileType;
}

const typeLabels: Record<FileType, { label: string; icon: React.ReactNode }> = {
  pdf: { label: 'PDF', icon: <FileText size={16} /> },
  image: { label: '图片', icon: <Image size={16} /> },
  markdown: { label: 'Markdown', icon: <FileText size={16} /> },
  docx: { label: 'Word (.docx)', icon: <FileText size={16} /> },
  doc: { label: 'Word (.doc)', icon: <FileText size={16} /> },
  text: { label: '文本', icon: <File size={16} /> },
  code: { label: '代码', icon: <FileCode size={16} /> },
  unknown: { label: '未知', icon: <File size={16} /> },
};

export default function PreviewToolbar({ fileName, fileType }: PreviewToolbarProps) {
  const typeInfo = typeLabels[fileType];

  return (
    <div
      className="flex items-center justify-between px-4 py-3"
      style={{
        backgroundColor: 'var(--bg-elevated)',
        borderBottom: '1px solid var(--border-primary)',
      }}
    >
      <div className="flex items-center gap-3">
        <span style={{ color: 'var(--accent-primary)' }}>
          {typeInfo.icon}
        </span>
        <div className="flex flex-col">
          <span className="text-sm font-medium truncate max-w-[300px]"
            style={{ color: 'var(--text-primary)' }}
            title={fileName}
          >
            {fileName}
          </span>
          <span className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
            {typeInfo.label}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button
          className="flex items-center gap-1 px-3 py-1.5 rounded text-sm transition-colors"
          style={{
            backgroundColor: 'var(--bg-base)',
            color: 'var(--text-secondary)',
            border: '1px solid var(--border-primary)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-base)';
          }}
        >
          <Download size={14} />
          下载
        </button>
        <button
          className="flex items-center gap-1 px-3 py-1.5 rounded text-sm transition-colors"
          style={{
            backgroundColor: 'var(--bg-base)',
            color: 'var(--text-secondary)',
            border: '1px solid var(--border-primary)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-base)';
          }}
        >
          <ExternalLink size={14} />
          外部打开
        </button>
      </div>
    </div>
  );
}
