//! 评估工具集。
//!
//! 提供行动审查、LLM 输出反思、忠实度评估、自一致性评估、G-Eval 等评估能力，
//! 用于评估 Agent 输出质量和行为正确性。

use serde::Deserialize;

/// 行动审查输入参数：比较预期与实际结果。
#[derive(Debug, Deserialize)]
pub struct ActionReviewInput {
    /// 执行的操作名称。
    pub action: String,
    /// 预期结果。
    pub expected: String,
    /// 实际结果。
    pub actual: String,
}

/// LLM 输出反思评估输入参数。
#[derive(Debug, Deserialize)]
pub struct LlmReflectionInput {
    /// LLM 输出文本。
    pub output: String,
    /// 评估标准列表。
    #[serde(default)]
    pub criteria: Vec<String>,
}

/// 忠实度评估输入参数：比较源文本与输出文本的内容一致性。
#[derive(Debug, Deserialize)]
pub struct FaithfulnessEvalInput {
    /// 源文本。
    pub source: String,
    /// 待评估的输出文本。
    pub output: String,
}

/// 自一致性评估输入参数：比较多个结果的相互一致性。
#[derive(Debug, Deserialize)]
pub struct SelfConsistencyEvalInput {
    /// 多个待比较的结果文本。
    #[serde(default)]
    pub results: Vec<String>,
}

/// 评分量规项定义。
#[derive(Debug, Deserialize)]
pub struct RubricItem {
    /// 评分标准名称。
    pub name: String,
    /// 权重。
    pub weight: f64,
}

/// G-Eval 综合评分输入参数。
#[derive(Debug, Deserialize)]
pub struct GEvalInput {
    /// 待评估的输出文本。
    pub output: String,
    /// 评分量规列表。
    #[serde(default)]
    pub rubric: Vec<RubricItem>,
}

/// 评估工具集。
pub struct EvaluationTools;

impl EvaluationTools {
    pub fn action_review(
        action: &str,
        expected: &str,
        actual: &str,
    ) -> Result<serde_json::Value, String> {
        let matches = actual.contains(expected);
        Ok(
            serde_json::json!({"action": action, "matches_expectation": matches, "expected": expected, "actual_summary": &actual[..actual.len().min(200)]}),
        )
    }

    pub fn llm_reflection(output: &str, criteria: &[&str]) -> Result<serde_json::Value, String> {
        let scores: Vec<serde_json::Value> = criteria
            .iter()
            .map(|c| serde_json::json!({"criterion": c, "met": output.contains(c)}))
            .collect();
        let met = scores
            .iter()
            .filter(|s| s["met"].as_bool().unwrap_or(false))
            .count();
        Ok(serde_json::json!({"total_criteria": criteria.len(), "met": met, "details": scores}))
    }

    pub fn faithfulness_eval(source: &str, output: &str) -> Result<serde_json::Value, String> {
        let s_words: std::collections::HashSet<_> = source.split_whitespace().collect();
        let o_words: std::collections::HashSet<_> = output.split_whitespace().collect();
        let overlap = s_words.intersection(&o_words).count();
        let ratio = if s_words.is_empty() {
            0.0
        } else {
            overlap as f64 / s_words.len() as f64
        };
        Ok(
            serde_json::json!({"faithfulness_score": ratio, "hallucination_risk": if ratio < 0.3 {"high"} else if ratio < 0.6 {"medium"} else {"low"}}),
        )
    }

    pub fn self_consistency_eval(results: &[String]) -> Result<serde_json::Value, String> {
        if results.len() <= 1 {
            return Ok(
                serde_json::json!({"consistency": 1.0, "samples": 1, "note": "需要至少2个结果"}),
            );
        }
        let mut pairs = 0;
        let mut similar = 0;
        for i in 0..results.len() {
            for j in i + 1..results.len() {
                pairs += 1;
                let a: std::collections::HashSet<_> = results[i].split_whitespace().collect();
                let b: std::collections::HashSet<_> = results[j].split_whitespace().collect();
                let overlap = a.intersection(&b).count();
                let union = a.union(&b).count();
                if union > 0 && (overlap as f64 / union as f64) > 0.5 {
                    similar += 1;
                }
            }
        }
        Ok(
            serde_json::json!({"consistency_score": if pairs > 0 {similar as f64 / pairs as f64} else {1.0}, "sample_count": results.len()}),
        )
    }

