/**
 * ResizeHandle —— 可拖拽面板分隔手柄
 *
 * 精确设计参数：
 * - 宽度 4px，热区全高
 * - 默认透明，悬停时显示品牌绿色竖线（2px 宽）
 * - 光标：col-resize
 * - 拖拽时：显示半透明遮罩 + 虚线指示器
 * - onDrag 回调调整面板宽度
 */
import { useCallback, useRef, useState } from 'react';
import { cn } from '@/lib/utils';

interface ResizeHandleProps {
  /** 拖拽方向 */
  direction?: 'horizontal' | 'vertical';
  /** 当前尺寸值 */
  size: number;
  /** 最小尺寸 */
  minSize: number;
  /** 最大尺寸 */
  maxSize: number;
  /** 尺寸变化回调 */
  onResize: (size: number) => void;
  /** 放置位置 */
  position: 'left' | 'right' | 'top' | 'bottom';
}

export default function ResizeHandle({
  direction = 'horizontal',
  size,
  minSize,
  maxSize,
  onResize,
  position,
}: ResizeHandleProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const startPosRef = useRef(0);
  const startSizeRef = useRef(size);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(true);
      startPosRef.current = direction === 'horizontal' ? e.clientX : e.clientY;
      startSizeRef.current = size;

      const handleMouseMove = (moveEvent: MouseEvent) => {
        const currentPos =
          direction === 'horizontal' ? moveEvent.clientX : moveEvent.clientY;
        const delta = currentPos - startPosRef.current;

        let newSize = startSizeRef.current;
        if (position === 'left') {
          newSize = startSizeRef.current + delta;
        } else if (position === 'right') {
          newSize = startSizeRef.current - delta;
        } else if (position === 'top') {
          newSize = startSizeRef.current + delta;
        } else if (position === 'bottom') {
          newSize = startSizeRef.current - delta;
        }

        newSize = Math.max(minSize, Math.min(maxSize, newSize));
        onResize(newSize);
      };

      const handleMouseUp = () => {
        setIsDragging(false);
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
        // 移除拖拽遮罩
        const overlay = document.getElementById('resize-overlay');
        if (overlay) overlay.remove();
      };

      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      document.body.style.cursor =
        direction === 'horizontal' ? 'col-resize' : 'row-resize';
      document.body.style.userSelect = 'none';

      // 创建全屏拖拽遮罩（避免 iframe 等拦截鼠标事件）
      const overlay = document.createElement('div');
      overlay.id = 'resize-overlay';
      overlay.className = 'fixed inset-0 z-[9999]';
      overlay.style.cursor = direction === 'horizontal' ? 'col-resize' : 'row-resize';
      document.body.appendChild(overlay);
    },
    [direction, size, minSize, maxSize, onResize, position]
  );

  const isHorizontal = direction === 'horizontal';

  return (
    <>
      {/* 拖拽时的半透明遮罩 + 虚线指示器 */}
      {isDragging && (
        <div className="fixed inset-0 z-[9998] pointer-events-none">
          {/* 半透明遮罩 */}
          <div className="absolute inset-0 bg-black/5" />
          {/* 虚线指示器 */}
          <div
            className={cn(
              'absolute bg-brand-500/30',
              isHorizontal
                ? 'top-0 bottom-0 w-0 border-l border-dashed border-brand-500'
                : 'left-0 right-0 h-0 border-t border-dashed border-brand-500'
            )}
            style={
              isHorizontal
                ? { left: position === 'left' ? size : undefined, right: position === 'right' ? size : undefined }
                : { top: position === 'top' ? size : undefined, bottom: position === 'bottom' ? size : undefined }
            }
          />
        </div>
      )}

      {/* 拖拽手柄热区 */}
      <div
        onMouseDown={handleMouseDown}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
        className={cn(
          'shrink-0 z-10 flex items-center justify-center relative',
          isHorizontal ? 'w-[7px] cursor-col-resize' : 'h-[7px] cursor-row-resize'
        )}
      >
        {/* 悬停/拖拽时显示的品牌绿色竖线 */}
        <div
          className={cn(
            'rounded-full bg-brand-500 transition-opacity duration-fast absolute',
            isHorizontal
              ? 'w-[2px] top-0 bottom-0 left-1/2 -translate-x-1/2'
              : 'h-[2px] left-0 right-0 top-1/2 -translate-y-1/2',
            isDragging || isHovered ? 'opacity-100' : 'opacity-0'
          )}
        />
      </div>
    </>
  );
}
