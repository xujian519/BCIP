//! 学习闭环引擎
//!
//! 连接反馈收集 → 策略调整 → 效果追踪，实现真正的自适应学习：
//! - 聚合 `LearningStore`、`ReflectionEngine` 的历史数据
//! - 生成可执行的 `PolicyRecommendation`
//! - 持久化策略供后续 agent 会话消费

use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::learning::FeedbackData;
use crate::learning::GroupBy;
use crate::learning::LearningStore;
use crate::reflection::ReflectionEngine;
use crate::reflection::ReflectionResult;

/// 重试策略建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 建议最大重试次数
    pub max_retries: u32,
    /// 建议的基础延迟（毫秒）
    pub base_delay_ms: u64,
    /// 是否建议指数退避
    pub exponential_backoff: bool,
    /// 触发重试的常见错误类别
    pub retry_on_errors: Vec<String>,
}

/// 单个角色的策略建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRecommendation {
    pub role_id: String,
    /// 推荐模型（基于历史成功率）
    pub preferred_model: Option<String>,
    /// 推荐的 provider（基于延迟和成功率）
    pub preferred_provider: Option<String>,
    /// 应避免的 provider（高失败率）
    pub avoid_providers: Vec<String>,
    /// 重试策略（基于错误模式）
    pub retry_policy: RetryPolicy,
    /// 角色维度的历史成功率
    pub success_rate: f64,
    /// 质量趋势（正/负/稳定）
    pub quality_trend: QualityTrend,
    /// 改进建议
    pub suggestions: Vec<String>,
}

/// 质量趋势方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityTrend {
    Improving,
    Stable,
    Declining,
    InsufficientData,
}

/// 完整的策略推荐集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRecommendation {
    /// 各角色的独立建议
    pub role_recommendations: Vec<RoleRecommendation>,
    /// 全局改进建议
    pub global_suggestions: Vec<String>,
    /// 推荐生成时间戳
    pub generated_at: String,
}

/// 学习闭环引擎，聚合多源反馈生成策略推荐
pub struct FeedbackLoop {
    learning_store: LearningStore,
    reflection_engine: ReflectionEngine,
}

impl FeedbackLoop {
    pub fn new(home_dir: &Path) -> Self {
        Self {
            learning_store: LearningStore::new(home_dir),
            reflection_engine: ReflectionEngine::new(home_dir),
        }
    }

    /// 执行完整的分析并生成策略推荐
    pub fn analyze(&self) -> Result<PolicyRecommendation, String> {
        let all_feedback = self.learning_store.load_all().map_err(|e| e.to_string())?;
        let quality_trend = self.reflection_engine.get_quality_trend(50);
        let role_stats = self
            .learning_store
            .get_stats(GroupBy::Role)
            .map_err(|e| e.to_string())?;

        let mut role_recommendations = Vec::new();
        let mut global_suggestions = Vec::new();

        for (role_id, stats) in &role_stats {
            let role_feedback: Vec<&FeedbackData> =
                all_feedback.iter().filter(|f| &f.role_id == role_id).collect();

            let preferred_model = self.learning_store.suggest_model(role_id);
            let preferred_provider = suggest_provider(&role_feedback);
            let avoid_providers = find_weak_providers(&role_feedback);
            let retry_policy = build_retry_policy(&role_feedback);

            let role_quality: Vec<&ReflectionResult> = quality_trend
                .iter()
                .filter(|r| {
                    // 近似匹配：role_id 可能与 agent_id 前缀关联
                    all_feedback
                        .iter()
                        .any(|f| f.role_id == *role_id && f.agent_id == r.agent_id)
                })
                .collect();

            let quality_trend_dir = compute_quality_trend(&role_quality);

            let suggestions =
                generate_role_suggestions(stats, &preferred_model, &quality_trend_dir, &avoid_providers);

            role_recommendations.push(RoleRecommendation {
                role_id: role_id.clone(),
                preferred_model,
                preferred_provider,
                avoid_providers,
                retry_policy,
                success_rate: stats.success_rate(),
                quality_trend: quality_trend_dir,
                suggestions,
            });
        }

        // 全局建议：基于整体失败模式
        let recent_failures = self
            .learning_store
            .get_recent_failures(10)
            .map_err(|e| e.to_string())?;
        global_suggestions.extend(analyze_failure_patterns(&recent_failures));

        // 全局建议：基于模型统计
        let model_stats = self
            .learning_store
            .get_stats(GroupBy::Model)
            .map_err(|e| e.to_string())?;
        global_suggestions.extend(analyze_model_diversity(&model_stats));

        Ok(PolicyRecommendation {
            role_recommendations,
            global_suggestions,
            generated_at: crate::agent_manifest::iso8601_now(),
        })
    }

