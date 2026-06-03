import { useState } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import { Download, Upload, Trash2, AlertTriangle } from 'lucide-react';
import SelectSetting from '../SelectSetting';
import ToggleSetting from '../ToggleSetting';

const languages = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'en', label: 'English' },
  { value: 'ja', label: '日本語' },
];

const patentOffices = [
  { value: 'CNIPA', label: '中国专利 (CNIPA)' },
  { value: 'USPTO', label: '美国专利 (USPTO)' },
  { value: 'EPO', label: '欧洲专利 (EPO)' },
  { value: 'WIPO', label: 'WIPO' },
];

const sessionDurations = [
  { value: '7', label: '7天' },
  { value: '30', label: '30天' },
  { value: '90', label: '90天' },
  { value: 'forever', label: '永久' },
];

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.04,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 8 },
  show: { opacity: 1, y: 0, transition: { duration: 0.2, ease: 'easeOut' as const } },
};

const GeneralSettings: FC = () => {
  const [language, setLanguage] = useState('zh-CN');
  const [patentOffice, setPatentOffice] = useState('CNIPA');
  const [sessionDuration, setSessionDuration] = useState('30');
  const [autoSave, setAutoSave] = useState(true);
  const [notifications, setNotifications] = useState(true);
  const [soundEffects, setSoundEffects] = useState(false);
  const [showClearConfirm, setShowClearConfirm] = useState(false);

  const handleExport = () => {
    const data = { export: true, timestamp: new Date().toISOString() };
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `yunxi-backup-${new Date().toISOString().split('T')[0]}.json`;
    a.click();
    URL.revokeObjectURL(url);
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
          通用设置
        </h2>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          配置应用的基本行为和偏好
        </p>
      </motion.div>

      {/* Language */}
      <motion.div variants={itemVariants}>
        <SelectSetting
          label="界面语言"
          value={language}
          options={languages}
          onChange={setLanguage}
        />
      </motion.div>

      {/* Patent Office */}
      <motion.div variants={itemVariants}>
        <SelectSetting
          label="默认专利数据库"
          value={patentOffice}
          options={patentOffices}
          onChange={setPatentOffice}
        />
      </motion.div>

      {/* Session Duration */}
      <motion.div variants={itemVariants}>
        <SelectSetting
          label="会话保留时长"
          value={sessionDuration}
          options={sessionDurations}
          onChange={setSessionDuration}
        />
      </motion.div>

      {/* Section Separator */}
      <motion.div
        variants={itemVariants}
        style={{
          height: 1,
          backgroundColor: 'var(--border-primary)',
          margin: '12px 0',
        }}
      />

      {/* Toggles */}
      <motion.div variants={itemVariants}>
        <ToggleSetting label="自动保存" checked={autoSave} onChange={setAutoSave} />
      </motion.div>
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="通知提醒"
          description="启用桌面通知以获取重要更新"
          checked={notifications}
          onChange={setNotifications}
        />
      </motion.div>
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="音效"
          description="启用操作音效反馈"
          checked={soundEffects}
          onChange={setSoundEffects}
        />
      </motion.div>

      {/* Section Separator */}
      <motion.div
        variants={itemVariants}
        style={{
          height: 1,
          backgroundColor: 'var(--border-primary)',
          margin: '12px 0',
        }}
      />

      {/* Data Management */}
      <motion.div variants={itemVariants} className="flex flex-col gap-3 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            lineHeight: 1.4,
          }}
        >
          数据管理
        </span>
        <div className="flex gap-3">
          <button
            onClick={handleExport}
            className="flex items-center gap-2 px-4 py-2 transition-colors"
            style={{
              borderRadius: 8,
              border: '1px solid var(--border-primary)',
              backgroundColor: 'var(--bg-surface)',
              color: 'var(--text-primary)',
              fontSize: 12,
              fontWeight: 500,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-surface)';
            }}
            type="button"
          >
            <Download size={14} />
            导出所有数据
          </button>
          <button
            className="flex items-center gap-2 px-4 py-2 transition-colors"
            style={{
              borderRadius: 8,
              border: '1px solid var(--border-primary)',
              backgroundColor: 'var(--bg-surface)',
              color: 'var(--text-primary)',
              fontSize: 12,
              fontWeight: 500,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-surface)';
            }}
            type="button"
          >
            <Upload size={14} />
            导入数据
          </button>
        </div>
      </motion.div>

      {/* Danger Zone */}
      <motion.div
        variants={itemVariants}
        style={{
          height: 1,
          backgroundColor: 'rgba(184, 92, 80, 0.3)',
          margin: '12px 0',
        }}
      />
      <motion.div variants={itemVariants} className="flex flex-col gap-3 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--status-error)',
            lineHeight: 1.4,
          }}
        >
          危险区域
        </span>
        {!showClearConfirm ? (
          <button
            onClick={() => setShowClearConfirm(true)}
            className="flex items-center gap-2 px-4 py-2 transition-colors self-start"
            style={{
              borderRadius: 8,
              border: '1px solid rgba(184, 92, 80, 0.4)',
              backgroundColor: 'transparent',
              color: 'var(--status-error)',
              fontSize: 12,
              fontWeight: 500,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'rgba(184, 92, 80, 0.08)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent';
            }}
            type="button"
          >
            <Trash2 size={14} />
            清除所有数据
          </button>
        ) : (
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            className="flex flex-col gap-3 p-4"
            style={{
              borderRadius: 10,
              backgroundColor: 'rgba(184, 92, 80, 0.06)',
              border: '1px solid rgba(184, 92, 80, 0.2)',
            }}
          >
            <div className="flex items-start gap-2">
              <AlertTriangle size={16} style={{ color: 'var(--status-error)', marginTop: 2 }} />
              <div>
                <p style={{ fontSize: 13, fontWeight: 500, color: 'var(--text-primary)' }}>
                  确定要清除所有数据吗？
                </p>
                <p style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 2 }}>
                  此操作不可撤销，所有会话、设置和数据将被永久删除。
                </p>
              </div>
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => setShowClearConfirm(false)}
                className="px-4 py-1.5 transition-colors"
                style={{
                  borderRadius: 6,
                  border: '1px solid var(--border-primary)',
                  backgroundColor: 'var(--bg-surface)',
                  color: 'var(--text-primary)',
                  fontSize: 12,
                }}
                type="button"
              >
                取消
              </button>
              <button
                onClick={() => setShowClearConfirm(false)}
                className="px-4 py-1.5 transition-colors"
                style={{
                  borderRadius: 6,
                  border: 'none',
                  backgroundColor: 'var(--status-error)',
                  color: '#FFFFFF',
                  fontSize: 12,
                }}
                type="button"
              >
                确认清除
              </button>
            </div>
          </motion.div>
        )}
      </motion.div>
    </motion.div>
  );
};

export default GeneralSettings;
