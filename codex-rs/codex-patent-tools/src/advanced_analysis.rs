//! 高级分析工具集。
//!
//! 提供语义对比、技术特征协同分析、高引专利检索、OA 成功率预测等高级分析能力。

use codex_patent_domain::compare;
use serde::Deserialize;

/// 语义对比输入参数。
#[derive(Debug, Deserialize)]
pub struct SemanticCompareInput {
    /// 第一段待比较文本。
    pub text_a: String,
    /// 第二段待比较文本。
    pub text_b: String,
    /// 对比模式：lexical / structural / hybrid（默认）。
    pub mode: Option<String>,
}

/// 技术特征协同作用分析输入参数。
#[derive(Debug, Deserialize)]
pub struct SynergyAnalysisInput {
    /// 技术特征列表。
    pub features: Vec<String>,
    /// 技术方案整体描述。
    pub description: String,
}

/// 高引专利检索输入参数。
#[derive(Debug, Deserialize)]
pub struct HighCitationInput {
    /// 目标专利号码。
    pub patent_number: String,
    /// 返回结果数量上限。
    pub limit: Option<usize>,
}

/// OA 答复成功率预测输入参数。
#[derive(Debug, Deserialize)]
pub struct SuccessPredictorInput {
    /// 驳回类型（如 "novelty", "inventiveness" 等）。
    pub rejection_type: String,
    /// 是否具备区别技术特征。
    pub has_differences: Option<bool>,
    /// 是否具备技术效果。
    pub has_technical_effect: Option<bool>,
    /// 论证次数。
    pub argument_count: Option<usize>,
}

/// 高级分析工具集。
pub struct AdvancedAnalysisTools;

impl AdvancedAnalysisTools {
    pub fn semantic_compare(input: SemanticCompareInput) -> Result<serde_json::Value, String> {
        let mode = input.mode.unwrap_or_else(|| "hybrid".to_string());

        let lexical = compare::lexical_similarity(&input.text_a, &input.text_b);

        let a_sentences: Vec<&str> = input.text_a.split(['。', '；', ';']).collect();
        let b_sentences: Vec<&str> = input.text_b.split(['。', '；', ';']).collect();
        let structural = if a_sentences.is_empty() || b_sentences.is_empty() {
            0.0
        } else {
            let ratio = a_sentences.len().min(b_sentences.len()) as f64
                / a_sentences.len().max(b_sentences.len()) as f64;
            let mut match_count = 0;
            for a in &a_sentences {
                for b in &b_sentences {
                    if compare::lexical_similarity(a, b) > 0.5 {
                        match_count += 1;
                        break;
                    }
                }
            }
            (match_count as f64 / a_sentences.len() as f64) * 0.5 + ratio * 0.5
        };

        let hybrid = match mode.as_str() {
            "lexical" => lexical,
            "structural" => structural,
            _ => lexical * 0.6 + structural * 0.4,
        };

        Ok(serde_json::json!({
            "lexical_similarity": lexical,
            "structural_similarity": structural,
            "hybrid_score": hybrid,
            "mode": mode,
        }))
    }

    pub fn synergy_analysis(input: SynergyAnalysisInput) -> Result<serde_json::Value, String> {
        if input.features.len() < 2 {
            return Ok(serde_json::json!({
                "has_synergy": false,
                "reason": "需要至少2个技术特征才能分析协同作用",
                "synergy_score": 0.0,
            }));
        }

        let mut synergy_pairs = Vec::new();
        for i in 0..input.features.len() {
            for j in i + 1..input.features.len() {
                let sim = compare::lexical_similarity(&input.features[i], &input.features[j]);
                let both_in_desc = input.description.contains(&input.features[i])
                    && input.description.contains(&input.features[j]);
                let synergy = if both_in_desc && sim < 0.5 {
                    0.8
                } else if both_in_desc && sim >= 0.5 {
                    0.4
                } else {
                    0.2
                };
                synergy_pairs.push(serde_json::json!({
                    "feature_a": input.features[i],
                    "feature_b": input.features[j],
                    "pair_synergy": synergy,
                }));
            }
        }

        let avg_synergy: f64 = if synergy_pairs.is_empty() {
            0.0
        } else {
            synergy_pairs
                .iter()
                .filter_map(|p| p["pair_synergy"].as_f64())
                .sum::<f64>()
                / synergy_pairs.len() as f64
        };

        Ok(serde_json::json!({
            "has_synergy": avg_synergy > 0.5,
            "synergy_score": avg_synergy,
            "feature_count": input.features.len(),
            "pairs_analyzed": synergy_pairs.len(),
            "details": synergy_pairs,
        }))
    }

    pub fn high_citation_patents(input: HighCitationInput) -> Result<serde_json::Value, String> {
        // 实际前向引用检索逻辑已移至 register_analysis_tools 的 HighCitationSearch handler，
        // 通过 crate::google_patents::fetch_google_patents 异步执行。
        // 此同步方法仅作为备用入口。
        Err(format!(
            "请通过 HighCitationSearch 工具调用，该工具会使用 Google Patents 执行 citedby:{} 查询",
            input.patent_number
        ))
    }

