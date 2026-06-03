use codex_patent_core::CaseContext;
use codex_patent_core::CompareFeature;
use codex_patent_core::FeatureType;
use codex_patent_core::ParsedFeature;
use codex_patent_domain::claim_parser::ClaimParser;
use codex_patent_domain::compare::FeatureMatcher;
use codex_patent_domain::rule_engine::QualitativeRuleEngine;
use codex_patent_knowledge::SearchConfig;
use codex_patent_knowledge::SearchMode;
use codex_patent_knowledge::UnifiedSearch;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClaimParseInput {
    pub claim_text: String,
    pub claim_number: u32,
}
#[derive(Debug, Deserialize)]
pub struct ClaimCompareInput {
    pub claim_a: String,
    pub claim_b: String,
}
#[derive(Debug, Deserialize)]
pub struct NoveltyAnalysisInput {
    pub invention_description: String,
    pub prior_art_descriptions: Option<Vec<String>>,
    pub differences: Option<Vec<String>>,
}
#[derive(Debug, Deserialize)]
pub struct InventivenessAnalysisInput {
    pub invention_description: String,
    pub technical_effect: Option<String>,
    pub performance_improvement: Option<f64>,
    pub obviousness: Option<bool>,
}
#[derive(Debug, Deserialize)]
pub struct InfringementAnalysisInput {
    pub claim_text: String,
    pub accused_product_description: String,
}
#[derive(Debug, Deserialize)]
pub struct LegalQAInput {
    pub question: String,
}
#[derive(Debug, Deserialize)]
pub struct KnowledgeSearchInput {
    pub query: String,
    pub limit: Option<usize>,
    pub semantic: Option<bool>,
}

pub struct AnalysisTools;

impl AnalysisTools {
    pub fn claim_parse(input: ClaimParseInput) -> Result<serde_json::Value, String> {
        let parser = ClaimParser::new();
        let result = parser.parse(input.claim_number, &input.claim_text);
        serde_json::to_value(result).map_err(|e| format!("{e}"))
    }

    pub fn claim_compare(input: ClaimCompareInput) -> Result<serde_json::Value, String> {
        let a = ParsedFeature {
            id: "A".into(),
            description: input.claim_a,
            feature_type: FeatureType::Element,
            component: None,
            parameters: vec![],
        };
        let b = ParsedFeature {
            id: "B".into(),
            description: input.claim_b,
            feature_type: FeatureType::Element,
            component: None,
            parameters: vec![],
        };
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
            technical_effect: None,
            performance_improvement: None,
            obviousness: None,
            rejection_type: None,
            technical_effects: None,
            prior_art_different_field: None,
        };
        let rule_result = engine.analyze_novelty(&ctx).map_err(|e| format!("{e}"))?;
        Ok(
            serde_json::json!({"rule_engine": rule_result, "text_analysis": "perform_novelty_analysis功能需要LegalReasoningEngine实例"}),
        )
    }

    pub fn inventiveness_analysis(
        input: InventivenessAnalysisInput,
    ) -> Result<serde_json::Value, String> {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            invention: Some(input.invention_description),
            prior_art_contains_all: None,
            differences: None,
            technical_effect: input.technical_effect,
            performance_improvement: input.performance_improvement,
            obviousness: input.obviousness,
            rejection_type: None,
            technical_effects: None,
            prior_art_different_field: None,
        };
        let r = engine
            .analyze_inventiveness(&ctx)
            .map_err(|e| format!("{e}"))?;
        serde_json::to_value(r).map_err(|e| format!("{e}"))
    }

    pub fn infringement_analysis(
        input: InfringementAnalysisInput,
    ) -> Result<serde_json::Value, String> {
        let parser = ClaimParser::new();
        let claim = parser.parse(1, &input.claim_text);
        let target: Vec<CompareFeature> = claim
            .features
            .iter()
            .map(|f| CompareFeature {
                id: f.id.clone(),
                description: f.description.clone(),
            })
            .collect();
        let prior = vec![CompareFeature {
            id: "P1".into(),
            description: input.accused_product_description,
        }];
        serde_json::to_value(FeatureMatcher::compare(&target, &prior)).map_err(|e| format!("{e}"))
    }

    pub fn legal_qa(input: LegalQAInput) -> Result<serde_json::Value, String> {
        Ok(
            serde_json::json!({"question": input.question, "response_type": "knowledge_based", "message": "请通过知识库搜索获取详细法律依据"}),
        )
    }

    pub fn knowledge_search(input: KnowledgeSearchInput) -> Result<serde_json::Value, String> {
        let search = UnifiedSearch::with_vector(
            Some(&codex_patent_knowledge::paths::kg_db_path()),
            Some(&codex_patent_knowledge::paths::law_db_path()),
            Some(&codex_patent_knowledge::paths::card_index_path()),
            Some(&codex_patent_knowledge::paths::semantic_index_path()),
            Some("http://localhost:8009"),
            std::env::var("BCIP_MLX_API_KEY").ok().as_deref(),
            Some("bge-m3-mlx-8bit"),
        );
        let mode = if input.semantic.unwrap_or(false) {
            SearchMode::Hybrid
        } else {
            SearchMode::KeywordEnhanced
        };
        let config = SearchConfig {
            query: input.query,
            limit: input.limit.unwrap_or(10),
            mode,
            ..Default::default()
        };
        serde_json::to_value(search.search(&config)).map_err(|e| format!("{e}"))
    }
}

