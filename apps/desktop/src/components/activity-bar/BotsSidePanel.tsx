import { useState } from 'react';
import { Switch } from '@/components/ui/switch';
import {
  BOT_CHANNEL_DEFINITIONS,
  loadBotChannelStates,
  saveBotChannelState,
  type BotChannelId,
  type BotChannelState,
} from '@/lib/botChannels';

const statusLabel: Record<BotChannelState['status'], string> = {
  disconnected: '未连接',
  configured: '已配置',
  connected: '已连接',
};

const statusColor: Record<BotChannelState['status'], string> = {
  disconnected: 'var(--text-tertiary)',
  configured: 'var(--status-warning)',
  connected: 'var(--status-success)',
};

export default function BotsSidePanel() {
  const [channels, setChannels] = useState(loadBotChannelStates);

  const updateChannel = (id: BotChannelId, patch: Partial<BotChannelState>) => {
    setChannels(saveBotChannelState(id, patch));
  };

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="shrink-0 px-3 py-2">
        <p className="text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
          外接渠道
        </p>
        <p className="mt-1 text-[11px]" style={{ color: 'var(--text-tertiary)' }}>
          配置 Webhook 后，可将 Agent 会话桥接到 IM 渠道（本地保存，后续版本接入同步服务）。
        </p>
      </div>

      <div className="min-h-0 flex-1 space-y-2 overflow-y-auto px-2 pb-2">
        {BOT_CHANNEL_DEFINITIONS.map((channel) => {
          const state = channels[channel.id];
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
                  checked={state.enabled}
                  onCheckedChange={(enabled) => updateChannel(channel.id, { enabled })}
                />
              </div>

              <div className="mb-2 flex items-center gap-1.5 text-[10px]">
                <span
                  className="inline-block h-1.5 w-1.5 rounded-full"
                  style={{ backgroundColor: statusColor[state.status] }}
                />
                <span style={{ color: statusColor[state.status] }}>
                  {statusLabel[state.status]}
                </span>
              </div>

              <input
                type="url"
                value={state.webhookUrl}
                disabled={!state.enabled}
                placeholder="Webhook URL"
                onChange={(e) =>
                  updateChannel(channel.id, { webhookUrl: e.target.value })
                }
                className="w-full rounded-md border px-2 py-1 text-[11px] outline-none transition-colors duration-fast disabled:opacity-50"
                style={{
                  backgroundColor: 'var(--bg-base)',
                  borderColor: 'var(--border-primary)',
                  color: 'var(--text-primary)',
                }}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
