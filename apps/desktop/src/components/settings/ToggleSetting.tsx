import type { FC } from 'react';
import { Switch } from '@/components/ui/switch';
import { motion } from 'framer-motion';

interface ToggleSettingProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

const ToggleSetting: FC<ToggleSettingProps> = ({ label, description, checked, onChange }) => {
  return (
    <motion.div
      className="flex items-center justify-between py-3"
      initial={{ opacity: 0, y: 4 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
    >
      <div className="flex flex-col gap-0.5 pr-4">
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
      <Switch
        checked={checked}
        onCheckedChange={onChange}
        style={
          {
            '--accent-primary': 'var(--accent-primary)',
          } as React.CSSProperties
        }
      />
    </motion.div>
  );
};

export default ToggleSetting;