pub fn register_analysis_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("ClaimParse".into(), |input| {
        Box::pin(async move {
            let parsed: ClaimParseInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::claim_parse(parsed)
        })
    });
    t.insert("ClaimCompare".into(), |input| {
        Box::pin(async move {
            let parsed: ClaimCompareInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::claim_compare(parsed)
        })
    });
    t.insert("NoveltyAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: NoveltyAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::novelty_analysis(parsed)
        })
    });
    t.insert("InventivenessAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: InventivenessAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::inventiveness_analysis(parsed)
        })
    });
    t.insert("InfringementAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: InfringementAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::infringement_analysis(parsed)
        })
    });
    t.insert("InnovationEvaluator".into(), |input| {
        Box::pin(async move {
            let invention = input
                .get("invention_description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let effect = input.get("technical_effect").and_then(|v| v.as_str());
            let improvement = input
                .get("performance_improvement")
                .and_then(|v| v.as_f64());
            let obvious = input.get("obviousness").and_then(|v| v.as_bool());
            super::drafting_tools::DraftingTools::innovation_evaluator(
                invention.into(),
                effect.map(|s| s.to_string()),
                improvement,
                obvious,
            )
        })
    });
    t.insert("SemanticCompare".into(), |input| {
        Box::pin(async move {
            let parsed: super::advanced_analysis::SemanticCompareInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            super::advanced_analysis::AdvancedAnalysisTools::semantic_compare(parsed)
        })
    });
    t.insert("TechTripleExtractor".into(), |input| {
        Box::pin(async move {
            let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
            use codex_patent_domain::disclosure::FeatureExtractor;
            let features = FeatureExtractor::extract_features(text, None);
            let pfe =
                FeatureExtractor::extract_problem_feature_effects(text, None, Some(&features));
            serde_json::to_value(&pfe).map_err(|e| format!("{e}"))
        })
    });
    t.insert("FeatureExtractor".into(), |input| {
        Box::pin(async move {
            let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
            use codex_patent_domain::disclosure::FeatureExtractor;
            let features = FeatureExtractor::extract_features(text, None);
            serde_json::to_value(&features).map_err(|e| format!("{e}"))
        })
    });
    t.insert("PatentInfringement".into(), |input| {
        Box::pin(async move {
            let parsed: InfringementAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::infringement_analysis(parsed)
        })
    });
    t.insert("PatentCompareTool".into(), |input| {
        Box::pin(async move {
            let target = input.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let prior = input
                .get("prior_art")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let sim = codex_patent_text::text_similarity(target, prior);
            Ok(serde_json::json!({"similarity": sim, "has_differences": sim < 0.9}))
        })
    });
    t.insert("InventionUnderstanding".into(), |input| Box::pin(async move {
        let title = input.get("invention_title").and_then(|v| v.as_str()).unwrap_or("");
        let field = input.get("technical_field").and_then(|v| v.as_str()).unwrap_or("");
        let disclosure = input.get("technical_disclosure").and_then(|v| v.as_str()).unwrap_or("");
        use codex_patent_domain::disclosure::DisclosureParser;
        let doc = DisclosureParser::parse(disclosure);
        Ok(serde_json::json!({"title": title, "field": field, "sections_found": doc.sections.len(), "confidence": doc.confidence}))
    }));
    t.insert("TechUnit".into(), |input| Box::pin(async move {
        let text = input.get("claim_text").and_then(|v| v.as_str()).unwrap_or("");
        let tokens = codex_patent_text::tokenize(text);
        let keywords = codex_patent_text::extract_keywords(text, 5);
        let ipc = codex_patent_text::IpcClassifier::new().classify(text);
        Ok(serde_json::json!({"token_count": tokens.len(), "keywords": keywords, "ipc_suggestions": ipc}))
    }));
    t.insert("Researcher".into(), |input| Box::pin(async move {
        let query = input.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let depth = input.get("depth").and_then(|v| v.as_u64()).unwrap_or(2);
        Ok(serde_json::json!({"query": query, "depth": depth, "note": "技术调研结果将基于知识库和网络搜索综合生成"}))
    }));
    t.insert("SynergyAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: super::advanced_analysis::SynergyAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            super::advanced_analysis::AdvancedAnalysisTools::synergy_analysis(parsed)
        })
    });
    t.insert("HighCitationSearch".into(), |input| {
        Box::pin(async move {
            let parsed: super::advanced_analysis::HighCitationInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            super::advanced_analysis::AdvancedAnalysisTools::high_citation_patents(parsed)
        })
    });
    t.insert("SuccessPredictor".into(), |input| {
        Box::pin(async move {
            let parsed: super::advanced_analysis::SuccessPredictorInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            super::advanced_analysis::AdvancedAnalysisTools::success_predictor(parsed)
        })
    });
    t
}
