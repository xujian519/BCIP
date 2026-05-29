use serde::Deserialize;
use codex_patent_domain::claim_parser::ClaimParser;
use codex_patent_domain::rule_engine::QualitativeRuleEngine;
use codex_patent_domain::compare::FeatureMatcher;
use codex_patent_knowledge::{SearchConfig, UnifiedSearch};
use codex_patent_core::{CaseContext, FeatureType, ParsedFeature, CompareFeature};

#[derive(Debug, Deserialize)] pub struct ClaimParseInput { pub claim_text: String, pub claim_number: u32 }
#[derive(Debug, Deserialize)] pub struct ClaimCompareInput { pub claim_a: String, pub claim_b: String }
#[derive(Debug, Deserialize)] pub struct NoveltyAnalysisInput { pub invention_description: String, pub prior_art_descriptions: Option<Vec<String>>, pub differences: Option<Vec<String>> }
#[derive(Debug, Deserialize)] pub struct InventivenessAnalysisInput { pub invention_description: String, pub technical_effect: Option<String>, pub performance_improvement: Option<f64>, pub obviousness: Option<bool> }
#[derive(Debug, Deserialize)] pub struct InfringementAnalysisInput { pub claim_text: String, pub accused_product_description: String }
#[derive(Debug, Deserialize)] pub struct LegalQAInput { pub question: String }
#[derive(Debug, Deserialize)] pub struct KnowledgeSearchInput { pub query: String, pub limit: Option<usize> }

pub struct AnalysisTools;

impl AnalysisTools {
    pub fn claim_parse(input: ClaimParseInput) -> Result<serde_json::Value, String> {
        let parser = ClaimParser::new();
        let result = parser.parse(input.claim_number, &input.claim_text);
        serde_json::to_value(result).map_err(|e| format!("{e}"))
    }

    pub fn claim_compare(input: ClaimCompareInput) -> Result<serde_json::Value, String> {
        let a = ParsedFeature { id: "A".into(), description: input.claim_a, feature_type: FeatureType::Element, component: None, parameters: vec![] };
        let b = ParsedFeature { id: "B".into(), description: input.claim_b, feature_type: FeatureType::Element, component: None, parameters: vec![] };
        let sim = ClaimParser::feature_similarity(&a, &b);
        let corr = ClaimParser::classify_correspondence(sim);
        Ok(serde_json::json!({"similarity": sim, "correspondence": format!("{corr:?}")}))
    }

    pub fn novelty_analysis(input: NoveltyAnalysisInput) -> Result<serde_json::Value, String> {
        let mut engine = QualitativeRuleEngine::new();
        let prior = input.prior_art_descriptions.unwrap_or_default().join("; ");
        let ctx = CaseContext {
            invention: Some(input.invention_description.clone()),
            prior_art_contains_all: Some(!prior.is_empty()),
            differences: input.differences,
            technical_effect: None, performance_improvement: None, obviousness: None,
            rejection_type: None, technical_effects: None, prior_art_different_field: None,
        };
        let rule_result = engine.analyze_novelty(&ctx).map_err(|e| format!("{e}"))?;
        Ok(serde_json::json!({"rule_engine": rule_result, "text_analysis": "perform_novelty_analysis功能需要LegalReasoningEngine实例"}))
    }

    pub fn inventiveness_analysis(input: InventivenessAnalysisInput) -> Result<serde_json::Value, String> {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            invention: Some(input.invention_description),
            prior_art_contains_all: None, differences: None,
            technical_effect: input.technical_effect,
            performance_improvement: input.performance_improvement,
            obviousness: input.obviousness,
            rejection_type: None, technical_effects: None, prior_art_different_field: None,
        };
        let r = engine.analyze_inventiveness(&ctx).map_err(|e| format!("{e}"))?;
        serde_json::to_value(r).map_err(|e| format!("{e}"))
    }

    pub fn infringement_analysis(input: InfringementAnalysisInput) -> Result<serde_json::Value, String> {
        let parser = ClaimParser::new();
        let claim = parser.parse(1, &input.claim_text);
        let target: Vec<CompareFeature> = claim.features.iter().map(|f| CompareFeature { id: f.id.clone(), description: f.description.clone() }).collect();
        let prior = vec![CompareFeature { id: "P1".into(), description: input.accused_product_description }];
        serde_json::to_value(FeatureMatcher::compare(&target, &prior)).map_err(|e| format!("{e}"))
    }

    pub fn legal_qa(input: LegalQAInput) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"question": input.question, "response_type": "knowledge_based", "message": "请通过知识库搜索获取详细法律依据"}))
    }

    pub fn knowledge_search(input: KnowledgeSearchInput) -> Result<serde_json::Value, String> {
        let search = UnifiedSearch::new(
            Some("../codex-patent-assets/patent_kg.db"),
            Some("../codex-patent-assets/laws.db"),
            Some("../codex-patent-assets/card-index.json"),
        );
        let config = SearchConfig { query: input.query, limit: input.limit.unwrap_or(10), ..Default::default() };
        serde_json::to_value(search.search(&config)).map_err(|e| format!("{e}"))
    }
}