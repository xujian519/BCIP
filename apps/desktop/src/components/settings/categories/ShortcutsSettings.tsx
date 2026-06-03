import { useState } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import { RotateCcw, Command, Search, Sidebar, PanelRight, MessageSquare, FilePlus, Sun, Save, Slash } from 'lucide-react';

interface ShortcutItem {
  id: string;
  name: string;
  keys: string[];
  category: string;
  icon: React.ReactNode;
}

const defaultShortcuts: ShortcutItem[] = [
  { id: '1', name: '新建会话', keys: ['⌘', 'N'], category: '应用', icon: <FilePlus size={14} /> },
  { id: '2', name: '发送消息', keys: ['↵'], category: '应用', icon: <MessageSquare size={14} /> },
  { id: '3', name: '切换侧边栏', keys: ['⌘', 'B'], category: '导航', icon: <Sidebar size={14} /> },
  { id: '4', name: '切换 AI 面板', keys: ['⌘', 'J'], category: '导航', icon: <PanelRight size={14} /> },
  { id: '5', name: '命令面板', keys: ['⇧', '⌘', 'P'], category: '应用', icon: <Command size={14} /> },
  { id: '6', name: '快速搜索', keys: ['⌘', 'K'], category: '应用', icon: <Search size={14} /> },
  { id: '7', name: '切换主题', keys: ['⌘', '/'], category: '应用', icon: <Sun size={14} /> },
  { id: '8', name: '保存', keys: ['⌘', 'S'], category: '编辑器', icon: <Save size={14} /> },
  { id: '9', name: '查看命令', keys: ['/'], category: '聊天', icon: <Slash size={14} /> },
  { id: '10', name: '换行', keys: ['⇧', '↵'], category: '聊天', icon: <MessageSquare size={14} /> },
];

const categories = ['全部', '应用', '编辑器', '聊天', '导航'];

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.02 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 6 },
  show: { opacity: 1, y: 0, transition: { duration: 0.15, ease: 'easeOut' as const } },
};

