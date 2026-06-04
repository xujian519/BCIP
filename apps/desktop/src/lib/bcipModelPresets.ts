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
    description: '智谱编程 API，仅文本（model_provider = ZhiPu）',
  },
  {
    model: 'glm-4.6v',
    displayName: 'GLM 4.6V',
    description: '智谱多模态：图像+文本+工具（model_provider = ZhiPuVLM）',
  },
  {
    model: 'glm-4.6v-flash',
    displayName: 'GLM 4.6V Flash',
    description: '智谱轻量多模态（model_provider = ZhiPuVLM）',
  },
  {
    model: 'glm-4.7',
    displayName: 'GLM 4.7',
    description: '智谱编程文本（model_provider = ZhiPu）',
  },
  {
    model: 'glm-4.7-flash',
    displayName: 'GLM 4.7 Flash',
    description: '智谱快速文本（model_provider = ZhiPu）',
  },
  {
    model: 'kimi-for-coding',
    displayName: 'Kimi 编程',
    description: 'Kimi 直连（config: model_provider = Kimi）',
  },
  {
    model: 'deepseek-v4-pro',
    displayName: 'DeepSeek V4 Pro',
    description: 'DeepSeek 直连（config: model_provider = DeepSeek）',
  },
  {
    model: 'deepseek-chat',
    displayName: 'DeepSeek Chat',
    description: 'DeepSeek 直连',
  },
  {
    model: 'glm-4-flash',
    displayName: 'GLM-4 Flash',
    description: '智谱 GLM 快速模型',
  },
];

export const BCIP_MODEL_PROVIDERS: Array<{ id: string; name: string }> = [
  { id: 'DeepSeek', name: 'DeepSeek 直连' },
  { id: 'ZhiPu', name: '智谱 GLM 编程' },
  { id: 'ZhiPuVLM', name: '智谱 GLM 视觉（多模态）' },
  { id: 'Kimi', name: 'Kimi 直连' },
  { id: 'KimiDirect', name: 'Kimi Moonshot 直连' },
  { id: 'LocalProxy', name: '本地代理 (8788，可选)' },
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
