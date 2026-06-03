import type { FC } from 'react';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { motion } from 'framer-motion';

interface SelectOption {
  value: string;
  label: string;
}

interface SelectSettingProps {
  label: string;
  description?: string;
  value: string;
  options: SelectOption[];
  onChange: (value: string) => void;
  placeholder?: string;
}

const SelectSetting: FC<SelectSettingProps> = ({
  label,
  description,
  value,
  options,
  onChange,
  placeholder,
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
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger
          className="w-full"
          style={{
            height: 36,
            backgroundColor: 'var(--bg-surface)',
            borderColor: 'var(--border-primary)',
            borderRadius: 8,
            fontSize: 13,
            color: 'var(--text-primary)',
          }}
        >
          <SelectValue placeholder={placeholder} />
        </SelectTrigger>
        <SelectContent
          style={{
            backgroundColor: 'var(--bg-elevated)',
            borderColor: 'var(--border-primary)',
          }}
        >
          {options.map((option) => (
            <SelectItem
              key={option.value}
              value={option.value}
              style={{ fontSize: 13 }}
            >
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </motion.div>
  );
};

export default SelectSetting;
