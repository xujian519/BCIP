use serde::Deserialize;
use codex_patent_core::CaseContext;
use codex_patent_domain::rule_engine::QualitativeRuleEngine;

#[derive(Debug, Deserialize)] pub struct SpecificationInput { pub title: String, pub technical_field: String, pub background: String, pub invention_content: String, pub embodiments: String }
#[derive(Debug, Deserialize)] pub struct ClaimGeneratorInput { pub invention_name: String, pub essential_features: Vec<String>, pub optional_features: Option<Vec<Vec<String>>> }
#[derive(Debug, Deserialize)] pub struct AbstractDraftInput { pub title: String, pub technical_problem: String, pub technical_solution: String, pub technical_effect: String }

pub struct DraftingTools;

impl DraftingTools {
    pub fn specification_draft(input: SpecificationInput) -> Result<serde_json::Value, String> {
        let spec = format!("说明书\n\n技术领域\n{}\n\n背景技术\n{}\n\n发明内容\n{}\n\n具体实施方式\n{}", input.technical_field, input.background, input.invention_content, input.embodiments);
        Ok(serde_json::json!({"title": input.title, "specification": spec, "word_count": spec.len()}))
    }

    pub fn claim_generator(input: ClaimGeneratorInput) -> Result<serde_json::Value, String> {
        let ind = format!("一种{}，其特征在于，包括：{}。", input.invention_name, input.essential_features.join("；"));
        let deps: Vec<String> = input.optional_features.unwrap_or_default().iter().enumerate().map(|(i, f)| format!("根据权利要求{}所述的{}，其特征在于，还包括：{}。", i+1, input.invention_name, f.join("；"))).collect();
        let mut all = vec![ind]; all.extend(deps);
        Ok(serde_json::json!({"claims": all, "independent_count": 1, "dependent_count": all.len()-1}))
    }

    pub fn abstract_draft(input: AbstractDraftInput) -> Result<serde_json::Value, String> {
        let text = format!("本发明公开了一种{}，解决{}的技术问题，方案是{}，达到{}的效果。", input.title, input.technical_problem, input.technical_solution, input.technical_effect);
        Ok(serde_json::json!({"abstract_text": text, "word_count": text.len()}))
    }

    pub fn innovation_evaluator(invention: String, effect: Option<String>, improvement: Option<f64>, obvious: Option<bool>) -> Result<serde_json::Value, String> {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext { invention: Some(invention), prior_art_contains_all: None, differences: None, technical_effect: effect, performance_improvement: improvement, obviousness: obvious, rejection_type: None, technical_effects: None, prior_art_different_field: None };
        let r = engine.analyze_inventiveness(&ctx).map_err(|e| format!("{e}"))?;
        let level = if r.net_score > 0.7 { "高" } else if r.net_score > 0.4 { "中" } else { "低" };
        Ok(serde_json::json!({"innovation_level": level, "score": r.net_score, "analysis": r}))
    }
}