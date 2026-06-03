use codex_patent_agents::scenario::ScenarioRegistry;
use codex_patent_core::CaseContext;
use codex_patent_domain::examiner_simulator::ExaminerSimulator;
use codex_patent_domain::oa_feedback::FeedbackAnalyzer;
use codex_patent_domain::oa_feedback::FeedbackRecord;
use codex_patent_domain::oa_feedback::FeedbackType;
use codex_patent_domain::oa_pattern::PatternExtractor;
use codex_patent_domain::rule_engine::QualitativeRuleEngine;
use serde::Deserialize;

pub struct SimulatorTools;

#[derive(Debug, Deserialize)]
pub struct ExaminerSimulateInput {
    pub oa_text: String,
    pub claims: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExaminerRespondInput {
    pub applicant_argument: String,
    pub rejection_type: Option<String>,
    pub round_number: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseEvaluateInput {
    pub response_text: String,
}

#[derive(Debug, Deserialize)]
pub struct RuleAnalysisInput {
    pub analysis_type: String,
    pub differences: Option<Vec<String>>,
    pub technical_effects: Option<Vec<String>>,
    pub rejection_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeedbackRecordInput {
    pub oa_id: String,
    pub patent_id: String,
    pub feedback_type: String,
    pub outcome: String,
    pub quality_score: f64,
    pub strategy_used: Option<String>,
    pub comments: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PatternExtractInput {
    pub min_support: Option<usize>,
    pub min_success_rate: Option<f64>,
}

impl SimulatorTools {
    pub fn examiner_simulate(input: ExaminerSimulateInput) -> Result<serde_json::Value, String> {
        let mut simulator = ExaminerSimulator::new();
        let prior_art_analysis = serde_json::json!({});
        let result =
            simulator.simulate_initial_review(&input.oa_text, &input.claims, &prior_art_analysis);
        Ok(result)
    }

    pub fn examiner_respond(input: ExaminerRespondInput) -> Result<serde_json::Value, String> {
        let simulator = ExaminerSimulator::new();
        if let Some(rt) = &input.rejection_type {
            let _ = ExaminerSimulator::detect_rejection_type(rt);
        }
        let prior_art_analysis = serde_json::json!({});
        let round = input.round_number.unwrap_or(1);
        let result = simulator.respond_to_applicant_argument(
            &input.applicant_argument,
            &prior_art_analysis,
            round,
        );
        Ok(result)
    }

    pub fn response_evaluate(input: ResponseEvaluateInput) -> Result<serde_json::Value, String> {
        let result = ExaminerSimulator::evaluate_final_response(&input.response_text);
        Ok(result)
    }

    pub fn rule_analysis(input: RuleAnalysisInput) -> Result<serde_json::Value, String> {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            differences: input.differences,
            rejection_type: input.rejection_type,
            technical_effects: input.technical_effects,
            ..Default::default()
        };
        match input.analysis_type.as_str() {
            "novelty" => {
                let r = engine.analyze_novelty(&ctx).map_err(|e| format!("{e}"))?;
                serde_json::to_value(r).map_err(|e| format!("{e}"))
            }
            "inventiveness" => {
                let r = engine
                    .analyze_inventiveness(&ctx)
                    .map_err(|e| format!("{e}"))?;
                serde_json::to_value(r).map_err(|e| format!("{e}"))
            }
            "oa_strategy" => {
                let r = engine
                    .suggest_oa_strategy(&ctx)
                    .map_err(|e| format!("{e}"))?;
                serde_json::to_value(r).map_err(|e| format!("{e}"))
            }
            other => Err(format!("unknown analysis_type: {other}")),
        }
    }

    pub fn feedback_record(input: FeedbackRecordInput) -> Result<serde_json::Value, String> {
        let feedback_type = match input.feedback_type.as_str() {
            "success" => FeedbackType::Success,
            "partial_success" => FeedbackType::PartialSuccess,
            "failure" => FeedbackType::Failure,
            "quality_issue" => FeedbackType::QualityIssue,
            other => return Err(format!("unknown feedback_type: {other}")),
        };
        let record = FeedbackRecord {
            feedback_id: format!("fb-{}", chrono::Utc::now().timestamp_millis()),
            oa_id: input.oa_id,
            patent_id: input.patent_id,
            feedback_type,
            outcome: input.outcome,
            quality_score: input.quality_score,
            strategy_used: input.strategy_used,
            comments: input.comments.unwrap_or_default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            analyzed: false,
        };
        let mut analyzer = FeedbackAnalyzer::new();
        analyzer.collect(&record);
        let suggestions = analyzer.generate_suggestions();
        Ok(serde_json::json!({
            "recorded": true,
            "feedback_id": record.feedback_id,
            "suggestions_count": suggestions.len(),
            "suggestions": suggestions,
        }))
    }

    pub fn pattern_extract(input: PatternExtractInput) -> Result<serde_json::Value, String> {
        let min_support = input.min_support.unwrap_or(3);
        let min_success_rate = input.min_success_rate.unwrap_or(0.6);
        let extractor = PatternExtractor::new(min_support, min_success_rate);
        let patterns = extractor.extract_patterns_for("all");
        Ok(serde_json::json!({
            "patterns_found": patterns.len(),
            "min_support": min_support,
            "min_success_rate": min_success_rate,
            "total_trajectories": extractor.total_count(),
            "patterns": patterns,
        }))
    }
}

pub struct ScenarioDispatchTools;

impl ScenarioDispatchTools {
    pub fn dispatch(task_type: &str) -> Result<serde_json::Value, String> {
        let mut registry = ScenarioRegistry::new();
        Self::load_builtin_rules(&mut registry)?;
        let Some(rule) = registry.find(task_type) else {
            let available: Vec<&str> = registry
                .list()
                .iter()
                .map(|r| r.scenario.task_type.as_str())
                .collect();
            return Ok(serde_json::json!({
                "found": false,
                "task_type": task_type,
                "available_scenarios": available,
            }));
        };
        let parallel_groups = rule.processing.parallel_groups();
        let total_steps: usize = parallel_groups.iter().map(|g| g.len()).sum();
        let groups_json: Vec<serde_json::Value> = parallel_groups
            .iter()
            .enumerate()
            .map(|(group_idx, group)| {
                let steps_json: Vec<serde_json::Value> = group
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "name": s.name,
                            "description": s.description,
                            "depends_on": s.depends_on,
                            "agent": s.agent,
                            "tool": s.tool,
                            "hitl": s.hitl,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "group": group_idx + 1,
                    "parallel": group.len() > 1,
                    "steps": steps_json,
                })
            })
            .collect();
        Ok(serde_json::json!({
            "found": true,
            "task_type": task_type,
            "rule_id": rule.scenario.rule_id,
            "domain": rule.scenario.domain,
            "phase": rule.scenario.phase,
            "agent_level": rule.scenario.agent_level,
            "total_steps": total_steps,
            "parallel_groups": groups_json,
            "legal_basis": rule.legal_basis.laws,
        }))
    }

    fn load_builtin_rules(registry: &mut ScenarioRegistry) -> Result<(), String> {
        let rules = [
            include_str!("../../codex-patent-agents/assets/scenario-rules/oa_strategy.toml"),
            include_str!("../../codex-patent-agents/assets/scenario-rules/novelty_analysis.toml"),
            include_str!(
                "../../codex-patent-agents/assets/scenario-rules/inventiveness_rejection.toml"
            ),
            include_str!(
                "../../codex-patent-agents/assets/scenario-rules/infringement_analysis.toml"
            ),
            include_str!("../../codex-patent-agents/assets/scenario-rules/quality_review.toml"),
        ];
        for rule_content in &rules {
            registry
                .register_from_toml(rule_content)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Input struct deserialization tests ---

    #[test]
    fn deserialize_examiner_simulate_input() {
        let json = serde_json::json!({
            "oa_text": "审查意见内容",
            "claims": ["权利要求1", "权利要求2"]
        });
        let input: ExaminerSimulateInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.oa_text, "审查意见内容");
        assert_eq!(input.claims.len(), 2);
    }

    #[test]
    fn deserialize_examiner_respond_input() {
        let json = serde_json::json!({
            "applicant_argument": "申请人答复",
            "rejection_type": "新颖性",
            "round_number": 2
        });
        let input: ExaminerRespondInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.applicant_argument, "申请人答复");
        assert_eq!(input.rejection_type.as_deref(), Some("新颖性"));
        assert_eq!(input.round_number, Some(2));
    }

    #[test]
    fn deserialize_examiner_respond_input_minimal() {
        let json = serde_json::json!({
            "applicant_argument": "简单答复"
        });
        let input: ExaminerRespondInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert!(input.rejection_type.is_none());
        assert!(input.round_number.is_none());
    }

    #[test]
    fn deserialize_response_evaluate_input() {
        let json = serde_json::json!({
            "response_text": "答复文本内容"
        });
        let input: ResponseEvaluateInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.response_text, "答复文本内容");
    }

