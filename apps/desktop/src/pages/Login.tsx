import type { FC } from 'react';
import { useState, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useNavigate } from 'react-router';
import { Key, Eye, EyeOff, Loader2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import MeshGradient from '../components/MeshGradient';

type LoginMode = 'select' | 'apikey' | 'local';

interface LoginFormData {
  apiKey: string;
  showKey: boolean;
  isLoading: boolean;
  error: string;
}

const easeOut = [0.16, 1, 0.3, 1] as [number, number, number, number];

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.08, delayChildren: 0.6 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 8 },
  visible: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.2, ease: 'easeOut' as const },
  },
};

const Login: FC = () => {
  const navigate = useNavigate();
  const [mode, setMode] = useState<LoginMode>('select');
  const [form, setForm] = useState<LoginFormData>({
    apiKey: '',
    showKey: false,
    isLoading: false,
    error: '',
  });

  const handleDeepSeekLogin = useCallback(() => {
    // DeepSeek OAuth 跳转（暂用模拟，后续接入 OAuth 回调）
    setForm((prev) => ({ ...prev, isLoading: true, error: '' }));
    // TODO: 替换为真实 OAuth 流程，回调后 invoke('write_config', { params: { api_key } })
    setTimeout(() => {
      setForm((prev) => ({ ...prev, isLoading: false }));
      navigate('/');
    }, 1500);
  }, [navigate]);

  const handleApiKeySubmit = useCallback(async () => {
    if (!form.apiKey.trim()) return;
    if (form.apiKey.length < 8) {
      setForm((prev) => ({ ...prev, error: 'API Key 无效，请检查后重试' }));
      return;
    }
    setForm((prev) => ({ ...prev, isLoading: true, error: '' }));
    try {
      // 通过 Tauri command 持久化 API Key 到 ~/.bcip/config.toml
      await invoke('write_config', {
        params: {
          api_key: form.apiKey.trim(),
          model: null,
          model_provider: null,
        },
      });
      navigate('/');
    } catch (err) {
      setForm((prev) => ({
        ...prev,
        isLoading: false,
        error: `保存失败：${err instanceof Error ? err.message : String(err)}`,
      }));
    }
  }, [form.apiKey, navigate]);

  const handleLocalMode = useCallback(() => {
    setForm((prev) => ({ ...prev, isLoading: true }));
    setTimeout(() => {
      setForm((prev) => ({ ...prev, isLoading: false }));
      navigate('/');
    }, 800);
  }, [navigate]);

  const toggleMode = useCallback((newMode: LoginMode) => {
    setMode((prev) => (prev === newMode ? 'select' : newMode));
    setForm({ apiKey: '', showKey: false, isLoading: false, error: '' });
  }, []);

  return (
    <div
      className="relative flex items-center justify-center overflow-hidden"
      style={{ width: '100vw', height: '100vh', backgroundColor: 'var(--bg-base)' }}
    >
      {/* Three.js Animated Background */}
      <div style={{ position: 'absolute', inset: 0, zIndex: 1 }}>
        <MeshGradient />
      </div>

      {/* Login Card */}
      <motion.div
        className="relative"
        style={{ zIndex: 10 }}
        initial={{ opacity: 0, y: 24, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        transition={{ duration: 0.5, ease: easeOut, delay: 0.2 }}
      >
        {/* Ambient glow behind card */}
        <div
          className="absolute animate-glow-pulse"
          style={{
            width: '110%',
            height: '110%',
            top: '-5%',
            left: '-5%',
            borderRadius: 20,
            background: 'radial-gradient(ellipse at center, var(--accent-primary) 0%, transparent 70%)',
            opacity: 0.12,
            filter: 'blur(60px)',
            zIndex: -1,
          }}
        />

        <div
          style={{
            width: 420,
            background: 'var(--glass-bg)',
            backdropFilter: 'var(--glass-backdrop)',
            WebkitBackdropFilter: 'var(--glass-backdrop)',
            border: '1px solid var(--border-primary)',
            borderRadius: 'var(--radius-xl)',
            padding: '40px 36px',
            boxShadow: 'var(--card)',
          }}
        >
          {/* Mascot Avatar */}
          <motion.div
            className="flex justify-center"
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, ease: easeOut, delay: 0.3 }}
          >
            <motion.img
              src="./mascot.png"
              alt="云熙智能体 Logo"
              className="animate-float"
              style={{
                width: 96,
                height: 96,
                borderRadius: 'var(--radius-full)',
                border: '3px solid var(--bg-elevated)',
                boxShadow: '0 4px 16px rgba(0,0,0,0.1)',
                marginBottom: 20,
                objectFit: 'cover',
              }}
            />
          </motion.div>

          {/* App Name */}
          <motion.h1
            className="text-center font-inter"
            style={{
              fontSize: 28,
              fontWeight: 600,
              letterSpacing: '-0.02em',
              lineHeight: 1.2,
              color: 'var(--text-primary)',
              marginBottom: 8,
            }}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.3, ease: 'easeOut', delay: 0.4 }}
          >
            云熙智能体
          </motion.h1>

          {/* Subtitle */}
          <motion.p
            className="text-center"
            style={{
              fontSize: 13,
              color: 'var(--text-secondary)',
              lineHeight: 1.5,
              marginBottom: 32,
            }}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.3, ease: 'easeOut', delay: 0.5 }}
          >
            YunXi Agent — 专业专利智能助手
          </motion.p>

          {/* Divider */}
          <div className="relative flex items-center" style={{ margin: '24px 0' }}>
            <div
              className="flex-1"
              style={{ height: 1, backgroundColor: 'var(--border-primary)' }}
            />
            <span
              className="shrink-0"
              style={{
                padding: '0 12px',
                fontSize: 11,
                fontWeight: 500,
                letterSpacing: '0.01em',
                color: 'var(--text-tertiary)',
                backgroundColor: 'var(--bg-elevated)',
              }}
            >
              选择登录方式
            </span>
            <div
              className="flex-1"
              style={{ height: 1, backgroundColor: 'var(--border-primary)' }}
            />
          </div>

          {/* Login Buttons */}
          <motion.div
            variants={containerVariants}
            initial="hidden"
            animate="visible"
            className="flex flex-col"
            style={{ gap: 12 }}
          >
            {/* DeepSeek OAuth */}
            <motion.button
              variants={itemVariants}
              className="w-full flex items-center justify-center font-inter"
              style={{
                height: 44,
                backgroundColor: 'var(--accent-primary)',
                color: 'var(--text-inverse)',
                fontSize: 13,
                fontWeight: 500,
                borderRadius: 'var(--radius-md)',
                border: 'none',
                cursor: form.isLoading ? 'not-allowed' : 'pointer',
                opacity: form.isLoading ? 0.6 : 1,
                gap: 8,
                transition: 'transform 0.2s ease, background-color 0.15s ease, box-shadow 0.2s ease',
              }}
              onClick={handleDeepSeekLogin}
              disabled={form.isLoading}
              whileHover={!form.isLoading ? { y: -1, boxShadow: '0 4px 12px rgba(74, 124, 111, 0.3)' } : {}}
              whileTap={!form.isLoading ? { y: 0 } : {}}
            >
              {form.isLoading ? (
                <Loader2 size={16} className="animate-spin" />
              ) : (
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <path
                    d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"
                    fill="currentColor"
                  />
                </svg>
              )}
              {form.isLoading ? '正在连接...' : '使用 DeepSeek 登录'}
            </motion.button>

            {/* API Key */}
            <motion.button
              variants={itemVariants}
              className="w-full flex items-center justify-center font-inter"
              style={{
                height: 44,
                backgroundColor: 'var(--bg-surface)',
                color: 'var(--text-primary)',
                fontSize: 13,
                fontWeight: 500,
                borderRadius: 'var(--radius-md)',
                border: '1px solid var(--border-primary)',
                cursor: 'pointer',
                gap: 8,
                transition: 'border-color 0.2s ease, background-color 0.15s ease',
              }}
              onClick={() => toggleMode('apikey')}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = 'var(--border-focus)';
                e.currentTarget.style.backgroundColor = 'var(--bg-elevated)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = 'var(--border-primary)';
                e.currentTarget.style.backgroundColor = 'var(--bg-surface)';
              }}
              type="button"
            >
              <Key size={16} />
              使用 API Key 登录
            </motion.button>

            {/* API Key Input Panel */}
            <AnimatePresence>
              {mode === 'apikey' && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: 'auto', opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.25, ease: 'easeOut' }}
                  className="overflow-hidden"
                >
                  <div
                    className="flex flex-col"
                    style={{ gap: 8, paddingTop: 4, paddingBottom: 4 }}
                  >
                    <div className="relative">
                      <input
                        type={form.showKey ? 'text' : 'password'}
                        value={form.apiKey}
                        onChange={(e) =>
                          setForm((prev) => ({ ...prev, apiKey: e.target.value, error: '' }))
                        }
                        onKeyDown={(e) => e.key === 'Enter' && handleApiKeySubmit()}
                        placeholder="输入你的 DeepSeek API Key..."
                        className="w-full font-mono"
                        style={{
                          height: 40,
                          backgroundColor: 'var(--bg-surface)',
                          border: `1px solid ${form.error ? 'var(--status-error)' : 'var(--border-primary)'}`,
                          borderRadius: 'var(--radius-md)',
                          padding: '10px 36px 10px 14px',
                          fontSize: 12,
                          color: 'var(--text-primary)',
                          outline: 'none',
                          transition: 'border-color 0.2s ease, box-shadow 0.15s ease-in-out',
                        }}
                        onFocus={(e) => {
                          if (!form.error) {
                            e.currentTarget.style.borderColor = 'var(--border-focus)';
                            e.currentTarget.style.boxShadow = '0 0 0 3px var(--accent-primary-muted)';
                          }
                        }}
                        onBlur={(e) => {
                          e.currentTarget.style.borderColor = form.error
                            ? 'var(--status-error)'
                            : 'var(--border-primary)';
                          e.currentTarget.style.boxShadow = 'none';
                        }}
                      />
                      <button
                        className="absolute flex items-center justify-center"
                        style={{
                          right: 8,
                          top: '50%',
                          transform: 'translateY(-50%)',
                          width: 24,
                          height: 24,
                          color: 'var(--text-tertiary)',
                          background: 'none',
                          border: 'none',
                          cursor: 'pointer',
                        }}
                        onClick={() =>
                          setForm((prev) => ({ ...prev, showKey: !prev.showKey }))
                        }
                        type="button"
                      >
                        {form.showKey ? <EyeOff size={14} /> : <Eye size={14} />}
                      </button>
                    </div>

                    {form.error && (
                      <motion.p
                        style={{
                          fontSize: 11,
                          color: 'var(--status-error)',
                          margin: 0,
                          paddingLeft: 2,
                        }}
                        initial={{ opacity: 0, x: -4 }}
                        animate={{ opacity: 1, x: 0 }}
                        transition={{ duration: 0.15 }}
                      >
                        {form.error}
                      </motion.p>
                    )}

                    {form.apiKey.trim().length > 0 && (
                      <motion.button
                        initial={{ opacity: 0, y: 4 }}
                        animate={{ opacity: 1, y: 0 }}
                        className="self-end font-inter"
                        style={{
                          height: 32,
                          padding: '0 16px',
                          backgroundColor: 'var(--accent-primary)',
                          color: 'var(--text-inverse)',
                          fontSize: 12,
                          fontWeight: 500,
                          borderRadius: 'var(--radius-md)',
                          border: 'none',
                          cursor: form.isLoading ? 'not-allowed' : 'pointer',
                          opacity: form.isLoading ? 0.6 : 1,
                          transition: 'background-color 0.15s ease',
                        }}
                        onClick={handleApiKeySubmit}
                        disabled={form.isLoading}
                        whileHover={!form.isLoading ? { backgroundColor: 'var(--accent-primary-hover)' } : {}}
                        type="button"
                      >
                        {form.isLoading ? (
                          <span className="flex items-center" style={{ gap: 6 }}>
                            <Loader2 size={12} className="animate-spin" />
                            连接中
                          </span>
                        ) : (
                          '连接'
                        )}
                      </motion.button>
                    )}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Local Mode */}
            <motion.button
              variants={itemVariants}
              className="w-full flex items-center justify-center font-inter"
              style={{
                height: 44,
                backgroundColor: 'transparent',
                color: 'var(--text-secondary)',
                fontSize: 12,
                fontWeight: 400,
                borderRadius: 'var(--radius-md)',
                border: '1px dashed var(--border-primary)',
                cursor: 'pointer',
                letterSpacing: '0.005em',
                transition: 'border-color 0.2s ease, color 0.15s ease',
              }}
              onClick={() => toggleMode('local')}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = 'var(--text-tertiary)';
                e.currentTarget.style.color = 'var(--text-primary)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = 'var(--border-primary)';
                e.currentTarget.style.color = 'var(--text-secondary)';
              }}
              type="button"
            >
              本地模式（无需登录）
            </motion.button>

            {/* Local Mode Info Panel */}
            <AnimatePresence>
              {mode === 'local' && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: 'auto', opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.25, ease: 'easeOut' }}
                  className="overflow-hidden"
                >
                  <div
                    className="flex flex-col"
                    style={{
                      gap: 12,
                      padding: 16,
                      backgroundColor: 'var(--bg-sidebar-active)',
                      borderRadius: 'var(--radius-md)',
                    }}
                  >
                    <p
                      style={{
                        fontSize: 12,
                        color: 'var(--text-secondary)',
                        lineHeight: 1.5,
                        margin: 0,
                      }}
                    >
                      本地模式下，部分功能受限。文档处理完全在本地进行，不会上传数据。
                    </p>
                    <button
                      className="self-stretch font-inter flex items-center justify-center"
                      style={{
                        height: 36,
                        backgroundColor: 'var(--bg-surface)',
                        color: 'var(--text-primary)',
                        fontSize: 12,
                        fontWeight: 500,
                        borderRadius: 'var(--radius-md)',
                        border: '1px solid var(--border-primary)',
                        cursor: form.isLoading ? 'not-allowed' : 'pointer',
                        opacity: form.isLoading ? 0.6 : 1,
                        gap: 6,
                        transition: 'border-color 0.2s ease, background-color 0.15s ease',
                      }}
                      onClick={handleLocalMode}
                      disabled={form.isLoading}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.borderColor = 'var(--border-focus)';
                        e.currentTarget.style.backgroundColor = 'var(--bg-elevated)';
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.borderColor = 'var(--border-primary)';
                        e.currentTarget.style.backgroundColor = 'var(--bg-surface)';
                      }}
                      type="button"
                    >
                      {form.isLoading ? (
                        <Loader2 size={14} className="animate-spin" />
                      ) : null}
                      进入本地模式
                    </button>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        </div>
      </motion.div>

      {/* Footer */}
      <motion.div
        className="absolute flex flex-col items-center"
        style={{
          bottom: 24,
          left: '50%',
          transform: 'translateX(-50%)',
          zIndex: 10,
          gap: 8,
        }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1, duration: 0.4 }}
      >
        <span
          style={{
            fontSize: 11,
            fontWeight: 500,
            letterSpacing: '0.01em',
            color: 'var(--text-tertiary)',
          }}
        >
          v2.1.0 · macOS · DeepSeek-V3
        </span>
        <div className="flex items-center" style={{ gap: 12 }}>
          {['隐私政策', '服务条款', '帮助'].map((text) => (
            <a
              key={text}
              href="#"
              className="no-underline"
              style={{
                fontSize: 11,
                fontWeight: 500,
                color: 'var(--text-tertiary)',
                transition: 'color 0.15s ease',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.color = 'var(--text-secondary)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.color = 'var(--text-tertiary)';
              }}
            >
              {text}
            </a>
          ))}
        </div>
      </motion.div>
    </div>
  );
};

export default Login;