    /// 持久化策略推荐到磁盘
    pub fn save_recommendation(
        &self,
        recommendation: &PolicyRecommendation,
    ) -> Result<(), String> {
        let home = self.learning_store.home_dir();
        let dir = home.join("policy");
        std::fs::create_dir_all(&dir).map_err(|e| format!("create policy dir: {e}"))?;
        let path = dir.join("current_recommendation.json");
        let json = serde_json::to_string_pretty(recommendation)
            .map_err(|e| format!("serialize recommendation: {e}"))?;
        std::fs::write(&path, json).map_err(|e| format!("write recommendation: {e}"))
    }

    /// 加载最近的策略推荐
    pub fn load_recommendation(&self) -> Option<PolicyRecommendation> {
        let home = self.learning_store.home_dir();
        let path = home.join("policy").join("current_recommendation.json");
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// 获取指定角色的推荐
    pub fn recommendation_for_role(
        &self,
        role_id: &str,
    ) -> Result<Option<RoleRecommendation>, String> {
        let rec = self.analyze()?;
        Ok(rec
            .role_recommendations
            .into_iter()
            .find(|r| r.role_id == role_id))
    }
}

/// 根据历史数据推荐最佳 provider（成功率加权延迟）
fn suggest_provider(feedback: &[&FeedbackData]) -> Option<String> {
    let mut provider_scores: std::collections::HashMap<String, (u32, u32, u64)> =
        std::collections::HashMap::new();

    for f in feedback {
        let entry = provider_scores
            .entry(f.provider.clone())
            .or_insert((0, 0, 0));
        entry.0 += 1;
        if f.success {
            entry.1 += 1;
        }
        entry.2 += f.latency_ms;
    }

    // 评分 = 成功率 * 0.7 + 速度分(延迟倒数归一化) * 0.3
    let max_latency = provider_scores
        .values()
        .map(|v| v.2 as f64 / v.0 as f64)
        .fold(1.0, f64::max);

    provider_scores
        .into_iter()
        .filter(|(_, (total, _, _))| *total >= 2)
        .max_by(|a, b| {
            let score_a = score_provider(&a.1, max_latency);
            let score_b = score_provider(&b.1, max_latency);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(provider, _)| provider)
}

fn score_provider(stats: &(u32, u32, u64), max_latency: f64) -> f64 {
    let (total, success, latency) = (stats.0, stats.1, stats.2);
    let success_rate = success as f64 / total as f64;
    let avg_latency = latency as f64 / total as f64;
    let speed_score = if max_latency > 0.0 {
        1.0 - (avg_latency / max_latency)
    } else {
        1.0
    };
    success_rate * 0.7 + speed_score * 0.3
}

/// 找出失败率高的 provider
fn find_weak_providers(feedback: &[&FeedbackData]) -> Vec<String> {
    let mut provider_stats: std::collections::HashMap<String, (u32, u32)> =
        std::collections::HashMap::new();

    for f in feedback {
        let entry = provider_stats
            .entry(f.provider.clone())
            .or_insert((0, 0));
        entry.0 += 1;
        if !f.success {
            entry.1 += 1;
        }
    }

    provider_stats
        .into_iter()
        .filter(|(_, (total, failures))| *total >= 3 && *failures as f64 / *total as f64 > 0.5)
        .map(|(provider, _)| provider)
        .collect()
}

/// 根据错误模式构建重试策略
fn build_retry_policy(feedback: &[&FeedbackData]) -> RetryPolicy {
    let failures: Vec<&&FeedbackData> = feedback.iter().filter(|f| !(**f).success).collect();
    let failure_rate = if feedback.is_empty() {
        0.0
    } else {
        failures.len() as f64 / feedback.len() as f64
    };

    // 收集常见错误类别
    let mut error_counts: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    for f in &failures {
        if let Some(err) = &(**f).error_category {
            *error_counts.entry(err.clone()).or_insert(0) += 1;
        }
    }
    let retry_on_errors: Vec<String> = error_counts
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .map(|(err, _)| err)
        .collect();

    // 根据失败率调整重试参数
    if failure_rate > 0.5 {
        RetryPolicy {
            max_retries: 1,
            base_delay_ms: 2000,
            exponential_backoff: true,
            retry_on_errors,
        }
    } else if failure_rate > 0.2 {
        RetryPolicy {
            max_retries: 2,
            base_delay_ms: 1000,
            exponential_backoff: true,
            retry_on_errors,
        }
    } else {
        RetryPolicy {
            max_retries: 3,
            base_delay_ms: 500,
            exponential_backoff: false,
            retry_on_errors,
        }
    }
}

/// 计算质量趋势方向
fn compute_quality_trend(reflections: &[&ReflectionResult]) -> QualityTrend {
    if reflections.len() < 3 {
        return QualityTrend::InsufficientData;
    }

    // 按时间排序（最新的在前）
    let mut sorted: Vec<&ReflectionResult> = reflections.to_vec();
    sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // 比较前半段和后半段的平均质量
    let mid = sorted.len() / 2;
    let recent_avg: f64 = sorted[..mid].iter().map(|r| r.quality_score).sum::<f64>() / mid as f64;
    let older_avg: f64 = sorted[mid..]
        .iter()
        .map(|r| r.quality_score)
        .sum::<f64>()
        / (sorted.len() - mid) as f64;

    let diff = recent_avg - older_avg;
    if diff > 0.05 {
        QualityTrend::Improving
    } else if diff < -0.05 {
        QualityTrend::Declining
    } else {
        QualityTrend::Stable
    }
}

/// 生成角色级别的改进建议
fn generate_role_suggestions(
    stats: &crate::learning::LearningStats,
    preferred_model: &Option<String>,
    trend: &QualityTrend,
    avoid_providers: &[String],
) -> Vec<String> {
    let mut suggestions = Vec::new();

    if stats.success_rate() < 0.6 && stats.total_calls >= 3 {
        suggestions.push(format!(
            "角色成功率仅 {:.0}%，建议检查 prompt 设计或切换模型",
            stats.success_rate() * 100.0
        ));
    }

    if let Some(model) = preferred_model {
        suggestions.push(format!("基于历史表现推荐模型: {model}"));
    }

    match trend {
        QualityTrend::Declining => {
            suggestions.push("质量呈下降趋势，建议审查最近的 prompt 变更".to_string());
        }
        QualityTrend::Improving => {
            suggestions.push("质量呈上升趋势，当前策略有效".to_string());
        }
        QualityTrend::Stable => {}
        QualityTrend::InsufficientData => {
            suggestions.push("数据不足，继续收集反馈以建立基线".to_string());
        }
    }

    if !avoid_providers.is_empty() {
        suggestions.push(format!(
            "避免使用高失败率 provider: {}",
            avoid_providers.join(", ")
        ));
    }

    if stats.avg_latency_ms() > 10000.0 {
        suggestions.push(format!(
            "平均延迟 {:.1}s 较高，考虑切换到更快的模型或 provider",
            stats.avg_latency_ms() / 1000.0
        ));
    }

    suggestions
}

/// 分析最近失败的模式
fn analyze_failure_patterns(failures: &[FeedbackData]) -> Vec<String> {
    if failures.is_empty() {
        return Vec::new();
    }

    let mut suggestions = Vec::new();

    // 错误类别统计
    let mut error_counts: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    for f in failures {
        if let Some(ref err) = f.error_category {
            *error_counts.entry(err.clone()).or_insert(0) += 1;
        }
    }

    for (error, count) in &error_counts {
        if *count >= 3 {
            suggestions.push(format!(
                "错误 '{error}' 频繁出现 ({count} 次)，建议排查根因"
            ));
        }
    }

    // 模型失败集中度
    let mut model_fails: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    for f in failures {
        *model_fails.entry(f.model.clone()).or_insert(0) += 1;
    }
    for (model, count) in &model_fails {
        if *count >= 3 {
            suggestions.push(format!("模型 '{model}' 最近失败 {count} 次，考虑降级"));
        }
    }

    suggestions
}

/// 分析模型多样性
fn analyze_model_diversity(
    model_stats: &std::collections::HashMap<String, crate::learning::LearningStats>,
) -> Vec<String> {
    let mut suggestions = Vec::new();

    // 找出成功率显著低于平均的模型
    let total_calls: u32 = model_stats.values().map(|s| s.total_calls).sum();
    if total_calls < 10 {
        return suggestions;
    }

    let avg_success: f64 = model_stats
        .values()
        .map(|s| s.success_count as f64)
        .sum::<f64>()
        / total_calls as f64;

    for (model, stats) in model_stats {
        if stats.total_calls >= 5 {
            let rate = stats.success_count as f64 / stats.total_calls as f64;
            if rate < avg_success - 0.2 {
                suggestions.push(format!(
                    "模型 '{model}' 成功率 {:.0}% 低于平均 {:.0}%，建议减少使用",
                    rate * 100.0,
                    avg_success * 100.0
                ));
            }
        }
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_manifest::AgentManifest;
    use crate::learning::FeedbackData;
    use tempfile::TempDir;

    fn setup() -> (TempDir, FeedbackLoop) {
        let dir = TempDir::new().unwrap();
        let feedback_loop = FeedbackLoop::new(dir.path());
        (dir, feedback_loop)
    }

    fn fake_manifest(subagent_type: &str, model: &str) -> AgentManifest {
        AgentManifest {
            agent_id: format!("test-{subagent_type}-1"),
            name: "test".to_string(),
            subagent_type: subagent_type.to_string(),
            model: model.to_string(),
            status: "completed".to_string(),
            output_file: std::path::PathBuf::from("/tmp/test.md"),
            manifest_file: std::path::PathBuf::from("/tmp/test.json"),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            completed_at: None,
            error: None,
        }
    }

    fn seed_feedback(
        store: &LearningStore,
        role: &str,
        model: &str,
        provider: &str,
        success: bool,
        error: Option<&str>,
        latency_ms: u64,
    ) {
        store
            .record_feedback(FeedbackData {
                agent_id: format!("test-{role}"),
                role_id: role.to_string(),
                model: model.to_string(),
                provider: provider.to_string(),
                success,
                latency_ms,
                token_count: None,
                quality_score: if success { Some(0.8) } else { None },
                error_category: error.map(|s| s.to_string()),
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            })
            .unwrap();
    }

    #[test]
    fn test_analyze_empty() {
        let (_dir, loop_engine) = setup();
        let rec = loop_engine.analyze().unwrap();
        assert!(rec.role_recommendations.is_empty());
        assert!(rec.global_suggestions.is_empty());
    }

    #[test]
    fn test_analyze_with_feedback() {
        let (_dir, loop_engine) = setup();
        let store = &loop_engine.learning_store;

        // 3 successes with model-a
        for _ in 0..3 {
            seed_feedback(store, "analyzer", "model-a", "deepseek", true, None, 500);
        }
        // 3 failures with model-b
        for _ in 0..3 {
            seed_feedback(
                store,
                "analyzer",
                "model-b",
                "openai",
                false,
                Some("timeout"),
                2000,
            );
        }

        let rec = loop_engine.analyze().unwrap();
        assert_eq!(rec.role_recommendations.len(), 1);

        let role_rec = &rec.role_recommendations[0];
        assert_eq!(role_rec.role_id, "analyzer");
        assert_eq!(role_rec.preferred_model.as_deref(), Some("model-a"));
        assert!(role_rec.success_rate > 0.0);
        assert!(!role_rec.suggestions.is_empty());
    }

    #[test]
    fn test_save_and_load_recommendation() {
        let (_dir, loop_engine) = setup();
        let store = &loop_engine.learning_store;

        seed_feedback(store, "writer", "model-x", "qwen", true, None, 800);

        let rec = loop_engine.analyze().unwrap();
        loop_engine.save_recommendation(&rec).unwrap();

        let loaded = loop_engine.load_recommendation().unwrap();
        assert_eq!(loaded.role_recommendations.len(), rec.role_recommendations.len());
    }

    #[test]
    fn test_retry_policy_high_failure() {
        let feedback: Vec<FeedbackData> = (0..5)
            .map(|i| FeedbackData {
                agent_id: format!("test-{i}"),
                role_id: "analyzer".to_string(),
                model: "test".to_string(),
                provider: "test".to_string(),
                success: i < 1,
                latency_ms: 1000,
                token_count: None,
                quality_score: None,
                error_category: if i >= 1 {
                    Some("timeout".to_string())
                } else {
                    None
                },
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            })
            .collect();
        let refs: Vec<&FeedbackData> = feedback.iter().collect();
        let policy = build_retry_policy(&refs);
        assert_eq!(policy.max_retries, 1); // 高失败率 → 少重试
        assert!(policy.exponential_backoff);
        assert!(policy.retry_on_errors.contains(&"timeout".to_string()));
    }

    #[test]
    fn test_quality_trend_direction() {
        let dir = TempDir::new().unwrap();
        let engine = ReflectionEngine::new(dir.path());

        // 生成下降趋势的质量数据
        for i in 0..6 {
            let mut manifest = fake_manifest("analyzer", "test");
            manifest.agent_id = format!("trend-{i}");
            let output = if i < 3 {
                "高质量输出：包含权利要求分析和技术特征描述"
            } else {
                "ok" // 短输出 → 低质量分
            };
            let result = engine.reflect_on_output(&manifest, output);
            engine.save_reflection(&result).unwrap();
        }

        let trend_data = engine.get_quality_trend(10);
        let refs: Vec<&ReflectionResult> = trend_data.iter().collect();
        let trend = compute_quality_trend(&refs);
        // 至少能检测到趋势变化
        assert_ne!(trend, QualityTrend::InsufficientData);
    }

    #[test]
    fn test_weak_provider_detection() {
        let feedback: Vec<FeedbackData> = (0..6)
            .map(|i| FeedbackData {
                agent_id: format!("test-{i}"),
                role_id: "analyzer".to_string(),
                model: "test".to_string(),
                provider: "bad-provider".to_string(),
                success: i < 2,
                latency_ms: 1000,
                token_count: None,
                quality_score: None,
                error_category: None,
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            })
            .collect();
        let refs: Vec<&FeedbackData> = feedback.iter().collect();
        let weak = find_weak_providers(&refs);
        assert!(weak.contains(&"bad-provider".to_string()));
    }

    #[test]
    fn test_provider_scoring() {
        let feedback: Vec<FeedbackData> = vec![
            FeedbackData {
                agent_id: "t1".to_string(),
                role_id: "r".to_string(),
                model: "m".to_string(),
                provider: "fast".to_string(),
                success: true,
                latency_ms: 200,
                token_count: None,
                quality_score: None,
                error_category: None,
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            },
            FeedbackData {
                agent_id: "t2".to_string(),
                role_id: "r".to_string(),
                model: "m".to_string(),
                provider: "fast".to_string(),
                success: true,
                latency_ms: 300,
                token_count: None,
                quality_score: None,
                error_category: None,
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            },
            FeedbackData {
                agent_id: "t3".to_string(),
                role_id: "r".to_string(),
                model: "m".to_string(),
                provider: "slow".to_string(),
                success: true,
                latency_ms: 5000,
                token_count: None,
                quality_score: None,
                error_category: None,
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            },
            FeedbackData {
                agent_id: "t4".to_string(),
                role_id: "r".to_string(),
                model: "m".to_string(),
                provider: "slow".to_string(),
                success: true,
                latency_ms: 6000,
                token_count: None,
                quality_score: None,
                error_category: None,
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            },
        ];
        let refs: Vec<&FeedbackData> = feedback.iter().collect();
        let provider = suggest_provider(&refs);
        assert_eq!(provider.as_deref(), Some("fast"));
    }
}
