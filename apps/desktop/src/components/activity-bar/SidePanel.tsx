import { AnimatePresence, motion } from 'framer-motion';
import { useState } from 'react';
import { useAppStore } from '@/hooks/useAppStore';
import ExplorerPanel from './ExplorerPanel';

function SearchPanel() {
  const [query, setQuery] = useState('');
  return (
    <div className="h-full overflow-auto">
      <div className="px-3 py-2 text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
        搜索
      </div>
      <div className="px-3 pb-2">
        <input
          type="text"
          placeholder="搜索文件内容..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="w-full rounded-md border px-2 py-1 text-xs outline-none"
          style={{
            backgroundColor: 'var(--bg-elevated)',
            borderColor: 'var(--border-primary)',
            color: 'var(--text-primary)',
            height: 30,
          }}
        />
      </div>
      {!query && (
        <div className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
          输入关键词搜索文件内容
        </div>
      )}
    </div>
  );
}

function SkillsPanel() {
  return (
    <div className="h-full overflow-auto">
      <div className="px-3 py-2 text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
        技能管理
      </div>
      <div className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
        技能列表将在此显示
      </div>
    </div>
  );
}

function BotsPanel() {
  return (
    <div className="h-full overflow-auto">
      <div className="px-3 py-2 text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
        AI 助手
      </div>
      <div className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
        管理微信、飞书、钉钉等外接渠道
      </div>
    </div>
  );
}

const panelMap: Record<string, () => React.ReactElement> = {
  files: ExplorerPanel,
  search: SearchPanel,
  skills: SkillsPanel,
  bots: BotsPanel,
};

export default function SidePanel() {
  const { state } = useAppStore();
  const tab = state.activityBarTab;

  if (!tab) return null;

  const Panel = panelMap[tab];
  if (!Panel) return null;

  const panelWidth = tab === 'files' ? Math.max(state.leftSidebarWidth, 320) : state.leftSidebarWidth;

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
