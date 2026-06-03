import { Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';
import { settingsTheme } from './settingsTheme';

export function SettingRow({
  label,
  description,
  children,
}: {
  label: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div
      className={cn(
        'flex items-center justify-between py-3',
        settingsTheme.rowDivider,
      )}
    >
      <div className="flex min-w-0 flex-col gap-0.5 pr-4">
        <span className="text-sm font-medium text-[var(--text-primary)]">
          {label}
        </span>
        {description && (
          <span className={settingsTheme.caption}>{description}</span>
        )}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

export function SettingsCard({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className={settingsTheme.card}>
      <h3 className="mb-1 text-sm font-semibold text-[var(--text-primary)]">
        {title}
      </h3>
      <div className="mt-3">{children}</div>
    </div>
  );
}

export function SettingsRpcBanner({
  rpcReady,
  loading,
  error,
  saving,
}: {
  rpcReady: boolean;
  loading: boolean;
  error: string | null;
  saving: boolean;
}) {
  if (!rpcReady) {
    return (
      <p className="mb-4 text-sm text-[var(--status-warning)]">
        请在桌面端连接 app-server 后编辑配置（与终端 config.toml 同步）。
      </p>
    );
  }
  return (
    <div
      className={cn(
        'mb-4 flex min-h-[20px] items-center gap-2 text-xs',
        settingsTheme.caption,
      )}
    >
      {(loading || saving) && <Loader2 size={14} className="animate-spin" />}
      {saving && <span>保存中…</span>}
      {error && <span className="text-[var(--status-error)]">{error}</span>}
    </div>
  );
}
