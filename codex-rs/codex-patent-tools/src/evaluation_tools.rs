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

pub fn register_evaluation_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("ActionReview".into(), |input| {
        Box::pin(async move {
            let action = input.get("action").and_then(|v| v.as_str()).unwrap_or("");
            let expected = input.get("expected").and_then(|v| v.as_str()).unwrap_or("");
            let actual = input.get("actual").and_then(|v| v.as_str()).unwrap_or("");
            EvaluationTools::action_review(action, expected, actual)
        })
    });
    t.insert("LlmReflection".into(), |input| {
        Box::pin(async move {
            let output = input.get("output").and_then(|v| v.as_str()).unwrap_or("");
            let criteria = input
                .get("criteria")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();
            EvaluationTools::llm_reflection(output, &criteria)
        })
    });
    t.insert("FaithfulnessEval".into(), |input| {
        Box::pin(async move {
            let source = input.get("source").and_then(|v| v.as_str()).unwrap_or("");
            let output = input.get("output").and_then(|v| v.as_str()).unwrap_or("");
            EvaluationTools::faithfulness_eval(source, output)
        })
    });
    t.insert("SelfConsistencyEval".into(), |input| {
        Box::pin(async move {
            let results = input
                .get("results")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            EvaluationTools::self_consistency_eval(&results)
        })
    });
    t.insert("GEval".into(), |input| {
        Box::pin(async move {
            let output = input.get("output").and_then(|v| v.as_str()).unwrap_or("");
            let rubric = input
                .get("rubric")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            let name = v.get("name")?.as_str()?;
                            let weight = v.get("weight")?.as_f64()?;
                            Some((name, weight))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            EvaluationTools::g_eval(output, &rubric)
        })
    });
    t
}
