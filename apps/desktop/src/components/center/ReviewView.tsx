import type { FC } from 'react';
import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { reviewData } from '@/data/mockData';
import { ChevronDown, AlertTriangle, Lightbulb, FileText, CheckCircle } from 'lucide-react';

const ReviewView: FC = () => {
  const [expandedObjections, setExpandedObjections] = useState<Set<string>>(new Set(['obj-1']));

  const toggleObjection = (id: string) => {
    setExpandedObjections((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'novelty':
        return <AlertTriangle size={14} style={{ color: 'var(--status-error)' }} />;
      case 'inventive':
        return <Lightbulb size={14} style={{ color: 'var(--status-warning)' }} />;
      case 'support':
        return <FileText size={14} style={{ color: 'var(--status-info)' }} />;
      default:
        return <AlertTriangle size={14} style={{ color: 'var(--status-error)' }} />;
    }
  };

  const getTypeLabel = (type: string) => {
    switch (type) {
      case 'novelty':
        return '新颖性';
      case 'inventive':
        return '创造性';
      case 'support':
        return '支持问题';
      default:
        return type;
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'novelty':
        return 'var(--status-error)';
      case 'inventive':
        return 'var(--status-warning)';
      case 'support':
        return 'var(--status-info)';
      default:
        return 'var(--text-tertiary)';
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="h-full overflow-auto custom-scrollbar"
      style={{ backgroundColor: 'var(--bg-surface)', padding: 20 }}
    >
      {/* Header */}
      <div className="mb-5">
        <h2
          style={{
            fontSize: 18,
            fontWeight: 600,
            color: 'var(--text-primary)',
            letterSpacing: '-0.01em',
            marginBottom: 4,
          }}
        >
          审查意见分析
        </h2>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)' }}>
          共 {reviewData.objections.length} 条审查意见，已回复 {reviewData.responses.length} 条
        </p>
      </div>

      {/* Objections List */}
      <div className="flex flex-col" style={{ gap: 12 }}>
        {reviewData.objections.map((obj, idx) => {
          const isExpanded = expandedObjections.has(obj.id);
          const response = reviewData.responses.find((r) => r.objectionId === obj.id);

          return (
            <motion.div
              key={obj.id}
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: idx * 0.08, duration: 0.3 }}
              style={{
                backgroundColor: 'var(--bg-elevated)',
                borderRadius: 12,
                border: '1px solid var(--border-primary)',
                overflow: 'hidden',
              }}
            >
              {/* Objection Header */}
              <button
                onClick={() => toggleObjection(obj.id)}
                className="flex w-full items-center"
                style={{
                  padding: '12px 16px',
                  gap: 10,
                  backgroundColor: isExpanded ? 'var(--bg-sidebar-active)' : 'transparent',
                  transition: 'background-color 0.15s ease',
                }}
                type="button"
              >
                {getTypeIcon(obj.type)}
                <div className="flex flex-1 flex-col items-start">
                  <div className="flex items-center" style={{ gap: 8 }}>
                    <span
                      style={{
                        fontSize: 12,
                        fontWeight: 600,
                        color: getTypeColor(obj.type),
                      }}
                    >
                      {getTypeLabel(obj.type)}
                    </span>
                    <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>
                      {obj.claim}
                    </span>
                  </div>
                  {obj.citation && (
                    <span style={{ fontSize: 10, color: 'var(--text-tertiary)', marginTop: 2 }}>
                      引用: {obj.citation}
                    </span>
                  )}
                </div>
                <ChevronDown
                  size={14}
                  style={{
                    color: 'var(--text-tertiary)',
                    transform: isExpanded ? 'rotate(180deg)' : 'rotate(0deg)',
                    transition: 'transform 0.2s ease',
                  }}
                />
              </button>

              {/* Expanded Content */}
              <AnimatePresence>
                {isExpanded && (
                  <motion.div
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: 'auto', opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    className="overflow-hidden"
                  >
                    <div style={{ padding: '12px 16px', borderTop: '1px solid var(--border-secondary)' }}>
                      <p
                        style={{
                          fontSize: 13,
                          lineHeight: 1.6,
                          color: 'var(--text-primary)',
                          marginBottom: response ? 12 : 0,
                        }}
                      >
                        {obj.content}
                      </p>

                      {response && (
                        <div
                          style={{
                            backgroundColor: 'var(--accent-primary-muted)',
                            borderRadius: 8,
                            padding: '10px 12px',
                            marginTop: 8,
                          }}
                        >
                          <div className="flex items-center" style={{ gap: 6, marginBottom: 6 }}>
                            <CheckCircle size={12} style={{ color: 'var(--accent-primary)' }} />
                            <span
                              style={{
                                fontSize: 11,
                                fontWeight: 600,
                                color: 'var(--accent-primary)',
                              }}
                            >
                              答复意见
                            </span>
                          </div>
                          <p
                            style={{
                              fontSize: 12,
                              lineHeight: 1.6,
                              color: 'var(--text-primary)',
                            }}
                          >
                            {response.content}
                          </p>
                        </div>
                      )}

                      {!response && (
                        <button
                          className="mt-2 transition-colors duration-150"
                          style={{
                            padding: '6px 12px',
                            fontSize: 11,
                            fontWeight: 500,
                            color: 'var(--accent-primary)',
                            backgroundColor: 'var(--accent-primary-muted)',
                            borderRadius: 6,
                          }}
                          type="button"
                        >
                          撰写答复...
                        </button>
                      )}
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.div>
          );
        })}
      </div>
    </motion.div>
  );
};

export default ReviewView;
