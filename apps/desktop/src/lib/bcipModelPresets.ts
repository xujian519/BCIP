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
  // Qwen 系列
  {
    model: 'qwen-plus',
    displayName: 'Qwen Plus',
    description: '阿里通义千问 Plus（config: model_provider = Qwen）',
  },
  {
    model: 'qwen-max',
    displayName: 'Qwen Max',
    description: '阿里通义千问 Max（config: model_provider = Qwen）',
  },
  {
    model: 'qwen-turbo',
    displayName: 'Qwen Turbo',
    description: '阿里通义千问 Turbo（config: model_provider = Qwen）',
  },
  {
    model: 'qwen3.5-plus',
    displayName: 'Qwen 3.5 Plus',
    description: '阿里通义千问 3.5 Plus，1M 上下文（config: model_provider = Qwen）',
  },
  {
    model: 'qwen3.6-flash',
    displayName: 'Qwen 3.6 Flash',
    description: '阿里通义千问 3.6 Flash（config: model_provider = Qwen）',
  },
  // MiniMax 系列
  {
    model: 'minimax-m2.5',
    displayName: 'MiniMax M2.5',
    description: 'MiniMax M2.5（config: model_provider = MiniMax）',
  },
  {
    model: 'minimax-m2.5-lightning',
    displayName: 'MiniMax M2.5 Lightning',
    description: 'MiniMax 快速版（config: model_provider = MiniMax）',
  },
  // DeepSeek 补充
  {
    model: 'deepseek-reasoner',
    displayName: 'DeepSeek Reasoner',
    description: 'DeepSeek 深度推理（config: model_provider = DeepSeek）',
  },
  // Kimi / Moonshot 补充
  {
    model: 'kimi-k2.5',
    displayName: 'Kimi K2.5',
    description: '月之暗面 Kimi 旗舰，262K 上下文（config: model_provider = Kimi）',
  },
  {
    model: 'moonshot-v1-128k',
    displayName: 'Moonshot 128K',
    description: 'Moonshot 128K 上下文（config: model_provider = KimiDirect）',
  },
];

export const BCIP_MODEL_PROVIDERS: Array<{ id: string; name: string }> = [
  { id: 'DeepSeek', name: 'DeepSeek 直连' },
  { id: 'ZhiPu', name: '智谱 GLM 编程' },
  { id: 'ZhiPuVLM', name: '智谱 GLM 视觉（多模态）' },
  { id: 'Kimi', name: 'Kimi 直连' },
  { id: 'KimiDirect', name: 'Kimi Moonshot 直连' },
  { id: 'Qwen', name: '阿里通义千问 (Dashscope)' },
  { id: 'MiniMax', name: 'MiniMax 直连' },
  { id: 'BaiDu', name: '百度千帆' },
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
