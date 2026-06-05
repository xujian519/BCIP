//! 智能模型路由策略
//!
//! 基于任务类型、成本、延迟和历史表现自动选择最优模型：
//! - 任务复杂度匹配（简单/中等/复杂）
//! - 成本感知路由（优先低成本模型）
//! - 降级策略（主模型不可用时自动切换）
//! - 与 LearningStore 联动（基于历史成功率）

use std::collections::HashMap;

use crate::learning::GroupBy;
use crate::learning::LearningStore;

/// 模型能力声明
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelCapabilities {
    pub model_id: String,
    pub provider: String,
    /// 是否支持 function calling / tool use
    pub tool_use: bool,
    /// 是否支持视觉（图片输入）
    pub vision: bool,
    /// 是否支持代码执行
    pub code_execution: bool,
    /// 上下文窗口大小（tokens）
    pub context_window: usize,
    /// 相对成本等级 1-5（1=最便宜）
    pub cost_tier: u8,
    /// 相对速度等级 1-5（1=最快）
    pub speed_tier: u8,
    /// 适合的任务复杂度
    pub complexity_range: (u8, u8),
}

/// 任务复杂度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskComplexity {
    /// 简单任务：格式转换、简单问答、摘要
    Simple,
    /// 中等任务：分析、比较、评估
    Medium,
    /// 复杂任务：多步推理、长文档撰写、跨领域综合
    Complex,
}

/// 路由策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// 优先选择成本最低的模型
    CostOptimized,
    /// 优先选择最快的模型
    SpeedOptimized,
    /// 优先选择质量最好的模型
    QualityOptimized,
    /// 基于历史表现自动选择
    PerformanceBased,
}

/// 路由决策结果
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub recommended_model: String,
    pub recommended_provider: String,
    pub fallback_models: Vec<String>,
    pub strategy_used: RoutingStrategy,
    pub reason: String,
}

/// 智能模型路由器
pub struct ModelRouter {
    capabilities: HashMap<String, ModelCapabilities>,
    learning_store: LearningStore,
}

impl ModelRouter {
    /// 使用默认模型能力表创建路由器
    pub fn new(home_dir: &std::path::Path) -> Self {
        Self {
            capabilities: default_capabilities(),
            learning_store: LearningStore::new(home_dir),
        }
    }

    /// 注册或更新模型能力
    pub fn register_capability(&mut self, cap: ModelCapabilities) {
        self.capabilities.insert(cap.model_id.clone(), cap);
    }

    /// 根据任务描述和策略选择最优模型
    pub fn route(
        &self,
        task_description: &str,
        role: &str,
        strategy: RoutingStrategy,
    ) -> RoutingDecision {
        let complexity = estimate_complexity(task_description);

        // 1. 基于历史表现（如果有足够数据）
        if strategy == RoutingStrategy::PerformanceBased
            && let Ok(stats) = self.learning_store.get_stats(GroupBy::Role)
            && let Some(role_stats) = stats.get(role)
            && role_stats.total_calls >= 5
            && let Some(model) = self.learning_store.suggest_model(role)
        {
            return RoutingDecision {
                recommended_model: model.clone(),
                recommended_provider: self.provider_for_model(&model),
                fallback_models: self.fallbacks_for(&model, complexity),
                strategy_used: RoutingStrategy::PerformanceBased,
                reason: format!(
                    "基于角色 '{role}' 的 {} 次历史调用，推荐模型",
                    role_stats.total_calls
                ),
            };
        }

        // 2. 基于策略和能力匹配
        let candidates = self.candidates_for_complexity(complexity);
        if candidates.is_empty() {
            return RoutingDecision {
                recommended_model: "deepseek-v4-pro".to_string(),
                recommended_provider: "deepseek".to_string(),
                fallback_models: vec![],
                strategy_used: strategy,
                reason: "无匹配模型，使用默认".to_string(),
            };
        }

        let (best, reason) = match strategy {
            RoutingStrategy::CostOptimized => {
                let b = candidates.iter().min_by_key(|c| c.cost_tier).unwrap();
                (b, format!("成本优先 (tier {})", b.cost_tier))
            }
            RoutingStrategy::SpeedOptimized => {
                let b = candidates.iter().min_by_key(|c| c.speed_tier).unwrap();
                (b, format!("速度优先 (tier {})", b.speed_tier))
            }
            RoutingStrategy::QualityOptimized => {
                let b = candidates
                    .iter()
                    .max_by_key(|c| c.complexity_range.1)
                    .unwrap();
                (b, "质量优先".to_string())
            }
            RoutingStrategy::PerformanceBased => {
                // 已在上面处理，这里是 fallback
                let b = candidates.iter().min_by_key(|c| c.cost_tier).unwrap();
                (b, "数据不足，使用成本优先".to_string())
            }
        };

        RoutingDecision {
            recommended_model: best.model_id.clone(),
            recommended_provider: best.provider.clone(),
            fallback_models: self
                .candidates_for_complexity(complexity)
                .iter()
                .filter(|c| c.model_id != best.model_id)
                .map(|c| c.model_id.clone())
                .collect(),
            strategy_used: strategy,
            reason,
        }
    }

