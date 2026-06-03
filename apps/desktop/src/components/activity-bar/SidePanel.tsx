import { AnimatePresence, motion } from 'framer-motion';
import { useAppStore } from '@/hooks/useAppStore';
import ExplorerPanel from './ExplorerPanel';
import SearchPanel from './SearchPanel';
import SkillsSidePanel from './SkillsSidePanel';

function BotsPanel() {
  return (
    <div className="h-full overflow-auto">
      <div className="px-3 py-2 text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
        AI 助手
      </div>
      <div className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
        外接渠道（微信 / 飞书 / 钉钉）配置即将上线
      </div>
    </div>
  );
}

const SIDE_PANEL_MIN_WIDTH: Record<string, number> = {
  files: 180,
  search: 240,
  skills: 240,
  bots: 280,
};

const panelMap: Record<string, () => React.ReactElement> = {
  files: ExplorerPanel,
  search: SearchPanel,
  skills: SkillsSidePanel,
  bots: BotsPanel,
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
