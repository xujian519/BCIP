import { useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '@/hooks/useAppStore';
import { openOAuthAuthorizationUrl } from '@/lib/mcpOAuth';
import { Lock, Loader2 } from 'lucide-react';
import { easePanel } from '@/lib/animations';

export default function OAuthWaitingSheet() {
  const { state, dispatch } = useAppStore();
  const oauthState = state.oAuthWaiting;
  const isOpen = oauthState !== null;
  const phase = oauthState?.phase ?? 'idle';

  useEffect(() => {
    if (phase === 'completed') {
      const timer = setTimeout(() => {
        dispatch({ type: 'SET_OAUTH_WAITING', payload: null });
      }, 1500);
      return () => clearTimeout(timer);
    }
  }, [phase, dispatch]);

  const handleOpenBrowser = () => {
    openOAuthAuthorizationUrl(oauthState?.authUrl);
    if (oauthState) {
      dispatch({
        type: 'SET_OAUTH_WAITING',
        payload: { ...oauthState, phase: 'waiting' },
      });
    }
  };

  const handleCancel = useCallback(() => {
    dispatch({ type: 'SET_OAUTH_WAITING', payload: null });
  }, [dispatch]);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleCancel();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, handleCancel]);

  const serverName = oauthState?.serverName ?? 'MCP 服务器';

  return (
    <AnimatePresence>
      {isOpen && oauthState && (
        <div className="fixed inset-0 z-50 flex items-end justify-center sm:items-center">
          <motion.div
            className="absolute inset-0 bg-[rgba(0,0,0,0.5)]"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            onClick={handleCancel}
          />

          <motion.div
            className="relative z-10 w-full sm:w-[400px] sm:rounded-xl bg-[#242424] shadow-[0_16px_48px_rgba(0,0,0,0.35)] rounded-t-xl"
            initial={{ opacity: 0, y: 60 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 60 }}
            transition={{ duration: 0.3, ease: easePanel }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="p-7">
              <div className="flex items-center gap-2 mb-3">
                <Lock size={20} className="text-[#4A7C6F]" />
                <h2 className="text-lg font-semibold text-[#E8E8E8]">需要认证</h2>
              </div>

              <p className="text-sm text-[#9CA3AF] mb-5 leading-relaxed">
                <span className="font-mono text-[#4A7C6F]">{serverName}</span> 需要 OAuth 认证。
              </p>

              <div className="space-y-2 mb-6">
                <div className="flex items-center gap-3">
                  <span className="text-[13px] font-semibold text-[#4A7C6F] w-5">1</span>
                  <span className="text-[13px] text-[#E8E8E8]">点击「打开浏览器」</span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-[13px] font-semibold text-[#4A7C6F] w-5">2</span>
                  <span className="text-[13px] text-[#E8E8E8]">在浏览器中完成认证</span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-[13px] font-semibold text-[#4A7C6F] w-5">3</span>
                  <span className="text-[13px] text-[#E8E8E8]">返回应用，将自动收到完成通知</span>
                </div>
              </div>

              {phase === 'idle' && (
                <motion.button
                  type="button"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  onClick={handleOpenBrowser}
                  disabled={!oauthState.authUrl}
                  className="w-full h-10 bg-[#4A7C6F] hover:bg-[#5A9A8C] text-white text-sm font-medium rounded-lg transition-colors duration-150 mb-3 disabled:opacity-50"
                >
                  打开浏览器
                </motion.button>
              )}

              {phase === 'waiting' && (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="flex items-center justify-center gap-2 h-10 mb-3"
                >
                  <Loader2 size={16} className="animate-spin text-[#4A7C6F]" />
                  <span className="text-sm text-[#9CA3AF]">等待认证完成...</span>
                </motion.div>
              )}

              {phase === 'completed' && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.95 }}
                  animate={{ opacity: 1, scale: 1 }}
                  className="flex items-center justify-center h-10 mb-3"
                >
                  <span className="text-sm font-medium text-[#4A7C6F]">认证成功</span>
                </motion.div>
              )}

              {phase === 'failed' && (
                <p className="text-sm text-red-400 mb-3">
                  {oauthState.error ?? '认证失败'}
                </p>
              )}

              {phase !== 'completed' && (
                <button
                  type="button"
                  onClick={handleCancel}
                  className="w-full h-9 text-[13px] text-[#6B7280] hover:text-[#E8E8E8] transition-colors duration-150"
                >
                  取消
                </button>
              )}
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
