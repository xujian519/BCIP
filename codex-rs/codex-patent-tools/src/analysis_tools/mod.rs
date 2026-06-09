//! 专利分析工具集。
//!
//! 提供专利领域中常见的分析功能，包括：
//! - 权利要求解析与比对 (`ClaimParseInput`, `ClaimCompareInput`)
//! - 新颖性/创造性/创新性评估 (`NoveltyAnalysisInput`, `InventivenessAnalysisInput`, `InnovationEvaluatorInput`)
//! - 侵权分析 (`InfringementAnalysisInput`)
//! - 法律问答与知识检索 (`LegalQAInput`, `KnowledgeSearchInput`)
//! - 技术特征提取与理解 (`TechTripleExtractorInput`, `FeatureExtractorInput`, `InventionUnderstandingInput`)
//! - 技术单元/保护范围分析 (`TechUnitInput`, `ClaimScopeInput`)
//! - 专利对比与深度研究 (`PatentCompareInput`, `ResearcherInput`)

pub mod types;

use codex_patent_core::CaseContext;
use codex_patent_core::CompareFeature;
use codex_patent_core::FeatureType;
use codex_patent_core::ParsedFeature;
use codex_patent_domain::claim_parser;
use codex_patent_domain::compare;
use codex_patent_domain::rule_engine::QualitativeRuleEngine;
use codex_patent_knowledge::SearchConfig;
use codex_patent_knowledge::SearchMode;
use codex_patent_knowledge::UnifiedSearch;
use regex::Regex;
use std::sync::LazyLock;
pub use types::*;

/// 功能性特征正则：匹配"用于..."、"配置为..."等模式
static FUNCTIONAL_FEATURE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:用于|配置为|配置成|适于|适用于|被配置为|被配置成)[^，。；]+")
        .expect("FUNCTIONAL_FEATURE_RE 正则字面量有效")
});

/// 参数范围正则：匹配数字+单位的模式
static PARAM_RANGE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\d+[\.\d]*\s*(?:%|度|mm|cm|m|kg|Hz|MHz|GHz|V|A|W|Pa)")
        .expect("PARAM_RANGE_RE 正则字面量有效")
});

pub fn claim_parse(input: ClaimParseInput) -> Result<serde_json::Value, String> {
    let result = claim_parser::parse(input.claim_number, &input.claim_text);
    serde_json::to_value(result).map_err(|e| format!("{e}"))
}

