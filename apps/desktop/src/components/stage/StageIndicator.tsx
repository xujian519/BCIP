import type { FC } from 'react';
import { Search, GitCompare, FileSearch, Edit3 } from 'lucide-react';
import { stageLabels } from '@/data/mockData';
import type { WorkStage } from '@/types';

interface StageIndicatorProps {
  activeStage: WorkStage | null;
  onStageClick?: (stage: WorkStage) => void;
}

const stageIcons: Record<WorkStage, typeof Search> = {
  search: Search,
  compare: GitCompare,
  review: FileSearch,
  draft: Edit3,
};

const stageOrder: WorkStage[] = ['search', 'compare', 'review', 'draft'];

const StageIndicator: FC<StageIndicatorProps> = ({ activeStage, onStageClick }) => {
  return (
    <div className="flex items-center" style={{ gap: 4 }}>
      {stageOrder.map((stage, idx) => {
        const Icon = stageIcons[stage];
        const isActive = activeStage === stage;
        return (
          <span key={stage} className="flex items-center" style={{ gap: 4 }}>
            <button
              onClick={() => onStageClick?.(stage)}
              className="flex items-center gap-1.5 transition-all duration-150"
              style={{
                padding: '4px 10px',
                borderRadius: 20,
                fontSize: 12,
                fontWeight: isActive ? 600 : 400,
                backgroundColor: isActive ? 'var(--accent-primary-muted)' : 'transparent',
                color: isActive ? 'var(--accent-primary)' : 'var(--text-tertiary)',
                border: isActive ? '1px solid var(--accent-primary)' : '1px solid transparent',
              }}
              type="button"
            >
              <Icon size={13} />
              {stageLabels[stage]}
            </button>
            {idx < stageOrder.length - 1 && (
              <span
                style={{
                  color: isActive ? 'var(--accent-primary)' : 'var(--border-primary)',
                  fontSize: 10,
                }}
              >
                ──→
              </span>
            )}
          </span>
        );
      })}
    </div>
  );
};

export default StageIndicator;
