import { Routes, Route, Navigate } from 'react-router'
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { isTauri } from '@/api/tauri'
import OnboardingWizard from './pages/OnboardingWizard'
import MainApp from './pages/MainApp'
import CodexShellPreview from './pages/CodexShellPreview'

function AuthGuard({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = useState<'loading' | 'authed' | 'guest'>('loading')

  useEffect(() => {
    let cancelled = false
    if (!isTauri()) {
      setStatus('authed')
      return
    }
    invoke<{ config: { api_key?: string; model?: string; model_provider?: string } }>('read_config')
      .then((res) => {
        if (cancelled) return
        const model = res.config?.model?.trim()
        const provider = res.config?.model_provider?.trim()
        if (
          res.config?.api_key?.trim()
          || model === 'local'
          || (model && model.length > 0)
          || provider === 'LocalProxy'
        ) {
          setStatus('authed')
        } else {
          setStatus('guest')
        }
      })
      .catch(() => {
        if (cancelled) return
        setStatus('guest')
      })
    return () => { cancelled = true }
  }, [])

  if (status === 'loading') {
    return (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          width: '100vw',
          height: '100vh',
          backgroundColor: 'var(--bg-base)',
        }}
      >
        <div
          style={{
            width: 32,
            height: 32,
            border: '3px solid var(--border-primary)',
            borderTopColor: 'var(--accent-primary)',
            borderRadius: '50%',
            animation: 'spin 1s linear infinite',
          }}
        />
        <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
      </div>
    )
  }

  if (status === 'authed') return <>{children}</>
  return <Navigate to="/login" replace />
}

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<AuthGuard><MainApp /></AuthGuard>} />
      <Route path="/login" element={<OnboardingWizard />} />
      <Route path="/settings/*" element={<MainApp />} />
      <Route path="/preview/codex-shell" element={<CodexShellPreview />} />
      <Route path="*" element={<MainApp />} />
    </Routes>
  )
}
