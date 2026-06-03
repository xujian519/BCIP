import { RefreshCw } from 'lucide-react';
import { Switch } from '@/components/ui/switch';
import { useAppStore } from '@/hooks/useAppStore';
import { useSkills } from '@/hooks/useSkills';
import { isDesktopRpcReady } from '@/lib/configAccess';

export default function SkillsSidePanel() {
  const { state } = useAppStore();
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const {
    skills,
    listErrors,
    loading,
    error,
    saving,
    refresh,
    setSkillEnabled,
  } = useSkills(rpcReady, state.workspaceCwd);

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="flex shrink-0 items-center justify-between px-3 py-2">
        <span className="text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
          技能管理
        </span>
        <button
          type="button"
          onClick={() => void refresh(true)}
          disabled={!rpcReady || loading}
          className="flex h-7 items-center gap-1 rounded-md px-2 text-[11px] text-[var(--text-secondary)] transition-colors duration-fast hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] disabled:opacity-50"
        >
          <RefreshCw size={12} className={loading ? 'animate-spin' : ''} />
          刷新
        </button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
        {!rpcReady && (
          <p className="p-4 text-center text-xs" style={{ color: 'var(--status-warning)' }}>
            连接 app-server 后显示技能列表
          </p>
        )}
        {error && (
          <p className="p-4 text-center text-xs" style={{ color: 'var(--status-error)' }}>
            {error}
          </p>
        )}
        {listErrors.length > 0 && (
          <div
            className="mb-2 rounded-md border p-2"
            style={{
              borderColor: 'rgba(var(--status-error-rgb, 184, 92, 80), 0.25)',
              backgroundColor: 'var(--status-error-bg)',
            }}
          >
            {listErrors.map((entry) => (
              <p
                key={`${entry.path}-${entry.message}`}
                className="font-mono text-[10px] text-[var(--text-secondary)]"
              >
                {entry.path}: {entry.message}
              </p>
            ))}
          </div>
        )}
        {!loading && rpcReady && skills.length === 0 && (
          <p className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
            当前工作区未发现技能
          </p>
        )}
        {skills.map((skill) => (
          <div
            key={`${skill.name}-${skill.path}`}
            className="mb-1 flex items-start justify-between gap-2 rounded-md border px-2 py-2"
            style={{
              borderColor: 'var(--border-default)',
              backgroundColor: 'var(--bg-elevated)',
            }}
          >
            <div className="min-w-0 flex-1">
              <p className="truncate text-xs font-medium text-[var(--text-primary)]">
                {skill.displayName}
              </p>
              <p className="truncate font-mono text-[10px] text-[var(--accent-primary)]">
                {skill.name}
              </p>
              {skill.description && (
                <p className="line-clamp-2 text-[11px] text-[var(--text-secondary)]">
                  {skill.description}
                </p>
              )}
            </div>
            <Switch
              checked={skill.enabled}
              disabled={!rpcReady || saving === skill.name}
              onCheckedChange={(checked) => void setSkillEnabled(skill, checked)}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
