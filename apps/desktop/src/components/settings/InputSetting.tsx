import type { FC } from 'react';
import { Input } from '@/components/ui/input';
import { motion } from 'framer-motion';

interface InputSettingProps {
  label: string;
  description?: string;
  value: string;
  onChange: (value: string) => void;
  type?: string;
  placeholder?: string;
  min?: number;
  max?: number;
  step?: number;
}

const InputSetting: FC<InputSettingProps> = ({
  label,
  description,
  value,
  onChange,
  type = 'text',
  placeholder,
  min,
  max,
  step,
}) => {
  return (
    <motion.div
      className="flex flex-col gap-2 py-3"
      initial={{ opacity: 0, y: 4 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
    >
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
      <Input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        min={min}
        max={max}
        step={step}
        style={{
          height: 36,
          backgroundColor: 'var(--bg-surface)',
          borderColor: 'var(--border-primary)',
          borderRadius: 8,
          fontSize: 13,
          color: 'var(--text-primary)',
        }}
      />
    </motion.div>
  );
};

export default InputSetting;