    #[test]
    fn deserialize_rule_analysis_input() {
        let json = serde_json::json!({
            "analysis_type": "novelty",
            "differences": ["差异1"],
            "technical_effects": ["效果1"],
            "rejection_type": "新颖性"
        });
        let input: RuleAnalysisInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.analysis_type, "novelty");
        assert_eq!(input.differences.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn deserialize_rule_analysis_input_minimal() {
        let json = serde_json::json!({
            "analysis_type": "inventiveness"
        });
        let input: RuleAnalysisInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert!(input.differences.is_none());
        assert!(input.technical_effects.is_none());
        assert!(input.rejection_type.is_none());
    }

    #[test]
    fn deserialize_feedback_record_input() {
        let json = serde_json::json!({
            "oa_id": "OA001",
            "patent_id": "P001",
            "feedback_type": "success",
            "outcome": "授权",
            "quality_score": 0.85,
            "strategy_used": "修改权利要求",
            "comments": "效果良好"
        });
        let input: FeedbackRecordInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.oa_id, "OA001");
        assert_eq!(input.quality_score, 0.85);
    }

    #[test]
    fn deserialize_pattern_extract_input_defaults() {
        let json = serde_json::json!({});
        let input: PatternExtractInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert!(input.min_support.is_none());
        assert!(input.min_success_rate.is_none());
    }

