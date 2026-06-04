import { useEffect, type ComponentType } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '@/hooks/useAppStore';
import SettingsNav from './SettingsNav';
import GeneralSettings from './pages/GeneralSettings';
import type { SettingsPage } from '@/types';
import { settingsTheme } from './settingsTheme';
import { easePanel } from '@/lib/animations';

const pageComponents: Partial<Record<SettingsPage, ComponentType>> = {
  model: () => null,
  approval: () => null,
  mcp: () => null,
  skills: () => null,
  plugins: () => null,
  appearance: () => null,
  shortcuts: () => null,
};

export default function SettingsLayout() {
  const { state, dispatch } = useAppStore();
  const settingsPage = state.settingsPage;
  const isOpen = state.settingsOpen;

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        dispatch({ type: 'CLOSE_SETTINGS' });
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, dispatch]);

  const ActivePage = pageComponents[settingsPage] || GeneralSettings;

  return (
    <AnimatePresence>
      {isOpen && (
        <div
          className={settingsTheme.shell}
          role="dialog"
          aria-modal="true"
          aria-label="设置"
        >
          <div className="flex min-h-0 w-full flex-1">
            <SettingsNav />

            <div className={settingsTheme.content}>
              <AnimatePresence mode="wait">
                <motion.div
                  key={settingsPage}
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  transition={{ duration: 0.15, ease: easePanel }}
                  className={settingsTheme.contentInner}
                >
                  <ActivePage />
                </motion.div>
              </AnimatePresence>
            </div>
          </div>
        </div>
      )}
    </AnimatePresence>
  );
}
