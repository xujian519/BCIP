import { useState } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import { Sun, Moon, Monitor } from 'lucide-react';
import SelectSetting from '../SelectSetting';
import ToggleSetting from '../ToggleSetting';

const fontFamilies = [
  { value: 'jetbrains-mono', label: 'JetBrains Mono' },
  { value: 'fira-code', label: 'Fira Code' },
  { value: 'sf-mono', label: 'SF Mono' },
  { value: 'system', label: '系统默认' },
];

const themes = [
  { value: 'light', label: '浅色', icon: Sun },
  { value: 'dark', label: '深色', icon: Moon },
  { value: 'system', label: '跟随系统', icon: Monitor },
];

const densities = [
  { value: 'compact', label: '紧凑' },
  { value: 'default', label: '默认' },
  { value: 'comfortable', label: '宽松' },
];

const fontSizes = [
  { value: 'small', label: '小' },
  { value: 'medium', label: '中' },
  { value: 'large', label: '大' },
];

const accentColors = [
  { value: 'sage', label: ' sage绿', color: '#4A7C6F', darkColor: '#5FA08F' },
  { value: 'blue', label: ' 蓝色', color: '#5A7D9A', darkColor: '#6B9DC0' },
  { value: 'purple', label: ' 紫色', color: '#7B6FA5', darkColor: '#9B8FC5' },
  { value: 'orange', label: ' 橙色', color: '#B8834A', darkColor: '#D4A06A' },
];

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.04 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 8 },
  show: { opacity: 1, y: 0, transition: { duration: 0.2, ease: 'easeOut' as const } },
};

const AppearanceSettings: FC = () => {
  const [theme, setTheme] = useState<'light' | 'dark' | 'system'>('system');
  const [fontSize, setFontSize] = useState('medium');
  const [editorFont, setEditorFont] = useState('jetbrains-mono');
  const [density, setDensity] = useState('default');
  const [accentColor, setAccentColor] = useState('sage');
  const [animations, setAnimations] = useState(true);

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
          外观设置
        </h2>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          自定义应用的主题、字体和界面密度
        </p>
      </motion.div>

      {/* Theme Selector */}
      <motion.div variants={itemVariants} className="flex flex-col gap-2 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            lineHeight: 1.4,
          }}
        >
          主题
        </span>
        <div className="flex gap-3">
          {themes.map((t) => {
            const Icon = t.icon;
            const isActive = theme === t.value;
            return (
              <motion.button
                key={t.value}
                onClick={() => setTheme(t.value as 'light' | 'dark' | 'system')}
                className="flex flex-col items-center gap-2 px-4 py-3 transition-all"
                style={{
                  width: 96,
                  borderRadius: 10,
                  border: isActive
                    ? '1.5px solid var(--accent-primary)'
                    : '1px solid var(--border-primary)',
                  backgroundColor: isActive
                    ? 'var(--accent-primary-muted)'
                    : 'var(--bg-surface)',
                }}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                type="button"
              >
                <Icon
                  size={22}
                  style={{
                    color: isActive ? 'var(--accent-primary)' : 'var(--text-secondary)',
                  }}
                />
                <span
                  style={{
                    fontSize: 12,
                    fontWeight: isActive ? 500 : 400,
                    color: isActive ? 'var(--accent-primary)' : 'var(--text-primary)',
                  }}
                >
                  {t.label}
                </span>
              </motion.button>
            );
          })}
        </div>
      </motion.div>

      {/* Font Size Segmented Control */}
      <motion.div variants={itemVariants} className="flex flex-col gap-2 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            lineHeight: 1.4,
          }}
        >
          界面字体大小
        </span>
        <div
          className="inline-flex"
          style={{
            borderRadius: 8,
            border: '1px solid var(--border-primary)',
            backgroundColor: 'var(--bg-surface)',
            padding: 3,
            width: 'fit-content',
          }}
        >
          {fontSizes.map((fs) => {
            const isActive = fontSize === fs.value;
            return (
              <button
                key={fs.value}
                onClick={() => setFontSize(fs.value)}
                className="relative px-5 py-1.5 transition-colors"
                style={{
                  borderRadius: 6,
                  fontSize: 12,
                  fontWeight: isActive ? 500 : 400,
                  color: isActive ? 'var(--text-inverse)' : 'var(--text-secondary)',
                  backgroundColor: isActive ? 'var(--accent-primary)' : 'transparent',
                  border: 'none',
                  zIndex: 1,
                }}
                type="button"
              >
                {fs.label}
              </button>
            );
          })}
        </div>
      </motion.div>

      {/* Editor Font */}
      <motion.div variants={itemVariants}>
        <SelectSetting
          label="编辑器字体"
          value={editorFont}
          options={fontFamilies}
          onChange={setEditorFont}
        />
      </motion.div>

      {/* UI Density Segmented Control */}
      <motion.div variants={itemVariants} className="flex flex-col gap-2 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            lineHeight: 1.4,
          }}
        >
          界面密度
        </span>
        <div
          className="inline-flex"
          style={{
            borderRadius: 8,
            border: '1px solid var(--border-primary)',
            backgroundColor: 'var(--bg-surface)',
            padding: 3,
            width: 'fit-content',
          }}
        >
          {densities.map((d) => {
            const isActive = density === d.value;
            return (
              <button
                key={d.value}
                onClick={() => setDensity(d.value)}
                className="relative px-5 py-1.5 transition-colors"
                style={{
                  borderRadius: 6,
                  fontSize: 12,
                  fontWeight: isActive ? 500 : 400,
                  color: isActive ? 'var(--text-inverse)' : 'var(--text-secondary)',
                  backgroundColor: isActive ? 'var(--accent-primary)' : 'transparent',
                  border: 'none',
                  zIndex: 1,
                }}
                type="button"
              >
                {d.label}
              </button>
            );
          })}
        </div>
      </motion.div>

      {/* Accent Color Picker */}
      <motion.div variants={itemVariants} className="flex flex-col gap-2 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            lineHeight: 1.4,
          }}
        >
          主题色
        </span>
        <div className="flex gap-3">
          {accentColors.map((ac) => {
            const isActive = accentColor === ac.value;
            return (
              <motion.button
                key={ac.value}
                onClick={() => setAccentColor(ac.value)}
                className="flex items-center gap-2 px-3 py-2 transition-all"
                style={{
                  borderRadius: 8,
                  border: isActive
                    ? '1.5px solid var(--accent-primary)'
                    : '1px solid var(--border-primary)',
                  backgroundColor: isActive
                    ? 'var(--accent-primary-muted)'
                    : 'var(--bg-surface)',
                }}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                type="button"
              >
                <div
                  style={{
                    width: 14,
                    height: 14,
                    borderRadius: '50%',
                    backgroundColor: ac.color,
                    border: '1.5px solid rgba(0,0,0,0.1)',
                  }}
                />
                <span
                  style={{
                    fontSize: 12,
                    fontWeight: isActive ? 500 : 400,
                    color: isActive ? 'var(--accent-primary)' : 'var(--text-primary)',
                  }}
                >
                  {ac.label}
                </span>
              </motion.button>
            );
          })}
        </div>
      </motion.div>

      {/* Animation Toggle */}
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="动画效果"
          description="启用界面过渡和动画效果"
          checked={animations}
          onChange={setAnimations}
        />
      </motion.div>
    </motion.div>
  );
};

export default AppearanceSettings;