const ShortcutsSettings: FC = () => {
  const [shortcuts, setShortcuts] = useState<ShortcutItem[]>(defaultShortcuts);
  const [activeCategory, setActiveCategory] = useState('全部');
  const [editingId, setEditingId] = useState<string | null>(null);

  const handleStartEdit = (id: string) => {
    setEditingId(id);
    // Auto-clear edit mode after 3 seconds
    setTimeout(() => setEditingId(null), 3000);
  };

  const filteredShortcuts =
    activeCategory === '全部'
      ? shortcuts
      : shortcuts.filter((s) => s.category === activeCategory);

  const handleReset = () => {
    setShortcuts(defaultShortcuts);
  };

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="show"
      className="flex flex-col"
      style={{ padding: '24px 32px' }}
    >
      {/* Section Header */}
      <motion.div variants={itemVariants} className="mb-5">
        <div className="flex items-center justify-between">
          <div>
            <h2
              style={{
                fontSize: 20,
                fontWeight: 600,
                color: 'var(--text-primary)',
                letterSpacing: '-0.01em',
                lineHeight: 1.4,
                marginBottom: 4,
              }}
            >
              快捷键设置
            </h2>
            <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
              自定义键盘快捷键以提升工作效率
            </p>
          </div>
          <button
            onClick={handleReset}
            className="flex items-center gap-2 px-3 py-1.5 transition-colors"
            style={{
              borderRadius: 8,
              border: '1px solid var(--border-primary)',
              backgroundColor: 'var(--bg-surface)',
              color: 'var(--text-secondary)',
              fontSize: 11,
              fontWeight: 500,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
              e.currentTarget.style.color = 'var(--text-primary)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-surface)';
              e.currentTarget.style.color = 'var(--text-secondary)';
            }}
            type="button"
          >
            <RotateCcw size={12} />
            恢复默认
          </button>
        </div>
      </motion.div>

      {/* Category Filter */}
      <motion.div variants={itemVariants} className="flex gap-1 mb-4">
        {categories.map((cat) => {
          const isActive = activeCategory === cat;
          return (
            <button
              key={cat}
              onClick={() => setActiveCategory(cat)}
              className="px-3 py-1 transition-colors"
              style={{
                borderRadius: 6,
                fontSize: 12,
                fontWeight: isActive ? 500 : 400,
                color: isActive ? 'var(--text-inverse)' : 'var(--text-secondary)',
                backgroundColor: isActive ? 'var(--accent-primary)' : 'transparent',
                border: 'none',
              }}
              type="button"
            >
              {cat}
            </button>
          );
        })}
      </motion.div>

      {/* Shortcuts Table Header */}
      <motion.div
        variants={itemVariants}
        className="flex items-center py-2 px-1"
        style={{
          borderBottom: '1px solid var(--border-primary)',
        }}
      >
        <span
          style={{
            flex: 1,
            fontSize: 11,
            fontWeight: 500,
            color: 'var(--text-tertiary)',
            letterSpacing: '0.01em',
          }}
        >
          操作
        </span>
        <span
          style={{
            fontSize: 11,
            fontWeight: 500,
            color: 'var(--text-tertiary)',
            letterSpacing: '0.01em',
            textAlign: 'right',
          }}
        >
          快捷键
        </span>
      </motion.div>

      {/* Shortcuts List */}
      <div className="flex flex-col">
        {filteredShortcuts.map((shortcut, index) => {
          const isEditing = editingId === shortcut.id;
          return (
            <motion.div
              key={shortcut.id}
              variants={itemVariants}
              className="flex items-center justify-between py-2.5 px-1 transition-colors cursor-pointer"
              style={{
                borderBottom: '1px solid var(--border-secondary)',
              }}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: index * 0.02 }}
              onClick={() => handleStartEdit(shortcut.id)}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              <div className="flex items-center gap-2.5">
                <span style={{ color: 'var(--text-tertiary)' }}>{shortcut.icon}</span>
                <span
                  style={{
                    fontSize: 12,
                    color: isEditing ? 'var(--accent-primary)' : 'var(--text-primary)',
                    fontWeight: isEditing ? 500 : 400,
                  }}
                >
                  {shortcut.name}
                </span>
              </div>
              <div className="flex items-center gap-1">
                {isEditing ? (
                  <span
                    style={{
                      fontSize: 11,
                      color: 'var(--accent-primary)',
                      fontFamily: 'JetBrains Mono, monospace',
                      fontWeight: 500,
                    }}
                  >
                    点击按键...
                  </span>
                ) : (
                  shortcut.keys.map((key, i) => (
                    <span key={i} className="flex items-center">
                      <kbd
                        className="inline-flex items-center justify-center"
                        style={{
                          minWidth: 22,
                          height: 22,
                          padding: '0 6px',
                          borderRadius: 5,
                          backgroundColor: 'var(--bg-surface)',
                          border: '1px solid var(--border-primary)',
                          fontSize: 11,
                          fontFamily: 'JetBrains Mono, monospace',
                          color: 'var(--text-secondary)',
                          boxShadow: '0 1px 0 var(--border-primary)',
                        }}
                      >
                        {key}
                      </kbd>
                      {i < shortcut.keys.length - 1 && (
                        <span
                          style={{
                            color: 'var(--text-tertiary)',
                            fontSize: 10,
                            margin: '0 2px',
                          }}
                        >
                          +
                        </span>
                      )}
                    </span>
                  ))
                )}
              </div>
            </motion.div>
          );
        })}
      </div>

      {/* Hint */}
      <motion.div
        variants={itemVariants}
        className="mt-4"
        style={{
          padding: '10px 12px',
          borderRadius: 8,
          backgroundColor: 'var(--accent-primary-muted)',
        }}
      >
        <p style={{ fontSize: 11, color: 'var(--accent-primary)', lineHeight: 1.5 }}>
          提示: 点击快捷键组合可进行编辑。部分系统快捷键无法修改。
        </p>
      </motion.div>
    </motion.div>
  );
};

export default ShortcutsSettings;
