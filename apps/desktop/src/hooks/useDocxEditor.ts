import { useRef, useState, useCallback } from 'react';

export function useDocxEditor() {
  const editorRef = useRef<any>(null);
  const [isDirty, setIsDirty] = useState(false);
  const [documentBuffer, setDocumentBuffer] = useState<ArrayBuffer | null>(null);

  const loadFile = useCallback(async (filePath: string) => {
    try {
      if (window.__TAURI__) {
        const { readFileBinary } = await import('@/lib/fileSystem');
        const data = await readFileBinary(filePath);
        setDocumentBuffer(data.buffer as ArrayBuffer);
      }
    } catch (err) {
      console.error('Failed to load DOCX file:', err);
    }
    setIsDirty(false);
  }, []);

  const saveFile = useCallback(async (filePath: string) => {
    if (!editorRef.current) return;
    try {
      const buffer = await editorRef.current.save();
      if (buffer && window.__TAURI__) {
        const { writeFileBinary } = await import('@/lib/fileSystem');
        const uint8 = new Uint8Array(buffer);
        await writeFileBinary(filePath, uint8);
      }
      setIsDirty(false);
    } catch (err) {
      console.error('Failed to save DOCX file:', err);
    }
  }, []);

  return { editorRef, documentBuffer, isDirty, setIsDirty, loadFile, saveFile };
}