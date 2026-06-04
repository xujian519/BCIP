import { createRoot } from 'react-dom/client'
import { HashRouter } from 'react-router'
import { applyThemeToDocument, readStoredThemeMode } from '@/lib/desktopAppearance'
import './index.css'
import App from './App.tsx'
import RootErrorBoundary from './components/RootErrorBoundary.tsx'

/** Tauri / 静态资源加载时保证 Hash 路由落在主壳（避免 location.replace 导致 WKWebView 崩溃） */
function ensureHashRouterBootstrap() {
  const { hash } = window.location;
  if (!hash || hash === '#') {
    window.location.hash = '#/';
    return;
  }
  if (hash.startsWith('#/settings')) {
    window.location.hash = '#/';
  }
}

ensureHashRouterBootstrap()

// 首屏 FOUC 抑制：与 store / config.toml 共用同一套 apply 逻辑
applyThemeToDocument(readStoredThemeMode());

createRoot(document.getElementById('root')!).render(
  <RootErrorBoundary>
    <HashRouter>
      <App />
    </HashRouter>
  </RootErrorBoundary>,
)
