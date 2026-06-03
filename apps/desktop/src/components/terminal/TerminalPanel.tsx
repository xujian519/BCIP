import { useEffect, useRef, useState, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import '@xterm/xterm/css/xterm.css';
import { Play, Square, Trash2, Copy, Terminal as TerminalIcon, AlertTriangle, Download, ExternalLink } from 'lucide-react';
import { spawnTerminal, killTerminal, checkBcipInstalled, type BcipCheckResult } from '@/lib/terminalBridge';

interface TerminalPanelProps {
  onClose?: () => void;
}

export default function TerminalPanel({ onClose }: TerminalPanelProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const terminalInstance = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const [status, setStatus] = useState<'idle' | 'connecting' | 'connected' | 'error'>('idle');
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [bcipStatus, setBcipStatus] = useState<BcipCheckResult | null>(null);
  const [checkingBcip, setCheckingBcip] = useState(false);

  // 检测 bcip 是否安装
  const checkBcip = useCallback(async () => {
    setCheckingBcip(true);
    try {
      const result = await checkBcipInstalled();
      setBcipStatus(result);
    } catch (err) {
      console.error('Failed to check bcip:', err);
      setBcipStatus({ installed: false });
    } finally {
      setCheckingBcip(false);
    }
  }, []);

  // 组件挂载时检测
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- async init
    checkBcip();
  }, [checkBcip]);

  const initTerminal = useCallback(() => {
    if (terminalRef.current && !terminalInstance.current) {
      const term = new Terminal({
        cursorBlink: true,
        fontSize: 14,
        fontFamily: 'JetBrains Mono, Menlo, Monaco, "Courier New", monospace',
        theme: {
          background: '#0D0D0D',
          foreground: '#d4d4d4',
          cursor: '#d4d4d4',
          selectionBackground: '#264f78',
          black: '#000000',
          red: '#cd3131',
          green: '#0dbc79',
          yellow: '#e5e510',
          blue: '#2472c8',
          magenta: '#bc3fbc',
          cyan: '#11a8cd',
          white: '#e5e5e5',
        },
      });

      const fit = new FitAddon();
      term.loadAddon(fit);
      term.loadAddon(new WebLinksAddon());

      term.open(terminalRef.current);
      fit.fit();

      terminalInstance.current = term;
      fitAddon.current = fit;

      // 处理输入
      term.onData((data) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
          wsRef.current.send(data);
        }
      });

      // 处理窗口大小变化
      const handleResize = () => {
        fit.fit();
      };
      window.addEventListener('resize', handleResize);

      return () => {
        window.removeEventListener('resize', handleResize);
      };
    }
  }, []);

  const startTerminal = useCallback(async () => {
    // 先检测 bcip
    if (!bcipStatus?.installed) {
      await checkBcip();
      if (!bcipStatus?.installed) {
        return;
      }
    }

    try {
      setStatus('connecting');
      
      // 启动 bcip tui
      const session = await spawnTerminal('bcip', ['tui']);
      setSessionId(session.id);

      // 连接 WebSocket（使用后端返回的实际 URL）
      const ws = new WebSocket(session.websocketUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setStatus('connected');
      };

      ws.onmessage = (event) => {
        if (event.data instanceof ArrayBuffer) {
          const data = new Uint8Array(event.data);
          terminalInstance.current?.write(data);
        } else {
          terminalInstance.current?.write(event.data);
        }
      };

      ws.onerror = () => {
        setStatus('error');
      };

      ws.onclose = () => {
        setStatus('idle');
      };
    } catch (error) {
      console.error('Failed to start terminal:', error);
      setStatus('error');
    }
  }, [bcipStatus, checkBcip]);

  const stopTerminal = useCallback(async () => {
    if (sessionId) {
      await killTerminal(sessionId);
      setSessionId(null);
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setStatus('idle');
  }, [sessionId]);

  useEffect(() => {
    const cleanup = initTerminal();
    return () => {
      cleanup?.();
      terminalInstance.current?.dispose();
      terminalInstance.current = null;
      wsRef.current?.close();
    };
  }, [initTerminal]);

  // 如果 bcip 未安装，显示安装提示
  if (bcipStatus && !bcipStatus.installed) {
    return (
      <div className="flex flex-col h-full">
        {/* 工具栏 */}
        <div
          className="flex items-center justify-between px-4 py-2"
          style={{
            backgroundColor: 'var(--bg-elevated)',
            borderBottom: '1px solid var(--border-primary)',
          }}
        >
          <div className="flex items-center gap-2">
            <TerminalIcon size={16} style={{ color: 'var(--accent-primary)' }} />
            <span className="text-sm font-medium" style={{ color: 'var(--text-primary)' }}>
              终端
            </span>
          </div>
          {onClose && (
            <button
              onClick={onClose}
              className="p-1.5 rounded transition-colors"
              style={{ color: 'var(--text-secondary)' }}
            >
              ✕
            </button>
          )}
        </div>

        {/* 未安装提示 */}
        <div className="flex-1 flex flex-col items-center justify-center p-8">
          <div
            className="flex flex-col items-center max-w-md"
            style={{
              padding: 32,
              backgroundColor: 'var(--bg-elevated)',
              borderRadius: 16,
              border: '1px solid var(--border-primary)',
            }}
          >
            <div
              className="flex items-center justify-center mb-4"
              style={{
                width: 56,
                height: 56,
                borderRadius: '50%',
                backgroundColor: 'rgba(184, 92, 80, 0.1)',
              }}
            >
              <AlertTriangle size={28} style={{ color: 'var(--status-error)' }} />
            </div>

            <h3
              className="text-lg font-semibold mb-2"
              style={{ color: 'var(--text-primary)' }}
            >
              BCIP CLI 未安装
            </h3>

            <p
              className="text-sm text-center mb-6"
              style={{ color: 'var(--text-secondary)', lineHeight: 1.6 }}
            >
              终端功能需要 BCIP CLI 工具。请先安装它，然后重新启动终端。
            </p>

            <div className="w-full mb-6">
              <div className="text-xs font-medium mb-2" style={{ color: 'var(--text-tertiary)' }}>
                安装步骤：
              </div>
              <div
                className="p-3 rounded-lg text-xs font-mono"
                style={{
                  backgroundColor: 'var(--bg-surface)',
                  border: '1px solid var(--border-secondary)',
                  color: 'var(--text-secondary)',
                  lineHeight: 1.8,
                }}
              >
                # 1. 克隆仓库（如果还没有）<br/>
                git clone https://github.com/xujian519/BCIP.git<br/>
                cd BCIP/codex-rs<br/>
                <br/>
                # 2. 编译安装<br/>
                cargo install --path .<br/>
                <br/>
                # 3. 验证安装<br/>
                bcip --version
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={checkBcip}
                disabled={checkingBcip}
                className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
                style={{
                  backgroundColor: 'var(--accent-primary)',
                  color: 'var(--text-inverse)',
                  opacity: checkingBcip ? 0.6 : 1,
                }}
              >
                {checkingBcip ? (
                  <>
                    <span className="animate-spin">⟳</span>
                    检测中...
                  </>
                ) : (
                  <>
                    <Download size={14} />
                    重新检测
                  </>
                )}
              </button>

              <a
                href="https://github.com/xujian519/BCIP#readme"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
                style={{
                  backgroundColor: 'var(--bg-sidebar-active)',
                  color: 'var(--text-secondary)',
                }}
              >
                <ExternalLink size={14} />
                查看文档
              </a>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* 工具栏 */}
      <div
        className="flex items-center justify-between px-4 py-2"
        style={{
          backgroundColor: 'var(--bg-elevated)',
          borderBottom: '1px solid var(--border-primary)',
        }}
      >
        <div className="flex items-center gap-2">
          <TerminalIcon size={16} style={{ color: 'var(--accent-primary)' }} />
          <span className="text-sm font-medium" style={{ color: 'var(--text-primary)' }}>
            终端
          </span>
          <span
            className="text-xs px-2 py-0.5 rounded-full"
            style={{
              backgroundColor:
                status === 'connected'
                  ? 'rgba(74, 124, 111, 0.2)'
                  : status === 'error'
                  ? 'rgba(184, 92, 80, 0.2)'
                  : 'var(--bg-sidebar-active)',
              color:
                status === 'connected'
                  ? 'var(--status-success)'
                  : status === 'error'
                  ? 'var(--status-error)'
                  : 'var(--text-tertiary)',
            }}
          >
            {status === 'connected' ? '运行中' : status === 'error' ? '错误' : status === 'connecting' ? '连接中' : '空闲'}
          </span>
          {bcipStatus?.version && (
            <span className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
              {bcipStatus.version}
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          {status === 'idle' || status === 'error' ? (
            <button
              onClick={startTerminal}
              className="flex items-center gap-1 px-3 py-1.5 rounded text-sm transition-colors"
              style={{
                backgroundColor: 'var(--accent-primary)',
                color: 'var(--text-inverse)',
              }}
            >
              <Play size={14} />
              启动 TUI
            </button>
          ) : (
            <button
              onClick={stopTerminal}
              className="flex items-center gap-1 px-3 py-1.5 rounded text-sm transition-colors"
              style={{
                backgroundColor: 'var(--status-error)',
                color: 'var(--text-inverse)',
              }}
            >
              <Square size={14} />
              停止
            </button>
          )}
          
          <button
            onClick={() => terminalInstance.current?.clear()}
            className="p-1.5 rounded transition-colors"
            style={{ color: 'var(--text-secondary)' }}
            title="清空"
          >
            <Trash2 size={14} />
          </button>
          
          <button
            onClick={() => {
              const text = terminalInstance.current?.getSelection();
              if (text) navigator.clipboard.writeText(text);
            }}
            className="p-1.5 rounded transition-colors"
            style={{ color: 'var(--text-secondary)' }}
            title="复制"
          >
            <Copy size={14} />
          </button>
          
          {onClose && (
            <button
              onClick={onClose}
              className="p-1.5 rounded transition-colors"
              style={{ color: 'var(--text-secondary)' }}
            >
              ✕
            </button>
          )}
        </div>
      </div>

      {/* 终端区域 */}
      <div className="flex-1 relative overflow-hidden">
        {status === 'idle' ? (
          <div className="flex flex-col items-center justify-center h-full">
            <TerminalIcon size={48} style={{ color: 'var(--text-tertiary)', marginBottom: 16 }} />
            <p className="text-lg mb-2" style={{ color: 'var(--text-primary)' }}>
              终端就绪
            </p>
            <p className="text-sm mb-4" style={{ color: 'var(--text-secondary)' }}>
              点击上方"启动 TUI"按钮启动 BCIP 终端界面
            </p>
            <button
              onClick={startTerminal}
              className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
              style={{
                backgroundColor: 'var(--accent-primary)',
                color: 'var(--text-inverse)',
              }}
            >
              启动 TUI
            </button>
          </div>
        ) : status === 'error' ? (
          <div className="flex flex-col items-center justify-center h-full">
            <p style={{ color: 'var(--status-error)' }}>终端启动失败</p>
            <button
              onClick={startTerminal}
              className="mt-4 px-4 py-2 rounded text-sm"
              style={{
                backgroundColor: 'var(--accent-primary)',
                color: 'var(--text-inverse)',
              }}
            >
              重试
            </button>
          </div>
        ) : (
          <div
            ref={terminalRef}
            className="absolute inset-0 p-2"
            style={{
              backgroundColor: '#1e1e1e',
            }}
          />
        )}
      </div>
    </div>
  );
}