import { useState, useCallback, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '@/hooks/useAppStore';
import { getAppServerClient } from '@/lib/appServerClient';
import { buildElicitationContent } from '@/lib/mcpElicitationFromRpc';
import { openOAuthAuthorizationUrl } from '@/lib/mcpOAuth';
import type { McpServerElicitationRequestResponse } from '@/generated/app-server/v2/McpServerElicitationRequestResponse';
import { Plug, ExternalLink } from 'lucide-react';
import { easePanel } from '@/lib/animations';

interface FormField {
  id: string;
  label: string;
  type: 'text' | 'password' | 'select';
  description?: string;
  options?: string[];
  value: string;
}

export default function McpElicitationModal() {
  const { state, dispatch } = useAppStore();
  const elicitation = state.mcpElicitation;
  const isOpen = elicitation !== null;

  const [fields, setFields] = useState<FormField[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [prevElicitation, setPrevElicitation] = useState(elicitation);

  if (elicitation !== prevElicitation) {
    if (elicitation?.mode === 'form' && elicitation.fields.length > 0) {
      setFields(
        elicitation.fields.map((f) => ({
          ...f,
          value: f.value ?? (f.type === 'select' ? f.options?.[0] ?? '' : ''),
        })),
      );
    } else {
      setFields([]);
    }
    setError(null);
    setPrevElicitation(elicitation);
  }

  const close = useCallback(() => {
    dispatch({ type: 'SET_MCP_ELICITATION', payload: null });
    setFields([]);
    setError(null);
    setSubmitting(false);
  }, [dispatch]);

  const respond = useCallback(
    async (response: McpServerElicitationRequestResponse) => {
      if (!elicitation?.rpcId) {
        close();
        return;
      }
      setSubmitting(true);
      setError(null);
      try {
        await getAppServerClient().respond(elicitation.rpcId, response);
        close();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setSubmitting(false);
      }
    },
    [elicitation, close],
  );

  const handleSubmitForm = () => {
    if (!elicitation) {
      return;
    }
    const values: Record<string, string> = {};
    for (const f of fields) {
      values[f.id] = f.value;
    }
    void respond({
      action: 'accept',
      content: buildElicitationContent(elicitation.fields, values),
      _meta: null,
    });
  };

  const handleUrlComplete = () => {
    void respond({ action: 'accept', content: {}, _meta: null });
  };

  const handleOpenUrl = () => {
    if (elicitation?.url) {
      openOAuthAuthorizationUrl(elicitation.url);
    }
  };

  const handleDecline = () => {
    void respond({ action: 'decline', content: null, _meta: null });
  };

  const handleCancel = useCallback(() => {
    void respond({ action: 'cancel', content: null, _meta: null });
  }, [respond]);

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

  const serverName = elicitation?.serverName ?? 'MCP 服务器';
  const isUrlMode = elicitation?.mode === 'url';

  return (
    <AnimatePresence>
      {isOpen && elicitation && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <motion.div
            className="absolute inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-[4px]"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2, ease: 'easeOut' }}
            onClick={handleCancel}
          />

          <motion.div
            className="relative z-10 w-full max-w-[420px] mx-4 bg-[#242424] rounded-xl shadow-[0_16px_48px_rgba(0,0,0,0.35)]"
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            transition={{ duration: 0.2, ease: easePanel }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="p-6">
              <div className="flex items-center gap-2 mb-2">
                <Plug size={20} className="text-[#4A7C6F]" />
                <h2 className="text-lg font-semibold text-[#E8E8E8]">MCP 服务器请求</h2>
              </div>

              <p className="text-sm text-[#9CA3AF] mb-4">
                <span className="font-mono text-[#4A7C6F]">{serverName}</span>
                {isUrlMode ? ' 需要你在浏览器中完成操作：' : ' 需要以下信息：'}
              </p>

              {elicitation.message && (
                <p className="text-[13px] text-[#E8E8E8] mb-4 leading-relaxed">
                  {elicitation.message}
                </p>
              )}

              {isUrlMode ? (
                <div className="space-y-3 mb-6">
                  <button
                    type="button"
                    onClick={handleOpenUrl}
                    className="w-full h-10 flex items-center justify-center gap-2 bg-[#4A7C6F] hover:bg-[#5A9A8C] text-white text-sm font-medium rounded-lg transition-colors duration-150"
                  >
                    <ExternalLink size={16} />
                    在浏览器中打开
                  </button>
                  <p className="text-[11px] text-[#4B5563] text-center">
                    完成后点击下方「已完成」
                  </p>
                </div>
              ) : (
                <div className="space-y-4 mb-6">
                  {fields.map((field, index) => (
                    <motion.div
                      key={field.id}
                      initial={{ opacity: 0, y: 8 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ delay: index * 0.05, duration: 0.15, ease: easePanel }}
                    >
                      <label className="block text-[13px] font-medium text-[#E8E8E8] mb-1.5">
                        {field.label}
                      </label>
                      {field.type === 'select' ? (
                        <select
                          value={field.value}
                          onChange={(e) =>
                            setFields((prev) =>
                              prev.map((f) =>
                                f.id === field.id ? { ...f, value: e.target.value } : f,
                              ),
                            )
                          }
                          className="w-full h-9 px-3 bg-[#1A1A1A] border border-[rgba(255,255,255,0.1)] rounded-md text-sm text-[#E8E8E8] focus:outline-none focus:border-[#4A7C6F] transition-colors duration-150"
                        >
                          {field.options?.map((opt) => (
                            <option key={opt} value={opt}>
                              {opt}
                            </option>
                          ))}
                        </select>
                      ) : (
                        <input
                          type={field.type}
                          value={field.value}
                          onChange={(e) =>
                            setFields((prev) =>
                              prev.map((f) =>
                                f.id === field.id ? { ...f, value: e.target.value } : f,
                              ),
                            )
                          }
                          placeholder={field.description}
                          className="w-full h-9 px-3 bg-[#1A1A1A] border border-[rgba(255,255,255,0.1)] rounded-md text-sm text-[#E8E8E8] placeholder:text-[#4B5563] focus:outline-none focus:border-[#4A7C6F] focus:shadow-[0_0_0_3px_rgba(74,124,111,0.15)] transition-all duration-150 font-mono"
                        />
                      )}
                      {field.description && field.type !== 'text' && (
                        <p className="text-[11px] text-[#4B5563] mt-1">{field.description}</p>
                      )}
                    </motion.div>
                  ))}
                </div>
              )}

              {error && (
                <p className="text-sm text-red-400 mb-3">{error}</p>
              )}

              <div className="flex items-center justify-end gap-2">
                <button
                  type="button"
                  onClick={handleDecline}
                  disabled={submitting}
                  className="h-9 px-4 text-[13px] font-medium text-[#9CA3AF] hover:text-[#E8E8E8] hover:bg-[rgba(255,255,255,0.06)] rounded-lg transition-all duration-150 disabled:opacity-50"
                >
                  拒绝
                </button>
                {isUrlMode ? (
                  <button
                    type="button"
                    onClick={handleUrlComplete}
                    disabled={submitting}
                    className="h-9 px-4 text-[13px] font-medium text-white bg-[#4A7C6F] hover:bg-[#5A9A8C] rounded-lg transition-all duration-150 disabled:opacity-50"
                  >
                    {submitting ? '发送中...' : '已完成'}
                  </button>
                ) : (
                  <button
                    type="button"
                    onClick={handleSubmitForm}
                    disabled={submitting || fields.some((f) => !f.value.trim())}
                    className="h-9 px-4 text-[13px] font-medium text-white bg-[#4A7C6F] hover:bg-[#5A9A8C] rounded-lg transition-all duration-150 disabled:opacity-50"
                  >
                    {submitting ? '发送中...' : '确认并发送'}
                  </button>
                )}
              </div>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