    /// 获取降级链（主模型 → 备选1 → 备选2）
    pub fn degradation_chain(&self, primary_model: &str, role: &str) -> Vec<String> {
        let mut chain = vec![primary_model.to_string()];

        // 从历史表现中找替代
        if let Some(model) = self.learning_store.suggest_model(role)
            && model != primary_model
        {
            chain.push(model);
        }

        // 从能力表中找同复杂度的替代
        if let Some(cap) = self.capabilities.get(primary_model) {
            let level = cap.complexity_range;
            for alt in self.capabilities.values() {
                if alt.model_id != primary_model
                    && alt.complexity_range.0 <= level.1
                    && alt.complexity_range.1 >= level.0
                    && !chain.contains(&alt.model_id)
                {
                    chain.push(alt.model_id.clone());
                }
                if chain.len() >= 3 {
                    break;
                }
            }
        }

        chain
    }

    // ---- 内部方法 ----

    fn candidates_for_complexity(&self, complexity: TaskComplexity) -> Vec<&ModelCapabilities> {
        let level = match complexity {
            TaskComplexity::Simple => 1,
            TaskComplexity::Medium => 2,
            TaskComplexity::Complex => 3,
        };
        self.capabilities
            .values()
            .filter(|c| c.complexity_range.0 <= level as u8 && c.complexity_range.1 >= level as u8)
            .collect()
    }

    fn provider_for_model(&self, model: &str) -> String {
        self.capabilities
            .get(model)
            .map(|c| c.provider.clone())
            .unwrap_or_else(|| {
                use crate::provider_router::registry::model_to_provider_name;
                model_to_provider_name(model).to_string()
            })
    }

    fn fallbacks_for(&self, model: &str, complexity: TaskComplexity) -> Vec<String> {
        self.candidates_for_complexity(complexity)
            .iter()
            .filter(|c| c.model_id != model)
            .map(|c| c.model_id.clone())
            .collect()
    }
}

/// 从任务描述估算复杂度
fn estimate_complexity(description: &str) -> TaskComplexity {
    let desc = description.to_lowercase();

    // 复杂任务关键词
    let complex_keywords = [
        "综合分析",
        "多步推理",
        "跨领域",
        "专利撰写",
        "全面评估",
        "对比分析",
        "战略",
        "规划",
        "架构设计",
    ];
    let complex_count = complex_keywords
        .iter()
        .filter(|k| desc.contains(*k))
        .count();

    // 中等任务关键词
    let medium_keywords = [
        "分析", "评估", "比较", "检索", "审查", "检查", "摘要", "翻译", "改写",
    ];
    let medium_count = medium_keywords.iter().filter(|k| desc.contains(*k)).count();

    // 简单任务关键词
    let simple_keywords = ["格式化", "转换", "复制", "简单", "列表", "提取"];
    let simple_count = simple_keywords.iter().filter(|k| desc.contains(*k)).count();

    if complex_count > 0 || desc.len() > 500 {
        TaskComplexity::Complex
    } else if medium_count > simple_count {
        TaskComplexity::Medium
    } else if simple_count > 0 && medium_count == 0 {
        TaskComplexity::Simple
    } else if desc.len() > 100 {
        TaskComplexity::Medium
    } else {
        TaskComplexity::Simple
    }
}

