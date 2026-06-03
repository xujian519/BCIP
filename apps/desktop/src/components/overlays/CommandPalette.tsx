import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import type { LucideIcon } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import { focusComposer, requestNewThread } from '@/lib/desktopEvents';
import {
  Search,
  Settings,
  FilePlus,
  PanelLeft,
  MessageSquare,
  FileText,
  Terminal,
  Sun,
  Keyboard,
  HelpCircle,
  RefreshCw,
  Columns2,
  Merge,
  Rows2,
} from 'lucide-react';
import { easePanel } from '@/lib/animations';

interface PaletteItem {
  id: string;
  name: string;
  category: string;
  icon: LucideIcon;
  shortcut?: string;
  action: () => void;
}

export default function CommandPalette() {
  const { state, dispatch } = useAppStore();
  const isOpen = state.commandPaletteOpen;
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [prevOpen, setPrevOpen] = useState(isOpen);
  const [prevQuery, setPrevQuery] = useState(query);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  if (isOpen && !prevOpen) {
    setQuery('');
    setSelectedIndex(0);
    setPrevOpen(true);
  }
  if (!isOpen && prevOpen) {
    setPrevOpen(false);
  }
  if (query !== prevQuery) {
    setSelectedIndex(0);
    setPrevQuery(query);
  }

  const paletteItems: PaletteItem[] = useMemo(() => [
    {
      id: 'settings',
      name: '打开设置',
      category: '最近使用',
      icon: Settings,
      shortcut: '⌘,',
      action: () => {
        dispatch({ type: 'OPEN_SETTINGS', payload: 'general' });
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'new-thread',
      name: '新建线程',
      category: '最近使用',
      icon: FilePlus,
      shortcut: '⌘N',
      action: () => {
        requestNewThread();
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'toggle-sidebar',
      name: '切换侧边栏',
      category: '工作区',
      icon: PanelLeft,
      shortcut: '⌘B',
      action: () => {
        dispatch({ type: 'TOGGLE_LEFT_SIDEBAR' });
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'focus-input',
      name: '聚焦输入框',
      category: '工作区',
      icon: MessageSquare,
      shortcut: '⌘J',
      action: () => {
        focusComposer();
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'toggle-terminal',
      name: '切换终端',
      category: '工作区',
      icon: Terminal,
      shortcut: '⌘⇧J',
      action: () => {
        dispatch({ type: 'TOGGLE_TERMINAL_OVERLAY' });
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'split-right',
      name: '当前标签右侧分屏',
      category: '工作区',
      icon: Columns2,
      shortcut: '⌘\\',
      action: () => {
        dispatch({ type: 'SPLIT_ACTIVE_TAB', payload: { side: 'right' } });
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'split-down',
      name: '当前标签下方分屏',
      category: '工作区',
      icon: Rows2,
      shortcut: '⌘⌥\\',
      action: () => {
        dispatch({ type: 'SPLIT_ACTIVE_TAB', payload: { side: 'bottom' } });
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'merge-pane',
      name: '关闭工作区分屏',
      category: '工作区',
      icon: Merge,
      shortcut: '⌘⌥M',
      action: () => {
        dispatch({ type: 'COLLAPSE_WORKSPACE_SPLITS' });
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'search-files',
      name: '搜索文件',
      category: '工作区',
      icon: FileText,
      shortcut: '⌘P',
      action: () => {
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'toggle-theme',
      name: '切换主题',
      category: '外观',
      icon: Sun,
      action: () => {
        dispatch({ type: 'TOGGLE_DARK' });
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'shortcuts',
      name: '快捷键',
      category: '帮助',
      icon: Keyboard,
      shortcut: '⌘/',
      action: () => {
        dispatch({ type: 'OPEN_SETTINGS', payload: 'shortcuts' });
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'help',
      name: '帮助中心',
      category: '帮助',
      icon: HelpCircle,
      action: () => {
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
    {
      id: 'reload',
      name: '重新加载窗口',
      category: '系统',
      icon: RefreshCw,
      action: () => {
        dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
      },
    },
  ], [dispatch]);

  // Filter items by query
  const filteredItems = useMemo(() => {
    if (!query.trim()) return paletteItems;
    const q = query.toLowerCase();
    return paletteItems.filter(
      item =>
        item.name.toLowerCase().includes(q) ||
        item.category.toLowerCase().includes(q)
    );
  }, [query, paletteItems]);

  // Group by category
  const grouped = useMemo(() => {
    const groups: Record<string, PaletteItem[]> = {};
    filteredItems.forEach(item => {
      if (!groups[item.category]) groups[item.category] = [];
      groups[item.category].push(item);
    });
    return groups;
  }, [filteredItems]);

  const allItems = useMemo(() => filteredItems, [filteredItems]);

  // Focus input on open
  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  // Keyboard navigation
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!isOpen) return;

      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault();
          setSelectedIndex(prev => (prev + 1) % allItems.length);
          break;
        case 'ArrowUp':
          e.preventDefault();
          setSelectedIndex(prev => (prev - 1 + allItems.length) % allItems.length);
          break;
        case 'Enter':
          e.preventDefault();
          if (allItems[selectedIndex]) {
            allItems[selectedIndex].action();
          }
          break;
        case 'Escape':
          e.preventDefault();
          dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
          break;
      }
    },
    [isOpen, allItems, selectedIndex, dispatch]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current && allItems.length > 0) {
      const selectedEl = listRef.current.querySelector(`[data-index="${selectedIndex}"]`);
      if (selectedEl) {
        selectedEl.scrollIntoView({ block: 'nearest' });
      }
    }
  }, [selectedIndex, allItems.length]);

  return (
    <AnimatePresence>
      {isOpen && (
        <div
          className="fixed inset-0 z-50 flex justify-center"
          style={{ top: '15%' }}
          role="dialog"
          aria-modal="true"
          aria-label="命令面板"
        >
          {/* Backdrop */}
          <motion.div
            className="absolute inset-0 bg-[rgba(0,0,0,0.5)]"
            style={{ top: '-15%', height: '115%' }}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.1 }}
            onClick={() => dispatch({ type: 'TOGGLE_COMMAND_PALETTE' })}
          />

          {/* Palette */}
          <motion.div
            className="relative z-10 w-full max-w-[560px] mx-4 bg-[#242424] rounded-xl shadow-[0_16px_48px_rgba(0,0,0,0.35)] border border-[rgba(255,255,255,0.06)] overflow-hidden"
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2, ease: easePanel }}
          >
            {/* Search input */}
            <div className="flex items-center h-[52px] px-4 border-b border-[rgba(255,255,255,0.06)]">
              <Search size={16} className="text-[#6B7280] shrink-0" />
              <input
                ref={inputRef}
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="输入命令或搜索文件..."
                className="flex-1 ml-3 bg-transparent text-[15px] text-[#E8E8E8] placeholder:text-[#6B7280] focus:outline-none"
              />
              <span className="px-1.5 py-0.5 text-[11px] font-mono text-[#4B5563] bg-[rgba(255,255,255,0.04)] rounded">
                ESC
              </span>
            </div>

            {/* Results list */}
            <div
              ref={listRef}
              className="max-h-[428px] overflow-y-auto py-1"
            >
              {Object.entries(grouped).map(([category, items]) => (
                <div key={category}>
                  {/* Category header */}
                  <div className="px-3 py-1.5">
                    <span className="text-[11px] font-semibold text-[#6B7280] uppercase tracking-wider">
                      {category}
                    </span>
                  </div>

                  {/* Items */}
                  {items.map((item) => {
                    const globalIndex = allItems.findIndex(i => i.id === item.id);
                    const isSelected = globalIndex === selectedIndex;
                    const Icon = item.icon;

                    return (
                      <button
                        key={item.id}
                        data-index={globalIndex}
                        onClick={() => item.action()}
                        onMouseEnter={() => setSelectedIndex(globalIndex)}
                        className={`w-full h-10 px-3 flex items-center gap-3 text-left transition-colors duration-100 ${
                          isSelected
                            ? 'bg-[rgba(74,124,111,0.12)]'
                            : 'hover:bg-[rgba(255,255,255,0.04)]'
                        }`}
                      >
                        <Icon
                          size={16}
                          className={isSelected ? 'text-[#4A7C6F]' : 'text-[#9CA3AF]'}
                        />
                        <span className={`flex-1 text-[13px] font-medium ${
                          isSelected ? 'text-[#E8E8E8]' : 'text-[#E8E8E8]'
                        }`}>
                          {item.name}
                        </span>
                        {item.shortcut && (
                          <span className="px-1.5 py-0.5 text-[11px] font-mono text-[#4B5563] bg-[rgba(255,255,255,0.04)] rounded">
                            {item.shortcut}
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>
              ))}

              {allItems.length === 0 && (
                <div className="py-8 text-center text-[13px] text-[#6B7280]">
                  未找到匹配的命令
                </div>
              )}
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
