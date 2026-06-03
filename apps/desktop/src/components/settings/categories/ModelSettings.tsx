import { useState, useEffect, useCallback } from 'react';
import type { FC } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown, ChevronUp, Eye, EyeOff, Check } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import SelectSetting from '../SelectSetting';
import SliderSetting from '../SliderSetting';
import InputSetting from '../InputSetting';

const modelOptions = [
  { value: 'deepseek-v4-pro', label: 'DeepSeek-V4 Pro (推荐)' },
  { value: 'deepseek-v4-flash', label: 'DeepSeek-V4 Flash' },
  { value: 'messages-opus', label: 'Claude 3.5 Opus' },
  { value: 'messages-sonnet', label: 'Claude 3.5 Sonnet' },
  { value: 'messages-haiku', label: 'Claude 3.5 Haiku' },
  { value: 'auto', label: '自动选择 (推荐模型)' },
];

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.04 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 8 },
  show: { opacity: 1, y: 0, transition: { duration: 0.2, ease: 'easeOut' as const } },
};

const ModelSettings: FC = () => {
  const [model, setModel] = useState('deepseek-v4-pro');
  const [temperature, setTemperature] = useState(0.7);
  const [maxTokens, setMaxTokens] = useState('4096');
  const [apiKey, setApiKey] = useState('');
  const [apiBaseUrl, setApiBaseUrl] = useState('https://api.deepseek.com/v1');
  const [showApiKey, setShowApiKey] = useState(false);
  const [timeout, setTimeout_] = useState(30);
  const [apiExpanded, setApiExpanded] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveMsg, setSaveMsg] = useState('');

  // 首次加载从 config.toml 读取已保存的 api_key 和 model
  useEffect(() => {
    invoke<{ config: { api_key?: string; model?: string }; path: string }>('read_config')
      .then((res) => {
        if (res.config?.api_key) setApiKey(res.config.api_key);
        if (res.config?.model) setModel(res.config.model);
      })
      .catch(() => {
        // read_config 失败（如首次启动无配置文件）静默忽略
      });
  }, []);

  // 持久化配置到 config.toml
  const persistConfig = useCallback(
    async (updates: { api_key?: string; model?: string; model_provider?: string }) => {
      setSaving(true);
      setSaveMsg('');
      try {
        await invoke('write_config', {
          params: {
            api_key: updates.api_key ?? null,
            model: updates.model ?? null,
            model_provider: updates.model_provider ?? null,
          },
        });
        setSaveMsg('已保存');
        setTimeout(() => setSaveMsg(''), 2000);
      } catch {
        setSaveMsg('保存失败');
      } finally {
        setSaving(false);
      }
    },
    [],
  );

  const handleModelChange = (val: string) => {
    setModel(val);
    void persistConfig({ model: val });
  };

  const handleApiKeyBlur = () => {
    if (apiKey.trim()) {
      void persistConfig({ api_key: apiKey.trim() });
    }
  };

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="show"
      className="flex flex-col"
      style={{ padding: '24px 32px' }}
    >
      {/* Section Header */}
      <motion.div variants={itemVariants} className="mb-5">
        <h2
          style={{
            fontSize: 20,
            fontWeight: 600,
            color: 'var(--text-primary)',
            letterSpacing: '-0.01em',
            lineHeight: 1.4,
            marginBottom: 4,
          }}
        >
          模型设置
        </h2>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          配置 AI 模型参数和 API 连接
        </p>
      </motion.div>

      {/* Model Selector */}
      <motion.div variants={itemVariants}>
        <SelectSetting
          label="AI 模型"
          description="选择默认使用的 AI 模型"
          value={model}
          options={modelOptions}
          onChange={handleModelChange}
        />
      </motion.div>

      {/* Temperature */}
      <motion.div variants={itemVariants}>
        <SliderSetting
          label="创造性 (Temperature)"
          description="较高的值会产生更有创意的输出"
          value={temperature}
          min={0}
          max={2}
          step={0.1}
          onChange={setTemperature}
          valueFormatter={(v) => v.toFixed(1)}
        />
      </motion.div>

      {/* Max Tokens */}
      <motion.div variants={itemVariants}>
        <InputSetting
          label="最大响应长度"
          description="单次响应的最大 token 数量 (256-8192)"
          value={maxTokens}
          onChange={(v) => {
            const num = parseInt(v) || 0;
            if (num >= 256 && num <= 8192) setMaxTokens(v);
            else if (v === '') setMaxTokens('');
            else setMaxTokens(String(Math.min(Math.max(num, 256), 8192)));
          }}
          type="number"
          min={256}
          max={8192}
        />
      </motion.div>

      {/* Section Separator */}
      <motion.div
        variants={itemVariants}
        style={{
          height: 1,
          backgroundColor: 'var(--border-primary)',
          margin: '12px 0',
        }}
      />

      {/* API Configuration Accordion */}
      <motion.div variants={itemVariants}>
        <button
          onClick={() => setApiExpanded(!apiExpanded)}
          className="flex items-center justify-between w-full py-2 transition-colors"
          style={{ color: 'var(--text-primary)' }}
          type="button"
        >
          <div className="flex flex-col items-start">
            <span
              style={{
                fontSize: 13,
                fontWeight: 500,
                color: 'var(--text-primary)',
                lineHeight: 1.4,
              }}
            >
              API 配置
            </span>
            <span style={{ fontSize: 12, color: 'var(--text-secondary)' }}>
              自定义 API 密钥和连接设置
            </span>
          </div>
          {apiExpanded ? (
            <ChevronUp size={16} style={{ color: 'var(--text-tertiary)' }} />
          ) : (
            <ChevronDown size={16} style={{ color: 'var(--text-tertiary)' }} />
          )}
        </button>

        <AnimatePresence>
          {apiExpanded && (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: 'auto', opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ duration: 0.2, ease: 'easeOut' }}
              className="overflow-hidden"
            >
              <div className="flex flex-col gap-1 pt-2">
                {/* API Base URL */}
                <div className="flex flex-col gap-2 py-2">
                  <span
                    style={{
                      fontSize: 13,
                      fontWeight: 500,
                      color: 'var(--text-primary)',
                    }}
                  >
                    API Base URL
                  </span>
                  <input
                    type="text"
                    value={apiBaseUrl}
                    onChange={(e) => setApiBaseUrl(e.target.value)}
                    className="w-full px-3 py-2 transition-colors"
                    style={{
                      height: 36,
                      backgroundColor: 'var(--bg-surface)',
                      border: '1px solid var(--border-primary)',
                      borderRadius: 8,
                      fontSize: 13,
                      color: 'var(--text-primary)',
                      fontFamily: 'JetBrains Mono, monospace',
                      outline: 'none',
                    }}
                    onFocus={(e) => {
                      e.currentTarget.style.borderColor = 'var(--border-focus)';
                      e.currentTarget.style.boxShadow = '0 0 0 3px var(--accent-primary-muted)';
                    }}
                    onBlur={(e) => {
                      e.currentTarget.style.borderColor = 'var(--border-primary)';
                      e.currentTarget.style.boxShadow = 'none';
                    }}
                  />
                </div>

                {/* API Key */}
                <div className="flex flex-col gap-2 py-2">
                  <span
                    style={{
                      fontSize: 13,
                      fontWeight: 500,
                      color: 'var(--text-primary)',
                    }}
                  >
                    API Key
                  </span>
                  <div className="relative">
                    <input
                      type={showApiKey ? 'text' : 'password'}
                      value={apiKey}
                      onChange={(e) => setApiKey(e.target.value)}
                      onBlur={(e) => {
                        handleApiKeyBlur();
                        e.currentTarget.style.borderColor = 'var(--border-primary)';
                        e.currentTarget.style.boxShadow = 'none';
                      }}
                      placeholder="sk-..."
                      className="w-full px-3 py-2 pr-10 transition-colors"
                      style={{
                        height: 36,
                        backgroundColor: 'var(--bg-surface)',
                        border: '1px solid var(--border-primary)',
                        borderRadius: 8,
                        fontSize: 13,
                        color: 'var(--text-primary)',
                        fontFamily: 'JetBrains Mono, monospace',
                        outline: 'none',
                      }}
                      onFocus={(e) => {
                        e.currentTarget.style.borderColor = 'var(--border-focus)';
                        e.currentTarget.style.boxShadow = '0 0 0 3px var(--accent-primary-muted)';
                      }}
                    />
                    <button
                      onClick={() => setShowApiKey(!showApiKey)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center justify-center"
                      style={{
                        width: 28,
                        height: 28,
                        borderRadius: 6,
                        color: 'var(--text-tertiary)',
                      }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.color = 'var(--text-secondary)';
                        e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.color = 'var(--text-tertiary)';
                        e.currentTarget.style.backgroundColor = 'transparent';
                      }}
                      type="button"
                    >
                      {showApiKey ? <EyeOff size={14} /> : <Eye size={14} />}
                    </button>
                  </div>
                </div>

                {/* Timeout */}
                <div className="py-2">
                  <SliderSetting
                    label="请求超时 (秒)"
                    value={timeout}
                    min={5}
                    max={60}
                    step={1}
                    onChange={setTimeout_}
                    valueFormatter={(v) => `${v}s`}
                  />
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>

      {/* Connection Test */}
      <motion.div
        variants={itemVariants}
        style={{
          height: 1,
          backgroundColor: 'var(--border-primary)',
          margin: '12px 0',
        }}
      />
      <motion.div variants={itemVariants} className="py-3">
        <div className="flex items-center justify-between">
          <div>
            <span
              style={{
                fontSize: 13,
                fontWeight: 500,
                color: 'var(--text-primary)',
                lineHeight: 1.4,
              }}
            >
              连接测试
            </span>
            <p style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 2 }}>
              {saving ? '正在保存…' : saveMsg || '验证 API 配置是否正确'}
            </p>
          </div>
          <button
            className="flex items-center gap-2 px-4 py-2 transition-all"
            style={{
              borderRadius: 8,
              border: '1px solid var(--accent-primary)',
              backgroundColor: 'var(--accent-primary-muted)',
              color: 'var(--accent-primary)',
              fontSize: 12,
              fontWeight: 500,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--accent-primary)';
              e.currentTarget.style.color = 'var(--text-inverse)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--accent-primary-muted)';
              e.currentTarget.style.color = 'var(--accent-primary)';
            }}
            type="button"
          >
            <Check size={14} />
            测试连接
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
};

export default ModelSettings;
