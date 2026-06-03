import { useCallback } from 'react';
import type { FC, ReactNode } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Settings,
  Cpu,
  Palette,
  FileText,
  Keyboard,
  CreditCard,
  Info,
  ChevronLeft,
} from 'lucide-react';
import { useNavigate } from 'react-router';

export type SettingsCategory =
  | 'general'
  | 'model'
  | 'appearance'
  | 'editor'
  | 'shortcuts'
  | 'cost'
  | 'about';

interface CategoryDef {
  id: SettingsCategory;
  label: string;
  icon: ReactNode;
}

const categories: CategoryDef[] = [
  { id: 'general', label: '通用', icon: <Settings size={16} /> },
  { id: 'model', label: '模型', icon: <Cpu size={16} /> },
  { id: 'appearance', label: '外观', icon: <Palette size={16} /> },
  { id: 'editor', label: '编辑器', icon: <FileText size={16} /> },
  { id: 'shortcuts', label: '快捷键', icon: <Keyboard size={16} /> },
  { id: 'cost', label: '费用', icon: <CreditCard size={16} /> },
  { id: 'about', label: '关于', icon: <Info size={16} /> },
];

interface SettingsLayoutProps {
  children: ReactNode;
  activeCategory: SettingsCategory;
  onCategoryChange: (category: SettingsCategory) => void;
}

const sidebarVariants = {
  hidden: { opacity: 0, x: -8 },
  show: {
    opacity: 1,
    x: 0,
    transition: {
      staggerChildren: 0.04,
      delayChildren: 0.1,
    },
  },
};

const navItemVariants = {
  hidden: { opacity: 0, x: -8 },
  show: { opacity: 1, x: 0, transition: { duration: 0.15 } },
};

const contentVariants = {
  enter: { opacity: 0, y: 6 },
  center: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -6 },
};

const SettingsLayout: FC<SettingsLayoutProps> = ({
  children,
  activeCategory,
  onCategoryChange,
}) => {
  const navigate = useNavigate();
  const contentKey = activeCategory;

  const handleBack = useCallback(() => {
    navigate('/');
  }, [navigate]);

  return (
    <div
      className="flex h-full"
      style={{
        backgroundColor: 'var(--bg-base)',
      }}
    >
      {/* Left Sidebar */}
      <motion.aside
        variants={sidebarVariants}
        initial="hidden"
        animate="show"
        className="shrink-0 flex flex-col"
        style={{
          width: 200,
          backgroundColor: 'var(--bg-surface)',
          borderRight: '1px solid var(--border-primary)',
          padding: '16px 10px',
        }}
      >
        {/* Back Button */}
        <motion.button
          variants={navItemVariants}
          onClick={handleBack}
          className="flex items-center gap-2 px-3 py-2 mb-4 transition-colors"
          style={{
            borderRadius: 8,
            border: 'none',
            backgroundColor: 'transparent',
            color: 'var(--text-secondary)',
            fontSize: 12,
            fontWeight: 500,
            cursor: 'pointer',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            e.currentTarget.style.color = 'var(--text-primary)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }}
          type="button"
        >
          <ChevronLeft size={16} />
          返回
        </motion.button>

        {/* Settings Title */}
        <motion.div
          variants={navItemVariants}
          className="px-3 mb-3"
        >
          <span
            style={{
              fontSize: 11,
              fontWeight: 600,
              color: 'var(--text-tertiary)',
              letterSpacing: '0.06em',
              textTransform: 'uppercase' as const,
            }}
          >
            设置
          </span>
        </motion.div>

        {/* Category List */}
        <nav className="flex flex-col gap-0.5">
          {categories.map((cat) => {
            const isActive = activeCategory === cat.id;
            return (
              <motion.button
                key={cat.id}
                variants={navItemVariants}
                onClick={() => onCategoryChange(cat.id)}
                className="flex items-center gap-2.5 px-3 py-2 transition-all"
                style={{
                  borderRadius: 7,
                  border: 'none',
                  backgroundColor: isActive
                    ? 'var(--accent-primary-muted)'
                    : 'transparent',
                  color: isActive
                    ? 'var(--accent-primary)'
                    : 'var(--text-secondary)',
                  fontSize: 13,
                  fontWeight: isActive ? 500 : 400,
                  cursor: 'pointer',
                  textAlign: 'left',
                  height: 34,
                }}
                whileHover={{
                  backgroundColor: isActive
                    ? 'var(--accent-primary-muted)'
                    : 'var(--bg-sidebar-active)',
                }}
                whileTap={{ scale: 0.98 }}
                type="button"
              >
                <span
                  style={{
                    color: isActive
                      ? 'var(--accent-primary)'
                      : 'var(--text-tertiary)',
                    display: 'flex',
                    alignItems: 'center',
                  }}
                >
                  {cat.icon}
                </span>
                {cat.label}
              </motion.button>
            );
          })}
        </nav>
      </motion.aside>

      {/* Right Content Panel */}
      <main
        className="flex-1 overflow-y-auto"
        style={{
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        <AnimatePresence mode="wait">
          <motion.div
            key={contentKey}
            variants={contentVariants}
            initial="enter"
            animate="center"
            exit="exit"
            transition={{
              duration: 0.15,
              ease: [0.16, 1, 0.3, 1] as [number, number, number, number],
            }}
          >
            {children}
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  );
};

export { categories };
export default SettingsLayout;
