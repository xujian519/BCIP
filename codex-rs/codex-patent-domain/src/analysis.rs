//! 多维度综合分析框架
//!
//! 将现有单维度分析（新颖性、创造性、侵权等）编排为统一报告。
//! 参考钱学森"定性定量相结合"方法论。

use codex_patent_core::FeatureMatchResult;
use serde::Deserialize;
use serde::Serialize;

/// 分析维度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisDimension {
    Novelty,
    Inventiveness,
    Utility,
    InfringementRisk,
    DraftQuality,
}

impl AnalysisDimension {
    /// 返回该维度的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Novelty => "新颖性",
            Self::Inventiveness => "创造性",
            Self::Utility => "实用性",
            Self::InfringementRisk => "侵权风险",
            Self::DraftQuality => "撰写质量",
        }
    }

    /// 返回该维度的权重系数（所有维度权重之和为 1.0）
    pub fn weight(&self) -> f64 {
        match self {
            Self::Novelty => 0.25,
            Self::Inventiveness => 0.30,
            Self::Utility => 0.10,
            Self::InfringementRisk => 0.20,
            Self::DraftQuality => 0.15,
        }
    }
}

/// 维度分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionResult {
    pub dimension: AnalysisDimension,
    pub score: f64,
    pub conclusion: String,
    pub details: Vec<String>,
    pub legal_basis: Vec<String>,
    pub risk_level: RiskLevel,
}

/// 风险评估等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    fn from_score(s: f64) -> Self {
        match s {
            x if x >= 0.8 => Self::Low,
            x if x >= 0.5 => Self::Medium,
            x if x >= 0.3 => Self::High,
            _ => Self::Critical,
        }
    }
}

/// 综合分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveAnalysisReport {
    pub dimensions: Vec<DimensionResult>,
    pub overall_score: f64,
    pub overall_conclusion: String,
    pub recommendations: Vec<String>,
    pub risk_summary: RiskSummary,
}

/// 风险摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummary {
    pub high_risk_dimensions: Vec<String>,
    pub total_risk_score: f64,
    pub recommended_actions: Vec<String>,
}

/// 构建新颖性维度结果
pub fn assess_novelty(matches: &FeatureMatchResult) -> DimensionResult {
    let total_features = (matches.exact_matches.len()
        + matches.equivalent_matches.len()
        + matches.different_features.len()
        + matches.missing_features.len())
    .max(1) as f64;

    let matched_features = (matches.exact_matches.len() + matches.equivalent_matches.len()) as f64;

    let score = if matches.missing_features.is_empty() && matches.different_features.is_empty() {
        0.1
    } else if matched_features / total_features < 0.5 {
        0.9
    } else {
        0.5
    };

    DimensionResult {
        dimension: AnalysisDimension::Novelty,
        score,
        conclusion: if score >= 0.7 {
            "具备新颖性".into()
        } else if score >= 0.4 {
            "新颖性存疑".into()
        } else {
            "缺乏新颖性".into()
        },
        details: vec![
            format!("完全匹配特征: {}个", matches.exact_matches.len()),
            format!("等同特征: {}个", matches.equivalent_matches.len()),
            format!("不同特征: {}个", matches.different_features.len()),
            format!("缺失特征: {}个", matches.missing_features.len()),
        ],
        legal_basis: vec!["专利法第22条第2款".into()],
        risk_level: RiskLevel::from_score(score),
    }
}

/// 构建创造性维度结果
pub fn assess_inventiveness(
    has_differences: bool,
    has_unexpected_effect: bool,
    is_obvious_combination: bool,
) -> DimensionResult {
    let mut score: f64 = 0.5;
    if has_differences {
        score += 0.2;
    }
    if has_unexpected_effect {
        score += 0.2;
    }
    if is_obvious_combination {
        score -= 0.3;
    }
    let score = score.clamp(0.0, 1.0);

    DimensionResult {
        dimension: AnalysisDimension::Inventiveness,
        score,
        conclusion: if score >= 0.7 {
            "具备创造性".into()
        } else if score >= 0.4 {
            "创造性存疑".into()
        } else {
            "缺乏创造性".into()
        },
        details: vec![
            format!("存在区别技术特征: {has_differences}"),
            format!("产生预料不到的效果: {has_unexpected_effect}"),
            format!("属于显而易见组合: {is_obvious_combination}"),
        ],
        legal_basis: vec!["专利法第22条第3款".into()],
        risk_level: RiskLevel::from_score(score),
    }
}

