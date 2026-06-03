import type { FC } from 'react';
import { useState } from 'react';
import { motion } from 'framer-motion';
import { searchResults } from '@/data/mockData';
import { Search, Filter } from 'lucide-react';

const statusLabels: Record<string, { text: string; color: string; bg: string }> = {
  draft: { text: '草稿', color: 'var(--text-tertiary)', bg: 'var(--bg-sidebar-active)' },
  published: { text: '已公开', color: 'var(--status-success)', bg: 'rgba(74, 124, 111, 0.12)' },
  examination: { text: '审查中', color: 'var(--status-warning)', bg: 'rgba(184, 146, 58, 0.12)' },
  rejected: { text: '驳回', color: 'var(--status-error)', bg: 'rgba(184, 92, 80, 0.12)' },
};

const SearchView: FC = () => {
  const [query, setQuery] = useState('');
  const [activeFilter, setActiveFilter] = useState('全部');
  const filters = ['全部', '发明专利', '实用新型', '外观设计', 'PCT'];

  const filtered = searchResults.filter((r) => {
    if (!query) return true;
    const q = query.toLowerCase();
    return (
      r.title.toLowerCase().includes(q) ||
      r.number.toLowerCase().includes(q) ||
      r.applicant.toLowerCase().includes(q)
    );
  });

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
        style={{
          padding: '16px 20px',
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        {/* Search Input */}
        <div className="relative" style={{ marginBottom: 12 }}>
          <Search
            size={16}
            className="pointer-events-none absolute"
            style={{ left: 14, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-tertiary)' }}
          />
          <input
            type="text"
            placeholder="搜索专利..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="w-full transition-all duration-150 focus:outline-none"
            style={{
              height: 38,
              padding: '8px 14px 8px 40px',
              fontSize: 14,
              borderRadius: 8,
              backgroundColor: 'var(--bg-surface)',
              border: '1px solid var(--border-primary)',
              color: 'var(--text-primary)',
            }}
            onFocus={(e) => {
              e.currentTarget.style.borderColor = 'var(--border-focus)';
              e.currentTarget.style.boxShadow = '0 0 0 3px var(--accent-primary-muted)';
            }}
            onBlur={(e) => {
              e.currentTarget.style.borderColor = 'var(--border-primary)';
              e.currentTarget.style.boxShadow = 'none';
            }}
          />
        </div>

        {/* Filters */}
        <div className="flex items-center justify-between">
          <div className="flex items-center" style={{ gap: 6 }}>
            {filters.map((f) => (
              <button
                key={f}
                onClick={() => setActiveFilter(f)}
                className="transition-all duration-150"
                style={{
                  padding: '4px 12px',
                  fontSize: 11,
                  fontWeight: 500,
                  borderRadius: 6,
                  backgroundColor: activeFilter === f ? 'var(--accent-primary-muted)' : 'transparent',
                  color: activeFilter === f ? 'var(--accent-primary)' : 'var(--text-secondary)',
                  border: activeFilter === f ? '1px solid var(--accent-primary)' : '1px solid var(--border-primary)',
                }}
                type="button"
              >
                {f}
              </button>
            ))}
          </div>
          <div className="flex items-center" style={{ gap: 4 }}>
            <Filter size={12} style={{ color: 'var(--text-tertiary)' }} />
            <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>
              共 {filtered.length} 条结果
            </span>
          </div>
        </div>
      </div>

      {/* Results */}
      <div className="flex-1 overflow-auto custom-scrollbar" style={{ padding: 16 }}>
        <div className="flex flex-col" style={{ gap: 12 }}>
          {filtered.map((result, idx) => {
            const status = statusLabels[result.status];
            return (
              <motion.div
                key={result.id}
                initial={{ opacity: 0, y: 12 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: idx * 0.04, duration: 0.25 }}
                className="transition-all duration-150"
                style={{
                  backgroundColor: 'var(--bg-elevated)',
                  borderRadius: 10,
                  border: '1px solid var(--border-primary)',
                  padding: '14px 16px',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-1px)';
                  e.currentTarget.style.boxShadow = '0 4px 12px rgba(0,0,0,0.06)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.boxShadow = 'none';
                }}
              >
                <div className="mb-2 flex items-start justify-between">
                  <h3
                    className="flex-1 cursor-pointer truncate"
                    style={{
                      fontSize: 14,
                      fontWeight: 600,
                      color: 'var(--accent-primary)',
                      lineHeight: 1.4,
                    }}
                  >
                    {result.title}
                  </h3>
                  <span
                    style={{
                      marginLeft: 8,
                      padding: '2px 8px',
                      fontSize: 10,
                      fontWeight: 500,
                      borderRadius: 4,
                      backgroundColor: status.bg,
                      color: status.color,
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {status.text}
                  </span>
                </div>

                <div className="flex items-center" style={{ gap: 12, marginBottom: 6 }}>
                  <span
                    style={{
                      fontSize: 11,
                      fontFamily: "'JetBrains Mono', monospace",
                      color: 'var(--text-secondary)',
                    }}
                  >
                    {result.number}
                  </span>
                  <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>
                    {result.applicant}
                  </span>
                  <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>
                    {result.date}
                  </span>
                </div>

                <p
                  style={{
                    fontSize: 12,
                    lineHeight: 1.5,
                    color: 'var(--text-primary)',
                    display: '-webkit-box',
                    WebkitLineClamp: 2,
                    WebkitBoxOrient: 'vertical',
                    overflow: 'hidden',
                    marginBottom: 8,
                  }}
                >
                  {result.abstract}
                </p>

                {/* Relevance Bar */}
                <div className="flex items-center" style={{ gap: 8 }}>
                  <span style={{ fontSize: 10, color: 'var(--text-tertiary)' }}>相关度</span>
                  <div
                    style={{
                      flex: 1,
                      height: 4,
                      borderRadius: 2,
                      backgroundColor: 'var(--border-primary)',
                      overflow: 'hidden',
                    }}
                  >
                    <motion.div
                      initial={{ width: 0 }}
                      animate={{ width: `${result.relevance * 100}%` }}
                      transition={{ duration: 0.6, delay: idx * 0.1 }}
                      style={{
                        height: '100%',
                        borderRadius: 2,
                        backgroundColor: 'var(--accent-primary)',
                      }}
                    />
                  </div>
                  <span
                    style={{
                      fontSize: 10,
                      fontWeight: 500,
                      color: 'var(--accent-primary)',
                      minWidth: 32,
                      textAlign: 'right',
                    }}
                  >
                    {Math.round(result.relevance * 100)}%
                  </span>
                </div>
              </motion.div>
            );
          })}
        </div>
      </div>
    </motion.div>
  );
};

export default SearchView;
