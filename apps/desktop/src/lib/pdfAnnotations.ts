/**
 * PDF 标注持久化到项目 `.bcip/annotations/`
 */
import { createDirectory, readFile, writeFile } from '@/lib/fileSystem';

export interface PdfAnnotationRecord {
  id: string;
  type: 'highlight' | 'underline' | 'note';
  page: number;
  x: number;
  y: number;
  w: number;
  h: number;
  color?: string;
  text?: string;
}

function annotationFileKey(filePath: string): string {
  return encodeURIComponent(filePath.replace(/[/\\]/g, '__'));
}

function annotationsDir(workspaceCwd: string): string {
  return `${workspaceCwd}/.bcip/annotations`;
}

function annotationFilePath(workspaceCwd: string, filePath: string): string {
  return `${annotationsDir(workspaceCwd)}/${annotationFileKey(filePath)}.json`;
}

export async function loadPdfAnnotations(
  workspaceCwd: string | null,
  filePath: string,
): Promise<PdfAnnotationRecord[]> {
  if (!workspaceCwd) {
    return [];
  }
  try {
    const raw = await readFile(annotationFilePath(workspaceCwd, filePath));
    const parsed = JSON.parse(raw) as { annotations?: PdfAnnotationRecord[] };
    return Array.isArray(parsed.annotations) ? parsed.annotations : [];
  } catch {
    return [];
  }
}

export async function savePdfAnnotations(
  workspaceCwd: string | null,
  filePath: string,
  annotations: PdfAnnotationRecord[],
): Promise<void> {
  if (!workspaceCwd) {
    return;
  }
  try {
    await createDirectory(annotationsDir(workspaceCwd));
    await writeFile(
      annotationFilePath(workspaceCwd, filePath),
      JSON.stringify({ filePath, annotations, updatedAt: Date.now() }, null, 2),
    );
  } catch {
    // 无工作区或 FS 不可写时静默跳过
  }
}
