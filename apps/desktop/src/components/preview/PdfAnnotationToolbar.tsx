/**
 * PDF 标注工具栏（阶段 6.5）：与页面导航/缩放分离，供 PdfPreview 复用
 */
import { useState } from 'react';
import {
  Highlighter,
  Underline,
  MessageSquare,
  MousePointer,
  Palette,
} from 'lucide-react';

export type PdfToolMode = 'pointer' | 'highlight' | 'underline' | 'note';

export const PDF_HIGHLIGHT_COLORS = ['#ffeb3b', '#4caf50', '#f44336', '#2196f3'];

interface PdfAnnotationToolbarProps {
  toolMode: PdfToolMode;
  onToolModeChange: (mode: PdfToolMode) => void;
  highlightColor: string;
  onHighlightColorChange: (color: string) => void;
  annotationCount: number;
}

function ToolBtn({
  active,
  onClick,
  title,
  children,
}: {
  active: boolean;
  onClick: () => void;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className="p-1.5 rounded transition-colors"
      style={{
        color: active ? 'var(--accent-primary)' : 'var(--text-tertiary)',
        backgroundColor: active ? 'var(--accent-primary-muted)' : 'transparent',
      }}
      title={title}
      type="button"
    >
      {children}
    </button>
  );
}

export default function PdfAnnotationToolbar({
  toolMode,
  onToolModeChange,
  highlightColor,
  onHighlightColorChange,
  annotationCount,
}: PdfAnnotationToolbarProps) {
  const [showColorPicker, setShowColorPicker] = useState(false);

  return (
    <div className="flex items-center gap-1">
      <ToolBtn
        active={toolMode === 'pointer'}
        onClick={() => onToolModeChange('pointer')}
        title="指针"
      >
        <MousePointer size={14} />
      </ToolBtn>
      <ToolBtn
        active={toolMode === 'highlight'}
        onClick={() => onToolModeChange('highlight')}
        title="高亮"
      >
        <Highlighter size={14} />
      </ToolBtn>
      <ToolBtn
        active={toolMode === 'underline'}
        onClick={() => onToolModeChange('underline')}
        title="下划线"
      >
        <Underline size={14} />
      </ToolBtn>
      <ToolBtn
        active={toolMode === 'note'}
        onClick={() => onToolModeChange('note')}
        title="批注"
      >
        <MessageSquare size={14} />
      </ToolBtn>
      {(toolMode === 'highlight' || toolMode === 'underline') && (
        <div className="relative">
          <button
            onClick={() => setShowColorPicker((v) => !v)}
            className="p-1 rounded"
            style={{ color: 'var(--text-secondary)' }}
            type="button"
          >
            <Palette size={14} />
          </button>
          {showColorPicker && (
            <div
              className="absolute top-full mt-1 flex gap-1 p-1.5 rounded shadow-lg z-20"
              style={{
                backgroundColor: 'var(--bg-elevated)',
                border: '1px solid var(--border-primary)',
              }}
            >
              {PDF_HIGHLIGHT_COLORS.map((c) => (
                <button
                  key={c}
                  onClick={() => {
                    onHighlightColorChange(c);
                    setShowColorPicker(false);
                  }}
                  className="rounded-full border-2 transition-transform"
                  style={{
                    width: 18,
                    height: 18,
                    backgroundColor: c,
                    borderColor:
                      highlightColor === c ? 'var(--text-primary)' : 'transparent',
                    transform: highlightColor === c ? 'scale(1.2)' : 'scale(1)',
                  }}
                  type="button"
                />
              ))}
            </div>
          )}
        </div>
      )}
      <span className="mx-1" style={{ color: 'var(--border-primary)' }}>
        |
      </span>
      <span className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
        {annotationCount > 0 ? `${annotationCount} 个标注` : '暂无标注'}
      </span>
    </div>
  );
}
