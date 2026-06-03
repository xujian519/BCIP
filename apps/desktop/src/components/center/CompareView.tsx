import type { FC } from 'react';
import { motion } from 'framer-motion';
import { diffComparison } from '@/data/mockData';
import { GitCompare } from 'lucide-react';

const DiffLine: FC<{
  type: 'add' | 'del' | 'unchanged';
  content: string;
  lineNum: number;
}> = ({ type, content, lineNum }) => {
  const bgColor =
    type === 'add'
      ? 'rgba(74, 124, 111, 0.12)'
      : type === 'del'
        ? 'rgba(184, 92, 80, 0.12)'
        : 'transparent';
  const borderColor =
    type === 'add'
      ? 'var(--status-success)'
      : type === 'del'
        ? 'var(--status-error)'
        : 'transparent';
  const textColor =
    type === 'add'
      ? 'var(--status-success)'
      : type === 'del'
        ? 'var(--status-error)'
        : 'var(--text-primary)';
  const prefix = type === 'add' ? '+ ' : type === 'del' ? '- ' : '  ';

  return (
    <div
      className="flex"
      style={{
        backgroundColor: bgColor,
        borderLeft: `3px solid ${borderColor}`,
        minHeight: 24,
      }}
    >
      <div
        className="select-none text-right"
        style={{
          width: 40,
          paddingRight: 8,
          fontSize: 10,
          fontFamily: "'JetBrains Mono', monospace",
          color: 'var(--text-tertiary)',
          lineHeight: '24px',
          flexShrink: 0,
        }}
      >
        {lineNum}
      </div>
      <div
        style={{
          flex: 1,
          fontSize: 12,
          fontFamily: "'JetBrains Mono', monospace",
          lineHeight: '24px',
          color: textColor,
          paddingLeft: 8,
          whiteSpace: 'pre',
          overflow: 'visible',
        }}
      >
        {prefix}{content}
      </div>
    </div>
  );
};

const CompareView: FC = () => {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="flex h-full flex-col"
      style={{ backgroundColor: 'var(--bg-surface)' }}
    >
      {/* Header */}
      <div
        className="flex items-center justify-between"
        style={{
          padding: '8px 16px',
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        <div className="flex items-center" style={{ gap: 8 }}>
          <GitCompare size={14} style={{ color: 'var(--accent-primary)' }} />
          <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--text-primary)' }}>
            权利要求对比
          </span>
        </div>
        <div className="flex items-center" style={{ gap: 12 }}>
          <div className="flex items-center" style={{ gap: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: 2, backgroundColor: 'var(--status-success)' }} />
            <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>新增</span>
          </div>
          <div className="flex items-center" style={{ gap: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: 2, backgroundColor: 'var(--status-error)' }} />
            <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>删除</span>
          </div>
          <div className="flex items-center" style={{ gap: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: 2, backgroundColor: 'var(--text-tertiary)' }} />
            <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>未变更</span>
          </div>
        </div>
      </div>

      {/* Diff Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Original */}
        <div className="flex-1 overflow-auto custom-scrollbar" style={{ borderRight: '1px solid var(--border-primary)' }}>
          <div
            className="sticky top-0"
            style={{
              padding: '6px 12px',
              fontSize: 11,
              fontWeight: 600,
              color: 'var(--text-secondary)',
              backgroundColor: 'var(--bg-elevated)',
              borderBottom: '1px solid var(--border-primary)',
            }}
          >
            原始版本
          </div>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ staggerChildren: 0.05 }}
          >
            {diffComparison.original.map((line, idx) => (
              <motion.div
                key={`orig-${idx}`}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: idx * 0.05 }}
              >
                <DiffLine type={line.type} content={line.content} lineNum={line.lineNum} />
              </motion.div>
            ))}
          </motion.div>
        </div>

        {/* Modified */}
        <div className="flex-1 overflow-auto custom-scrollbar">
          <div
            className="sticky top-0"
            style={{
              padding: '6px 12px',
              fontSize: 11,
              fontWeight: 600,
              color: 'var(--text-secondary)',
              backgroundColor: 'var(--bg-elevated)',
              borderBottom: '1px solid var(--border-primary)',
            }}
          >
            修改版本
          </div>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ staggerChildren: 0.05 }}
          >
            {diffComparison.modified.map((line, idx) => (
              <motion.div
                key={`mod-${idx}`}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: idx * 0.05 }}
              >
                <DiffLine type={line.type} content={line.content} lineNum={line.lineNum} />
              </motion.div>
            ))}
          </motion.div>
        </div>
      </div>
    </motion.div>
  );
};

export default CompareView;
