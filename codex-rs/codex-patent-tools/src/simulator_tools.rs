use codex_patent_agents::scenario::ScenarioRegistry;
use codex_patent_core::CaseContext;
use codex_patent_domain::examiner_simulator::ExaminerSimulator;
use codex_patent_domain::oa_feedback::FeedbackAnalyzer;
use codex_patent_domain::oa_feedback::FeedbackRecord;
use codex_patent_domain::oa_feedback::FeedbackType;
use codex_patent_domain::oa_pattern::OaResponseTrajectory;
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
        let mut simulator = ExaminerSimulator::new();
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
            invention: None,
            prior_art_contains_all: None,
            differences: input.differences,
            technical_effect: None,
            performance_improvement: None,
            obviousness: None,
            rejection_type: input.rejection_type,
            technical_effects: input.technical_effects,
            prior_art_different_field: None,
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
            registry.register_from_toml(rule_content)?;
        }
        Ok(())
    }
}
