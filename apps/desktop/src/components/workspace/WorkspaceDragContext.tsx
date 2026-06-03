import {
  createContext,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import type { TabDragPayload } from '@/lib/workspaceLayout';

interface WorkspaceDragContextValue {
  dragging: TabDragPayload | null;
  setDragging: (payload: TabDragPayload | null) => void;
}

const WorkspaceDragContext = createContext<WorkspaceDragContextValue | null>(
  null,
);

export function WorkspaceDragProvider({ children }: { children: ReactNode }) {
  const [dragging, setDragging] = useState<TabDragPayload | null>(null);
  const value = useMemo(
    () => ({ dragging, setDragging }),
    [dragging],
  );

  return (
    <WorkspaceDragContext.Provider value={value}>
      {children}
    </WorkspaceDragContext.Provider>
  );
}

export function useWorkspaceDrag(): WorkspaceDragContextValue {
  const ctx = useContext(WorkspaceDragContext);
  if (!ctx) {
    throw new Error('useWorkspaceDrag 必须在 WorkspaceDragProvider 内使用');
  }
  return ctx;
}
