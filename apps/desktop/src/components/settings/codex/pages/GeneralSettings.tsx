import { Switch } from '@/components/ui/switch';
import { useAppStore } from '@/hooks/useAppStore';
import { useCodexConfig } from '@/hooks/useCodexConfig';
import { isDesktopRpcReady } from '@/lib/configAccess';
import {
  SettingRow,
  SettingsCard,
  SettingsRpcBanner,
} from '../SettingPrimitives';

function readBool(value: unknown, fallback: boolean): boolean {
  return typeof value === 'boolean' ? value : fallback;
}

export default function GeneralSettings() {
  const { state } = useAppStore();
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const { loading, error, saving, writeValue, get } = useCodexConfig(
    rpcReady,
    state.workspaceCwd,
  );

  const preventSleep = readBool(get('prevent_idle_sleep'), false);
  const notifications = readBool(get('tui.notifications'), true);
  const autoConnect = readBool(get('desktop.auto_connect'), true);

  return (
    <div>
      <h1 className="text-2xl font-semibold text-[var(--text-primary)] mb-2">通用</h1>
      <SettingsRpcBanner
        rpcReady={rpcReady}
        loading={loading}
        error={error}
        saving={saving}
      />

      <SettingsCard title="桌面端">
        <SettingRow
          label="启动时连接 app-server"
          description="desktop.auto_connect"
        >
          <Switch
            checked={autoConnect}
            disabled={!rpcReady || saving}
            onCheckedChange={(v) =>
              void writeValue('desktop.auto_connect', v, 'upsert')
            }
          />
        </SettingRow>
      </SettingsCard>

      <SettingsCard title="会话">
        <SettingRow
          label="运行时防止休眠"
          description="prevent_idle_sleep"
        >
          <Switch
            checked={preventSleep}
            disabled={!rpcReady || saving}
            onCheckedChange={(v) => void writeValue('prevent_idle_sleep', v)}
          />
        </SettingRow>
      </SettingsCard>

      <SettingsCard title="通知">
        <SettingRow label="启用通知" description="tui.notifications">
          <Switch
            checked={notifications}
            disabled={!rpcReady || saving}
            onCheckedChange={(v) =>
              void writeValue('tui.notifications', v, 'upsert')
            }
          />
        </SettingRow>
      </SettingsCard>

      <p className="text-xs text-[var(--text-secondary)]">
        线程详情级别、跟进行为等项尚未在 config.toml 中暴露稳定键名，后续与 Codex 桌面 parity 对齐后再绑定。
      </p>
    </div>
  );
}
