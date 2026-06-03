import { RefreshCw } from 'lucide-react';
import { Switch } from '@/components/ui/switch';
import { useAppStore } from '@/hooks/useAppStore';
import { useSkills } from '@/hooks/useSkills';
import { isDesktopRpcReady } from '@/lib/configAccess';
import {
  SettingsRpcBanner,
} from '../SettingPrimitives';
import { settingsTheme } from '../settingsTheme';

export default function SkillsSettings() {
  const { state } = useAppStore();
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const {
    skills,
    listErrors,
    scopeCwd,
    loading,
    error,
    saving,
    refresh,
    setSkillEnabled,
  } = useSkills(rpcReady, state.workspaceCwd);

  return (
    <div>
      <div className="flex items-center justify-between mb-2">
        <h1 className={settingsTheme.pageTitle}>技能</h1>
        <button
          type="button"
          onClick={() => void refresh(true)}
          disabled={!rpcReady || loading}
          className="h-8 px-3 flex items-center gap-1.5 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] disabled:opacity-50"
        >
          <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
          从磁盘刷新
        </button>
      </div>

      <SettingsRpcBanner
        rpcReady={rpcReady}
        loading={loading}
        error={error}
        saving={saving !== null}
      />

      {scopeCwd && (
        <p className="text-xs text-[var(--text-secondary)] mb-3 font-mono truncate">
          作用域：{scopeCwd}
        </p>
      )}

      {!rpcReady && (
        <p className="text-sm text-[var(--status-warning)] mb-4">
          连接 app-server 后显示与 TUI 同步的技能列表。
        </p>
      )}

      {listErrors.length > 0 && (
        <div className="mb-4 p-3 rounded-lg bg-[var(--status-error-bg)] border border-[var(--status-error)]/25">
          <p className="text-xs font-medium text-[var(--status-error)] mb-1">扫描错误</p>
          {listErrors.map((e) => (
            <p key={`${e.path}-${e.message}`} className="text-[11px] text-[var(--text-secondary)] font-mono">
              {e.path}: {e.message}
            </p>
          ))}
        </div>
      )}

      <div className="bg-[var(--bg-elevated)] rounded-xl border border-[var(--border-default)]">
        {!loading && skills.length === 0 && (
          <p className="px-4 py-6 text-sm text-[var(--text-secondary)] text-center">
            当前工作区未发现技能。请在 ~/.bcip/skills 或项目 .codex/skills 中添加 SKILL.md。
          </p>
        )}
        {skills.map((skill, index) => (
          <div
            key={`${skill.name}-${skill.path}`}
            className={`flex items-center justify-between px-4 py-3 gap-3 ${
              index < skills.length - 1
                ? 'border-b border-[var(--border-default)]'
                : ''
            }`}
          >
            <div className="flex flex-col gap-0.5 min-w-0">
              <span className="text-sm font-medium text-[var(--text-primary)]">
                {skill.displayName}
              </span>
              <span className="text-xs font-mono text-[var(--accent-primary)]">{skill.name}</span>
              {skill.description && (
                <span className="text-xs text-[var(--text-secondary)]">{skill.description}</span>
              )}
              <span className="text-[11px] text-[var(--text-tertiary)] font-mono truncate">
                {skill.path}
              </span>
            </div>
            <Switch
              checked={skill.enabled}
              disabled={!rpcReady || saving === skill.name}
              onCheckedChange={(checked) =>
                void setSkillEnabled(skill, checked)
              }
            />
          </div>
        ))}
      </div>

      <p className="text-xs text-[var(--text-secondary)] mt-4">
        开关写入用户级技能配置（skills/config/write）。文件变更时会收到 skills/changed 并自动刷新。
      </p>
    </div>
  );
}