/// 构建侵权风险维度结果
pub fn assess_infringement_risk(coverage_ratio: f64) -> DimensionResult {
    let score = 1.0 - coverage_ratio;

    DimensionResult {
        dimension: AnalysisDimension::InfringementRisk,
        score,
        conclusion: if coverage_ratio < 0.3 {
            "侵权风险低".into()
        } else if coverage_ratio < 0.7 {
            "需要关注".into()
        } else {
            "侵权风险较高".into()
        },
        details: vec![format!("特征覆盖度: {:.0}%", coverage_ratio * 100.0)],
        legal_basis: vec!["专利法第11条".into(), "全面覆盖原则".into()],
        risk_level: RiskLevel::from_score(score),
    }
}

/// 构建撰写质量维度结果
pub fn assess_draft_quality(quality_score: f64, issues: &[String]) -> DimensionResult {
    DimensionResult {
        dimension: AnalysisDimension::DraftQuality,
        score: quality_score,
        conclusion: if quality_score >= 0.7 {
            "撰写质量良好".into()
        } else if quality_score >= 0.5 {
            "撰写质量一般".into()
        } else {
            "需要大幅修改".into()
        },
        details: issues.to_vec(),
        legal_basis: vec!["专利法第26条第4款".into(), "审查指南第二部分第二章".into()],
        risk_level: RiskLevel::from_score(quality_score),
    }
}

