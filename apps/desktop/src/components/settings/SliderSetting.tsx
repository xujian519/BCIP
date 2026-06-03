import type { FC } from 'react';
import { Slider } from '@/components/ui/slider';
import { motion } from 'framer-motion';

interface SliderSettingProps {
  label: string;
  description?: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (value: number) => void;
  valueFormatter?: (value: number) => string;
}

const SliderSetting: FC<SliderSettingProps> = ({
  label,
  description,
  value,
  min,
  max,
  step = 1,
  onChange,
  valueFormatter = (v) => String(v),
}) => {
  return (
    <motion.div
      className="flex flex-col gap-2 py-3"
      initial={{ opacity: 0, y: 4 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
    >
      <div className="flex items-center justify-between">
        <div className="flex flex-col gap-0.5">
          <span
            style={{
              fontSize: 13,
              fontWeight: 500,
              color: 'var(--text-primary)',
              lineHeight: 1.4,
            }}
          >
            {label}
          </span>
          {description && (
            <span
              style={{
                fontSize: 12,
                color: 'var(--text-secondary)',
                lineHeight: 1.5,
              }}
            >
              {description}
            </span>
          )}
        </div>
        <span
          style={{
            fontSize: 12,
            fontFamily: 'JetBrains Mono, monospace',
            color: 'var(--accent-primary)',
            fontWeight: 500,
            minWidth: 40,
            textAlign: 'right',
          }}
        >
          {valueFormatter(value)}
        </span>
      </div>
      <Slider
        value={[value]}
        min={min}
        max={max}
        step={step}
        onValueChange={(vals) => onChange(vals[0])}
        className="w-full"
      />
    </motion.div>
  );
};

export default SliderSetting;
