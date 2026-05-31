/**
 * 按扩展名路由到对应预览组件（spec §8.2）
 * PDF 使用自带工具栏，避免与 PreviewToolbar 重复。
 */
import { useMemo, useState, Suspense } from 'react';
import { getFileType } from '@/lib/fileType';
import PdfPreview from './PdfPreview';
import ImagePreview from './ImagePreview';
import MarkdownPreview from './MarkdownPreview';
import DocxPreview from './DocxPreview';
import DocxEditorView from './DocxEditorView';
import TextPreview from './TextPreview';
import PreviewToolbar from './PreviewToolbar';

interface FilePreviewRouterProps {
  filePath: string;
}

function DocxEditorWithErrorFallback({ filePath }: { filePath: string }) {
  const [editorError, setEditorError] = useState<Error | null>(null);

  if (editorError) {
    return <DocxPreview filePath={filePath} />;
  }

  return (
    <ErrorBoundary onError={setEditorError}>
      <DocxEditorView filePath={filePath} />
    </ErrorBoundary>
  );
}

class ErrorBoundary extends React.Component<{ children: React.ReactNode; onError: (error: Error) => void }, { hasError: boolean }> {
  constructor(props: { children: React.ReactNode; onError: (error: Error) => void }) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError() {
    return { hasError: true };
  }

  componentDidCatch(error: Error) {
    this.props.onError(error);
  }

  render() {
    if (this.state.hasError) {
      return null;
    }
    return this.props.children;
  }
}

export default function FilePreviewRouter({ filePath }: FilePreviewRouterProps) {
  const fileName = useMemo(() => filePath.split('/').pop() || '', [filePath]);
  const fileType = useMemo(() => getFileType(fileName), [fileName]);

  if (fileType === 'pdf') {
    return <PdfPreview filePath={filePath} />;
  }

  const renderPreview = () => {
    switch (fileType) {
      case 'image':
        return <ImagePreview filePath={filePath} />;
      case 'markdown':
        return <MarkdownPreview filePath={filePath} />;
      case 'docx':
        return <DocxEditorWithErrorFallback filePath={filePath} />;
      case 'text':
      case 'code':
        return <TextPreview filePath={filePath} fileType={fileType} />;
      default:
        return (
          <div className="flex flex-col items-center justify-center h-full">
            <div className="text-6xl mb-4">📎</div>
            <h3
              className="text-lg font-medium mb-2"
              style={{ color: 'var(--text-primary)' }}
            >
              无法预览此文件
            </h3>
            <p
              className="text-sm text-center max-w-md"
              style={{ color: 'var(--text-secondary)' }}
            >
              该文件格式暂不支持预览。您可以在外部应用中打开它。
            </p>
          </div>
        );
    }
  };

  return (
    <div className="flex flex-col h-full">
      <PreviewToolbar fileName={fileName} fileType={fileType} />
      <div className="flex-1 overflow-auto">{renderPreview()}</div>
    </div>
  );
}
