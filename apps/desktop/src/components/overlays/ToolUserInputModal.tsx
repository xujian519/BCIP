/**
 * Agent 工具提问（item/tool/requestUserInput）
 */
import { useCallback, useMemo, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '@/hooks/useAppStore';
import { sendToolUserInputResponse } from '@/lib/approvalRespond';
import type { ToolRequestUserInputAnswer } from '@/generated/app-server/v2/ToolRequestUserInputAnswer';
import type { ToolRequestUserInputResponse } from '@/generated/app-server/v2/ToolRequestUserInputResponse';
import { cn } from '@/lib/utils';
import { Loader2 } from 'lucide-react';

export default function ToolUserInputModal() {
  const { state, dispatch } = useAppStore();
  const request = state.toolUserInput;
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isOpen = request !== null;

  const reset = useCallback(() => {
    setAnswers({});
    setProcessing(false);
    setError(null);
  }, []);

  const handleClose = useCallback(() => {
    dispatch({ type: 'SET_TOOL_USER_INPUT', payload: null });
    reset();
  }, [dispatch, reset]);

  const buildResponse = useMemo((): ToolRequestUserInputResponse | null => {
    if (!request) {
      return null;
    }
    const out: Record<string, ToolRequestUserInputAnswer> = {};
    for (const q of request.questions) {
      const value = answers[q.id]?.trim();
      if (value) {
        out[q.id] = { answers: [value] };
        continue;
      }
      const first = q.options?.[0]?.label;
      if (first) {
        out[q.id] = { answers: [first] };
      }
    }
    return { answers: out };
  }, [request, answers]);

  const submit = useCallback(async () => {
    if (!request || !buildResponse) {
      return;
    }
    setProcessing(true);
    setError(null);
    try {
      await sendToolUserInputResponse(request.rpcId, buildResponse);
      handleClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setProcessing(false);
    }
  }, [request, buildResponse, handleClose]);

  if (!isOpen || !request) {
    return null;
  }

  return (
    <AnimatePresence>
      <motion.div
        className="fixed inset-0 z-[90] flex items-center justify-center bg-[var(--bg-overlay)] p-6"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        role="dialog"
        aria-modal="true"
        aria-labelledby="tool-user-input-title"
      >
        <motion.div
          className={cn(
            'w-full max-w-md rounded-xl border border-[var(--border-default)]',
            'bg-[var(--bg-elevated)] shadow-lg p-5',
          )}
          initial={{ scale: 0.96, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.96, opacity: 0 }}
        >
          <h2
            id="tool-user-input-title"
            className="text-sm font-semibold text-[var(--text-primary)] mb-3"
          >
            Agent 需要你的输入
          </h2>
          <div className="space-y-4 max-h-[50vh] overflow-y-auto">
            {request.questions.map((q) => (
              <div key={q.id}>
                <label className="block text-xs font-medium text-[var(--text-secondary)] mb-1">
                  {q.header}
                </label>
                <p className="text-2xs text-[var(--text-tertiary)] mb-2">{q.question}</p>
                {q.options && q.options.length > 0 ? (
                  <select
                    className="w-full h-9 px-2 rounded-md border border-[var(--border-default)] bg-[var(--bg-surface)] text-sm"
                    value={answers[q.id] ?? q.options[0]?.label ?? ''}
                    onChange={(e) =>
                      setAnswers((prev) => ({ ...prev, [q.id]: e.target.value }))
                    }
                  >
                    {q.options.map((opt) => (
                      <option key={opt.label} value={opt.label}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                ) : (
                  <input
                    type={q.isSecret ? 'password' : 'text'}
                    className="w-full h-9 px-2 rounded-md border border-[var(--border-default)] bg-[var(--bg-surface)] text-sm"
                    value={answers[q.id] ?? ''}
                    onChange={(e) =>
                      setAnswers((prev) => ({ ...prev, [q.id]: e.target.value }))
                    }
                  />
                )}
              </div>
            ))}
          </div>
          {error && (
            <p className="mt-3 text-2xs text-[var(--status-error)]">{error}</p>
          )}
          <div className="mt-4 flex justify-end gap-2">
            <button
              type="button"
              className="h-9 px-4 text-sm text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] rounded-lg"
              onClick={handleClose}
              disabled={processing}
            >
              取消
            </button>
            <button
              type="button"
              className="h-9 px-4 text-sm font-medium bg-brand-500 text-[var(--text-inverse)] rounded-lg disabled:opacity-50"
              onClick={() => void submit()}
              disabled={processing}
            >
              {processing ? (
                <Loader2 className="w-4 h-4 animate-spin inline" />
              ) : (
                '提交'
              )}
            </button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