pub fn claim_compare(input: ClaimCompareInput) -> Result<serde_json::Value, String> {
    let parsed_a = claim_parser::parse(1, &input.claim_a);
    let parsed_b = claim_parser::parse(2, &input.claim_b);
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
    let lexical_result = compare::compare(&features_a, &features_b);

    // 第二层: 语义层 (整段文本相似度)
    let semantic_score = codex_patent_text::text_similarity(&input.claim_a, &input.claim_b);

    // 第三层 & 第四层: 功能层/效果层 (基于特征类型统计)
    let (functional_score, effect_score) =
        compute_functional_effect_scores(&parsed_a.features, &parsed_b.features);

    // 特征矩阵
    let matrix = codex_patent_domain::compare::build_feature_matrix(&features_a, &features_b);

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

    // 自动提取区别特征
    let (claim_feats, prior_feats, distinguishing, coverage) =
        match (&input.claim_text, &input.closest_prior_art) {
            (Some(claim), Some(prior)) if !claim.is_empty() && !prior.is_empty() => {
                let claim_parsed = claim_parser::parse(1, claim);
                let prior_parsed = claim_parser::parse(1, prior);
                let target: Vec<CompareFeature> = claim_parsed
                    .features
                    .iter()
                    .map(|f| CompareFeature {
                        id: f.id.clone(),
                        description: f.description.clone(),
                    })
                    .collect();
                let prior_target: Vec<CompareFeature> = prior_parsed
                    .features
                    .iter()
                    .map(|f| CompareFeature {
                        id: f.id.clone(),
                        description: f.description.clone(),
                    })
                    .collect();
                let result = compare::compare(&target, &prior_target);
                let dists: Vec<String> = result
                    .different_features
                    .iter()
                    .chain(result.missing_features.iter())
                    .cloned()
                    .collect();
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
        // 自动检索创造性相关知识卡片
        if let Ok(knowledge) = search_creativity_knowledge(&ctx)
            && !knowledge.is_empty()
        {
            obj.insert("knowledge_references".into(), serde_json::json!(knowledge));
        }
    }
    Ok(output)
}

pub fn infringement_analysis(
    input: InfringementAnalysisInput,
) -> Result<serde_json::Value, String> {
    let claim = claim_parser::parse(1, &input.claim_text);
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
    serde_json::to_value(compare::compare(&target, &prior)).map_err(|e| format!("{e}"))
}

pub fn legal_qa(input: LegalQAInput) -> Result<serde_json::Value, String> {
    Ok(
        serde_json::json!({"question": input.question, "response_type": "knowledge_based", "message": "请通过知识库搜索获取详细法律依据"}),
    )
}

pub fn knowledge_search(input: KnowledgeSearchInput) -> Result<serde_json::Value, String> {
    let search = UnifiedSearch::global();
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

/// 根据分析上下文自动检索创造性相关知识卡片。
/// 检索失败不阻塞分析流程。
fn search_creativity_knowledge(ctx: &CaseContext) -> Result<Vec<serde_json::Value>, String> {
    let query = match (&ctx.invention_type, &ctx.distinguishing_features) {
        (Some(t), _) => format!("创造性 {:?}", t),
        (_, Some(dists)) if !dists.is_empty() => format!("创造性 技术启示 {}", dists.join(" ")),
        _ => "创造性 三步法".to_string(),
    };

    let search = UnifiedSearch::global();
    let config = SearchConfig {
        query,
        limit: 3,
        mode: SearchMode::KeywordEnhanced,
        ..Default::default()
    };
    let results = search.search(&config);
    Ok(results
        .into_iter()
        .take(3)
        .filter_map(|r| serde_json::to_value(r).ok())
        .collect())
}

fn compute_functional_effect_scores(
    features_a: &[ParsedFeature],
    features_b: &[ParsedFeature],
) -> (f64, f64) {
    let func_a: Vec<&ParsedFeature> = features_a
        .iter()
        .filter(|f| {
            matches!(
                f.feature_type,
                FeatureType::Element | FeatureType::Parameter
            )
        })
        .collect();
    let func_b: Vec<&ParsedFeature> = features_b
        .iter()
        .filter(|f| {
            matches!(
                f.feature_type,
                FeatureType::Element | FeatureType::Parameter
            )
        })
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
            .map(|fb| {
                codex_patent_domain::compare::lexical_similarity(&fa.description, &fb.description)
            })
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

fn compute_overall_correspondence(
    lexical: f64,
    semantic: f64,
    functional: f64,
    effect: f64,
) -> &'static str {
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

/// 注册分析工具集到工具注册表。
///
/// 注册所有分析工具（权利要求解析、新颖性评估、侵权分析等）到统一的 `ToolHandler` 映射中，
/// 供上层 Agent 按名称调用。
pub fn register_analysis_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("ClaimParse".into(), |input| {
        Box::pin(async move {
            let parsed: ClaimParseInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            claim_parse(parsed)
        })
    });
    t.insert("ClaimCompare".into(), |input| {
        Box::pin(async move {
            let parsed: ClaimCompareInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            claim_compare(parsed)
        })
    });
    t.insert("NoveltyAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: NoveltyAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            novelty_analysis(parsed)
        })
    });
    t.insert("InventivenessAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: InventivenessAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            inventiveness_analysis(parsed)
        })
    });
    t.insert("InfringementAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: InfringementAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            infringement_analysis(parsed)
        })
    });
    t.insert("InnovationEvaluator".into(), |input| {
        Box::pin(async move {
            let parsed: InnovationEvaluatorInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            super::drafting_tools::innovation_evaluator(
                parsed.invention_description,
                parsed.technical_effect,
                parsed.performance_improvement,
                parsed.obviousness,
            )
        })
    });
    t.insert("SemanticCompare".into(), |input| {
        Box::pin(async move {
            let parsed: super::advanced_analysis::SemanticCompareInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            super::advanced_analysis::semantic_compare(parsed)
        })
    });
    t.insert("TechTripleExtractor".into(), |input| {
        Box::pin(async move {
            let parsed: TechTripleExtractorInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            use codex_patent_domain::disclosure::{
                extract_features, extract_problem_feature_effects,
            };
            let features = extract_features(&parsed.text, None);
            let pfe = extract_problem_feature_effects(&parsed.text, None, Some(&features));
            serde_json::to_value(&pfe).map_err(|e| format!("{e}"))
        })
    });
    t.insert("FeatureExtractor".into(), |input| {
        Box::pin(async move {
            let parsed: FeatureExtractorInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            use codex_patent_domain::disclosure::extract_features;
            let features = extract_features(&parsed.text, None);
            serde_json::to_value(&features).map_err(|e| format!("{e}"))
        })
    });
    t.insert("PatentInfringement".into(), |input| {
        Box::pin(async move {
            let parsed: InfringementAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            infringement_analysis(parsed)
        })
    });
    t.insert("PatentCompareTool".into(), |input| {
        Box::pin(async move {
            let parsed: PatentCompareInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let sim = codex_patent_text::text_similarity(&parsed.target, &parsed.prior_art);
            Ok(serde_json::json!({"similarity": sim, "has_differences": sim < 0.9}))
        })
    });
    t.insert("InventionUnderstanding".into(), |input| {
        Box::pin(async move {
            let parsed: InventionUnderstandingInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let doc = codex_patent_domain::disclosure::parse(&parsed.technical_disclosure);
            Ok(serde_json::json!({"title": parsed.invention_title, "field": parsed.technical_field, "sections_found": doc.sections.len(), "confidence": doc.confidence}))
        })
    });
    t.insert("TechUnit".into(), |input| {
        Box::pin(async move {
            let parsed: TechUnitInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let tokens = codex_patent_text::tokenize(&parsed.claim_text);
            let keywords = codex_patent_text::extract_keywords(&parsed.claim_text, 5);
            let ipc = codex_patent_text::IpcClassifier::new()
                .classify(&parsed.claim_text);
            Ok(serde_json::json!({"token_count": tokens.len(), "keywords": keywords, "ipc_suggestions": ipc}))
        })
    });
    t.insert("Researcher".into(), |input| {
        Box::pin(async move {
            let parsed: ResearcherInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;

            let query = &parsed.query;
            let limit = match parsed.depth {
                0..=1 => 3,
                2 => 5,
                _ => 10,
            };

            // 聚合多个知识源
            let mut results = serde_json::Map::new();

            // 1. 知识图谱搜索
            let kg_result = super::legal_tools::LegalTools::knowledge_search(query, limit, false);
            if let Ok(v) = kg_result {
                results.insert("knowledge_graph".into(), v);
            }

            // 2. 知识卡片搜索
            let card_result = super::legal_tools::LegalTools::card_search(query, limit);
            if let Ok(v) = card_result {
                results.insert("knowledge_cards".into(), v);
            }

            // 3. IPC 分类搜索
            let ipc_result =
                super::legal_tools::LegalTools::ipc_search(super::legal_tools::IpcSearchInput {
                    query: query.clone(),
                    limit: Some(3),
                });
            if let Ok(v) = ipc_result {
                results.insert("ipc_classification".into(), v);
            }

            let sources_count = results.len();
            Ok(serde_json::json!({
                "query": query,
                "depth": parsed.depth,
                "sources_used": sources_count,
                "results": results,
            }))
        })
    });
    t.insert("SynergyAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: super::advanced_analysis::SynergyAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            super::advanced_analysis::synergy_analysis(parsed)
        })
    });
    t.insert("HighCitationSearch".into(), |input| {
        Box::pin(async move {
            let parsed: super::advanced_analysis::HighCitationInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let limit = parsed.limit.unwrap_or(20);
            let patent_number = parsed.patent_number.clone();

            // 使用 Google Patents 的 citedby 查询语法执行前向引用检索
            let query = format!("citedby:{}", patent_number);
            let google_input = crate::google_patents::GooglePatentsInput {
                query,
                limit,
                patent_number: None,
            };
            let citing_patents = crate::google_patents::fetch_google_patents(google_input).await?;

            let results: Vec<serde_json::Value> = citing_patents
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "patent_number": p.patent_number,
                        "title": p.title,
                        "assignee": p.assignee,
                        "publication_date": p.publication_date,
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "patent_number": patent_number,
                "citing_patents": results,
                "total": results.len(),
            }))
        })
    });
    t.insert("SuccessPredictor".into(), |input| {
        Box::pin(async move {
            let parsed: super::advanced_analysis::SuccessPredictorInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            super::advanced_analysis::success_predictor(parsed)
        })
    });
    t.insert("ClaimScopeAnalyzer".into(), |input| {
        Box::pin(async move {
            let parsed: ClaimScopeInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;

            if parsed.claim_text.trim().is_empty() {
                return Err("权利要求文本不能为空".to_string());
            }

            let claim = &parsed.claim_text;

            // 1. 功能性特征识别（"用于..."、"配置为..."、"适于..."、"配置成..."）
            let functional_features: Vec<String> = FUNCTIONAL_FEATURE_RE
                .captures_iter(claim)
                .filter_map(|cap| cap.get(0).map(|m| m.as_str().to_string()))
                .collect();

            // 2. 最宽合理解释分析
            let broad_terms = [
                "装置", "设备", "系统", "方法", "模块", "单元", "组件", "部件", "机构",
            ];
            let identified_broad_terms: Vec<&str> = broad_terms
                .iter()
                .filter(|t| claim.contains(**t))
                .copied()
                .collect();

            // 3. 等同原则适用性判断
            let has_means_plus_function = claim.contains("用于") || claim.contains("装置用于");
            let has_parameter_range = PARAM_RANGE_RE.is_match(claim);
            let has_method_steps = claim.contains("步骤") || claim.contains("包括以下步骤");

            let equivalence_applicable =
                has_means_plus_function || !identified_broad_terms.is_empty();

            // 4. 保护范围宽度评估
            let scope_width =
                if !functional_features.is_empty() && identified_broad_terms.len() >= 2 {
                    "宽" // 功能性限定+上位概念 = 宽范围
                } else if identified_broad_terms.len() >= 2 {
                    "较宽" // 多个上位概念
                } else if !functional_features.is_empty() {
                    "中等" // 有功能性限定
                } else if has_parameter_range {
                    "窄" // 有具体参数范围
                } else {
                    "适中"
                };

            // 5. 风险提示
            let mut warnings = Vec::new();
            if !functional_features.is_empty() {
                warnings.push("含功能性限定，可能面临112(f)/第26条第4款问题，建议补充结构特征");
            }
            if identified_broad_terms.len() >= 3 {
                warnings.push("上位概念较多，保护范围较宽但可能面临新颖性挑战");
            }
            if has_parameter_range {
                warnings.push("含具体参数范围，保护范围受限，等同原则适用空间小");
            }

            Ok(serde_json::json!({
                "claim_text": claim.chars().take(200).collect::<String>(),
                "scope_width": scope_width,
                "functional_features": functional_features,
                "functional_feature_count": functional_features.len(),
                "broad_terms": identified_broad_terms,
                "equivalence_applicable": equivalence_applicable,
                "has_parameter_range": has_parameter_range,
                "has_method_steps": has_method_steps,
                "warnings": warnings,
            }))
        })
    });
    t
}
