import { useCallback, useEffect, useState } from 'react';
import { useCodexConfig } from '@/hooks/useCodexConfig';
import {
  probeImBridge,
  resolveImBridgeUrl,
  type ImBridgeProbeState,
} from '@/lib/imBridgeProbe';

const PROBE_INTERVAL_MS = 30_000;

export function useImBridgeStatus(rpcReady: boolean, workspaceCwd: string | null) {
  const { get } = useCodexConfig(rpcReady, workspaceCwd);
  const [state, setState] = useState<ImBridgeProbeState>('checking');
  const [bridgeUrl, setBridgeUrl] = useState('ws://127.0.0.1:3456');

  const runProbe = useCallback(async () => {
    setState('checking');
    const url = resolveImBridgeUrl(get('im.bridge.server_url'));
    setBridgeUrl(url);
    const online = await probeImBridge(url);
    setState(online ? 'online' : 'offline');
  }, [get]);

  useEffect(() => {
    void runProbe();
    const timer = window.setInterval(() => {
      void runProbe();
    }, PROBE_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [runProbe]);

  return {
    bridgeUrl,
    state,
    refresh: runProbe,
  };
}
