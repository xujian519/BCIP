import { AnimatePresence, motion } from 'framer-motion';
import { useAppStore } from '@/hooks/useAppStore';
import ExplorerPanel from './ExplorerPanel';
import SearchPanel from './SearchPanel';
import SkillsSidePanel from './SkillsSidePanel';
import BotsSidePanel from './BotsSidePanel';
import NewTaskSidePanel from './NewTaskSidePanel';

const SIDE_PANEL_MIN_WIDTH: Record<string, number> = {
  files: 180,
  'new-task': 200,
  search: 240,
  skills: 240,
  bots: 280,
};

const panelMap: Record<string, () => React.ReactElement> = {
  files: ExplorerPanel,
  'new-task': NewTaskSidePanel,
  search: SearchPanel,
  skills: SkillsSidePanel,
  bots: BotsSidePanel,
};

export default function SidePanel() {
  const { state } = useAppStore();
  const tab = state.activityBarTab;

  if (!tab) return null;

  const Panel = panelMap[tab];
  if (!Panel) return null;

  const minWidth = SIDE_PANEL_MIN_WIDTH[tab] ?? 240;
  const panelWidth = Math.max(state.leftSidebarWidth, minWidth);

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={tab}
        initial={{ opacity: 0, x: -8 }}
        animate={{ opacity: 1, x: 0 }}
        exit={{ opacity: 0, x: -8 }}
        transition={{ duration: 0.15 }}
        className="h-full shrink-0 overflow-hidden"
        style={{
          width: panelWidth,
          backgroundColor: 'var(--bg-sidebar)',
          borderRight: '1px solid var(--border-primary)',
        }}
      >
        <Panel />
      </motion.div>
    </AnimatePresence>
  );
}
