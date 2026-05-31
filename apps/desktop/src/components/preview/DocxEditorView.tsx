import { useEffect, useRef, forwardRef, useImperativeHandle, useState } from 'react';
import { DocxEditor } from '@eigenpal/docx-editor-react';
import '@eigenpal/docx-editor-react/dist/styles.css';
import { readFileBinary, isTauri } from '@/lib/fileSystem';
import type { DocxEditorRef } from '@eigenpal/docx-editor-react';

interface DocxEditorViewProps {
  filePath: string;
}

export interface DocxEditorViewRef {
  save: () => Promise<void>;
  load: () => Promise<void>;
}

export default forwardRef<DocxEditorViewRef, DocxEditorViewProps>(
  function DocxEditorView({ filePath }, ref) {
    const editorRef = useRef<DocxEditorRef>(null);
    const [buffer, setBuffer] = useState<ArrayBuffer | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [isDirty, setIsDirty] = useState(false);

    const loadFile = async () => {
      setLoading(true);
      setError(null);
      try {
        const data = await readFileBinary(filePath);
        setBuffer(data.buffer as ArrayBuffer);
      } catch (err) {
        console.error('Failed to load DOCX file:', err);
        setError('无法加载 Word 文档');
      } finally {
        setLoading(false);
      }
    };

    const saveFile = async () => {
      if (!editorRef.current) return;
      try {
        const savedBuffer = await editorRef.current.save();
        if (savedBuffer && isTauri()) {
          const { writeFileBinary } = await import('@/lib/fileSystem');
          const uint8 = new Uint8Array(savedBuffer);
          await writeFileBinary(filePath, uint8);
          setIsDirty(false);
        }
      } catch (err) {
        console.error('Failed to save DOCX file:', err);
      }
    };

    useImperativeHandle(ref, () => ({
      save: saveFile,
      load: loadFile,
    }));

    useEffect(() => {
      loadFile();
    }, [filePath]);

    if (loading) {
      return (
        <div className="flex items-center justify-center h-full">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2" style={{ borderColor: 'var(--accent-primary)' }} />
        </div>
      );
    }

    if (error) {
      return (
        <div className="flex flex-col items-center justify-center h-full">
          <div className="text-4xl mb-4">📄</div>
          <p style={{ color: 'var(--status-error)' }}>{error}</p>
        </div>
      );
    }

    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center justify-between px-4 py-2" style={{ backgroundColor: 'var(--bg-elevated)', borderBottom: '1px solid var(--border-subtle)' }}>
          <span className="text-sm" style={{ color: 'var(--text-secondary)' }}>文档编辑器</span>
          {isDirty && (
            <button
              onClick={saveFile}
              className="px-3 py-1 rounded text-sm transition-colors"
              style={{
                backgroundColor: 'var(--accent-primary)',
                color: 'white',
              }}
            >
              保存
            </button>
          )}
        </div>
        <div className="flex-1 overflow-hidden">
          <DocxEditor
            ref={editorRef}
            documentBuffer={buffer}
            onSave={() => setIsDirty(false)}
            onChange={() => setIsDirty(true)}
            className="h-full"
            style={{ height: '100%' }}
          />
        </div>
      </div>
    );
  }
);