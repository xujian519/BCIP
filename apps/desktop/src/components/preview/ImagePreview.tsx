import { useState, useEffect, useCallback, useRef } from 'react';
import { ZoomIn, ZoomOut, RotateCw, Download } from 'lucide-react';

interface ImagePreviewProps {
  filePath: string;
}

export default function ImagePreview({ filePath }: ImagePreviewProps) {
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [scale, setScale] = useState(1);
  const [rotation, setRotation] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const prevUrlRef = useRef<string | null>(null);

  useEffect(() => {
    let currentUrl: string | null = null;
    const loadImage = async () => {
      try {
        if (window.__TAURI__) {
          const { readFileBinary } = await import('@/lib/fileSystem');
          const data = await readFileBinary(filePath);
          const blob = new Blob([data.buffer as ArrayBuffer]);
          currentUrl = URL.createObjectURL(blob);
          setImageUrl(currentUrl);
        } else {
          setImageUrl(filePath);
        }
      } catch (_err) {
        setError('无法加载图片');
      } finally {
        setLoading(false);
      }
    };

    loadImage();

    return () => {
      if (prevUrlRef.current?.startsWith('blob:')) {
        URL.revokeObjectURL(prevUrlRef.current);
      }
      prevUrlRef.current = currentUrl;
    };
  }, [filePath]);

  const zoomIn = useCallback(() => setScale(prev => Math.min(prev + 0.2, 5)), []);
  const zoomOut = useCallback(() => setScale(prev => Math.max(prev - 0.2, 0.2)), []);
  const rotate = useCallback(() => setRotation(prev => (prev + 90) % 360), []);
  const reset = useCallback(() => {
    setScale(1);
    setRotation(0);
  }, []);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      if (e.deltaY < 0) {
        zoomIn();
      } else {
        zoomOut();
      }
    }
  }, [zoomIn, zoomOut]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2" style={{ borderColor: 'var(--accent-primary)' }} />
      </div>
    );
  }

  if (error || !imageUrl) {
    return (
      <div className="flex items-center justify-center h-full">
        <p style={{ color: 'var(--status-error)' }}>{error || '无法加载图片'}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* 工具栏 */}
      <div
        className="flex items-center justify-center gap-4 py-2 px-4"
        style={{
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        <button onClick={zoomOut} className="p-1 rounded transition-colors" style={{ color: 'var(--text-secondary)' }}>
          <ZoomOut size={18} />
        </button>
        <span className="text-sm w-12 text-center" style={{ color: 'var(--text-primary)' }}>
          {Math.round(scale * 100)}%
        </span>
        <button onClick={zoomIn} className="p-1 rounded transition-colors" style={{ color: 'var(--text-secondary)' }}>
          <ZoomIn size={18} />
        </button>
        <button onClick={rotate} className="p-1 rounded transition-colors" style={{ color: 'var(--text-secondary)' }}>
          <RotateCw size={18} />
        </button>
        <button onClick={reset} className="px-2 py-1 text-sm rounded transition-colors" style={{ color: 'var(--text-secondary)' }}>
          重置
        </button>
        <a
          href={imageUrl}
          download
          className="p-1 rounded transition-colors"
          style={{ color: 'var(--text-secondary)' }}
        >
          <Download size={18} />
        </a>
      </div>

      {/* 图片 */}
      <div
        className="flex-1 overflow-auto flex items-center justify-center p-4 relative"
        onWheel={handleWheel}
      >
        <img
          src={imageUrl}
          alt="预览"
          className="max-w-full max-h-full object-contain transition-transform"
          style={{
            transform: `scale(${scale}) rotate(${rotation}deg)`,
            transformOrigin: 'center center',
          }}
          draggable={false}
        />
        {/* 信息叠加层 */}
        <div
          className="absolute bottom-4 left-1/2 -translate-x-1/2 px-3 py-1.5 rounded-lg text-xs"
          style={{
            backgroundColor: 'rgba(0, 0, 0, 0.6)',
            color: '#fff',
            backdropFilter: 'blur(8px)',
          }}
        >
          {filePath.split('/').pop()}
        </div>
      </div>
    </div>
  );
}
