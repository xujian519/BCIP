//! 侵权分析流水线
//!
//! 全流程：权利要求解析 → 特征对比 → 全面覆盖原则 → 等同原则 → 判定结论

use crate::claim_parser::ClaimParser;
use codex_patent_core::*;

/// 侵权分析上下文
#[derive(Debug, Clone)]
pub struct InfringementContext {
    pub patent_claims: Vec<String>,
    pub accused_product: CompareDocument,
    pub legal_standard: InfringementStandard,
}

/// 侵权判定标准
#[derive(Debug, Clone)]
pub enum InfringementStandard {
    ChinaAllElementsPlusDoE,
    LiteralOnly,
}

/// 侵权判定结果
#[derive(Debug, Clone)]
pub struct InfringementResult {
    pub claim_results: Vec<FeatureMatchResult>,
    pub overall_conclusion: InfringementType,
    pub legal_basis: Vec<String>,
    pub opinion: String,
}

/// 侵权分析流水线
pub struct InfringementPipeline;

impl InfringementPipeline {
    pub fn new() -> Self {
        Self
    }

    /// 执行全流程侵权分析
    pub fn analyze(&self, context: &InfringementContext) -> InfringementResult {
        let parser = ClaimParser::new();
        let mut claim_results = Vec::new();
        let mut any_infringement = false;

        for (i, claim_text) in context.patent_claims.iter().enumerate() {
            let parsed = parser.parse((i + 1) as u32, claim_text);
            let feature_texts: Vec<String> = parsed
                .features
                .iter()
                .map(|f| f.description.clone())
                .collect();
            let result = self.analyze_single_claim(
                &parsed.preamble,
                &feature_texts,
                &context.accused_product.features,
            );
            let infringes = result.infringement_type != Some(InfringementType::NoInfringement);
            if infringes {
                any_infringement = true;
            }
            claim_results.push(result);
        }

        let conclusion = if any_infringement {
            InfringementType::Literal
        } else {
            InfringementType::NoInfringement
        };

        let opinion = match conclusion {
            InfringementType::Literal => "落入专利权的保护范围，构成字面侵权。".into(),
            InfringementType::DoctrineOfEquivalents => {
                "虽不构成字面侵权，但根据等同原则，仍落入保护范围。".into()
            }
            InfringementType::NoInfringement => "未落入专利权的保护范围，不构成侵权。".into(),
        };

        InfringementResult {
            claim_results,
            overall_conclusion: conclusion,
            legal_basis: vec![
                "专利法第11条".into(),
                "全面覆盖原则".into(),
                "等同原则".into(),
            ],
            opinion,
        }
    }

    fn analyze_single_claim(
        &self,
        _preamble: &str,
        claim_features: &[String],
        product_features: &[CompareFeature],
    ) -> FeatureMatchResult {
        let mut exact_matches = Vec::new();
        let mut equivalent_matches = Vec::new();
        let mut missing = Vec::new();

        for cf in claim_features {
            let match_found = product_features
                .iter()
                .find(|pf| ClaimParser::feature_text_similarity(cf, &pf.description) >= 0.6);

            match match_found {
                Some(pf) => {
                    let sim = ClaimParser::feature_text_similarity(cf, &pf.description);
                    let match_type = if sim >= 0.9 {
                        CorrespondenceType::Exact
                    } else {
                        CorrespondenceType::Equivalent
                    };
                    let fm = FeatureMatch {
                        target_feature: cf.clone(),
                        prior_feature: pf.description.clone(),
                        similarity_score: sim,
                        match_type,
                    };
                    match match_type {
                        CorrespondenceType::Exact => exact_matches.push(fm),
                        _ => equivalent_matches.push(fm),
                    }
                }
                None => missing.push(cf.clone()),
            }
        }

        let total = claim_features.len().max(1) as f64;
        let matched = (exact_matches.len() + equivalent_matches.len()) as f64;
        let coverage = matched / total;

        let infringement_type = if coverage >= 1.0 {
            Some(InfringementType::Literal)
        } else if !equivalent_matches.is_empty() && coverage >= 0.6 {
            Some(InfringementType::DoctrineOfEquivalents)
        } else {
            Some(InfringementType::NoInfringement)
        };

        FeatureMatchResult {
            exact_matches,
            equivalent_matches,
            different_features: Vec::new(),
            missing_features: missing,
            coverage_ratio: coverage,
            infringement_type,
        }
    }
}

impl Default for InfringementPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_claims_results_in_no_infringement() {
        let pipeline = InfringementPipeline::new();
        let context = InfringementContext {
            patent_claims: vec![],
            accused_product: CompareDocument::default(),
            legal_standard: InfringementStandard::ChinaAllElementsPlusDoE,
        };
        let result = pipeline.analyze(&context);
        assert_eq!(result.overall_conclusion, InfringementType::NoInfringement);
    }

    #[test]
    fn matching_features_yield_literal_infringement() {
        let pipeline = InfringementPipeline::new();
        let context = InfringementContext {
            patent_claims: vec!["其特征在于组件A；组件B".into()],
            accused_product: CompareDocument {
                features: vec![
                    CompareFeature {
                        id: "f1".into(),
                        description: "组件A".into(),
                    },
                    CompareFeature {
                        id: "f2".into(),
                        description: "组件B".into(),
                    },
                ],
                ..Default::default()
            },
            legal_standard: InfringementStandard::ChinaAllElementsPlusDoE,
        };
        let result = pipeline.analyze(&context);
        assert_eq!(result.overall_conclusion, InfringementType::Literal);
        assert!(result.opinion.contains("字面侵权"));
    }

    #[test]
    fn missing_features_no_infringement() {
        let pipeline = InfringementPipeline::new();
        let context = InfringementContext {
            patent_claims: vec!["其特征在于组件A；组件C".into()],
            accused_product: CompareDocument {
                features: vec![CompareFeature {
                    id: "f1".into(),
                    description: "组件A".into(),
                }],
                ..Default::default()
            },
            legal_standard: InfringementStandard::ChinaAllElementsPlusDoE,
        };
        let result = pipeline.analyze(&context);
        assert_eq!(result.overall_conclusion, InfringementType::NoInfringement);
    }
}
