import { useState, useEffect } from 'react';
import { FolderOpen, FileText, FileCode, FilePen, Terminal } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import { cn } from '@/lib/utils';

const features = [
  { icon: FileText, label: 'Markdown 编辑', shortcut: '⌘N' },
  { icon: FileCode, label: 'PDF 预览', shortcut: '' },
  { icon: FilePen, label: 'DOCX 编辑', shortcut: '' },
  { icon: Terminal, label: '代码查看', shortcut: '' },
];

function FeatureBadge({
  icon: Icon,
  label,
  index,
}: {
  icon: typeof FileText;
  label: string;
  index: number;
}) {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => setVisible(true), 400 + index * 80);
    return () => clearTimeout(timer);
  }, [index]);

  return (
    <div
      className={cn(
        'flex items-center gap-2 rounded-full px-3 py-1.5',
        'bg-[var(--bg-surface)] border border-[var(--border-default)]',
        'shadow-sm',
        'transition-all duration-300',
        visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2',
      )}
      style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
    >
      <Icon size={13} className="text-[var(--text-tertiary)]" />
      <span className="text-xs text-[var(--text-secondary)]">{label}</span>
    </div>
  );
}

export default function WelcomeScreen() {
  const { state, dispatch } = useAppStore();
  const hasWorkspace = !!state.workspaceCwd;
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const timer = requestAnimationFrame(() => setVisible(true));
    return () => cancelAnimationFrame(timer);
  }, []);

  const handleOpenFiles = () => {
    dispatch({ type: 'SET_ACTIVITY_BAR_TAB', payload: 'files' });
  };

  return (
    <div className="flex h-full flex-col items-center justify-center gap-5 px-8 select-none">
      {/* 品牌图标 */}
      <div
        className={cn(
          'relative flex h-16 w-16 items-center justify-center rounded-2xl',
          'bg-[var(--bg-surface)] border border-[var(--border-default)]',
          'shadow-md',
          'transition-all duration-500',
          visible ? 'opacity-100 scale-100' : 'opacity-0 scale-90',
        )}
        style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
      >
        <FolderOpen size={28} className="text-[var(--accent-primary)]" />
        {/* 品牌色微光 */}
        <div
          className="absolute inset-0 rounded-2xl opacity-0 hover:opacity-100 transition-opacity duration-300"
          style={{
            boxShadow: '0 0 20px rgba(74, 124, 111, 0.15)',
          }}
        />
      </div>

      {/* 主标题 */}
      <div
        className={cn(
          'flex flex-col items-center gap-1.5',
          'transition-all duration-500 delay-100',
          visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-3',
        )}
      >
        <h2 className="text-base font-semibold text-[var(--text-primary)] tracking-tight">
          {hasWorkspace ? '选择文件开始工作' : '打开工作区'}
        </h2>
        <p className="text-sm text-[var(--text-secondary)] text-center max-w-[280px] leading-relaxed">
          {hasWorkspace
            ? '从左侧文件浏览器选择文件，或拖拽文件到此处'
            : '点击左侧资源管理器图标，打开一个项目目录'}
        </p>
      </div>

      {/* 功能标签 */}
      <div
        className={cn(
          'flex flex-wrap items-center justify-center gap-2 mt-1',
          'transition-all duration-500 delay-200',
          visible ? 'opacity-100' : 'opacity-0',
        )}
      >
        {features.map((feat, i) => (
          <FeatureBadge key={feat.label} icon={feat.icon} label={feat.label} index={i} />
        ))}
      </div>

      {/* 快速操作提示 */}
      {hasWorkspace && (
        <button
          type="button"
          onClick={handleOpenFiles}
          className={cn(
            'mt-2 flex items-center gap-2 rounded-xl px-4 py-2',
            'bg-[var(--bg-surface)] border border-[var(--border-default)]',
            'text-sm text-[var(--text-secondary)]',
            'hover:bg-[var(--bg-hover)] hover:border-[var(--border-hover)]',
            'hover:text-[var(--text-primary)]',
            'transition-all duration-200',
            'cursor-pointer',
          )}
          style={{ transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }}
        >
          <FolderOpen size={14} />
          浏览文件
        </button>
      )}
    </div>
  );
}
