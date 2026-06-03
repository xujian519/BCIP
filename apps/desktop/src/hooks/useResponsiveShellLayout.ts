import { useEffect, useState } from 'react';
import { useAppStore } from '@/hooks/useAppStore';

const BP_NARROW_OVERLAY = 900;
const BP_HIDE_THREAD_LIST = 1200;

export function useResponsiveShellLayout(_centerRef: React.RefObject<HTMLElement | null>) {
  const { state, dispatch } = useAppStore();
  const [isNarrowViewport, setIsNarrowViewport] = useState(
    () => typeof window !== 'undefined' && window.innerWidth < BP_NARROW_OVERLAY,
  );

  useEffect(() => {
    const applyBreakpoint = () => {
      const w = window.innerWidth;
      setIsNarrowViewport(w < BP_NARROW_OVERLAY);
      const band = w < BP_NARROW_OVERLAY ? 'narrow' : w < BP_HIDE_THREAD_LIST ? 'medium' : 'wide';

      if (band !== 'wide' && state.threadListOpen) {
        dispatch({ type: 'SET_THREAD_LIST_OPEN', payload: false });
      }

      if (band === 'narrow' && state.layoutMode !== 'horizontal-split') {
        dispatch({ type: 'SET_LAYOUT_MODE', payload: 'horizontal-split' });
      }
    };

    applyBreakpoint();
    window.addEventListener('resize', applyBreakpoint);
    return () => window.removeEventListener('resize', applyBreakpoint);
  }, [dispatch, state.threadListOpen, state.layoutMode]);

  return { isNarrowViewport };
}