/// 生成综合分析报告
pub fn generate_report(dimensions: Vec<DimensionResult>) -> ComprehensiveAnalysisReport {
    let overall: f64 = if dimensions.is_empty() {
        0.0
    } else {
        let total_weight: f64 = dimensions.iter().map(|d| d.dimension.weight()).sum();
        dimensions
            .iter()
            .map(|d| d.score * d.dimension.weight())
            .sum::<f64>()
            / total_weight
    };

    let mut recommendations = Vec::new();
    let mut high_risk = Vec::new();

    for dim in &dimensions {
        if matches!(dim.risk_level, RiskLevel::High | RiskLevel::Critical) {
            high_risk.push(dim.dimension.name().to_string());
            recommendations.push(format!("[{}] {}", dim.dimension.name(), dim.conclusion));
        }
    }

    let overall_conclusion = if overall >= 0.7 {
        "该专利整体评估良好，建议提交申请。".into()
    } else if overall >= 0.4 {
        "该专利存在一定风险，建议针对具体缺陷修改后再提交。".into()
    } else {
        "该专利风险较高，建议重新评估技术方案或进行重大修改。".into()
    };

    ComprehensiveAnalysisReport {
        overall_score: overall,
        overall_conclusion,
        risk_summary: RiskSummary {
            high_risk_dimensions: high_risk,
            total_risk_score: 1.0 - overall,
            recommended_actions: recommendations.clone(),
        },
        recommendations,
        dimensions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_patent_core::CorrespondenceType;
    use codex_patent_core::FeatureMatch;
    use codex_patent_core::InfringementType;

    #[test]
    fn weight_sum_is_one() {
        let all = [
            AnalysisDimension::Novelty,
            AnalysisDimension::Inventiveness,
            AnalysisDimension::Utility,
            AnalysisDimension::InfringementRisk,
            AnalysisDimension::DraftQuality,
        ];
        let sum: f64 = all.iter().map(|d| d.weight()).sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn high_novelty_score_when_all_missing() {
        let matches = FeatureMatchResult {
            exact_matches: vec![],
            equivalent_matches: vec![],
            different_features: vec!["特征A".into(), "特征B".into()],
            missing_features: vec!["特征C".into()],
            coverage_ratio: 0.0,
            infringement_type: Some(InfringementType::NoInfringement),
        };
        let result = assess_novelty(&matches);
        assert!(result.score > 0.7);
        assert_eq!(result.risk_level, RiskLevel::Low);
    }

    #[test]
    fn low_novelty_score_when_all_matched() {
        let matches = FeatureMatchResult {
            exact_matches: vec![FeatureMatch {
                target_feature: "A".into(),
                prior_feature: "A".into(),
                similarity_score: 1.0,
                match_type: CorrespondenceType::Exact,
            }],
            equivalent_matches: vec![],
            different_features: vec![],
            missing_features: vec![],
            coverage_ratio: 1.0,
            infringement_type: Some(InfringementType::Literal),
        };
        let result = assess_novelty(&matches);
        assert!(result.score < 0.3);
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn report_generates_with_all_dimensions() {
        let dims = vec![
            assess_inventiveness(true, true, false),
            assess_infringement_risk(0.1),
            assess_draft_quality(0.85, &[]),
        ];
        let report = generate_report(dims);
        assert_eq!(report.dimensions.len(), 3);
        assert!(report.overall_score > 0.0);
        assert!(!report.overall_conclusion.is_empty());
    }

    #[test]
    fn high_risk_inventiveness_adds_recommendation() {
        let dims = vec![assess_inventiveness(false, false, true)];
        let report = generate_report(dims);
        assert!(!report.risk_summary.high_risk_dimensions.is_empty());
    }

    #[test]
    fn dimension_name_returns_chinese() {
        assert_eq!(AnalysisDimension::Novelty.name(), "新颖性");
        assert_eq!(AnalysisDimension::Inventiveness.name(), "创造性");
        assert_eq!(AnalysisDimension::Utility.name(), "实用性");
        assert_eq!(AnalysisDimension::InfringementRisk.name(), "侵权风险");
        assert_eq!(AnalysisDimension::DraftQuality.name(), "撰写质量");
    }

    #[test]
    fn dimension_weight_all_positive() {
        for dim in [
            AnalysisDimension::Novelty,
            AnalysisDimension::Inventiveness,
            AnalysisDimension::Utility,
            AnalysisDimension::InfringementRisk,
            AnalysisDimension::DraftQuality,
        ] {
            assert!(dim.weight() > 0.0, "{dim:?} weight should be positive");
        }
    }

    #[test]
    fn risk_level_from_score_boundaries() {
        assert_eq!(RiskLevel::from_score(0.9), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(0.8), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(0.7), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(0.5), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(0.4), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(0.3), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(0.1), RiskLevel::Critical);
        assert_eq!(RiskLevel::from_score(0.0), RiskLevel::Critical);
    }

    #[test]
    fn assess_inventiveness_all_true() {
        let result = assess_inventiveness(true, true, false);
        assert!(result.score > 0.8);
        assert_eq!(result.dimension, AnalysisDimension::Inventiveness);
        assert_eq!(result.risk_level, RiskLevel::Low);
    }

    #[test]
    fn assess_inventiveness_obvious_combination() {
        let result = assess_inventiveness(false, false, true);
        assert!(result.score < 0.3);
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn assess_inventiveness_baseline() {
        let result = assess_inventiveness(false, false, false);
        assert!((result.score - 0.5).abs() < 0.001);
    }

    #[test]
    fn assess_infringement_risk_low_coverage() {
        let result = assess_infringement_risk(0.1);
        assert!(result.score > 0.8);
        assert!(result.conclusion.contains("低"));
    }

    #[test]
    fn assess_infringement_risk_high_coverage() {
        let result = assess_infringement_risk(0.9);
        assert!(result.score < 0.2);
        assert!(result.conclusion.contains("较高"));
    }

    #[test]
    fn assess_infringement_risk_medium_coverage() {
        let result = assess_infringement_risk(0.5);
        assert!(result.conclusion.contains("关注"));
    }

    #[test]
    fn assess_draft_quality_good() {
        let result = assess_draft_quality(0.85, &[]);
        assert_eq!(result.dimension, AnalysisDimension::DraftQuality);
        assert!(result.conclusion.contains("良好"));
    }

    #[test]
    fn assess_draft_quality_poor() {
        let result = assess_draft_quality(0.3, &["权利要求不清楚".into()]);
        assert!(result.conclusion.contains("修改"));
        assert_eq!(result.details.len(), 1);
    }

    #[test]
    fn generate_report_empty_dimensions() {
        let report = generate_report(vec![]);
        assert_eq!(report.overall_score, 0.0);
        assert!(report.recommendations.is_empty());
        assert!(report.risk_summary.high_risk_dimensions.is_empty());
    }

    #[test]
    fn generate_report_overall_conclusion_high() {
        let dims = vec![
            assess_inventiveness(true, true, false),
            assess_infringement_risk(0.1),
            assess_draft_quality(0.9, &[]),
        ];
        let report = generate_report(dims);
        assert!(report.overall_conclusion.contains("良好"));
    }

    #[test]
    fn generate_report_overall_conclusion_low() {
        let dims = vec![
            assess_inventiveness(false, false, true),
            assess_draft_quality(0.1, &["问题".into()]),
        ];
        let report = generate_report(dims);
        assert!(report.overall_score < 0.4);
        assert!(report.risk_summary.total_risk_score > 0.5);
    }

    #[test]
    fn dimension_result_serialize_deserialize() {
        let result = assess_inventiveness(true, true, false);
        let json = serde_json::to_string(&result).unwrap();
        let back: DimensionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.dimension, result.dimension);
        assert!((back.score - result.score).abs() < 0.001);
    }

    #[test]
    fn novelty_medium_score() {
        let matches = FeatureMatchResult {
            exact_matches: vec![FeatureMatch {
                target_feature: "A".into(),
                prior_feature: "A".into(),
                similarity_score: 1.0,
                match_type: CorrespondenceType::Exact,
            }],
            equivalent_matches: vec![],
            different_features: vec!["B".into()],
            missing_features: vec![],
            coverage_ratio: 0.5,
            infringement_type: Some(InfringementType::Literal),
        };
        let result = assess_novelty(&matches);
        assert!((result.score - 0.5).abs() < 0.001);
        assert!(result.conclusion.contains("存疑"));
    }
}
