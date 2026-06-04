import type { Model } from '@/generated/app-server/v2/Model';
import type { ReasoningEffort } from '@/generated/app-server/ReasoningEffort';
import type { ReasoningEffortOption } from '@/generated/app-server/v2/ReasoningEffortOption';

/** LocalProxy / BCIP 常用模型（model/list 为空时的桌面端回退） */
const BCIP_MODEL_ENTRIES: Array<{
  model: string;
  displayName: string;
  description: string;
}> = [
  {
    model: 'glm-5.1',
    displayName: 'GLM 5.1',
    description: '经本地 LiteLLM 代理 (8788)，推荐默认',
  },
  {
    model: 'kimi-for-coding',
    displayName: 'Kimi 编程',
    description: '经本地 LiteLLM 代理 (8788)',
  },
  {
    model: 'deepseek-chat',
    displayName: 'DeepSeek Chat',
    description: '经本地代理或直连 DeepSeek',
  },
  {
    model: 'glm-4-flash',
    displayName: 'GLM-4 Flash',
    description: '智谱 GLM 快速模型',
  },
];

export const BCIP_MODEL_PROVIDERS: Array<{ id: string; name: string }> = [
  { id: 'LocalProxy', name: '本地代理 (8788)' },
  { id: 'DeepSeek', name: 'DeepSeek 直连' },
  { id: 'ZhiPu', name: '智谱 GLM' },
  { id: 'Kimi', name: 'Kimi' },
];

export const STANDARD_REASONING_EFFORTS: ReasoningEffort[] = [
  'none',
  'minimal',
  'low',
  'medium',
  'high',
  'xhigh',
];

export const STANDARD_REASONING_OPTIONS: ReasoningEffortOption[] =
  STANDARD_REASONING_EFFORTS.map((effort) => ({
    reasoningEffort: effort,
    description: effort,
  }));

function toPickerModel(entry: {
  model: string;
  displayName: string;
  description: string;
}): Model {
  return {
    id: entry.model,
    model: entry.model,
    upgrade: null,
    upgradeInfo: null,
    availabilityNux: null,
    displayName: entry.displayName,
    description: entry.description,
    hidden: false,
    supportedReasoningEfforts: STANDARD_REASONING_OPTIONS,
    defaultReasoningEffort: 'medium',
    inputModalities: ['text'],
    supportsPersonality: false,
    additionalSpeedTiers: [],
    serviceTiers: [],
    defaultServiceTier: null,
    isDefault: entry.model === 'glm-5.1',
  };
}

/** 合并 app-server 目录与 BCIP 预设；catalog 为空时仍可改模型。 */
export function mergeModelCatalog(
  catalog: Model[],
  configModel: string | null | undefined,
): Model[] {
  if (catalog.length > 0) {
    return catalog;
  }

  const byModel = new Map<string, Model>();
  for (const entry of BCIP_MODEL_ENTRIES) {
    byModel.set(entry.model, toPickerModel(entry));
  }

  const trimmed = configModel?.trim();
  if (trimmed && !byModel.has(trimmed)) {
    byModel.set(
      trimmed,
      toPickerModel({
        model: trimmed,
        displayName: trimmed,
        description: '当前 config.toml 中的模型',
      }),
    );
  }

  return [...byModel.values()];
}

export function reasoningOptionsForModel(
  selected: Model | undefined,
): ReasoningEffortOption[] {
  const fromCatalog = selected?.supportedReasoningEfforts ?? [];
  if (fromCatalog.length > 0) {
    return fromCatalog;
  }
  return STANDARD_REASONING_OPTIONS;
}
