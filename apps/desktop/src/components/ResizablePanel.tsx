import type { FC, ReactNode } from 'react';
import { useCallback, useRef, useState, useEffect } from 'react';

interface ResizablePanelProps {
  children: ReactNode;
  defaultWidth: number;
  minWidth: number;
  maxWidth: number;
  side: 'left' | 'right';
  className?: string;
  style?: React.CSSProperties;
  width?: number;
  onResize?: (width: number) => void;
}

const ResizablePanel: FC<ResizablePanelProps> = ({
  children,
  defaultWidth,
  minWidth,
  maxWidth,
  side,
  className = '',
  style = {},
  width: controlledWidth,
  onResize,
}) => {
  const [internalWidth, setInternalWidth] = useState(defaultWidth);
  const width = controlledWidth ?? internalWidth;
  const [isResizing, setIsResizing] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const startXRef = useRef(0);
  const startWidthRef = useRef(0);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsResizing(true);
      startXRef.current = e.clientX;
      startWidthRef.current = width;

      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    },
    [width]
  );

  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const delta = e.clientX - startXRef.current;
      const newWidth =
        side === 'left'
          ? Math.min(maxWidth, Math.max(minWidth, startWidthRef.current + delta))
          : Math.min(maxWidth, Math.max(minWidth, startWidthRef.current - delta));
      if (controlledWidth !== undefined) {
        onResize?.(newWidth);
      } else {
        setInternalWidth(newWidth);
      }
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing, minWidth, maxWidth, side, controlledWidth, onResize]);

  return (
    <div
      ref={panelRef}
      className={`relative flex-shrink-0 ${className}`}
      style={{
        width,
        transition: isResizing ? 'none' : 'width 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
        ...style,
      }}
    >
      {children}
      <div
        className="absolute top-0 bottom-0 z-10 flex items-center justify-center"
        style={{
          [side === 'left' ? 'right' : 'left']: -4,
          width: 8,
          cursor: 'col-resize',
        }}
        onMouseDown={handleMouseDown}
        role="separator"
        aria-label="Resize panel"
      >
        <div
          className="h-8 rounded-full transition-colors duration-150"
          style={{
            width: 3,
            backgroundColor: isResizing
              ? 'var(--accent-primary)'
              : 'transparent',
          }}
        />
      </div>
      {isResizing && (
        <div
          className="pointer-events-none fixed inset-0 z-50"
          style={{ cursor: 'col-resize' }}
        />
      )}
    </div>
  );
};

export default ResizablePanel;
