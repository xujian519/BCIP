import { useState } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import { TrendingUp, AlertCircle } from 'lucide-react';
import SliderSetting from '../SliderSetting';

interface CostRecord {
  date: string;
  model: string;
  requests: number;
  tokens: number;
  cost: number;
}

interface ModelPrice {
  model: string;
  inputPrice: number;
  outputPrice: number;
}

const sampleCostHistory: CostRecord[] = [
  { date: '2024-12-15', model: 'DeepSeek-V3', requests: 42, tokens: 15600, cost: 1.25 },
  { date: '2024-12-14', model: 'DeepSeek-V3', requests: 38, tokens: 12200, cost: 0.98 },
  { date: '2024-12-13', model: 'DeepSeek-V3', requests: 55, tokens: 23400, cost: 1.87 },
  { date: '2024-12-12', model: 'DeepSeek-V3', requests: 28, tokens: 8900, cost: 0.71 },
  { date: '2024-12-11', model: 'DeepSeek-V3', requests: 31, tokens: 10200, cost: 0.82 },
  { date: '2024-12-10', model: 'DeepSeek-V3', requests: 47, tokens: 18900, cost: 1.51 },
  { date: '2024-12-09', model: 'DeepSeek-V3', requests: 22, tokens: 6700, cost: 0.54 },
];

