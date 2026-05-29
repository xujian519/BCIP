use serde::Deserialize;
use codex_patent_domain::compare;

#[derive(Debug, Deserialize)]
pub struct SemanticCompareInput {
    pub text_a: String,
    pub text_b: String,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SynergyAnalysisInput {
    pub features: Vec<String>,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct HighCitationInput {
    pub patent_number: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SuccessPredictorInput {
    pub rejection_type: String,
    pub has_differences: Option<bool>,
    pub has_technical_effect: Option<bool>,
    pub argument_count: Option<usize>,
}

pub struct AdvancedAnalysisTools;

impl AdvancedAnalysisTools {
    pub fn semantic_compare(input: SemanticCompareInput) -> Result<serde_json::Value, String> {
        let mode = input.mode.unwrap_or_else(|| "hybrid".to_string());

        let lexical = compare::lexical_similarity(&input.text_a, &input.text_b);

        let a_sentences: Vec<&str> = input.text_a.split(|c| c == '。' || c == '；' || c == ';').collect();
        let b_sentences: Vec<&str> = input.text_b.split(|c| c == '。' || c == '；' || c == ';').collect();
        let structural = if a_sentences.is_empty() || b_sentences.is_empty() {
            0.0
        } else {
            let ratio = a_sentences.len().min(b_sentences.len()) as f64 / a_sentences.len().max(b_sentences.len()) as f64;
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
                let both_in_desc = input.description.contains(&input.features[i]) && input.description.contains(&input.features[j]);
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
            synergy_pairs.iter().filter_map(|p| p["pair_synergy"].as_f64()).sum::<f64>() / synergy_pairs.len() as f64
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
        let limit = input.limit.unwrap_or(20);
        Ok(serde_json::json!({
            "patent_number": input.patent_number,
            "query": format!("https://patents.google.com/?q={}&num={}", input.patent_number, limit),
            "message": "请使用 GooglePatentsFetch 工具检索引用该专利的后续专利",
            "estimated_citations": "需要通过外部检索获取",
        }))
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