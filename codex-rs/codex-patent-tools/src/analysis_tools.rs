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
    pub invention_description: Option<String>,
    pub technical_effect: Option<String>,
    pub performance_improvement: Option<f64>,
    pub obviousness: Option<bool>,
    // ── 新增：三步法增强字段 ──
    pub claim_text: Option<String>,
    pub closest_prior_art: Option<String>,
    pub has_teaching_away: Option<bool>,
    pub has_technical_prejudice: Option<bool>,
    pub has_unexpected_effect: Option<bool>,
    pub has_long_felt_need: Option<bool>,
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
        let parser = ClaimParser::new();
        let parsed_a = parser.parse(1, &input.claim_a);
        let parsed_b = parser.parse(2, &input.claim_b);

        let features_a: Vec<CompareFeature> = parsed_a
            .features
            .iter()
            .map(|f| CompareFeature {
                id: f.id.clone(),
                description: f.description.clone(),
            })
            .collect();
        let features_b: Vec<CompareFeature> = parsed_b
            .features
            .iter()
            .map(|f| CompareFeature {
                id: f.id.clone(),
                description: f.description.clone(),
            })
            .collect();

        // 第一层: 词法对比 (bigram Jaccard 逐特征匹配)
        let lexical_result = FeatureMatcher::compare(&features_a, &features_b);

        // 第二层: 语义层 (整段文本相似度)
        let semantic_score =
            codex_patent_text::text_similarity(&input.claim_a, &input.claim_b);

        // 第三层 & 第四层: 功能层/效果层 (基于特征类型统计)
        let (functional_score, effect_score) =
            compute_functional_effect_scores(&parsed_a.features, &parsed_b.features);

        // 特征矩阵
        let matrix =
            codex_patent_domain::compare::build_feature_matrix(&features_a, &features_b);

        // 综合判定
        let overall = compute_overall_correspondence(
            lexical_result.coverage_ratio,
            semantic_score,
            functional_score,
            effect_score,
        );

        Ok(serde_json::json!({
            "layers": {
                "lexical": {
                    "exact_count": lexical_result.exact_matches.len(),
                    "equivalent_count": lexical_result.equivalent_matches.len(),
                    "different_count": lexical_result.different_features.len(),
                    "missing_count": lexical_result.missing_features.len(),
                    "coverage_ratio": lexical_result.coverage_ratio,
                    "matches": lexical_result.exact_matches.iter()
                        .chain(lexical_result.equivalent_matches.iter())
                        .map(|m| serde_json::json!({
                            "target": m.target_feature,
                            "prior": m.prior_feature,
                            "score": (m.similarity_score * 100.0).round() / 100.0,
                            "type": format!("{:?}", m.match_type),
                        }))
                        .collect::<Vec<_>>(),
                },
                "semantic": {
                    "score": (semantic_score * 100.0).round() / 100.0,
                    "level": classify_score(semantic_score),
                },
                "functional": {
                    "score": (functional_score * 100.0).round() / 100.0,
                    "matched_types": count_matching_types(&parsed_a.features, &parsed_b.features),
                },
                "effect": {
                    "score": (effect_score * 100.0).round() / 100.0,
                },
            },
            "matrix": {
                "overlap_ratio": (matrix.overlap_ratio * 100.0).round() / 100.0,
                "target_only_count": matrix.target_only.len(),
                "prior_only_count": matrix.prior_only.len(),
            },
            "overall_correspondence": overall,
            "feature_counts": {
                "claim_a": parsed_a.features.len(),
                "claim_b": parsed_b.features.len(),
            },
        }))
    }

    pub fn novelty_analysis(input: NoveltyAnalysisInput) -> Result<serde_json::Value, String> {
        let mut engine = QualitativeRuleEngine::new();
        let prior = input.prior_art_descriptions.unwrap_or_default().join("; ");
        let ctx = CaseContext {
            invention: Some(input.invention_description.clone()),
            prior_art_contains_all: Some(!prior.is_empty()),
            differences: input.differences,
            ..Default::default()
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
        let parser = ClaimParser::new();

        // 自动提取区别特征
        let (claim_feats, prior_feats, distinguishing, coverage) =
            match (&input.claim_text, &input.closest_prior_art) {
                (Some(claim), Some(prior)) if !claim.is_empty() && !prior.is_empty() => {
                    let claim_parsed = parser.parse(1, claim);
                    let prior_parsed = parser.parse(1, prior);
                    let target: Vec<CompareFeature> = claim_parsed.features.iter()
                        .map(|f| CompareFeature { id: f.id.clone(), description: f.description.clone() })
                        .collect();
                    let prior_target: Vec<CompareFeature> = prior_parsed.features.iter()
                        .map(|f| CompareFeature { id: f.id.clone(), description: f.description.clone() })
                        .collect();
                    let result = FeatureMatcher::compare(&target, &prior_target);
                    let dists: Vec<String> = result.different_features.iter()
                        .chain(result.missing_features.iter()).cloned().collect();
                    let cov = result.coverage_ratio;
                    (
                        Some(claim_parsed.features),
                        Some(prior_parsed.features),
                        if dists.is_empty() { None } else { Some(dists) },
                        cov,
                    )
                }
                _ => (None, None, None, 0.0),
            };

        let ctx = CaseContext {
            invention: input.invention_description,
            technical_effect: input.technical_effect,
            performance_improvement: input.performance_improvement,
            obviousness: input.obviousness,
            closest_prior_art: input.closest_prior_art,
            claim_features: claim_feats,
            prior_art_features: prior_feats,
            distinguishing_features: distinguishing.clone(),
            has_teaching_away: input.has_teaching_away,
            has_technical_prejudice: input.has_technical_prejudice,
            has_unexpected_effect: input.has_unexpected_effect,
            has_long_felt_need: input.has_long_felt_need,
            ..Default::default()
        };
        let r = engine
            .analyze_inventiveness(&ctx)
            .map_err(|e| format!("{e}"))?;

        let mut output = serde_json::to_value(r).map_err(|e| format!("{e}"))?;
        if let Some(obj) = output.as_object_mut() {
            if let Some(ref dists) = distinguishing {
                obj.insert("distinguishing_features".into(), serde_json::json!(dists));
            }
            if coverage > 0.0 {
                obj.insert("coverage_ratio".into(), serde_json::json!(coverage));
            }
        }
        Ok(output)
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

fn compute_functional_effect_scores(
    features_a: &[ParsedFeature],
    features_b: &[ParsedFeature],
) -> (f64, f64) {
    let func_a: Vec<&ParsedFeature> = features_a
        .iter()
        .filter(|f| matches!(f.feature_type, FeatureType::Element | FeatureType::Parameter))
        .collect();
    let func_b: Vec<&ParsedFeature> = features_b
        .iter()
        .filter(|f| matches!(f.feature_type, FeatureType::Element | FeatureType::Parameter))
        .collect();
    let func_score = set_overlap_ratio(&func_a, &func_b);

    let eff_a: Vec<&ParsedFeature> = features_a
        .iter()
        .filter(|f| matches!(f.feature_type, FeatureType::Result | FeatureType::Action))
        .collect();
    let eff_b: Vec<&ParsedFeature> = features_b
        .iter()
        .filter(|f| matches!(f.feature_type, FeatureType::Result | FeatureType::Action))
        .collect();
    let eff_score = set_overlap_ratio(&eff_a, &eff_b);

    (func_score, eff_score)
}

fn set_overlap_ratio(a: &[&ParsedFeature], b: &[&ParsedFeature]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let mut matched = 0usize;
    for fa in a {
        let best = b
            .iter()
            .map(|fb| codex_patent_domain::compare::lexical_similarity(&fa.description, &fb.description))
            .fold(0.0f64, f64::max);
        if best >= 0.6 {
            matched += 1;
        }
    }
    matched as f64 / a.len().max(b.len()) as f64
}

fn classify_score(score: f64) -> &'static str {
    if score >= 0.9 {
        "identical"
    } else if score >= 0.6 {
        "similar"
    } else if score >= 0.3 {
        "partially_similar"
    } else {
        "different"
    }
}

fn compute_overall_correspondence(lexical: f64, semantic: f64, functional: f64, effect: f64) -> &'static str {
    let weighted = lexical * 0.35 + semantic * 0.25 + functional * 0.2 + effect * 0.2;
    if weighted >= 0.9 {
        "Exact"
    } else if weighted >= 0.6 {
        "Equivalent"
    } else if weighted >= 0.3 {
        "Different"
    } else {
        "Missing"
    }
}

fn count_matching_types(
    features_a: &[ParsedFeature],
    features_b: &[ParsedFeature],
) -> std::collections::HashMap<String, (usize, usize)> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, (usize, usize)> = HashMap::new();
    for f in features_a {
        let key = format!("{:?}", f.feature_type);
        counts.entry(key).or_insert((0, 0)).0 += 1;
    }
    for f in features_b {
        let key = format!("{:?}", f.feature_type);
        counts.entry(key).or_insert((0, 0)).1 += 1;
    }
    counts
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