    // --- examiner_simulate tests ---

    #[test]
    fn examiner_simulate_basic() {
        let input = ExaminerSimulateInput {
            oa_text: "该申请不符合新颖性要求".into(),
            claims: vec!["一种装置，包括特征A".into()],
        };
        let result = SimulatorTools::examiner_simulate(input).unwrap();
        // Should return a valid JSON result
        assert!(result.is_object());
    }

    #[test]
    fn examiner_simulate_multiple_claims() {
        let input = ExaminerSimulateInput {
            oa_text: "审查意见".into(),
            claims: vec![
                "一种装置，包括特征A".into(),
                "根据权利要求1所述的装置，还包括特征B".into(),
            ],
        };
        let result = SimulatorTools::examiner_simulate(input).unwrap();
        assert!(result.is_object());
    }

    // --- examiner_respond tests ---

    #[test]
    fn examiner_respond_basic() {
        let input = ExaminerRespondInput {
            applicant_argument: "申请人认为具有创造性".into(),
            rejection_type: None,
            round_number: None,
        };
        let result = SimulatorTools::examiner_respond(input).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn examiner_respond_with_rejection_type() {
        let input = ExaminerRespondInput {
            applicant_argument: "申请人修改了权利要求".into(),
            rejection_type: Some("创造性".into()),
            round_number: Some(2),
        };
        let result = SimulatorTools::examiner_respond(input).unwrap();
        assert!(result.is_object());
    }

    // --- response_evaluate tests ---

    #[test]
    fn response_evaluate_basic() {
        let input = ResponseEvaluateInput {
            response_text: "答复：修改后的权利要求1具备新颖性和创造性".into(),
        };
        let result = SimulatorTools::response_evaluate(input).unwrap();
        assert!(result.is_object());
    }

    // --- rule_analysis tests ---

    #[test]
    fn rule_analysis_novelty() {
        let input = RuleAnalysisInput {
            analysis_type: "novelty".into(),
            differences: Some(vec!["差异A".into()]),
            technical_effects: None,
            rejection_type: None,
        };
        let result = SimulatorTools::rule_analysis(input).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn rule_analysis_inventiveness() {
        let input = RuleAnalysisInput {
            analysis_type: "inventiveness".into(),
            differences: Some(vec!["差异A".into(), "差异B".into()]),
            technical_effects: Some(vec!["效果1".into()]),
            rejection_type: Some("创造性".into()),
        };
        let result = SimulatorTools::rule_analysis(input).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn rule_analysis_oa_strategy() {
        let input = RuleAnalysisInput {
            analysis_type: "oa_strategy".into(),
            differences: None,
            technical_effects: None,
            rejection_type: Some("新颖性".into()),
        };
        let result = SimulatorTools::rule_analysis(input).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn rule_analysis_unknown_type() {
        let input = RuleAnalysisInput {
            analysis_type: "unknown_type".into(),
            differences: None,
            technical_effects: None,
            rejection_type: None,
        };
        let result = SimulatorTools::rule_analysis(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown analysis_type"));
    }

    // --- feedback_record tests ---

    #[test]
    fn feedback_record_success() {
        let input = FeedbackRecordInput {
            oa_id: "OA001".into(),
            patent_id: "P001".into(),
            feedback_type: "success".into(),
            outcome: "授权".into(),
            quality_score: 0.9,
            strategy_used: Some("修改权利要求".into()),
            comments: Some("效果良好".into()),
        };
        let result = SimulatorTools::feedback_record(input).unwrap();
        assert_eq!(result["recorded"], true);
        assert!(result["feedback_id"].as_str().unwrap().starts_with("fb-"));
    }

    #[test]
    fn feedback_record_partial_success() {
        let input = FeedbackRecordInput {
            oa_id: "OA002".into(),
            patent_id: "P002".into(),
            feedback_type: "partial_success".into(),
            outcome: "部分授权".into(),
            quality_score: 0.6,
            strategy_used: None,
            comments: None,
        };
        let result = SimulatorTools::feedback_record(input).unwrap();
        assert_eq!(result["recorded"], true);
    }

    #[test]
    fn feedback_record_unknown_type() {
        let input = FeedbackRecordInput {
            oa_id: "OA003".into(),
            patent_id: "P003".into(),
            feedback_type: "invalid_type".into(),
            outcome: "测试".into(),
            quality_score: 0.5,
            strategy_used: None,
            comments: None,
        };
        let result = SimulatorTools::feedback_record(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown feedback_type"));
    }

    // --- pattern_extract tests ---

    #[test]
    fn pattern_extract_defaults() {
        let input = PatternExtractInput {
            min_support: None,
            min_success_rate: None,
        };
        let result = SimulatorTools::pattern_extract(input).unwrap();
        assert_eq!(result["min_support"], 3);
        assert_eq!(result["min_success_rate"], 0.6);
    }

    #[test]
    fn pattern_extract_custom_params() {
        let input = PatternExtractInput {
            min_support: Some(5),
            min_success_rate: Some(0.8),
        };
        let result = SimulatorTools::pattern_extract(input).unwrap();
        assert_eq!(result["min_support"], 5);
        assert_eq!(result["min_success_rate"], 0.8);
    }

    // --- ScenarioDispatchTools::dispatch tests ---

    #[test]
    fn dispatch_known_scenario() {
        let result = ScenarioDispatchTools::dispatch("oa_strategy").unwrap();
        assert_eq!(result["found"], true);
        assert_eq!(result["task_type"], "oa_strategy");
        assert!(result["rule_id"].is_string());
        assert!(result["parallel_groups"].is_array());
        assert!(result["total_steps"].is_number());
    }

    #[test]
    fn dispatch_unknown_scenario() {
        let result = ScenarioDispatchTools::dispatch("nonexistent_task").unwrap();
        assert_eq!(result["found"], false);
        assert_eq!(result["task_type"], "nonexistent_task");
        assert!(result["available_scenarios"].is_array());
    }

    #[test]
    fn dispatch_novelty_analysis() {
        let result = ScenarioDispatchTools::dispatch("novelty_analysis").unwrap();
        assert_eq!(result["found"], true);
        assert_eq!(result["task_type"], "novelty_analysis");
    }

    #[test]
    fn dispatch_quality_review() {
        let result = ScenarioDispatchTools::dispatch("quality_review").unwrap();
        assert_eq!(result["found"], true);
    }
}

pub fn register_simulator_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("ExaminerSimulate".into(), |input| {
        Box::pin(async move {
            let parsed: ExaminerSimulateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::examiner_simulate(parsed)
        })
    });
    t.insert("ExaminerRespond".into(), |input| {
        Box::pin(async move {
            let parsed: ExaminerRespondInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::examiner_respond(parsed)
        })
    });
    t.insert("ResponseEvaluate".into(), |input| {
        Box::pin(async move {
            let parsed: ResponseEvaluateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::response_evaluate(parsed)
        })
    });
    t.insert("RuleAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: RuleAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::rule_analysis(parsed)
        })
    });
    t.insert("OaFeedbackRecord".into(), |input| {
        Box::pin(async move {
            let parsed: FeedbackRecordInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::feedback_record(parsed)
        })
    });
    t.insert("OaPatternExtract".into(), |input| {
        Box::pin(async move {
            let parsed: PatternExtractInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::pattern_extract(parsed)
        })
    });
    t.insert("ScenarioDispatch".into(), |input| {
        Box::pin(async move {
            let task_type = input
                .get("task_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            ScenarioDispatchTools::dispatch(task_type)
        })
    });
    t
}
