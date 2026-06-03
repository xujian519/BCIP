import type { FC } from 'react';
import { useState } from 'react';
import { motion } from 'framer-motion';
import { sampleClaims } from '@/data/mockData';

const ClaimsView: FC = () => {
  const [activeLine, setActiveLine] = useState<number | null>(null);
  const lines = sampleClaims.split('\n');

  const renderLine = (line: string, idx: number) => {
    const isClaimNumber = line.trim().match(/^\[\d{4}\\]\s+\d+\./);
    const isActive = activeLine === idx;

    return (
      <div
        key={idx}
        className="flex transition-colors duration-100"
        style={{
          backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
          minHeight: 24,
        }}
        onClick={() => setActiveLine(idx)}
      >
        {/* Line Number */}
        <div
          className="flex-shrink-0 select-none text-right"
          style={{
            width: 48,
            paddingRight: 12,
            fontSize: 11,
            fontFamily: "'JetBrains Mono', monospace",
            color: 'var(--text-tertiary)',
            lineHeight: '24px',
          }}
        >
          {line.trim() ? idx + 1 : ''}
        </div>
        {/* Content */}
        <div
          className="flex-1"
          style={{
            fontSize: 13,
            fontFamily: "'JetBrains Mono', monospace",
            lineHeight: '24px',
            color: 'var(--text-primary)',
            paddingLeft: 8,
            whiteSpace: 'pre',
            overflow: 'visible',
          }}
        >
          {isClaimNumber ? (
            <span>
              <span style={{ color: 'var(--accent-secondary)', fontWeight: 600 }}>
                {line.match(/^\[\d{4}\]\s+(\d+\.)?/)?.[0] || line}
              </span>
              <span>{line.slice((line.match(/^\[\d{4}\]\s+\d+\./)?.[0] || '').length)}</span>
            </span>
          ) : (
            <span style={{ color: isActive ? 'var(--text-primary)' : 'var(--text-secondary)' }}>
              {line}
            </span>
          )}
        </div>
      </div>
    );
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="h-full overflow-auto custom-scrollbar"
      style={{ backgroundColor: 'var(--bg-surface)' }}
    >
      <div style={{ padding: '16px 0', minWidth: 'fit-content' }}>
        {lines.map((line, idx) => renderLine(line, idx))}
      </div>
    </motion.div>
  );
};

export default ClaimsView;
