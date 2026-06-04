import { useMemo } from 'react';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useAppStore } from '@/hooks/useAppStore';
import { useCodexConfig } from '@/hooks/useCodexConfig';
import { findModelByConfigValue, useModelCatalog } from '@/hooks/useModelCatalog';
import { isDesktopRpcReady } from '@/lib/configAccess';
import {
  BCIP_MODEL_PROVIDERS,
  mergeModelCatalog,
  reasoningOptionsForModel,
} from '@/lib/bcipModelPresets';
import type { ReasoningEffort } from '@/generated/app-server/ReasoningEffort';
import type { Verbosity } from '@/generated/app-server/Verbosity';
import {
  SettingRow,
  SettingsCard,
  SettingsRpcBanner,
} from '../SettingPrimitives';
import { settingsTheme } from '../settingsTheme';

const reasoningLabels: Record<ReasoningEffort, string> = {
  none: '无',
  minimal: '极低',
  low: '低',
  medium: '中',
  high: '高',
  xhigh: '极高',
};

const verbosityLabels: Record<Verbosity, string> = {
  low: '简洁',
  medium: '标准',
  high: '详细',
};

export default function ModelSettings() {
  const { state, dispatch } = useAppStore();
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const { config, loading, error, saving, writeValue } = useCodexConfig(
    rpcReady,
    state.workspaceCwd,
  );
  const { models: catalogModels, loading: modelsLoading, error: modelsError } =
    useModelCatalog(rpcReady);

  const displayModels = useMemo(
    () => mergeModelCatalog(catalogModels, config?.model),
    [catalogModels, config?.model],
  );

  const usingFallbackCatalog = catalogModels.length === 0 && displayModels.length > 0;

  const selectedModel = useMemo(
    () => findModelByConfigValue(displayModels, config?.model ?? null),
    [displayModels, config?.model],
  );

  const reasoningEffort =
    (config?.model_reasoning_effort as ReasoningEffort | null) ??
    selectedModel?.defaultReasoningEffort ??
    'medium';

  const modelProvider = config?.model_provider ?? 'LocalProxy';

  const serviceTier = config?.service_tier ?? selectedModel?.defaultServiceTier ?? '';
  const verbosity = (config?.model_verbosity as Verbosity | null) ?? 'medium';

  const effortOptions = reasoningOptionsForModel(selectedModel);
  const tierOptions = selectedModel?.serviceTiers ?? [];

  const handleModelChange = async (modelId: string) => {
    const entry = displayModels.find((m) => m.id === modelId);
    if (!entry) {
      return;
    }
    await writeValue('model', entry.model);
    dispatch({ type: 'SET_CURRENT_MODEL', payload: entry.displayName });
  };

  const handleProviderChange = async (providerId: string) => {
    await writeValue('model_provider', providerId);
  };

  return (
    <div>
      <h1 className={`${settingsTheme.pageTitle} mb-2`}>模型与推理</h1>
      <SettingsRpcBanner
        rpcReady={rpcReady}
        loading={loading || modelsLoading}
        error={error ?? (modelsError && !usingFallbackCatalog ? modelsError : null)}
        saving={saving}
      />

      {usingFallbackCatalog && rpcReady && (
        <p className="mb-4 text-xs text-[var(--text-secondary)]">
          未从 app-server 拉取到模型目录，已显示 BCIP 本地常用模型；修改会写入{' '}
          <span className="font-mono">~/.bcip/config.toml</span>。
        </p>
      )}

      <SettingsCard title="模型提供商">
        <SettingRow label="提供商" description="model_provider，切换后请选对应模型">
          <Select
            value={modelProvider}
            onValueChange={(id) => void handleProviderChange(id)}
            disabled={!rpcReady || saving}
          >
            <SelectTrigger className="w-[200px] h-9 bg-[var(--bg-elevated)] border-[var(--border-default)] text-[var(--text-primary)] text-xs">
              <SelectValue placeholder="选择提供商" />
            </SelectTrigger>
            <SelectContent className="bg-[var(--bg-base)] border-[var(--border-default)]">
              {BCIP_MODEL_PROVIDERS.map((p) => (
                <SelectItem key={p.id} value={p.id} className="text-xs">
                  {p.name}
                </SelectItem>
              ))}
              {!BCIP_MODEL_PROVIDERS.some((p) => p.id === modelProvider) && (
                <SelectItem value={modelProvider} className="text-xs">
                  {modelProvider}
                </SelectItem>
              )}
            </SelectContent>
          </Select>
        </SettingRow>
      </SettingsCard>

      <div className="bg-[var(--bg-base)] rounded-2xl border border-[var(--border-default)] p-3 mb-6">
        {modelsLoading && displayModels.length === 0 ? (
          <div
            className="h-10 w-full animate-pulse rounded-lg bg-[var(--bg-elevated)]"
            aria-busy="true"
            aria-label="加载模型列表"
          />
        ) : (
          <Select
            value={selectedModel?.id ?? config?.model ?? ''}
            onValueChange={(id) => void handleModelChange(id)}
            disabled={!rpcReady || displayModels.length === 0}
          >
            <SelectTrigger className="w-full h-10 bg-[var(--bg-elevated)] border-[var(--border-default)] text-[var(--text-primary)] text-sm">
              <SelectValue
                placeholder={modelsLoading ? '加载模型…' : '选择模型'}
              />
            </SelectTrigger>
            <SelectContent className="bg-[var(--bg-base)] border-[var(--border-default)] max-h-[280px]">
              {displayModels.map((m) => (
                <SelectItem
                  key={m.id}
                  value={m.id}
                  className="text-xs"
                  disabled={m.hidden}
                >
                  {m.displayName}
                  {m.isDefault ? ' · 默认' : ''}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
        {selectedModel && (
          <p className="text-xs text-[var(--text-secondary)] mt-2 px-1">
            {selectedModel.description || selectedModel.model}
          </p>
        )}
      </div>

      <SettingsCard title="推理设置">
        <SettingRow
          label="推理力度"
          description="越高推理越深，但回复越慢；日常建议「中」"
        >
          <div className="flex flex-wrap bg-[var(--bg-base)] rounded-md p-0.5 gap-0.5 max-w-[320px] justify-end">
            {effortOptions.map((opt) => (
              <button
                key={opt.reasoningEffort}
                type="button"
                disabled={!rpcReady || saving}
                onClick={() =>
                  void writeValue('model_reasoning_effort', opt.reasoningEffort)
                }
                className={`px-2 py-1 text-xs rounded-md transition-all duration-150 ${
                  reasoningEffort === opt.reasoningEffort
                    ? 'bg-[var(--bg-active)] text-[var(--text-primary)]'
                    : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                }`}
                title={opt.description}
              >
                {reasoningLabels[opt.reasoningEffort] ?? opt.reasoningEffort}
              </button>
            ))}
          </div>
        </SettingRow>
        {(reasoningEffort === 'high' || reasoningEffort === 'xhigh') && (
          <p className="text-xs text-[var(--status-warning)] -mt-2 mb-1 px-1">
            当前推理力度偏高，会显著增加等待时间。若只需快速对话，请改为「中」或「低」。
          </p>
        )}

        {tierOptions.length > 0 && (
          <SettingRow label="服务层级" description="写入 service_tier">
            <Select
              value={serviceTier || tierOptions[0]?.id}
              onValueChange={(v) => void writeValue('service_tier', v)}
              disabled={!rpcReady}
            >
              <SelectTrigger className="w-[140px] h-8 bg-[var(--bg-base)] border-[var(--border-default)] text-[var(--text-primary)] text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="bg-[var(--bg-base)] border-[var(--border-default)]">
                {tierOptions.map((tier) => (
                  <SelectItem key={tier.id} value={tier.id} className="text-xs">
                    {tier.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </SettingRow>
        )}

        <SettingRow label="输出详略" description="model_verbosity（GPT-5 等）">
          <Select
            value={verbosity}
            onValueChange={(v) =>
              void writeValue('model_verbosity', v as Verbosity)
            }
            disabled={!rpcReady}
          >
            <SelectTrigger className="w-[140px] h-8 bg-[var(--bg-base)] border-[var(--border-default)] text-[var(--text-primary)] text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent className="bg-[var(--bg-base)] border-[var(--border-default)]">
              {(['low', 'medium', 'high'] as const).map((v) => (
                <SelectItem key={v} value={v} className="text-xs">
                  {verbosityLabels[v]}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </SettingRow>
      </SettingsCard>
    </div>
  );
}
