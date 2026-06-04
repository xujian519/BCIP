import type { FC } from 'react'
import { useState, useCallback, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { useNavigate } from 'react-router'
import { invoke } from '@tauri-apps/api/core'
import {
  Loader2, ArrowLeft, ArrowRight, Check, Eye, EyeOff,
  Sparkles, Cpu, Moon, Monitor,
} from 'lucide-react'
import MeshGradient from '../components/MeshGradient'
import { easePageOut } from '@/lib/animations'

type WizardStep = 'welcome' | 'apikey' | 'done'

interface ProviderInfo {
  id: string
  name: string
  model: string
  apiBase: string
  modelProvider: string
  description: string
}

const providers: ProviderInfo[] = [
  {
    id: 'deepseek',
    name: 'DeepSeek',
    model: 'deepseek-v4-pro',
    apiBase: 'https://api.deepseek.com/v1',
    modelProvider: 'DeepSeek',
    description: '直连官方 API，推荐默认',
  },
  {
    id: 'local',
    name: '本地代理 (8788，可选)',
    model: 'glm-5.1',
    apiBase: 'http://127.0.0.1:8788/v1',
    modelProvider: 'LocalProxy',
    description: '需本地 LiteLLM 已启动，否则 502',
  },
  {
    id: 'zhipu',
    name: '智谱 GLM 编程',
    model: 'glm-5.1',
    apiBase: 'https://open.bigmodel.cn/api/coding/paas/v4',
    modelProvider: 'ZhiPu',
    description: '编程 API：glm-5.1 / glm-4.7 等纯文本',
  },
  {
    id: 'zhipu-vlm',
    name: '智谱 GLM 视觉',
    model: 'glm-4.6v',
    apiBase: 'https://open.bigmodel.cn/api/paas/v4',
    modelProvider: 'ZhiPuVLM',
    description: '多模态：图像+文本+工具（glm-4.6v）',
  },
  {
    id: 'kimi',
    name: 'Kimi',
    model: 'kimi-for-coding',
    apiBase: 'https://api.kimi.com/coding/v1',
    modelProvider: 'Kimi',
    description: 'Kimi 编程套餐直连',
  },
]

const stepLabels = ['选择提供商', '配置 API Key', '完成']

const ProviderIcon: FC<{ id: string }> = ({ id }) => {
  const cls = 'w-6 h-6'
  switch (id) {
    case 'deepseek':
      return <Sparkles className={cls} />
    case 'zhipu':
      return <Cpu className={cls} />
    case 'kimi':
      return <Moon className={cls} />
    case 'local':
      return <Monitor className={cls} />
    default:
      return <Sparkles className={cls} />
  }
}

const OnboardingWizard: FC = () => {
  const navigate = useNavigate()
  const [step, setStep] = useState<WizardStep>('welcome')
  const [selectedProvider, setSelectedProvider] = useState<ProviderInfo>(providers[0])
  const [apiKey, setApiKey] = useState('')
  const [apiBase, setApiBase] = useState(providers[0].apiBase)
  const [showKey, setShowKey] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [error, setError] = useState('')
  const [stepDirection, setStepDirection] = useState<1 | -1>(1)
  const [verifying, setVerifying] = useState(false)
  const [verifyResult, setVerifyResult] = useState<'success' | 'fail' | null>(null)
  const [depsChecked, setDepsChecked] = useState(false)
  const [libreOfficeAvailable, setLibreOfficeAvailable] = useState(false)
  const [omlxAvailable, setOmlxAvailable] = useState(false)

  const goToStep = useCallback((nextStep: WizardStep, dir: 1 | -1 = 1) => {
    setStepDirection(dir)
    setStep(nextStep)
  }, [])

  const handleVerify = useCallback(async () => {
    if (!apiKey.trim() || !apiBase.trim()) {
      setError('请先输入 API Key')
      return
    }
    setVerifying(true)
    setVerifyResult(null)
    setError('')
    try {
      const resp = await fetch(`${apiBase}/models`, {
        headers: { Authorization: `Bearer ${apiKey.trim()}` },
      })
      if (resp.ok) {
        setVerifyResult('success')
      } else {
        setVerifyResult('fail')
        setError('API Key 无效，请检查后重试')
      }
    } catch {
      setVerifyResult('fail')
      setError('网络连接失败，请检查网络或 API 地址')
    } finally {
      setVerifying(false)
    }
  }, [apiKey, apiBase])

  useEffect(() => {
    if (step !== 'done' || depsChecked) return
    setDepsChecked(true)
    void (async () => {
      const [loStatus, omlxStatus] = await Promise.allSettled([
        invoke<{ available: boolean }>('libreoffice_status'),
        invoke<{ available: boolean }>('check_omlx_installed'),
      ])
      if (loStatus.status === 'fulfilled') setLibreOfficeAvailable(loStatus.value.available)
      if (omlxStatus.status === 'fulfilled') setOmlxAvailable(omlxStatus.value.available)
    })()
  }, [step, depsChecked])

  const handleSelectProvider = useCallback((provider: ProviderInfo) => {
    setSelectedProvider(provider)
    setApiBase(provider.apiBase)
    setError('')
    goToStep('apikey')
  }, [goToStep])

  const handleLocalMode = useCallback(async () => {
    setIsSaving(true)
    setError('')
    try {
      await invoke('write_config', {
        params: {
          api_key: 'local',
          model: 'glm-5.1',
          model_provider: 'LocalProxy',
        },
      })
      goToStep('done')
    } catch (err) {
      setError(`保存失败：${err instanceof Error ? err.message : String(err)}`)
    } finally {
      setIsSaving(false)
    }
  }, [goToStep])

  const handleSaveApiKey = useCallback(async () => {
    if (!apiKey.trim()) {
      setError('请输入 API Key')
      return
    }
    if (apiKey.trim().length < 8) {
      setError('API Key 无效，请检查后重试')
      return
    }
    setIsSaving(true)
    setError('')
    try {
      await invoke('write_config', {
        params: {
          api_key: apiKey.trim(),
          model: selectedProvider.model,
          model_provider: selectedProvider.modelProvider,
        },
      })
      goToStep('done')
    } catch (err) {
      setError(`保存失败：${err instanceof Error ? err.message : String(err)}`)
    } finally {
      setIsSaving(false)
    }
  }, [apiKey, selectedProvider, goToStep])

  const handleStart = useCallback(() => {
    navigate('/')
  }, [navigate])

  const stepIndex = step === 'welcome' ? 0 : step === 'apikey' ? 1 : 2

  const slideVariants = {
    enter: (dir: number) => ({ x: dir > 0 ? 40 : -40, opacity: 0 }),
    center: { x: 0, opacity: 1 },
    exit: (dir: number) => ({ x: dir > 0 ? -40 : 40, opacity: 0 }),
  }

  return (
    <div
      className="relative flex items-center justify-center overflow-hidden"
      style={{ width: '100vw', height: '100vh', backgroundColor: 'var(--bg-base)' }}
    >
      <div style={{ position: 'absolute', inset: 0, zIndex: 1 }}>
        <MeshGradient />
      </div>

      <motion.div
        className="relative"
        style={{ zIndex: 10 }}
        initial={{ opacity: 0, y: 24, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        transition={{ duration: 0.5, ease: easePageOut, delay: 0.2 }}
      >
        {/* Ambient glow */}
        <div
          className="absolute"
          style={{
            width: '110%', height: '110%', top: '-5%', left: '-5%',
            borderRadius: 20,
            background: 'radial-gradient(ellipse at center, var(--accent-primary) 0%, transparent 70%)',
            opacity: 0.12, filter: 'blur(60px)', zIndex: -1,
          }}
        />

        <div
          style={{
            width: 440,
            background: 'var(--glass-bg)',
            backdropFilter: 'var(--glass-backdrop)',
            WebkitBackdropFilter: 'var(--glass-backdrop)',
            border: '1px solid var(--border-primary)',
            borderRadius: 'var(--radius-xl)',
            padding: '32px 36px 36px',
            boxShadow: 'var(--card)',
          }}
        >
          {/* Step Indicator */}
          <div className="flex items-center justify-between" style={{ marginBottom: 28 }}>
            {stepLabels.map((label, i) => (
              <div key={label} className="flex items-center" style={{ flex: i < stepLabels.length - 1 ? 1 : undefined }}>
                <div className="flex items-center gap-2">
                  <div
                    style={{
                      width: 28, height: 28, borderRadius: '50%',
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                      backgroundColor: i < stepIndex
                        ? 'var(--accent-primary)'
                        : i === stepIndex
                          ? 'var(--accent-primary-muted)'
                          : 'var(--bg-surface)',
                      border: i === stepIndex
                        ? '2px solid var(--accent-primary)'
                        : i < stepIndex
                          ? '2px solid var(--accent-primary)'
                          : '2px solid var(--border-primary)',
                      transition: 'all 0.3s ease',
                    }}
                  >
                    {i < stepIndex ? (
                      <Check size={14} style={{ color: 'var(--text-inverse)' }} />
                    ) : (
                      <span
                        style={{
                          fontSize: 12, fontWeight: 600,
                          color: i === stepIndex ? 'var(--accent-primary)' : 'var(--text-tertiary)',
                        }}
                      >
                        {i + 1}
                      </span>
                    )}
                  </div>
                  <span
                    style={{
                      fontSize: 11, fontWeight: 500,
                      color: i <= stepIndex ? 'var(--text-primary)' : 'var(--text-tertiary)',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {label}
                  </span>
                </div>
                {i < stepLabels.length - 1 && (
                  <div
                    style={{
                      flex: 1, height: 1, margin: '0 8px',
                      backgroundColor: i < stepIndex ? 'var(--accent-primary)' : 'var(--border-primary)',
                      transition: 'background-color 0.3s ease',
                    }}
                  />
                )}
              </div>
            ))}
          </div>

          {/* Step Content */}
          <AnimatePresence mode="wait" custom={stepDirection}>
            {step === 'welcome' && (
              <motion.div
                key="welcome"
                custom={stepDirection}
                variants={slideVariants}
                initial="enter"
                animate="center"
                exit="exit"
                transition={{ duration: 0.2, ease: 'easeOut' }}
              >
                {/* Mascot */}
                <div className="flex justify-center" style={{ marginBottom: 16 }}>
                  <img
                    src="./app-icon.png"
                    alt="云熙智能体 Logo"
                    style={{
                      width: 72, height: 72, borderRadius: 'var(--radius-full)',
                      border: '3px solid var(--bg-elevated)',
                      boxShadow: '0 4px 16px rgba(0,0,0,0.1)',
                      objectFit: 'cover',
                    }}
                  />
                </div>

                <h1
                  style={{
                    fontSize: 22, fontWeight: 600, letterSpacing: '-0.02em',
                    textAlign: 'center', color: 'var(--text-primary)',
                    marginBottom: 4, lineHeight: 1.2,
                  }}
                >
                  欢迎使用云熙智能体
                </h1>
                <p
                  style={{
                    fontSize: 12, color: 'var(--text-secondary)',
                    textAlign: 'center', lineHeight: 1.5, marginBottom: 20,
                  }}
                >
                  专业专利智能助手，请选择模型提供商开始使用
                </p>

                {/* Provider Cards */}
                <div className="grid grid-cols-2" style={{ gap: 10 }}>
                  {providers.map((provider) => (
                    <button
                      key={provider.id}
                      className="flex flex-col items-center text-left"
                      style={{
                        padding: 14, borderRadius: 'var(--radius-md)',
                        border: '1px solid var(--border-primary)',
                        backgroundColor: 'var(--bg-surface)',
                        cursor: 'pointer',
                        gap: 8,
                        transition: 'border-color 0.2s ease, background-color 0.15s ease, transform 0.15s ease',
                      }}
                      onClick={() => {
                        if (provider.id === 'local') {
                          void handleLocalMode()
                        } else {
                          handleSelectProvider(provider)
                        }
                      }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.borderColor = 'var(--border-focus)'
                        e.currentTarget.style.backgroundColor = 'var(--bg-elevated)'
                        e.currentTarget.style.transform = 'translateY(-1px)'
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.borderColor = 'var(--border-primary)'
                        e.currentTarget.style.backgroundColor = 'var(--bg-surface)'
                        e.currentTarget.style.transform = 'none'
                      }}
                      type="button"
                    >
                      <div
                        style={{
                          width: 36, height: 36, borderRadius: 'var(--radius-md)',
                          backgroundColor: 'var(--accent-primary-muted)',
                          color: 'var(--accent-primary)',
                          display: 'flex', alignItems: 'center', justifyContent: 'center',
                        }}
                      >
                        <ProviderIcon id={provider.id} />
                      </div>
                      <span
                        style={{
                          fontSize: 13, fontWeight: 600,
                          color: 'var(--text-primary)',
                        }}
                      >
                        {provider.name}
                      </span>
                      <span
                        style={{
                          fontSize: 10, color: 'var(--text-tertiary)',
                          textAlign: 'center', lineHeight: 1.4,
                        }}
                      >
                        {provider.description}
                      </span>
                    </button>
                  ))}
                </div>

                {/* Footer links */}
                <div className="flex items-center justify-center" style={{ marginTop: 20, gap: 16 }}>
                  {['隐私政策', '服务条款'].map((text) => (
                    <a
                      key={text} href="#" className="no-underline"
                      style={{
                        fontSize: 11, fontWeight: 500, color: 'var(--text-tertiary)',
                        transition: 'color 0.15s ease',
                      }}
                      onMouseEnter={(e) => { e.currentTarget.style.color = 'var(--text-secondary)' }}
                      onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--text-tertiary)' }}
                    >
                      {text}
                    </a>
                  ))}
                </div>
              </motion.div>
            )}

            {step === 'apikey' && (
              <motion.div
                key="apikey"
                custom={stepDirection}
                variants={slideVariants}
                initial="enter"
                animate="center"
                exit="exit"
                transition={{ duration: 0.2, ease: 'easeOut' }}
              >
                {/* Back + Provider info */}
                <div className="flex items-center" style={{ marginBottom: 16, gap: 8 }}>
                  <button
                    onClick={() => goToStep('welcome', -1)}
                    className="flex items-center justify-center"
                    style={{
                      width: 28, height: 28, borderRadius: 'var(--radius-sm)',
                      backgroundColor: 'transparent', border: 'none',
                      color: 'var(--text-secondary)', cursor: 'pointer',
                      transition: 'color 0.15s ease, background-color 0.15s ease',
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.color = 'var(--text-primary)'
                      e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)'
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.color = 'var(--text-secondary)'
                      e.currentTarget.style.backgroundColor = 'transparent'
                    }}
                    type="button"
                  >
                    <ArrowLeft size={16} />
                  </button>
                  <div
                    style={{
                      display: 'flex', alignItems: 'center', gap: 6,
                      padding: '4px 10px', borderRadius: 'var(--radius-sm)',
                      backgroundColor: 'var(--bg-sidebar-active)',
                    }}
                  >
                    <div style={{ color: 'var(--accent-primary)' }}>
                      <ProviderIcon id={selectedProvider.id} />
                    </div>
                    <span style={{ fontSize: 12, fontWeight: 500, color: 'var(--text-primary)' }}>
                      {selectedProvider.name}
                    </span>
                  </div>
                </div>

                <h2
                  style={{
                    fontSize: 16, fontWeight: 600, color: 'var(--text-primary)',
                    marginBottom: 4,
                  }}
                >
                  配置 API Key
                </h2>
                <p
                  style={{
                    fontSize: 12, color: 'var(--text-secondary)',
                    lineHeight: 1.5, marginBottom: 16,
                  }}
                >
                  输入你的 {selectedProvider.name} API Key，将安全保存到本地配置文件
                </p>

                {/* API Key Input */}
                <div className="flex flex-col" style={{ gap: 12 }}>
                  <div className="relative">
                    <input
                      type={showKey ? 'text' : 'password'}
                      value={apiKey}
                      onChange={(e) => {
                        setApiKey(e.target.value)
                        setError('')
                      }}
                      onKeyDown={(e) => e.key === 'Enter' && handleSaveApiKey()}
                      placeholder={`输入 ${selectedProvider.name} API Key...`}
                      className="w-full font-mono"
                      style={{
                        height: 40,
                        backgroundColor: 'var(--bg-surface)',
                        border: `1px solid ${error ? 'var(--status-error)' : 'var(--border-primary)'}`,
                        borderRadius: 'var(--radius-md)',
                        padding: '10px 36px 10px 14px',
                        fontSize: 12, color: 'var(--text-primary)',
                        outline: 'none',
                        transition: 'border-color 0.2s ease, box-shadow 0.15s ease-in-out',
                      }}
                      onFocus={(e) => {
                        if (!error) {
                          e.currentTarget.style.borderColor = 'var(--border-focus)'
                          e.currentTarget.style.boxShadow = '0 0 0 3px var(--accent-primary-muted)'
                        }
                      }}
                      onBlur={(e) => {
                        e.currentTarget.style.borderColor = error ? 'var(--status-error)' : 'var(--border-primary)'
                        e.currentTarget.style.boxShadow = 'none'
                      }}
                    />
                    <button
                      className="absolute flex items-center justify-center"
                      style={{
                        right: 8, top: '50%', transform: 'translateY(-50%)',
                        width: 24, height: 24, color: 'var(--text-tertiary)',
                        background: 'none', border: 'none', cursor: 'pointer',
                      }}
                      onClick={() => setShowKey((p) => !p)}
                      type="button"
                    >
                      {showKey ? <EyeOff size={14} /> : <Eye size={14} />}
                    </button>
                  </div>

                  {error && (
                    <motion.p
                      style={{ fontSize: 11, color: 'var(--status-error)', margin: 0, paddingLeft: 2 }}
                      initial={{ opacity: 0, x: -4 }}
                      animate={{ opacity: 1, x: 0 }}
                      transition={{ duration: 0.15 }}
                    >
                      {error}
                    </motion.p>
                  )}

                  {/* API Base URL (collapsed, pre-filled) */}
                  <details style={{ opacity: 0.7 }}>
                    <summary
                      style={{
                        fontSize: 11, color: 'var(--text-tertiary)', cursor: 'pointer',
                        padding: '2px 0',
                      }}
                    >
                      API 地址（通常无需修改）
                    </summary>
                    <input
                      type="text"
                      value={apiBase}
                      onChange={(e) => setApiBase(e.target.value)}
                      className="w-full font-mono"
                      style={{
                        marginTop: 6,
                        height: 32,
                        backgroundColor: 'var(--bg-surface)',
                        border: '1px solid var(--border-primary)',
                        borderRadius: 'var(--radius-md)',
                        padding: '6px 10px',
                        fontSize: 11, color: 'var(--text-secondary)',
                        outline: 'none',
                      }}
                    />
                  </details>

                  {/* Verify + Save buttons */}
                  <div className="flex" style={{ gap: 8, marginTop: 4 }}>
                    <button
                      onClick={handleVerify}
                      disabled={verifying || isSaving}
                      className="flex items-center justify-center font-inter"
                      style={{
                        height: 40, padding: '0 16px',
                        backgroundColor: verifyResult === 'success'
                          ? 'var(--status-success-muted)'
                          : 'var(--bg-surface)',
                        color: verifyResult === 'success'
                          ? 'var(--status-success)'
                          : 'var(--text-primary)',
                        fontSize: 12, fontWeight: 500,
                        borderRadius: 'var(--radius-md)',
                        border: verifyResult === 'success'
                          ? '1px solid var(--status-success)'
                          : '1px solid var(--border-primary)',
                        cursor: verifying || isSaving ? 'not-allowed' : 'pointer',
                        opacity: verifying || isSaving ? 0.6 : 1,
                        gap: 6,
                        transition: 'border-color 0.2s ease, background-color 0.15s ease',
                        whiteSpace: 'nowrap',
                      }}
                      onMouseEnter={(e) => {
                        if (!verifying && !isSaving) {
                          e.currentTarget.style.borderColor = 'var(--border-focus)'
                          e.currentTarget.style.backgroundColor = 'var(--bg-elevated)'
                        }
                      }}
                      onMouseLeave={(e) => {
                        if (!verifying && !isSaving) {
                          e.currentTarget.style.borderColor = verifyResult === 'success' ? 'var(--status-success)' : 'var(--border-primary)'
                          e.currentTarget.style.backgroundColor = verifyResult === 'success' ? 'var(--status-success-muted)' : 'var(--bg-surface)'
                        }
                      }}
                      type="button"
                    >
                      {verifying ? (
                        <Loader2 size={14} className="animate-spin" />
                      ) : verifyResult === 'success' ? (
                        <Check size={14} />
                      ) : null}
                      验证
                    </button>

                    {/* Save Button */}
                    <button
                      onClick={handleSaveApiKey}
                      disabled={isSaving}
                      className="flex-1 flex items-center justify-center font-inter"
                    style={{
                      height: 40,
                      backgroundColor: 'var(--accent-primary)',
                      color: 'var(--text-inverse)',
                      fontSize: 13, fontWeight: 500,
                      borderRadius: 'var(--radius-md)',
                      border: 'none',
                      cursor: isSaving ? 'not-allowed' : 'pointer',
                      opacity: isSaving ? 0.6 : 1,
                      gap: 8,
                      transition: 'background-color 0.15s ease, transform 0.15s ease',
                    }}
                    onMouseEnter={(e) => {
                      if (!isSaving) {
                        e.currentTarget.style.backgroundColor = 'var(--accent-primary-hover)'
                        e.currentTarget.style.transform = 'translateY(-1px)'
                      }
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.backgroundColor = 'var(--accent-primary)'
                      e.currentTarget.style.transform = 'none'
                    }}
                    type="button"
                  >
                    {isSaving ? (
                      <>
                        <Loader2 size={14} className="animate-spin" />
                        保存中...
                      </>
                    ) : (
                      <>
                        <ArrowRight size={14} />
                        保存并继续
                      </>
                    )}
                  </button>
                  </div>
                </div>
              </motion.div>
            )}

            {step === 'done' && (
              <motion.div
                key="done"
                custom={stepDirection}
                variants={slideVariants}
                initial="enter"
                animate="center"
                exit="exit"
                transition={{ duration: 0.2, ease: 'easeOut' }}
              >
                {/* Success checkmark */}
                <div className="flex justify-center" style={{ marginBottom: 16 }}>
                  <motion.div
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    transition={{ type: 'spring', stiffness: 200, damping: 16, delay: 0.1 }}
                    style={{
                      width: 56, height: 56, borderRadius: '50%',
                      backgroundColor: 'var(--accent-primary-muted)',
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                    }}
                  >
                    <Check size={28} style={{ color: 'var(--accent-primary)' }} />
                  </motion.div>
                </div>

                <h2
                  style={{
                    fontSize: 18, fontWeight: 600, color: 'var(--text-primary)',
                    textAlign: 'center', marginBottom: 8,
                  }}
                >
                  配置完成
                </h2>
                <p
                  style={{
                    fontSize: 12, color: 'var(--text-secondary)',
                    textAlign: 'center', lineHeight: 1.5, marginBottom: 20,
                  }}
                >
                  {selectedProvider.modelProvider === 'local'
                    ? '已启用本地模式，部分 AI 功能受限'
                    : `已成功配置 ${selectedProvider.name}`}
                </p>

                {/* Summary card */}
                <div
                  style={{
                    padding: 14, borderRadius: 'var(--radius-md)',
                    backgroundColor: 'var(--bg-sidebar-active)',
                    marginBottom: 20,
                  }}
                >
                  {selectedProvider.modelProvider !== 'local' ? (
                    <div className="flex flex-col" style={{ gap: 8 }}>
                      <div className="flex justify-between">
                        <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>提供商</span>
                        <span style={{ fontSize: 12, fontWeight: 500, color: 'var(--text-primary)' }}>
                          {selectedProvider.name}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>模型</span>
                        <span style={{ fontSize: 12, fontWeight: 500, color: 'var(--text-primary)' }}>
                          {selectedProvider.model}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>API Key</span>
                        <span style={{ fontSize: 12, color: 'var(--text-secondary)', fontFamily: 'monospace' }}>
                          {apiKey.slice(0, 6)}••••••{apiKey.slice(-4)}
                        </span>
                      </div>
                    </div>
                  ) : (
                    <p style={{ fontSize: 12, color: 'var(--text-secondary)', textAlign: 'center', margin: 0 }}>
                      无需 API Key，可在设置中随时切换至云端模型
                    </p>
                  )}
                </div>

                {/* Optional Components Status */}
                <div style={{ marginBottom: 16 }}>
                  <p style={{ fontSize: 11, color: 'var(--text-tertiary)', marginBottom: 8 }}>
                    可选组件（缺少不影响基本使用）
                  </p>
                  <div className="flex flex-col" style={{ gap: 6 }}>
                    <div className="flex items-center justify-between" style={{ fontSize: 12 }}>
                      <span style={{ color: 'var(--text-secondary)' }}>语义搜索 (oMLX)</span>
                      <span style={{
                        color: omlxAvailable ? 'var(--status-success)' : 'var(--text-tertiary)',
                        fontSize: 11,
                      }}>
                        {omlxAvailable ? '已安装' : '未安装'}
                      </span>
                    </div>
                    <div className="flex items-center justify-between" style={{ fontSize: 12 }}>
                      <span style={{ color: 'var(--text-secondary)' }}>.doc 转换 (LibreOffice)</span>
                      <span style={{
                        color: libreOfficeAvailable ? 'var(--status-success)' : 'var(--text-tertiary)',
                        fontSize: 11,
                      }}>
                        {libreOfficeAvailable ? '已安装' : '未安装'}
                      </span>
                    </div>
                  </div>
                </div>

                <button
                  onClick={handleStart}
                  className="w-full flex items-center justify-center font-inter"
                  style={{
                    height: 40,
                    backgroundColor: 'var(--accent-primary)',
                    color: 'var(--text-inverse)',
                    fontSize: 13, fontWeight: 500,
                    borderRadius: 'var(--radius-md)',
                    border: 'none', cursor: 'pointer',
                    gap: 8,
                    transition: 'background-color 0.15s ease, transform 0.15s ease',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.backgroundColor = 'var(--accent-primary-hover)'
                    e.currentTarget.style.transform = 'translateY(-1px)'
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.backgroundColor = 'var(--accent-primary)'
                    e.currentTarget.style.transform = 'none'
                  }}
                  type="button"
                >
                  开始使用
                </button>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Version */}
          <div className="flex justify-center" style={{ marginTop: 20 }}>
            <span style={{ fontSize: 11, fontWeight: 500, color: 'var(--text-tertiary)' }}>
              v2.1.0 · macOS
            </span>
          </div>
        </div>
      </motion.div>
    </div>
  )
}

export default OnboardingWizard
