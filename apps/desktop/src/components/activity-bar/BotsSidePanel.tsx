import { Loader2 } from 'lucide-react';
import { Switch } from '@/components/ui/switch';
import { useAppStore } from '@/hooks/useAppStore';
import { useBotChannels } from '@/hooks/useBotChannels';
import { isDesktopRpcReady } from '@/lib/configAccess';
import {
  BOT_CHANNEL_DEFINITIONS,
  IM_BRIDGE_DEFAULT_URL,
  type BotChannelId,
} from '@/lib/botChannels';

const statusLabel = {
  disconnected: '未连接',
  configured: '已配置',
  connected: '已连接',
} as const;

const statusColor = {
  disconnected: 'var(--text-tertiary)',
  configured: 'var(--status-warning)',
  connected: 'var(--status-success)',
} as const;

function ChannelFields({
  channelId,
  enabled,
  webhookUrl,
  appId,
  appSecret,
  onPatch,
}: {
  channelId: BotChannelId;
  enabled: boolean;
  webhookUrl: string;
  appId: string;
  appSecret: string;
  onPatch: (patch: {
    webhookUrl?: string;
    appId?: string;
    appSecret?: string;
  }) => void;
}) {
  const inputClass =
    'w-full rounded-md border px-2 py-1 text-[11px] outline-none transition-colors duration-fast disabled:opacity-50';
  const inputStyle = {
    backgroundColor: 'var(--bg-base)',
    borderColor: 'var(--border-primary)',
    color: 'var(--text-primary)',
  };

  if (channelId === 'feishu') {
    return (
      <div className="space-y-2">
        <input
          type="text"
          value={appId}
          disabled={!enabled}
          placeholder="App ID"
          onChange={(e) => onPatch({ appId: e.target.value })}
          className={inputClass}
          style={inputStyle}
        />
        <input
          type="password"
          value={appSecret}
          disabled={!enabled}
          placeholder="App Secret"
          onChange={(e) => onPatch({ appSecret: e.target.value })}
          className={inputClass}
          style={inputStyle}
        />
        <input
          type="url"
          value={webhookUrl}
          disabled={!enabled}
          placeholder="Webhook URL（可选）"
          onChange={(e) => onPatch({ webhookUrl: e.target.value })}
          className={inputClass}
          style={inputStyle}
        />
      </div>
    );
  }

  return (
    <input
      type="url"
      value={webhookUrl}
      disabled={!enabled}
      placeholder="Webhook URL"
      onChange={(e) => onPatch({ webhookUrl: e.target.value })}
      className={inputClass}
      style={inputStyle}
    />
  );
}

export default function BotsSidePanel() {
  const { state } = useAppStore();
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const { channels, loading, saving, error, updateChannel } = useBotChannels(
    rpcReady,
    state.workspaceCwd,
  );

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="shrink-0 px-3 py-2">
        <p className="text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
          外接渠道
        </p>
        <p className="mt-1 text-[11px]" style={{ color: 'var(--text-tertiary)' }}>
          配置写入 <span className="font-mono">desktop.bot_channels</span>，与
          codex-im-bridge（{IM_BRIDGE_DEFAULT_URL}）及 codex-im-feishu 对齐。
        </p>
        {!rpcReady && (
          <p className="mt-1 text-[11px]" style={{ color: 'var(--status-warning)' }}>
            未连接 app-server 时暂存于浏览器本地，连接后自动同步至 config.toml。
          </p>
        )}
        {(loading || saving) && (
          <p className="mt-1 flex items-center gap-1 text-[11px]" style={{ color: 'var(--text-tertiary)' }}>
            <Loader2 size={12} className="animate-spin" />
            同步配置…
          </p>
        )}
        {error && (
          <p className="mt-1 text-[11px]" style={{ color: 'var(--status-error)' }}>
            {error}
          </p>
        )}
      </div>

      <div className="min-h-0 flex-1 space-y-2 overflow-y-auto px-2 pb-2">
        {BOT_CHANNEL_DEFINITIONS.map((channel) => {
          const channelState = channels[channel.id];
          return (
            <div
              key={channel.id}
              className="rounded-md border px-3 py-2"
              style={{
                borderColor: 'var(--border-default)',
                backgroundColor: 'var(--bg-elevated)',
              }}
            >
              <div className="mb-2 flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <p className="text-xs font-medium text-[var(--text-primary)]">
                    {channel.name}
                  </p>
                  <p className="text-[11px] text-[var(--text-secondary)]">
                    {channel.description}
                  </p>
                </div>
                <Switch
                  checked={channelState.enabled}
                  disabled={saving}
                  onCheckedChange={(enabled) =>
                    void updateChannel(channel.id, { enabled })
                  }
                />
              </div>

              <div className="mb-2 flex items-center gap-1.5 text-[10px]">
                <span
                  className="inline-block h-1.5 w-1.5 rounded-full"
                  style={{ backgroundColor: statusColor[channelState.status] }}
                />
                <span style={{ color: statusColor[channelState.status] }}>
                  {statusLabel[channelState.status]}
                </span>
              </div>

              <ChannelFields
                channelId={channel.id}
                enabled={channelState.enabled}
                webhookUrl={channelState.webhookUrl}
                appId={channelState.appId}
                appSecret={channelState.appSecret}
                onPatch={(patch) =>
                  void updateChannel(channel.id, patch, { debounceMs: 400 })
                }
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