    pub fn g_eval(output: &str, rubric: &[(&str, f64)]) -> Result<serde_json::Value, String> {
        let scores: Vec<serde_json::Value> = rubric
            .iter()
            .map(|(name, weight)| {
                let score = if output.contains(name) {
                    *weight
                } else {
                    weight * 0.3
                };
                serde_json::json!({"criterion": name, "weight": weight, "score": score})
            })
            .collect();
        let total: f64 = scores
            .iter()
            .map(|s| s["score"].as_f64().unwrap_or(0.0))
            .sum();
        Ok(serde_json::json!({"total_score": total, "dimensions": scores}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn action_review_matches() {
        let result = EvaluationTools::action_review("search", "found", "result found ok").unwrap();
        assert_eq!(result["matches_expectation"], true);
        assert_eq!(result["action"], "search");
    }

    #[test]
    fn action_review_no_match() {
        let result = EvaluationTools::action_review("search", "xyz", "abc def").unwrap();
        assert_eq!(result["matches_expectation"], false);
    }

    #[test]
    fn llm_reflection_all_met() {
        let result =
            EvaluationTools::llm_reflection("quality accuracy completeness", &["quality", "accuracy"]).unwrap();
        assert_eq!(result["total_criteria"], 2);
        assert_eq!(result["met"], 2);
    }

    #[test]
    fn llm_reflection_none_met() {
        let result = EvaluationTools::llm_reflection("hello world", &["quality", "accuracy"]).unwrap();
        assert_eq!(result["met"], 0);
    }

    #[test]
    fn faithfulness_identical_text() {
        let result = EvaluationTools::faithfulness_eval("the quick brown fox", "the quick brown fox").unwrap();
        let score = result["faithfulness_score"].as_f64().unwrap();
        assert_eq!(score, 1.0);
        assert_eq!(result["hallucination_risk"], "low");
    }

    #[test]
    fn faithfulness_empty_source() {
        let result = EvaluationTools::faithfulness_eval("", "some output").unwrap();
        assert_eq!(result["faithfulness_score"], 0.0);
    }

    #[test]
    fn self_consistency_single_result() {
        let result = EvaluationTools::self_consistency_eval(&["only one".into()]).unwrap();
        assert_eq!(result["consistency"], 1.0);
        assert_eq!(result["samples"], 1);
    }

    #[test]
    fn self_consistency_identical_results() {
        let result = EvaluationTools::self_consistency_eval(&[
            "same text here".into(),
            "same text here".into(),
            "same text here".into(),
        ]).unwrap();
        let score = result["consistency_score"].as_f64().unwrap();
        assert!(score > 0.9, "identical results should have high consistency, got {score}");
    }

    #[test]
    fn g_eval_matching_criterion() {
        let result = EvaluationTools::g_eval("quality accuracy output", &[("quality", 1.0), ("accuracy", 0.5)]).unwrap();
        let total = result["total_score"].as_f64().unwrap();
        assert!(total > 0.0, "matching criteria should yield positive score, got {total}");
    }
}

pub fn register_evaluation_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("ActionReview".into(), |input| {
        Box::pin(async move {
            let parsed: ActionReviewInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            EvaluationTools::action_review(&parsed.action, &parsed.expected, &parsed.actual)
        })
    });
    t.insert("LlmReflection".into(), |input| {
        Box::pin(async move {
            let parsed: LlmReflectionInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let criteria: Vec<&str> = parsed.criteria.iter().map(|s| s.as_str()).collect();
            EvaluationTools::llm_reflection(&parsed.output, &criteria)
        })
    });
    t.insert("FaithfulnessEval".into(), |input| {
        Box::pin(async move {
            let parsed: FaithfulnessEvalInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            EvaluationTools::faithfulness_eval(&parsed.source, &parsed.output)
        })
    });
    t.insert("SelfConsistencyEval".into(), |input| {
        Box::pin(async move {
            let parsed: SelfConsistencyEvalInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            EvaluationTools::self_consistency_eval(&parsed.results)
        })
    });
    t.insert("GEval".into(), |input| {
        Box::pin(async move {
            let parsed: GEvalInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let rubric: Vec<(&str, f64)> = parsed
                .rubric
                .iter()
                .map(|r| (r.name.as_str(), r.weight))
                .collect();
            EvaluationTools::g_eval(&parsed.output, &rubric)
        })
    });
    t
}
