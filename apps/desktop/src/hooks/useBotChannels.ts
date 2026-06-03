import { useCallback, useEffect, useRef, useState } from 'react';
import { useCodexConfig } from '@/hooks/useCodexConfig';
import {
  applyBotChannelPatch,
  botChannelsFromConfig,
  botChannelsToConfigValue,
  loadBotChannelStates,
  persistBotChannelStates,
  type BotChannelId,
  type BotChannelState,
} from '@/lib/botChannels';

export function useBotChannels(rpcReady: boolean, workspaceCwd: string | null) {
  const { get, writeValue, loading, saving, error } = useCodexConfig(
    rpcReady,
    workspaceCwd,
  );
  const [channels, setChannels] = useState(loadBotChannelStates);
  const migratedRef = useRef(false);
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!rpcReady || loading) {
      return;
    }
    const fromConfig = botChannelsFromConfig(get('desktop.bot_channels'));
    if (fromConfig) {
      setChannels(fromConfig);
      persistBotChannelStates(fromConfig);
      migratedRef.current = true;
      return;
    }
    if (!migratedRef.current) {
      const local = loadBotChannelStates();
      setChannels(local);
      migratedRef.current = true;
      void writeValue(
        'desktop.bot_channels',
        botChannelsToConfigValue(local),
        'upsert',
      );
    }
  }, [rpcReady, loading, get, writeValue]);

  useEffect(
    () => () => {
      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current);
      }
    },
    [],
  );

  const flushToConfig = useCallback(async () => {
    if (!rpcReady) {
      return;
    }
    const latest = loadBotChannelStates();
    await writeValue(
      'desktop.bot_channels',
      botChannelsToConfigValue(latest),
      'upsert',
    );
  }, [rpcReady, writeValue]);

  const updateChannel = useCallback(
    async (
      id: BotChannelId,
      patch: Partial<Omit<BotChannelState, 'status'>>,
      options?: { debounceMs?: number },
    ) => {
      setChannels((prev) => {
        const next = applyBotChannelPatch(prev, id, patch);
        persistBotChannelStates(next);
        return next;
      });

      if (options?.debounceMs && options.debounceMs > 0) {
        if (saveTimerRef.current) {
          clearTimeout(saveTimerRef.current);
        }
        saveTimerRef.current = setTimeout(() => {
          void flushToConfig();
        }, options.debounceMs);
        return;
      }

      await flushToConfig();
    },
    [flushToConfig],
  );

  return {
    channels,
    loading,
    saving,
    error,
    rpcReady,
    updateChannel,
  };
}
