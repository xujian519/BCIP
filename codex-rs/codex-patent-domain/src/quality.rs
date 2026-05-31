use codex_patent_core::*;

pub struct QualityAssessor;

impl QualityAssessor {
    pub fn assess_claims(claims: &[ClaimDraft]) -> QualityAssessment {
        let clarity_score = Self::assess_clarity(claims);
        let support_score = Self::assess_support(claims);
        let scope_score = Self::assess_scope(claims);
        let overall = clarity_score * 0.35 + support_score * 0.30 + scope_score * 0.35;

        let mut issues = Vec::new();
        if clarity_score < 0.6 {
            issues.push(QualityIssue {
                dimension: "清晰性".into(),
                severity: "高".into(),
                description: "权利要求表述不够清楚".into(),
                suggestion: "检查模糊用语，确保每个技术特征有明确定义".into(),
            });
        }
        if support_score < 0.6 {
            issues.push(QualityIssue {
                dimension: "支持性".into(),
                severity: "高".into(),
                description: "权利要求可能未得到说明书充分支持".into(),
                suggestion: "确保说明书中包含对应技术特征的实施例".into(),
            });
        }
        if scope_score < 0.5 {
            issues.push(QualityIssue {
                dimension: "保护范围".into(),
                severity: "中".into(),
                description: "保护范围可能过窄".into(),
                suggestion: "考虑使用开放式表达（包括/包含）替代封闭式表达".into(),
            });
        }

        QualityAssessment {
            clarity_score,
            support_score,
            scope_score,
            overall_score: overall,
            issues,
        }
    }

    fn assess_clarity(claims: &[ClaimDraft]) -> f32 {
        let mut score: f32 = 0.8;
        let vague_words = ["大约", "左右", "基本上", "适当", "一定", "某种"];
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
        }
        let dep_count = claims
            .iter()
            .filter(|c| c.claim_type == ClaimType::Dependent)
            .count();
        if dep_count >= 2 {
            score += 0.15;
        }
        score.min(1.0)
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
                elements: vec!["特征C".into()],
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
        let a = QualityAssessor::assess_claims(&claims);
        assert!(a.overall_score > 0.5);
        assert!(a.clarity_score > 0.0);
        assert!(a.scope_score > 0.0);
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
        let a = QualityAssessor::assess_claims(&claims);
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
        let a = QualityAssessor::assess_claims(&claims);
        assert!(a.support_score < 0.5);
    }
}