const modelPricing: ModelPrice[] = [
  { model: 'DeepSeek-V4 Pro', inputPrice: 0.02, outputPrice: 0.08 },
  { model: 'DeepSeek-V4 Flash', inputPrice: 0.005, outputPrice: 0.02 },
  { model: 'Claude 3.5 Opus', inputPrice: 0.15, outputPrice: 0.75 },
  { model: 'Claude 3.5 Sonnet', inputPrice: 0.03, outputPrice: 0.15 },
  { model: 'Claude 3.5 Haiku', inputPrice: 0.008, outputPrice: 0.04 },
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

const CostSettings: FC = () => {
  const [budget, setBudget] = useState('50.00');
  const [threshold, setThreshold] = useState(80);

  const currentUsed = 7.68;
  const budgetNum = parseFloat(budget) || 50;
  const costPercent = Math.min((currentUsed / budgetNum) * 100, 100);
  const costColor =
    costPercent > threshold
      ? 'var(--status-error)'
      : costPercent > threshold * 0.7
        ? 'var(--status-warning)'
        : 'var(--status-success)';

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
          费用管理
        </h2>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          管理 API 调用费用和预算设置
        </p>
      </motion.div>

      {/* Monthly Budget */}
      <motion.div variants={itemVariants}>
        <div className="flex items-center gap-1 mb-1">
          <span
            style={{
              fontSize: 13,
              fontWeight: 500,
              color: 'var(--text-primary)',
              lineHeight: 1.4,
            }}
          >
            月度预算 (CNY)
          </span>
        </div>
        <div className="relative">
          <span
            className="absolute left-3 top-1/2 -translate-y-1/2"
            style={{
              fontSize: 13,
              color: 'var(--text-tertiary)',
              fontWeight: 500,
            }}
          >
            ¥
          </span>
          <input
            type="number"
            value={budget}
            onChange={(e) => setBudget(e.target.value)}
            min={10}
            max={1000}
            step={0.01}
            className="w-full pl-7 pr-3 py-2 transition-colors"
            style={{
              height: 36,
              backgroundColor: 'var(--bg-surface)',
              border: '1px solid var(--border-primary)',
              borderRadius: 8,
              fontSize: 13,
              color: 'var(--text-primary)',
              fontFamily: 'JetBrains Mono, monospace',
              outline: 'none',
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
      </motion.div>

      {/* Cost Warning Threshold */}
      <motion.div variants={itemVariants}>
        <SliderSetting
          label="费用提醒阈值"
          description="当费用达到此比例时发送提醒"
          value={threshold}
          min={50}
          max={95}
          step={5}
          onChange={setThreshold}
          valueFormatter={(v) => `${v}%`}
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

      {/* Current Usage Card */}
      <motion.div
        variants={itemVariants}
        className="flex flex-col gap-3 p-4"
        style={{
          borderRadius: 10,
          backgroundColor: 'var(--bg-sidebar-active)',
          border: '1px solid var(--border-primary)',
        }}
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <TrendingUp size={14} style={{ color: costColor }} />
            <span
              style={{
                fontSize: 13,
                fontWeight: 500,
                color: 'var(--text-primary)',
              }}
            >
              本月费用
            </span>
          </div>
          <span
            style={{
              fontSize: 14,
              fontWeight: 600,
              fontFamily: 'JetBrains Mono, monospace',
              color: costPercent > threshold ? 'var(--status-error)' : 'var(--text-primary)',
            }}
          >
            ¥{currentUsed.toFixed(2)} / ¥{budgetNum.toFixed(2)}
          </span>
        </div>

        {/* Progress Bar */}
        <div
          style={{
            width: '100%',
            height: 6,
            borderRadius: 3,
            backgroundColor: 'var(--border-secondary)',
            overflow: 'hidden',
          }}
        >
          <motion.div
            initial={{ width: 0 }}
            animate={{ width: `${costPercent}%` }}
            transition={{ duration: 0.8, ease: 'easeOut' }}
            style={{
              height: '100%',
              borderRadius: 3,
              backgroundColor: costColor,
              transition: 'background-color 0.3s ease',
            }}
          />
        </div>

        <div className="flex items-center justify-between">
          <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>
            已使用 {costPercent.toFixed(1)}%
          </span>
          {costPercent > threshold && (
            <div className="flex items-center gap-1">
              <AlertCircle size={11} style={{ color: 'var(--status-error)' }} />
              <span style={{ fontSize: 11, color: 'var(--status-error)' }}>
                费用超出阈值
              </span>
            </div>
          )}
        </div>

        <div
          style={{
            height: 1,
            backgroundColor: 'var(--border-primary)',
            margin: '4px 0',
          }}
        />

        <div className="flex items-center justify-between">
          <span style={{ fontSize: 12, color: 'var(--text-secondary)' }}>
            预计剩余可用
          </span>
          <span
            style={{
              fontSize: 12,
              fontWeight: 500,
              fontFamily: 'JetBrains Mono, monospace',
              color: 'var(--text-secondary)',
            }}
          >
            {(budgetNum - currentUsed).toFixed(2)} CNY
          </span>
        </div>
      </motion.div>

      {/* Cost History Table */}
      <motion.div variants={itemVariants} className="mt-5">
        <h3
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            marginBottom: 8,
          }}
        >
          费用明细 (最近7天)
        </h3>
        <div
          style={{
            borderRadius: 10,
            border: '1px solid var(--border-primary)',
            overflow: 'hidden',
          }}
        >
          {/* Table Header */}
          <div
            className="flex items-center py-2 px-3"
            style={{
              backgroundColor: 'var(--bg-surface)',
              borderBottom: '1px solid var(--border-primary)',
            }}
          >
            {['日期', '模型', '请求', 'Token', '费用'].map((h, i) => (
              <span
                key={h}
                style={{
                  flex: i === 0 ? 1.2 : i === 1 ? 1.5 : 0.8,
                  fontSize: 11,
                  fontWeight: 500,
                  color: 'var(--text-tertiary)',
                  letterSpacing: '0.01em',
                  textAlign: i === 4 ? 'right' : 'left',
                }}
              >
                {h}
              </span>
            ))}
          </div>
          {/* Table Rows */}
          {sampleCostHistory.map((record, idx) => (
            <div
              key={idx}
              className="flex items-center py-2 px-3 transition-colors"
              style={{
                borderBottom:
                  idx < sampleCostHistory.length - 1
                    ? '1px solid var(--border-secondary)'
                    : 'none',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              <span
                style={{
                  flex: 1.2,
                  fontSize: 11,
                  fontFamily: 'JetBrains Mono, monospace',
                  color: 'var(--text-secondary)',
                }}
              >
                {record.date}
              </span>
              <span
                style={{
                  flex: 1.5,
                  fontSize: 11,
                  color: 'var(--text-primary)',
                }}
              >
                {record.model}
              </span>
              <span
                style={{
                  flex: 0.8,
                  fontSize: 11,
                  color: 'var(--text-secondary)',
                }}
              >
                {record.requests}
              </span>
              <span
                style={{
                  flex: 0.8,
                  fontSize: 11,
                  fontFamily: 'JetBrains Mono, monospace',
                  color: 'var(--text-secondary)',
                }}
              >
                {record.tokens.toLocaleString()}
              </span>
              <span
                style={{
                  flex: 0.8,
                  fontSize: 11,
                  fontWeight: 500,
                  fontFamily: 'JetBrains Mono, monospace',
                  color: 'var(--text-primary)',
                  textAlign: 'right',
                }}
              >
                ¥{record.cost.toFixed(2)}
              </span>
            </div>
          ))}
        </div>
      </motion.div>

      {/* Model Pricing Table */}
      <motion.div variants={itemVariants} className="mt-5">
        <h3
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            marginBottom: 8,
          }}
        >
          模型定价 (每 1K Tokens)
        </h3>
        <div
          style={{
            borderRadius: 10,
            border: '1px solid var(--border-primary)',
            overflow: 'hidden',
          }}
        >
          <div
            className="flex items-center py-2 px-3"
            style={{
              backgroundColor: 'var(--bg-surface)',
              borderBottom: '1px solid var(--border-primary)',
            }}
          >
            {['模型', '输入', '输出'].map((h, i) => (
              <span
                key={h}
                style={{
                  flex: i === 0 ? 2 : 1,
                  fontSize: 11,
                  fontWeight: 500,
                  color: 'var(--text-tertiary)',
                  letterSpacing: '0.01em',
                  textAlign: i > 0 ? 'right' : 'left',
                }}
              >
                {h}
              </span>
            ))}
          </div>
          {modelPricing.map((mp, idx) => (
            <div
              key={idx}
              className="flex items-center py-2 px-3"
              style={{
                borderBottom:
                  idx < modelPricing.length - 1
                    ? '1px solid var(--border-secondary)'
                    : 'none',
              }}
            >
              <span style={{ flex: 2, fontSize: 11, color: 'var(--text-primary)' }}>
                {mp.model}
              </span>
              <span
                style={{
                  flex: 1,
                  fontSize: 11,
                  fontFamily: 'JetBrains Mono, monospace',
                  color: 'var(--text-secondary)',
                  textAlign: 'right',
                }}
              >
                ¥{mp.inputPrice.toFixed(3)}
              </span>
              <span
                style={{
                  flex: 1,
                  fontSize: 11,
                  fontFamily: 'JetBrains Mono, monospace',
                  color: 'var(--text-secondary)',
                  textAlign: 'right',
                }}
              >
                ¥{mp.outputPrice.toFixed(3)}
              </span>
            </div>
          ))}
        </div>
      </motion.div>
    </motion.div>
  );
};

export default CostSettings;
