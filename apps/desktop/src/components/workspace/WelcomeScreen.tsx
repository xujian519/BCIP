import { FolderOpen } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';

export default function WelcomeScreen() {
  const { state } = useAppStore();
  const hasWorkspace = !!state.workspaceCwd;

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 px-8">
      <FolderOpen size={36} style={{ color: 'var(--text-tertiary)', opacity: 0.4 }} />
      {hasWorkspace ? (
        <>
          <p className="text-sm" style={{ color: 'var(--text-secondary)' }}>
            从左侧文件浏览器选择文件开始工作
          </p>
          <p className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
            支持 Markdown 编辑、PDF 预览、DOCX 编辑、代码查看
          </p>
        </>
      ) : (
        <>
          <p className="text-sm" style={{ color: 'var(--text-secondary)' }}>
            打开一个工作区目录开始工作
          </p>
          <p className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
            点击左侧文件浏览器图标打开目录
          </p>
        </>
      )}
    </div>
  );
}