    pub fn success_predictor(input: SuccessPredictorInput) -> Result<serde_json::Value, String> {
        let mut score = 50.0;

        match input.rejection_type.as_str() {
            "novelty" | "LackOfNovelty" => score -= 15.0,
            "inventiveness" | "Inventiveness" => score -= 10.0,
            "Obviousness" => score -= 8.0,
            "InsufficientDisclosure" => score -= 5.0,
            "UnpatentableSubject" => score -= 20.0,
            _ => {}
        }

        if input.has_differences.unwrap_or(false) {
            score += 20.0;
        }
        if input.has_technical_effect.unwrap_or(false) {
            score += 15.0;
        }

        let arg_count = input.argument_count.unwrap_or(1);
        if arg_count >= 3 {
            score += 10.0;
        } else if arg_count >= 2 {
            score += 5.0;
        }

        let probability = f64::clamp(score / 100.0, 0.0, 1.0);
        let assessment = if probability > 0.7 {
            "likely_success"
        } else if probability > 0.4 {
            "possible"
        } else {
            "challenging"
        };

        Ok(serde_json::json!({
            "success_probability": probability,
            "assessment": assessment,
            "score_breakdown": {
                "base": 50.0,
                "rejection_type_impact": match input.rejection_type.as_str() {
                    "novelty" | "LackOfNovelty" => -15.0,
                    "inventiveness" | "Inventiveness" => -10.0,
                    "Obviousness" => -8.0,
                    "InsufficientDisclosure" => -5.0,
                    "UnpatentableSubject" => -20.0,
                    _ => 0.0,
                },
                "differences_bonus": if input.has_differences.unwrap_or(false) { 20.0 } else { 0.0 },
                "technical_effect_bonus": if input.has_technical_effect.unwrap_or(false) { 15.0 } else { 0.0 },
                "argument_quality": match input.argument_count.unwrap_or(1) {
                    n if n >= 3 => 10.0,
                    n if n >= 2 => 5.0,
                    _ => 0.0,
                },
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn semantic_compare_lexical_identical() {
        let input = SemanticCompareInput {
            text_a: "一种传感器装置".into(),
            text_b: "一种传感器装置".into(),
            mode: Some("lexical".into()),
        };
        let result = AdvancedAnalysisTools::semantic_compare(input).unwrap();
        assert_eq!(result["mode"], "lexical");
        let lex = result["lexical_similarity"].as_f64().unwrap();
        assert!(lex > 0.9, "identical text should have high lexical similarity, got {lex}");
    }

    #[test]
    fn semantic_compare_hybrid_default() {
        let input = SemanticCompareInput {
            text_a: "技术方案A包含特征X。".into(),
            text_b: "完全不同的文本Y。".into(),
            mode: None,
        };
        let result = AdvancedAnalysisTools::semantic_compare(input).unwrap();
        assert_eq!(result["mode"], "hybrid");
        let hybrid = result["hybrid_score"].as_f64().unwrap();
        assert!(hybrid < 1.0, "different text should have lower hybrid score, got {hybrid}");
    }

    #[test]
    fn synergy_analysis_less_than_two_features() {
        let input = SynergyAnalysisInput {
            features: vec!["仅一个特征".into()],
            description: "描述文本".into(),
        };
        let result = AdvancedAnalysisTools::synergy_analysis(input).unwrap();
        assert_eq!(result["has_synergy"], false);
        assert_eq!(result["synergy_score"], 0.0);
    }

    #[test]
    fn synergy_analysis_features_in_description() {
        let input = SynergyAnalysisInput {
            features: vec!["特征Alpha".into(), "特征Beta".into()],
            description: "本方案包含特征Alpha和特征Beta的协同作用".into(),
        };
        let result = AdvancedAnalysisTools::synergy_analysis(input).unwrap();
        assert_eq!(result["feature_count"], 2);
        assert_eq!(result["pairs_analyzed"], 1);
    }

    #[test]
    fn success_predictor_novelty_with_bonuses() {
        let input = SuccessPredictorInput {
            rejection_type: "novelty".into(),
            has_differences: Some(true),
            has_technical_effect: Some(true),
            argument_count: Some(3),
        };
        let result = AdvancedAnalysisTools::success_predictor(input).unwrap();
        let prob = result["success_probability"].as_f64().unwrap();
        assert!(prob > 0.5, "with differences + effect + 3 args, should be > 0.5, got {prob}");
        assert_eq!(result["assessment"], "likely_success");
    }

    #[test]
    fn success_predictor_unknown_rejection_no_bonuses() {
        let input = SuccessPredictorInput {
            rejection_type: "UnknownType".into(),
            has_differences: Some(false),
            has_technical_effect: Some(false),
            argument_count: Some(1),
        };
        let result = AdvancedAnalysisTools::success_predictor(input).unwrap();
        let prob = result["success_probability"].as_f64().unwrap();
        assert!((0.3..=0.6).contains(&prob), "base case should be around 0.5, got {prob}");
        assert_eq!(result["score_breakdown"]["rejection_type_impact"], 0.0);
    }
}
