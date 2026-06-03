import { Component, type ErrorInfo, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

/** 捕获渲染错误，避免 Tauri 窗口只剩空白底图 */
export default class RootErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('[BCIP] UI render error:', error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return (
        <div
          className="flex h-screen w-screen flex-col items-center justify-center gap-4 p-8"
          style={{
            backgroundColor: 'var(--bg-base, #f5f2ee)',
            color: 'var(--text-primary, #1a1814)',
          }}
        >
          <h1 className="text-lg font-semibold">界面加载失败</h1>
          <pre className="max-h-[50vh] max-w-full overflow-auto rounded-lg bg-black/5 p-4 text-xs">
            {this.state.error.message}
          </pre>
          <button
            type="button"
            className="rounded-md bg-[#4A7C6F] px-4 py-2 text-sm text-white"
            onClick={() => {
              window.location.hash = '#/';
              window.location.reload();
            }}
          >
            返回主界面
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
