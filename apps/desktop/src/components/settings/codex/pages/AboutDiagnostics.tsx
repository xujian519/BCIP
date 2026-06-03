import { useState, useEffect, useCallback } from 'react';
import { FolderOpen, FileText, Copy, Check, RefreshCw } from 'lucide-react';
import { api } from '@/api';
import { useAppStore } from '@/hooks/useAppStore';
import { getAppServerClient } from '@/lib/appServerClient';
import { isDesktopRpcReady } from '@/lib/configAccess';

export default function AboutDiagnostics() {
  const { state, dispatch } = useAppStore();
  const [copied, setCopied] = useState(false);
  const [bcipHome, setBcipHome] = useState<string | null>(null);
  const [configPath, setConfigPath] = useState('~/.bcip/config.toml');
  const [bcipInstalled, setBcipInstalled] = useState<boolean | null>(null);
  const [bcipVersion, setBcipVersion] = useState<string | null>(null);
  const [bcipPath, setBcipPath] = useState<string | null>(null);
  const [appServerTransport, setAppServerTransport] = useState<string>('—');
  const [appServerConnected, setAppServerConnected] = useState(false);

  const version = '0.1.0';
  const buildInfo = 'BCIP Desktop (Tauri)';
  const rpcReady = isDesktopRpcReady(state.connectionStatus);

  const refreshDiagnostics = useCallback(async () => {
    try {
      const info = await api.getCodexHomeInfo();
      setBcipHome(info.codexHome);
      setConfigPath(info.configToml);
    } catch {
      // Web mock
    }

    try {
      const check = await api.checkBcip();
      setBcipInstalled(check.installed);
      setBcipVersion(check.version ?? null);
      setBcipPath(check.path ?? null);
    } catch {
      setBcipInstalled(null);
    }

    if (rpcReady && getAppServerClient().isInitialized()) {
      setAppServerConnected(true);
      setAppServerTransport(state.appServerTransport ?? 'stdio');
    } else {
      setAppServerConnected(state.connectionStatus === 'connected');
      setAppServerTransport(
        state.appServerTransport ?? (rpcReady ? 'stdio (未初始化)' : '—'),
      );
    }
  }, [rpcReady, state.connectionStatus, state.appServerTransport]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- initial data load
    void refreshDiagnostics();
  }, [refreshDiagnostics]);

  const diagnosticText = [
    `云熙 · 专利助手 v${version}`,
    buildInfo,
    `BCIP_HOME: ${bcipHome ?? '—'}`,
    `config.toml: ${configPath}`,
    `bcip CLI: ${bcipInstalled === null ? '—' : bcipInstalled ? `是 (${bcipVersion ?? state.bcipVersion ?? 'unknown'})` : '否'}`,
    (bcipPath ?? state.bcipPath) ? `bcip path: ${bcipPath ?? state.bcipPath}` : null,
    state.bcipSource ? `bcip source: ${state.bcipSource}` : null,
    `app-server: ${appServerConnected ? '已连接' : state.connectionStatus} (${appServerTransport})`,
    `工作区 cwd: ${state.workspaceCwd ?? '—'}`,
    `当前模型: ${state.currentModel}`,
  ]
    .filter(Boolean)
    .join('\n');

  const handleCopyInfo = () => {
    void navigator.clipboard.writeText(diagnosticText).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };

  const handleOpenConfigDir = () => {
    if (bcipHome) {
      void api.revealPathInFileManager(bcipHome);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-semibold text-[var(--text-primary)]">关于与诊断</h1>
        <button
          type="button"
          onClick={() => void refreshDiagnostics()}
          className="h-8 px-3 flex items-center gap-1.5 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
        >
          <RefreshCw size={14} />
          刷新
        </button>
      </div>

      <div className="bg-[var(--bg-elevated)] rounded-xl p-6 mb-4 border border-[var(--border-default)] flex items-start gap-4">
        <div className="w-12 h-12 rounded-xl bg-[var(--accent-primary-muted)] flex items-center justify-center shrink-0">
          <span className="text-lg font-bold text-[var(--accent-primary)]">B</span>
        </div>
        <div>
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">云熙 · 专利助手</h2>
          <p className="text-[13px] font-mono text-[var(--text-secondary)] mt-1">v{version}</p>
          <p className="text-[11px] font-mono text-[var(--text-tertiary)] mt-0.5">{buildInfo}</p>
        </div>
      </div>

      <div className="flex flex-wrap gap-2 mb-4">
        <button
          type="button"
          onClick={() => dispatch({ type: 'OPEN_SETTINGS', payload: 'general' })}
          className="h-8 px-4 flex items-center gap-2 bg-[var(--accent-primary)] hover:bg-[var(--accent-primary-hover)] text-white text-xs font-medium rounded-lg transition-colors duration-150"
        >
          <FileText size={14} />
          连接设置
        </button>
        <button
          type="button"
          onClick={handleOpenConfigDir}
          disabled={!bcipHome}
          className="h-8 px-4 flex items-center gap-2 bg-[var(--bg-hover)] hover:bg-[var(--bg-active)] text-[var(--text-primary)] text-xs rounded-lg transition-colors duration-150 disabled:opacity-50"
        >
          <FolderOpen size={14} />
          打开 BCIP 配置目录
        </button>
        <button
          type="button"
          onClick={handleCopyInfo}
          className="h-8 px-4 flex items-center gap-2 bg-[var(--bg-hover)] hover:bg-[var(--bg-active)] text-[var(--text-primary)] text-xs rounded-lg transition-colors duration-150"
        >
          {copied ? (
            <Check size={14} className="text-[var(--accent-primary)]" />
          ) : (
            <Copy size={14} />
          )}
          {copied ? '已复制' : '复制系统信息'}
        </button>
      </div>

      <div className="bg-[var(--bg-base)] rounded-lg p-3 border border-[var(--border-default)]">
        <h3 className="text-[13px] font-semibold text-[var(--text-primary)] mb-2">诊断信息</h3>
        <pre className="text-[11px] font-mono text-[var(--text-secondary)] leading-relaxed whitespace-pre-wrap">
          {diagnosticText}
        </pre>
      </div>
    </div>
  );
}
