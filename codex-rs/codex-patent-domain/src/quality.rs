//! 权利要求质量评估引擎
//!
//! 从清晰性、支持性、保护范围、可实施性四个维度评估专利权利要求书的质量。
//! 评分均为 0.0~1.0，高于阈值表示该维度通过评估。

use crate::quality_rules;
use codex_patent_core::*;
use serde::Deserialize;
use serde::Serialize;

const CLARITY_THRESHOLD: f32 = 0.6;
const SUPPORT_THRESHOLD: f32 = 0.6;
const SCOPE_THRESHOLD: f32 = 0.5;

/// 对权利要求集进行全面质量评估
///
/// 从清晰性、支持性、保护范围、可实施性四个维度分别评分，
/// 加权计算总分，并识别具体质量问题。
pub fn assess_claims(claims: &[ClaimDraft]) -> QualityAssessment {
    let clarity_score = assess_clarity(claims);
    let support_score = assess_support(claims);
    let scope_score = assess_scope(claims);
    let enablement_score = assess_enablement(claims);
    let overall =
        clarity_score * 0.25 + support_score * 0.25 + scope_score * 0.25 + enablement_score * 0.25;

    let mut issues = Vec::new();
    if clarity_score < CLARITY_THRESHOLD {
        issues.push(QualityIssue {
            dimension: "清晰性".into(),
            severity: "高".into(),
            description: "权利要求表述不够清楚".into(),
            suggestion: "检查模糊用语，确保每个技术特征有明确定义".into(),
        });
    }
    if support_score < SUPPORT_THRESHOLD {
        issues.push(QualityIssue {
            dimension: "支持性".into(),
            severity: "高".into(),
            description: "权利要求可能未得到说明书充分支持".into(),
            suggestion: "确保说明书中包含对应技术特征的实施例".into(),
        });
    }
    if scope_score < SCOPE_THRESHOLD {
        issues.push(QualityIssue {
            dimension: "保护范围".into(),
            severity: "中".into(),
            description: "保护范围可能过窄".into(),
            suggestion: "考虑使用开放式表达（包括/包含）替代封闭式表达".into(),
        });
    }
    if enablement_score < SUPPORT_THRESHOLD {
        issues.push(QualityIssue {
            dimension: "可实施性".into(),
            severity: "高".into(),
            description: "说明书可能未充分公开".into(),
            suggestion: "补充实施例和实验数据，确保本领域技术人员能够实现".into(),
        });
    }

    QualityAssessment {
        clarity_score,
        support_score,
        scope_score,
        enablement_score,
        overall_score: overall,
        issues,
    }
}

fn assess_clarity(claims: &[ClaimDraft]) -> f32 {
    let mut score: f32 = 0.8;
    let vague_words = quality_rules::vague_words();
    for claim in claims {
        let vague_count = vague_words
            .iter()
            .filter(|w| claim.elements.iter().any(|e| e.contains(**w)))
            .count();
        score -= vague_count as f32 * 0.15;

        if claim.claim_type == ClaimType::Independent {
            if claim.preamble.is_empty() {
                score -= 0.15;
            }
            if claim.transitional_phrase.is_empty() {
                score -= 0.1;
            }
            if claim.elements.is_empty() {
                score -= 0.2;
            }
        }
    }
    score.clamp(0.0, 1.0)
}

fn assess_support(claims: &[ClaimDraft]) -> f32 {
    let mut score: f32 = 0.7;
    for claim in claims {
        if let Some(ref dep) = claim.dependent_on
            && !claims.iter().any(|c| c.id == *dep)
        {
            score -= 0.3;
        }
    }
    let has_ind = claims
        .iter()
        .any(|c| c.claim_type == ClaimType::Independent);
    let has_dep = claims.iter().any(|c| c.claim_type == ClaimType::Dependent);
    if has_ind && has_dep {
        score += 0.2;
    }
    score.min(1.0)
}

fn assess_scope(claims: &[ClaimDraft]) -> f32 {
    let mut score: f32 = 0.6;
    if let Some(ind) = claims
        .iter()
        .find(|c| c.claim_type == ClaimType::Independent)
    {
        match ind.elements.len() {
            0 => score -= 0.3,
            1..=3 => score += 0.2,
            4..=6 => score += 0.1,
            _ => score -= 0.1,
        }
        let has_open =
            ind.transitional_phrase.contains("包括") || ind.transitional_phrase.contains("包含");
        if has_open {
            score += 0.15;
        }
    }
    let dep_count = claims
        .iter()
        .filter(|c| c.claim_type == ClaimType::Dependent)
        .count();
    if dep_count >= 3 {
        score += 0.2;
    } else if dep_count >= 2 {
        score += 0.1;
    }
    score.min(1.0)
}

