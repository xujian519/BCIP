//! 侵权分析流水线
//!
//! 全流程：权利要求解析 → 特征对比 → 全面覆盖原则 → 等同原则 → 判定结论

use crate::claim_parser;
use codex_patent_core::*;
use serde::{Deserialize, Serialize};

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

/// 执行全流程侵权分析
pub fn analyze_infringement(context: &InfringementContext) -> InfringementResult {
    let mut claim_results = Vec::new();
    let mut any_infringement = false;

    for (i, claim_text) in context.patent_claims.iter().enumerate() {
        let parsed = claim_parser::parse((i + 1) as u32, claim_text);
        let feature_texts: Vec<String> = parsed
            .features
            .iter()
            .map(|f| f.description.clone())
            .collect();
        let result = analyze_single_claim(
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
            .find(|pf| claim_parser::feature_text_similarity(cf, &pf.description) >= 0.6);

        match match_found {
            Some(pf) => {
                let sim = claim_parser::feature_text_similarity(cf, &pf.description);
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

// ── 等同侵权分析 ──

/// 等同类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EquivalenceType {
    /// 已知替换手段
    KnownSubstitution,
    /// 相同功能/方式/结果
    SameFunctionWayResult,
    /// 非实质性差异
    InsubstantialDifference,
}

/// 等同特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalentFeature {
    pub claim_feature: String,
    pub accused_feature: String,
    pub equivalence_type: EquivalenceType,
    pub reasoning: String,
    pub confidence: f64,
}

/// 字面侵权中的逐特征匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteralFeatureMatch {
    pub claim_feature: String,
    pub matched_in_accused: String,
    pub similarity: f64,
}

/// 字面侵权结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteralResult {
    pub all_features_covered: bool,
    pub missing_features: Vec<String>,
    pub matching_features: Vec<LiteralFeatureMatch>,
}

/// 等同侵权结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceResult {
    pub equivalent_features: Vec<EquivalentFeature>,
    pub non_equivalent_features: Vec<String>,
}

/// 禁止反悔结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstoppelResult {
    pub estoppel_applies: bool,
    pub surrendered_subject_matter: Vec<String>,
}

/// 侵权结论
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InfringementConclusion {
    LiteralInfringement,
    EquivalenceInfringement,
    NoInfringement,
    Indeterminate,
}

/// 综合侵权分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceAnalysis {
    pub literal_infringement: LiteralResult,
    pub doctrine_of_equivalents: EquivalenceResult,
    pub prosecution_history_estoppel: Option<EstoppelResult>,
    pub overall: InfringementConclusion,
}

/// 全面侵权分析
pub fn analyze_infringement_comprehensive(
    claim_text: &str,
    accused_description: &str,
) -> EquivalenceAnalysis {
    let claim = claim_parser::parse(1, claim_text);
    let accused = claim_parser::parse(2, accused_description);

    // 第一层：字面侵权
    let literal = analyze_literal(&claim, &accused);

    // 第二层：等同侵权
    let equivalence = analyze_equivalence_layer(&claim, &accused);

    // 综合判断
    let overall = determine_conclusion(&literal, &equivalence);

    EquivalenceAnalysis {
        literal_infringement: literal,
        doctrine_of_equivalents: equivalence,
        prosecution_history_estoppel: None,
        overall,
    }
}

