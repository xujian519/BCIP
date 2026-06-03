import { AlertTriangle } from 'lucide-react';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useAppStore } from '@/hooks/useAppStore';
import { useCodexConfig } from '@/hooks/useCodexConfig';
import { isDesktopRpcReady } from '@/lib/configAccess';
import type { AskForApproval } from '@/generated/app-server/v2/AskForApproval';
import type { SandboxMode } from '@/generated/app-server/v2/SandboxMode';
import {
  SettingRow,
  SettingsCard,
  SettingsRpcBanner,
} from '../SettingPrimitives';

type SimpleApproval = Extract<
  AskForApproval,
  'untrusted' | 'on-failure' | 'on-request' | 'never'
>;

const approvalOptions: { value: SimpleApproval; label: string }[] = [
  { value: 'on-request', label: '每次询问' },
  { value: 'on-failure', label: '失败时询问' },
  { value: 'untrusted', label: '不信任命令时询问' },
  { value: 'never', label: '从不询问' },
];

const sandboxOptions: { value: SandboxMode; label: string }[] = [
  { value: 'workspace-write', label: '工作区可写' },
  { value: 'read-only', label: '只读' },
  { value: 'danger-full-access', label: '完全访问（危险）' },
];

function approvalSelectValue(
  policy: AskForApproval | null | undefined,
): SimpleApproval {
  if (
    policy === 'untrusted' ||
    policy === 'on-failure' ||
    policy === 'on-request' ||
    policy === 'never'
  ) {
    return policy;
  }
  return 'on-request';
}

export default function ApprovalSandboxSettings() {
  const { state } = useAppStore();
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const { config, loading, error, saving, writeValue } = useCodexConfig(
    rpcReady,
    state.workspaceCwd,
  );

  const approvalPolicy = config?.approval_policy ?? 'on-request';
  const sandboxMode = config?.sandbox_mode ?? 'workspace-write';
  const approvalValue = approvalSelectValue(approvalPolicy);

  const showDangerWarning =
    approvalValue === 'never' || sandboxMode === 'danger-full-access';

  return (
    <div>
      <h1 className="text-2xl font-semibold text-[var(--text-primary)] mb-2">审批与沙箱</h1>
      <SettingsRpcBanner
        rpcReady={rpcReady}
        loading={loading}
        error={error}
        saving={saving}
      />

      <SettingsCard title="审批策略">
        <SettingRow label="命令审批" description="approval_policy">
          <Select
            value={approvalValue}
            onValueChange={(v) =>
              void writeValue('approval_policy', v as SimpleApproval)
            }
            disabled={!rpcReady}
          >
            <SelectTrigger className="w-[180px] h-8 bg-[var(--bg-base)] border-[var(--border-default)] text-[var(--text-primary)] text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent className="bg-[var(--bg-base)] border-[var(--border-default)]">
              {approvalOptions.map((opt) => (
                <SelectItem key={opt.value} value={opt.value} className="text-xs">
                  {opt.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </SettingRow>
      </SettingsCard>

      <SettingsCard title="沙箱">
        <SettingRow label="沙箱模式" description="sandbox_mode">
          <Select
            value={sandboxMode}
            onValueChange={(v) =>
              void writeValue('sandbox_mode', v as SandboxMode)
            }
            disabled={!rpcReady}
          >
            <SelectTrigger className="w-[180px] h-8 bg-[var(--bg-base)] border-[var(--border-default)] text-[var(--text-primary)] text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent className="bg-[var(--bg-base)] border-[var(--border-default)]">
              {sandboxOptions.map((opt) => (
                <SelectItem key={opt.value} value={opt.value} className="text-xs">
                  {opt.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </SettingRow>
      </SettingsCard>

      {showDangerWarning && (
        <div className="flex items-start gap-2 p-3 bg-[var(--status-error-bg)] border border-[var(--status-error)]/30 rounded-lg">
          <AlertTriangle size={16} className="text-[var(--status-error)] shrink-0 mt-0.5" />
          <p className="text-xs text-[var(--status-error)] leading-relaxed">
            当前设置可能允许未审批命令或扩大文件系统访问范围，请谨慎使用。
          </p>
        </div>
      )}
    </div>
  );
}
