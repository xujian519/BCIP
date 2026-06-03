import { useCallback, useLayoutEffect, useRef, useState } from 'react';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/hooks/useAppStore';
import type { WorkspaceNode, WorkspaceSplitNode } from '@/types';
import ResizeHandle from '@/components/shell/ResizeHandle';
import WorkspacePane from './WorkspacePane';

interface WorkspaceSplitProps {
  node: WorkspaceNode;
}

function SplitNodeView({ node }: { node: WorkspaceSplitNode }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const { dispatch } = useAppStore();
  const isVertical = node.direction === 'vertical';
  const [containerSize, setContainerSize] = useState(0);

  useLayoutEffect(() => {
    const element = containerRef.current;
    if (!element) {
      return;
    }
    const observer = new ResizeObserver(() => {
      setContainerSize(
        isVertical ? element.clientHeight : element.clientWidth,
      );
    });
    observer.observe(element);
    setContainerSize(isVertical ? element.clientHeight : element.clientWidth);
    return () => observer.disconnect();
  }, [isVertical]);

  const firstSize = Math.round(node.ratio * containerSize);
  const minSize = Math.max(160, Math.floor(containerSize * 0.15));
  const maxSize = Math.max(minSize, Math.floor(containerSize * 0.85));

  const handleResize = useCallback(
    (size: number) => {
      if (containerSize <= 0) {
        return;
      }
      dispatch({
        type: 'SET_WORKSPACE_SPLIT_RATIO',
        payload: {
          splitId: node.id,
          ratio: size / containerSize,
        },
      });
    },
    [containerSize, dispatch, node.id],
  );

  return (
    <div
      ref={containerRef}
      className={cn(
        'flex min-h-0 min-w-0 flex-1 overflow-hidden',
        isVertical ? 'flex-col' : 'flex-row',
      )}
    >
      <div
        className="flex min-h-0 min-w-0 flex-col overflow-hidden"
        style={
          isVertical
            ? { height: containerSize > 0 ? firstSize : `${node.ratio * 100}%` }
            : { width: containerSize > 0 ? firstSize : `${node.ratio * 100}%` }
        }
      >
        <WorkspaceSplit node={node.first} />
      </div>
      {containerSize > 0 && (
        <ResizeHandle
          direction={isVertical ? 'vertical' : 'horizontal'}
          size={firstSize}
          minSize={minSize}
          maxSize={maxSize}
          onResize={handleResize}
          position={isVertical ? 'top' : 'left'}
        />
      )}
      <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
        <WorkspaceSplit node={node.second} />
      </div>
    </div>
  );
}

export default function WorkspaceSplit({ node }: WorkspaceSplitProps) {
  if (node.type === 'leaf') {
    return <WorkspacePane node={node} />;
  }
  return <SplitNodeView node={node} />;
}
