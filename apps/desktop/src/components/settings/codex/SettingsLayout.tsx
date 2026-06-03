import { useEffect, type ComponentType } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '@/hooks/useAppStore';
import SettingsNav from './SettingsNav';
import GeneralSettings from './pages/GeneralSettings';
import ModelSettings from './pages/ModelSettings';
import ApprovalSandboxSettings from './pages/ApprovalSandboxSettings';
import McpServersSettings from './pages/McpServersSettings';
import SkillsSettings from './pages/SkillsSettings';
import PluginsSettings from './pages/PluginsSettings';
import AppearanceSettings from './pages/AppearanceSettings';
import ShortcutsSettings from './pages/ShortcutsSettings';
import AboutDiagnostics from './pages/AboutDiagnostics';
import type { SettingsPage } from '@/types';
import { settingsTheme } from './settingsTheme';
import { easePanel } from '@/lib/animations';

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
