import { useState, useEffect, useCallback, useRef } from 'react';
import { Document, Page, pdfjs } from 'react-pdf';
import 'react-pdf/dist/Page/AnnotationLayer.css';
import 'react-pdf/dist/Page/TextLayer.css';
import { ZoomIn, ZoomOut, RotateCw, ChevronLeft, ChevronRight, Trash2 } from 'lucide-react';
import { readFileBinary } from '@/lib/fileSystem';
import { useAppStore } from '@/hooks/useAppStore';
import {
  loadPdfAnnotations,
  savePdfAnnotations,
  type PdfAnnotationRecord,
} from '@/lib/pdfAnnotations';
import PdfAnnotationToolbar, {
  type PdfToolMode,
  PDF_HIGHLIGHT_COLORS,
} from './PdfAnnotationToolbar';

pdfjs.GlobalWorkerOptions.workerSrc = `//cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjs.version}/pdf.worker.min.js`;

const DEFAULT_COLOR = PDF_HIGHLIGHT_COLORS[0];

export default function PdfPreview({ filePath }: { filePath: string }) {
  return <PdfPreviewInner key={filePath} filePath={filePath} />;
}

function PdfPreviewInner({ filePath }: { filePath: string }) {
  const { state } = useAppStore();
  const workspaceCwd = state.workspaceCwd;
  const [numPages, setNumPages] = useState(0);
  const [pageNumber, setPageNumber] = useState(1);
  const [scale, setScale] = useState(1.2);
  const [rotation, setRotation] = useState(0);
  const [pdfData, setPdfData] = useState<Uint8Array | null>(null);
  const [loading, setLoading] = useState(true);
  const [toolMode, setToolMode] = useState<PdfToolMode>('pointer');
  const [highlightColor, setHighlightColor] = useState(DEFAULT_COLOR);
  const [annotations, setAnnotations] = useState<PdfAnnotationRecord[]>([]);
  const [annotationsLoaded, setAnnotationsLoaded] = useState(false);
  const [isDrawing, setIsDrawing] = useState(false);
  const [drawStart, setDrawStart] = useState<{ x: number; y: number } | null>(null);
  const [drawRect, setDrawRect] = useState<{ x: number; y: number; w: number; h: number } | null>(null);
  const [noteInput, setNoteInput] = useState<{ x: number; y: number } | null>(null);
  const [noteText, setNoteText] = useState('');
  const pageContainerRef = useRef<HTMLDivElement>(null);
  const fileName = filePath.split('/').pop() ?? filePath;

  useEffect(() => {
    let cancelled = false;
    const loadPdf = async () => {
      try {
        const data = await readFileBinary(filePath);
        if (!cancelled) {
          setPdfData(data);
        }
      } catch {
        if (!cancelled) {
          setPdfData(null);
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };
    void loadPdf();
    return () => {
      cancelled = true;
    };
  }, [filePath]);

  useEffect(() => {
    let cancelled = false;
    void loadPdfAnnotations(workspaceCwd, filePath).then((loaded) => {
      if (!cancelled) {
        setAnnotations(loaded);
        setAnnotationsLoaded(true);
        setPageNumber(1);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [filePath, workspaceCwd]);

  useEffect(() => {
    if (!annotationsLoaded) {
      return;
    }
    const timer = window.setTimeout(() => {
      void savePdfAnnotations(workspaceCwd, filePath, annotations);
    }, 600);
    return () => window.clearTimeout(timer);
  }, [annotations, annotationsLoaded, workspaceCwd, filePath]);

  const onDocumentLoadSuccess = ({ numPages }: { numPages: number }) => {
    setNumPages(numPages);
    setPageNumber(1);
    setLoading(false);
  };

  const zoomIn = () => setScale((prev) => Math.min(prev + 0.2, 3));
  const zoomOut = () => setScale((prev) => Math.max(prev - 0.2, 0.5));
  const rotate = () => setRotation((prev) => (prev + 90) % 360);
  const prevPage = () => setPageNumber((prev) => Math.max(prev - 1, 1));
  const nextPage = () => setPageNumber((prev) => Math.min(prev + 1, numPages));

  const getPageCoords = useCallback(
    (e: React.MouseEvent) => {
      const el = pageContainerRef.current;
      if (!el) return { x: 0, y: 0 };
      const rect = el.getBoundingClientRect();
      return {
        x: (e.clientX - rect.left) / scale,
        y: (e.clientY - rect.top) / scale,
      };
    },
    [scale]
  );

  const handleMouseDown = (e: React.MouseEvent) => {
    if (toolMode === 'pointer' || toolMode === 'note') return;
    const coords = getPageCoords(e);
    setIsDrawing(true);
    setDrawStart(coords);
    setDrawRect({ x: coords.x, y: coords.y, w: 0, h: 0 });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDrawing || !drawStart) return;
    const coords = getPageCoords(e);
    setDrawRect({
      x: Math.min(drawStart.x, coords.x),
      y: Math.min(drawStart.y, coords.y),
      w: Math.abs(coords.x - drawStart.x),
      h: Math.abs(coords.y - drawStart.y),
    });
  };

  const handleMouseUp = () => {
    if (!isDrawing || !drawRect) {
      setIsDrawing(false);
      setDrawStart(null);
      setDrawRect(null);
      return;
    }
    if (drawRect.w > 5 && drawRect.h > 5) {
      const ann: PdfAnnotationRecord = {
        id: `ann-${Date.now()}`,
        type: toolMode === 'underline' ? 'underline' : 'highlight',
        page: pageNumber,
        x: drawRect.x,
        y: drawRect.y,
        w: drawRect.w,
        h: drawRect.h,
        color: toolMode === 'underline' ? '#f44336' : highlightColor,
      };
      setAnnotations((prev) => [...prev, ann]);
    }
    setIsDrawing(false);
    setDrawStart(null);
    setDrawRect(null);
  };

  const handlePageClick = (e: React.MouseEvent) => {
    if (toolMode !== 'note') return;
    const coords = getPageCoords(e);
    setNoteInput(coords);
  };

  const handleAddNote = () => {
    if (!noteInput || !noteText.trim()) {
      setNoteInput(null);
      setNoteText('');
      return;
    }
    const ann: PdfAnnotationRecord = {
      id: `ann-${Date.now()}`,
      type: 'note',
      page: pageNumber,
      x: noteInput.x,
      y: noteInput.y,
      w: 20,
      h: 20,
      text: noteText.trim(),
      color: '#2196f3',
    };
    setAnnotations((prev) => [...prev, ann]);
    setNoteInput(null);
    setNoteText('');
  };

  const handleDeleteAnnotation = (id: string) => {
    setAnnotations((prev) => prev.filter((a) => a.id !== id));
  };

  const pageAnnotations = annotations.filter((a) => a.page === pageNumber);

  return (
    <div className="flex flex-col h-full">
      <div
        className="flex items-center justify-between px-4 py-1"
        style={{
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
          minHeight: 40,
        }}
      >
        <div className="flex items-center gap-3 min-w-0">
          <span
            className="text-xs truncate max-w-[140px]"
            style={{ color: 'var(--text-tertiary)' }}
            title={filePath}
          >
            {fileName}
          </span>
          <div className="flex items-center gap-1">
            <button onClick={prevPage} disabled={pageNumber <= 1}
              className="p-1 rounded disabled:opacity-30" style={{ color: 'var(--text-secondary)' }}>
              <ChevronLeft size={16} />
            </button>
            <span className="text-xs" style={{ color: 'var(--text-primary)' }}>
              {pageNumber} / {numPages}
            </span>
            <button onClick={nextPage} disabled={pageNumber >= numPages}
              className="p-1 rounded disabled:opacity-30" style={{ color: 'var(--text-secondary)' }}>
              <ChevronRight size={16} />
            </button>
          </div>
          <div className="flex items-center gap-1">
            <button onClick={zoomOut} className="p-1 rounded" style={{ color: 'var(--text-secondary)' }}>
              <ZoomOut size={16} />
            </button>
            <span className="text-xs w-10 text-center" style={{ color: 'var(--text-primary)' }}>
              {Math.round(scale * 100)}%
            </span>
            <button onClick={zoomIn} className="p-1 rounded" style={{ color: 'var(--text-secondary)' }}>
              <ZoomIn size={16} />
            </button>
            <button onClick={rotate} className="p-1 rounded" style={{ color: 'var(--text-secondary)' }}>
              <RotateCw size={16} />
            </button>
          </div>
        </div>
        <PdfAnnotationToolbar
          toolMode={toolMode}
          onToolModeChange={setToolMode}
          highlightColor={highlightColor}
          onHighlightColorChange={setHighlightColor}
          annotationCount={pageAnnotations.length}
        />
      </div>

      <div
        className="flex-1 overflow-auto"
        ref={pageContainerRef}
        style={{ backgroundColor: '#E8E5E0' }}
      >
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2"
              style={{ borderColor: 'var(--accent-primary)' }} />
          </div>
        ) : !pdfData ? (
          <div className="flex items-center justify-center h-full">
            <p style={{ color: 'var(--status-error)' }}>无法加载 PDF 文件</p>
          </div>
        ) : (
          <div
            className="flex justify-center p-4 relative"
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            onClick={handlePageClick}
            style={{ cursor: toolMode === 'pointer' ? 'default' : 'crosshair' }}
          >
            <div style={{ transform: `scale(${scale})`, transformOrigin: 'top center' }}>
              <Document
                file={{ data: pdfData }}
                onLoadSuccess={onDocumentLoadSuccess}
                loading={null}
              >
                <Page
                  pageNumber={pageNumber}
                  rotate={rotation}
                  renderTextLayer
                  renderAnnotationLayer
                  className="shadow-lg"
                />
              </Document>
            </div>
            <svg
              className="absolute pointer-events-none"
              style={{ width: '100%', height: '100%', left: 0, top: 0 }}
            >
              {pageAnnotations.map((ann) => {
                const x = ann.x * scale;
                const y = ann.y * scale;
                const w = ann.w * scale;
                const h = ann.h * scale;
                const color = ann.color || '#ffeb3b';
                if (ann.type === 'highlight') {
                  return (
                    <rect key={ann.id} x={x} y={y} width={w} height={h}
                      fill={`${color}44`} stroke={color} strokeWidth={1} rx={2} />
                  );
                }
                if (ann.type === 'underline') {
                  return (
                    <line key={ann.id} x1={x} y1={y + h} x2={x + w} y2={y + h}
                      stroke={color} strokeWidth={2} strokeLinecap="round" />
                  );
                }
                if (ann.type === 'note') {
                  return (
                    <g key={ann.id}>
                      <rect x={x} y={y} width={w} height={h}
                        fill={`${color}88`} stroke={color} strokeWidth={1.5} rx={3} />
                      <text x={x + w / 2} y={y + h / 2 + 1}
                        textAnchor="middle" dominantBaseline="middle"
                        fill="#fff" fontSize={10} fontWeight="bold">
                        !
                      </text>
                      <title>{ann.text}</title>
                    </g>
                  );
                }
                return null;
              })}
              {isDrawing && drawRect && (
                <rect
                  x={drawRect.x * scale} y={drawRect.y * scale}
                  width={drawRect.w * scale} height={drawRect.h * scale}
                  fill={toolMode === 'underline' ? 'transparent' : `${highlightColor}33`}
                  stroke={toolMode === 'underline' ? '#f44336' : highlightColor}
                  strokeWidth={1} strokeDasharray="4 2" />
              )}
            </svg>
            {noteInput && (
              <div
                className="absolute z-20 p-2 rounded shadow-lg"
                style={{
                  left: noteInput.x * scale + 20,
                  top: noteInput.y * scale,
                  backgroundColor: 'var(--bg-elevated)',
                  border: '1px solid var(--border-primary)',
                  minWidth: 160,
                }}
                onClick={(e) => e.stopPropagation()}
              >
                <textarea
                  value={noteText}
                  onChange={(e) => setNoteText(e.target.value)}
                  placeholder="输入批注..."
                  className="w-full bg-transparent text-xs resize-none focus:outline-none mb-1"
                  style={{ fontSize: 12, color: 'var(--text-primary)', minHeight: 40 }}
                  rows={2}
                  autoFocus
                />
                <div className="flex justify-end gap-1">
                  <button
                    onClick={() => { setNoteInput(null); setNoteText(''); }}
                    className="text-xs px-2 py-0.5 rounded"
                    style={{ color: 'var(--text-tertiary)' }}
                    type="button"
                  >
                    取消
                  </button>
                  <button
                    onClick={handleAddNote}
                    className="text-xs px-2 py-0.5 rounded"
                    style={{ backgroundColor: 'var(--accent-primary)', color: 'var(--text-inverse)' }}
                    type="button"
                  >
                    添加
                  </button>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {pageAnnotations.length > 0 && (
        <div
          className="overflow-y-auto"
          style={{
            maxHeight: 120,
            borderTop: '1px solid var(--border-primary)',
            backgroundColor: 'var(--bg-elevated)',
            padding: '4px 8px',
          }}
        >
          {pageAnnotations.map((ann) => (
            <div key={ann.id} className="flex items-center gap-2 py-0.5">
              <span
                className="rounded-full flex-shrink-0"
                style={{ width: 8, height: 8, backgroundColor: ann.color || '#ffeb3b' }}
              />
              <span className="text-xs truncate flex-1" style={{ color: 'var(--text-secondary)' }}>
                {ann.type === 'note' ? `📝 ${ann.text}` : ann.type === 'underline' ? '下划线' : '高亮'}
              </span>
              <button
                onClick={() => handleDeleteAnnotation(ann.id)}
                className="flex-shrink-0 p-0.5 rounded"
                style={{ color: 'var(--text-tertiary)' }}
                type="button"
              >
                <Trash2 size={11} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
