//! OA 答复反馈闭环
//!
//! 跟踪审查意见答复结果，从成功和失败中学习，自动调整答复策略。

use serde::Deserialize;
use serde::Serialize;

/// 反馈类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    /// 答复成功，专利授权
    Success,
    /// 部分成功，需要修改
    PartialSuccess,
    /// 答复失败，专利驳回
    Failure,
    /// 质量问题
    QualityIssue,
}

/// 优化动作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationAction {
    /// 调整超时参数
    AdjustTimeout,
    /// 更新模式权重
    UpdatePattern,
    /// 重试失败操作
    RetryFailed,
    /// 改进质量控制
    ImproveQuality,
    /// 告警需要人工介入
    AlertHuman,
}

/// 反馈记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub feedback_id: String,
    pub oa_id: String,
    pub patent_id: String,
    pub feedback_type: FeedbackType,
    /// 实际审查结果（allowed / rejected / partial）
    pub outcome: String,
    pub quality_score: f64,
    pub strategy_used: Option<String>,
    pub comments: String,
    pub timestamp: String,
    pub analyzed: bool,
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_id: String,
    pub action_type: OptimizationAction,
    pub description: String,
    pub reason: String,
    pub confidence: f64,
    pub auto_apply: bool,
}

/// 反馈模式分析器
pub struct FeedbackAnalyzer {
    /// 历史成功答复的特征权重
    pub success_patterns: Vec<FeedbackPattern>,
    /// 历史失败答复的特征
    pub failure_patterns: Vec<FeedbackPattern>,
}

/// 答复模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackPattern {
    pub oa_type: String,
    pub strategy: String,
    pub argument_count: usize,
    pub success_rate: f64,
    pub sample_count: usize,
}

impl FeedbackAnalyzer {
    pub fn new() -> Self {
        Self {
            success_patterns: Vec::new(),
            failure_patterns: Vec::new(),
        }
    }

    /// 收集一条反馈记录，更新模式统计
    pub fn collect(&mut self, record: &FeedbackRecord) {
        let strategy = record.strategy_used.clone().unwrap_or_default();
        match record.feedback_type {
            FeedbackType::Success | FeedbackType::PartialSuccess => {
                Self::update_pattern(&mut self.success_patterns, "success", &strategy, 1.0);
            }
            FeedbackType::Failure => {
                Self::update_pattern(&mut self.failure_patterns, "failure", &strategy, 0.0);
            }
            FeedbackType::QualityIssue => {
                Self::update_pattern(&mut self.failure_patterns, "quality", &strategy, 0.0);
            }
        }
    }

    fn update_pattern(
        patterns: &mut Vec<FeedbackPattern>,
        oa_type: &str,
        strategy: &str,
        new_success: f64,
    ) {
        if let Some(p) = patterns.iter_mut().find(|p| p.strategy == strategy) {
            let total = p.sample_count as f64;
            p.success_rate = (p.success_rate * total + new_success) / (total + 1.0);
            p.sample_count += 1;
        } else {
            patterns.push(FeedbackPattern {
                oa_type: oa_type.into(),
                strategy: strategy.into(),
                argument_count: 0,
                success_rate: if new_success > 0.5 { 1.0 } else { 0.0 },
                sample_count: 1,
            });
        }
    }

    /// 根据历史反馈推荐最佳策略
    pub fn recommend_strategy(&self, available_strategies: &[&str]) -> Option<String> {
        available_strategies
            .iter()
            .filter_map(|&s| {
                self.success_patterns
                    .iter()
                    .find(|p| p.strategy == s)
                    .map(|p| (s, p.success_rate))
            })
            .max_by(|(_, rate_a), (_, rate_b)| {
                rate_a
                    .partial_cmp(rate_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(s, _)| s.to_string())
    }

    /// 生成优化建议
    pub fn generate_suggestions(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        for pattern in &self.failure_patterns {
            if pattern.sample_count >= 3 && pattern.success_rate < 0.3 {
                suggestions.push(OptimizationSuggestion {
                    suggestion_id: format!("opt-{}", pattern.strategy),
                    action_type: OptimizationAction::UpdatePattern,
                    description: format!(
                        "策略 '{}' 成功率低({:.0}%, {}次)，建议降低权重",
                        pattern.strategy,
                        pattern.success_rate * 100.0,
                        pattern.sample_count
                    ),
                    reason: "低成功率".into(),
                    confidence: 0.8,
                    auto_apply: pattern.sample_count >= 5,
                });
            }
        }

        for pattern in &self.success_patterns {
            if pattern.sample_count >= 3 && pattern.success_rate > 0.8 {
                suggestions.push(OptimizationSuggestion {
                    suggestion_id: format!("opt-{}", pattern.strategy),
                    action_type: OptimizationAction::UpdatePattern,
                    description: format!(
                        "策略 '{}' 成功率较高({:.0}%, {}次)，建议优先使用",
                        pattern.strategy,
                        pattern.success_rate * 100.0,
                        pattern.sample_count
                    ),
                    reason: "高成功率".into(),
                    confidence: 0.9,
                    auto_apply: false,
                });
            }
        }

        suggestions
    }
}

impl Default for FeedbackAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(
        strategy: &str,
        outcome: &str,
        feedback_type: FeedbackType,
        quality: f64,
    ) -> FeedbackRecord {
        FeedbackRecord {
            feedback_id: "fb1".into(),
            oa_id: "oa1".into(),
            patent_id: "p1".into(),
            feedback_type,
            outcome: outcome.into(),
            quality_score: quality,
            strategy_used: Some(strategy.into()),
            comments: String::new(),
            timestamp: String::new(),
            analyzed: false,
        }
    }

    #[test]
    fn empty_analyzer_returns_none() {
        let analyzer = FeedbackAnalyzer::new();
        assert!(analyzer.recommend_strategy(&["argue"]).is_none());
    }

    #[test]
    fn collect_success_updates_pattern() {
        let mut analyzer = FeedbackAnalyzer::new();
        for _ in 0..5 {
            analyzer.collect(&make_record("argue", "allowed", FeedbackType::Success, 0.9));
        }
        let suggestion = analyzer.recommend_strategy(&["argue", "amend"]);
        assert_eq!(suggestion, Some("argue".into()));
    }

    #[test]
    fn failure_generates_suggestion() {
        let mut analyzer = FeedbackAnalyzer::new();
        for _ in 0..4 {
            analyzer.collect(&make_record(
                "withdraw",
                "rejected",
                FeedbackType::Failure,
                0.2,
            ));
        }
        let suggestions = analyzer.generate_suggestions();
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn empty_failures_no_suggestions() {
        let analyzer = FeedbackAnalyzer::new();
        let suggestions = analyzer.generate_suggestions();
        assert!(suggestions.is_empty());
    }
}