fn analyze_literal(claim: &ParsedClaim, accused: &ParsedClaim) -> LiteralResult {
    let mut matching = Vec::new();
    let mut missing = Vec::new();

    for cf in &claim.features {
        let best = accused
            .features
            .iter()
            .map(|af| (af, claim_parser::feature_similarity(cf, af)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((af, score)) = best {
            if score > 0.7 {
                matching.push(LiteralFeatureMatch {
                    claim_feature: cf.description.clone(),
                    matched_in_accused: af.description.clone(),
                    similarity: score,
                });
            } else {
                missing.push(cf.description.clone());
            }
        } else {
            missing.push(cf.description.clone());
        }
    }

    LiteralResult {
        all_features_covered: missing.is_empty(),
        missing_features: missing,
        matching_features: matching,
    }
}

fn analyze_equivalence_layer(claim: &ParsedClaim, accused: &ParsedClaim) -> EquivalenceResult {
    let mut equivalent = Vec::new();
    let mut non_equivalent = Vec::new();

    for cf in &claim.features {
        let best = accused
            .features
            .iter()
            .map(|af| (af, claim_parser::feature_similarity(cf, af)))
            .filter(|(_, s)| *s > 0.4)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((af, score)) = best {
            let eq_type = if score > 0.8 {
                EquivalenceType::KnownSubstitution
            } else if score > 0.6 {
                EquivalenceType::SameFunctionWayResult
            } else {
                EquivalenceType::InsubstantialDifference
            };
            equivalent.push(EquivalentFeature {
                claim_feature: cf.description.clone(),
                accused_feature: af.description.clone(),
                equivalence_type: eq_type,
                reasoning: format!("特征相似度 {:.0}%", score * 100.0),
                confidence: score,
            });
        } else {
            non_equivalent.push(cf.description.clone());
        }
    }

    EquivalenceResult {
        equivalent_features: equivalent,
        non_equivalent_features: non_equivalent,
    }
}

fn determine_conclusion(
    literal: &LiteralResult,
    equivalence: &EquivalenceResult,
) -> InfringementConclusion {
    if literal.all_features_covered {
        InfringementConclusion::LiteralInfringement
    } else if equivalence.non_equivalent_features.is_empty() {
        InfringementConclusion::EquivalenceInfringement
    } else if !equivalence.equivalent_features.is_empty() {
        InfringementConclusion::Indeterminate
    } else {
        InfringementConclusion::NoInfringement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_claims_results_in_no_infringement() {
        let context = InfringementContext {
            patent_claims: vec![],
            accused_product: CompareDocument::default(),
            legal_standard: InfringementStandard::ChinaAllElementsPlusDoE,
        };
        let result = analyze_infringement(&context);
        assert_eq!(result.overall_conclusion, InfringementType::NoInfringement);
    }

    #[test]
    fn matching_features_yield_literal_infringement() {
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
        let result = analyze_infringement(&context);
        assert_eq!(result.overall_conclusion, InfringementType::Literal);
        assert!(result.opinion.contains("字面侵权"));
    }

    #[test]
    fn missing_features_no_infringement() {
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
        let result = analyze_infringement(&context);
        assert_eq!(result.overall_conclusion, InfringementType::NoInfringement);
    }

    #[test]
    fn test_literal_infringement_found() {
        // Identical features → literal infringement
        let result = analyze_infringement_comprehensive(
            "一种装置，其特征在于包括壳体；处理器",
            "一种装置，其特征在于包括壳体；处理器",
        );
        assert_eq!(result.overall, InfringementConclusion::LiteralInfringement);
        assert!(result.literal_infringement.all_features_covered);
        assert!(result.literal_infringement.missing_features.is_empty());
    }

    #[test]
    fn test_equivalence_analysis() {
        // Similar but not identical features → equivalence
        let result = analyze_infringement_comprehensive(
            "一种装置，其特征在于包括壳体；散热模块",
            "一种装置，其特征在于包括壳体；冷却单元",
        );
        // "散热模块" vs "冷却单元" should produce at least an indeterminate result
        assert_ne!(result.overall, InfringementConclusion::LiteralInfringement);
        assert!(
            !result
                .doctrine_of_equivalents
                .equivalent_features
                .is_empty()
                || !result.literal_infringement.matching_features.is_empty()
        );
    }

    #[test]
    fn test_no_infringement() {
        // Completely different features
        let result = analyze_infringement_comprehensive(
            "一种装置，其特征在于包括壳体；处理器；散热模块；显示屏；电池",
            "一种装置，其特征在于包括发动机；轮胎；方向盘；座椅",
        );
        assert_eq!(result.overall, InfringementConclusion::NoInfringement);
    }
}