fn assess_enablement(claims: &[ClaimDraft]) -> f32 {
    let mut score: f32 = 0.7;
    let min_words = quality_rules::enablement_min_words();
    for claim in claims {
        let total_chars: usize = claim.elements.iter().map(|e| e.chars().count()).sum();
        if total_chars < min_words {
            score -= 0.1;
        }
    }
    if claims.iter().any(|c| {
        c.elements
            .iter()
            .any(|e| e.contains("实施例") || e.contains("实施方式"))
    }) {
        score += 0.2;
    }
    score.min(1.0)
}

/// 质量评估器配置
///
/// 可自定义各维度的评分阈值。默认值与模块级常量保持一致。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessorConfig {
    pub clarity_threshold: f32,
    pub support_threshold: f32,
    pub scope_threshold: f32,
}

impl Default for QualityAssessorConfig {
    fn default() -> Self {
        Self {
            clarity_threshold: CLARITY_THRESHOLD,
            support_threshold: SUPPORT_THRESHOLD,
            scope_threshold: SCOPE_THRESHOLD,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_claims() -> Vec<ClaimDraft> {
        vec![
            ClaimDraft {
                id: "1".into(),
                claim_type: ClaimType::Independent,
                preamble: "一种装置".into(),
                transitional_phrase: "其特征在于".into(),
                elements: vec!["特征A".into(), "特征B".into()],
                dependent_on: None,
            },
            ClaimDraft {
                id: "2".into(),
                claim_type: ClaimType::Dependent,
                preamble: "根据权利要求1".into(),
                transitional_phrase: String::new(),
                elements: vec!["实施例的特征C".into()],
                dependent_on: Some("1".into()),
            },
            ClaimDraft {
                id: "3".into(),
                claim_type: ClaimType::Dependent,
                preamble: "根据权利要求2".into(),
                transitional_phrase: String::new(),
                elements: vec!["特征D".into()],
                dependent_on: Some("2".into()),
            },
        ]
    }

    #[test]
    fn test_assess_claims() {
        let claims = test_claims();
        let a = assess_claims(&claims);
        assert!(a.overall_score > 0.4);
        assert!(a.clarity_score > 0.0);
        assert!(a.scope_score > 0.0);
        assert!(a.enablement_score > 0.0);
    }

    #[test]
    fn test_vague_penalized() {
        let claims = vec![ClaimDraft {
            id: "1".into(),
            claim_type: ClaimType::Independent,
            preamble: "一种装置".into(),
            transitional_phrase: "其特征在于".into(),
            elements: vec!["大约特征A".into()],
            dependent_on: None,
        }];
        let a = assess_claims(&claims);
        assert!(a.clarity_score < 0.7);
    }

    #[test]
    fn test_broken_dependency() {
        let claims = vec![ClaimDraft {
            id: "2".into(),
            claim_type: ClaimType::Dependent,
            preamble: "根据权利要求1".into(),
            transitional_phrase: String::new(),
            elements: vec!["特征C".into()],
            dependent_on: Some("99".into()),
        }];
        let a = assess_claims(&claims);
        assert!(a.support_score < 0.5);
    }

    #[test]
    fn test_enablement_boosted_by_embodiment_ref() {
        let claims = vec![ClaimDraft {
            id: "1".into(),
            claim_type: ClaimType::Independent,
            preamble: "一种装置".into(),
            transitional_phrase: "包括".into(),
            elements: vec!["实施例中描述的模块A".into(), "模块B".into()],
            dependent_on: None,
        }];
        let a = assess_claims(&claims);
        assert!(a.enablement_score > 0.7);
    }

    #[test]
    fn test_scope_open_transition_boosted() {
        let claims = vec![ClaimDraft {
            id: "1".into(),
            claim_type: ClaimType::Independent,
            preamble: "一种装置".into(),
            transitional_phrase: "包括".into(),
            elements: vec!["特征A".into(), "特征B".into(), "特征C".into()],
            dependent_on: None,
        }];
        let a = assess_claims(&claims);
        assert!(a.scope_score > 0.7);
    }
}