/// 默认模型能力表
fn default_capabilities() -> HashMap<String, ModelCapabilities> {
    let entries = vec![
        // DeepSeek 系列
        ModelCapabilities {
            model_id: "deepseek-v4-pro".to_string(),
            provider: "deepseek".to_string(),
            tool_use: true,
            vision: false,
            code_execution: true,
            context_window: 128_000,
            cost_tier: 2,
            speed_tier: 3,
            complexity_range: (1, 3),
        },
        ModelCapabilities {
            model_id: "deepseek-v4-chat".to_string(),
            provider: "deepseek".to_string(),
            tool_use: true,
            vision: false,
            code_execution: true,
            context_window: 64_000,
            cost_tier: 1,
            speed_tier: 2,
            complexity_range: (1, 2),
        },
        // Qwen 系列
        ModelCapabilities {
            model_id: "qwen-max".to_string(),
            provider: "qwen".to_string(),
            tool_use: true,
            vision: true,
            code_execution: false,
            context_window: 32_000,
            cost_tier: 3,
            speed_tier: 3,
            complexity_range: (2, 3),
        },
        ModelCapabilities {
            model_id: "qwen-turbo".to_string(),
            provider: "qwen".to_string(),
            tool_use: true,
            vision: false,
            code_execution: false,
            context_window: 128_000,
            cost_tier: 1,
            speed_tier: 1,
            complexity_range: (1, 2),
        },
        // Claude 系列
        ModelCapabilities {
            model_id: "claude-sonnet-4-6".to_string(),
            provider: "anthropic".to_string(),
            tool_use: true,
            vision: true,
            code_execution: false,
            context_window: 200_000,
            cost_tier: 4,
            speed_tier: 4,
            complexity_range: (2, 3),
        },
        ModelCapabilities {
            model_id: "claude-haiku-4-5".to_string(),
            provider: "anthropic".to_string(),
            tool_use: true,
            vision: true,
            code_execution: false,
            context_window: 200_000,
            cost_tier: 2,
            speed_tier: 1,
            complexity_range: (1, 2),
        },
        // OpenAI 系列
        ModelCapabilities {
            model_id: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            tool_use: true,
            vision: true,
            code_execution: false,
            context_window: 128_000,
            cost_tier: 5,
            speed_tier: 3,
            complexity_range: (2, 3),
        },
        // GLM 系列
        ModelCapabilities {
            model_id: "glm-4-plus".to_string(),
            provider: "glm".to_string(),
            tool_use: true,
            vision: true,
            code_execution: false,
            context_window: 128_000,
            cost_tier: 2,
            speed_tier: 2,
            complexity_range: (1, 3),
        },
    ];

    entries
        .into_iter()
        .map(|cap| (cap.model_id.clone(), cap))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, ModelRouter) {
        let dir = tempfile::TempDir::new().unwrap();
        let router = ModelRouter::new(dir.path());
        (dir, router)
    }

    #[test]
    fn test_estimate_complexity_simple() {
        assert_eq!(
            estimate_complexity("简单列表提取格式化"),
            TaskComplexity::Simple
        );
    }

    #[test]
    fn test_estimate_complexity_complex() {
        assert_eq!(
            estimate_complexity("请对这篇专利进行全面评估和综合分析"),
            TaskComplexity::Complex
        );
    }

    #[test]
    fn test_estimate_complexity_medium() {
        assert_eq!(
            estimate_complexity("分析这个专利权利要求的新颖性"),
            TaskComplexity::Medium
        );
    }

    #[test]
    fn test_route_cost_optimized() {
        let (_dir, router) = setup();
        let decision = router.route("简单格式化", "writer", RoutingStrategy::CostOptimized);
        assert!(decision.reason.contains("成本优先"));
        // 最低 cost_tier 的模型
        assert!(
            decision.recommended_model.contains("turbo")
                || decision.recommended_model.contains("deepseek-v4-chat")
                || decision.recommended_model.contains("haiku")
                || decision.recommended_model.contains("glm")
        );
    }

    #[test]
    fn test_route_speed_optimized() {
        let (_dir, router) = setup();
        let decision = router.route("快速检索", "retriever", RoutingStrategy::SpeedOptimized);
        assert!(decision.reason.contains("速度优先"));
    }

    #[test]
    fn test_route_quality_optimized() {
        let (_dir, router) = setup();
        let decision = router.route(
            "综合分析专利的创造性",
            "analyzer",
            RoutingStrategy::QualityOptimized,
        );
        assert!(decision.reason.contains("质量优先"));
    }

    #[test]
    fn test_degradation_chain() {
        let (_dir, router) = setup();
        let chain = router.degradation_chain("deepseek-v4-pro", "analyzer");
        assert!(!chain.is_empty());
        assert_eq!(chain[0], "deepseek-v4-pro");
    }

    #[test]
    fn test_default_capabilities_loaded() {
        let (_dir, router) = setup();
        assert!(router.capabilities.len() >= 7);
    }

    #[test]
    fn test_register_custom_capability() {
        let (_dir, mut router) = setup();
        router.register_capability(ModelCapabilities {
            model_id: "custom-model".to_string(),
            provider: "custom".to_string(),
            tool_use: false,
            vision: false,
            code_execution: false,
            context_window: 4096,
            cost_tier: 1,
            speed_tier: 1,
            complexity_range: (1, 1),
        });
        assert!(router.capabilities.contains_key("custom-model"));
    }
}
