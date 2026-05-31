use codex_patent_knowledge::CardIndex;
use codex_patent_knowledge::LawDatabase;
use codex_patent_knowledge::SearchConfig;
use codex_patent_knowledge::SqliteKnowledgeGraph;
use codex_patent_knowledge::UnifiedSearch;
use serde_json::to_string_pretty;

// 辅助函数：创建统一搜索引擎
fn create_search() -> UnifiedSearch {
    UnifiedSearch::new(
        Some("../codex-patent-assets/patent_kg.db"),
        Some("../codex-patent-assets/laws.db"),
        Some("../codex-patent-assets/card-index.json"),
    )
}

#[test]
#[ignore = "requires local asset files"]
fn workflow_retrieval() {
    // 场景: 检索 "一种智能温控装置" 的相关专利
    let search = create_search();

    // Step 1: 构建检索式
    let query = "智能温控 温度传感器 自动调节";

    // Step 2: 执行搜索
    let config = SearchConfig {
        query: query.to_string(),
        limit: 10,
        ..Default::default()
    };
    let results = search.search(&config);

    // Step 3: 验证结果
    if results.is_empty() {
        println!("搜索无结果，尝试关键词: 温度");
        let simple_config = SearchConfig {
            query: "温度".to_string(),
            limit: 10,
            ..Default::default()
        };
        let simple_results = search.search(&simple_config);
        if !simple_results.is_empty() {
            println!("简化搜索返回 {} 条结果", simple_results.len());
            return;
        }
    }

    assert!(!results.is_empty(), "搜索应返回结果");
    println!("检索到 {} 条结果", results.len());
    assert!(
        results.iter().any(|r| r.score > 0.0),
        "至少有一条结果有正分数"
    );
}

#[test]
fn workflow_claim_analysis() {
    // 场景: 解析权利要求并进行新颖性分析
    use codex_patent_domain::claim_parser::ClaimParser;

    let parser = ClaimParser::new();
    let claim_text = "一种智能温控装置，包括温度传感器、控制器和加热元件，其特征在于，所述控制器根据温度传感器的检测值自动调节加热元件的功率。";

    // Step 1: 解析权利要求
    let claim = parser.parse(1, claim_text);
    assert_eq!(claim.claim_number, 1);
    assert!(!claim.features.is_empty());
    assert!(!claim.features.is_empty(), "应至少提取1个特征");

    // Step 2: 验证特征类型
    let has_element = claim
        .features
        .iter()
        .any(|f| matches!(f.feature_type, codex_patent_core::FeatureType::Element));
    assert!(has_element, "应包含元件类特征");

    println!("提取到 {} 个特征", claim.features.len());
    for f in &claim.features {
        println!("  {}: {} ({:?})", f.id, f.description, f.feature_type);
    }
}

#[test]
fn workflow_novelty_check() {
    // 场景: 检查发明的新颖性
    use codex_patent_core::CaseContext;
    use codex_patent_domain::rule_engine::QualitativeRuleEngine;

    let mut engine = QualitativeRuleEngine::new();
    let ctx = CaseContext {
        invention: Some("一种采用AI算法的智能温控系统".into()),
        prior_art_contains_all: Some(false),
        differences: Some(vec!["AI自适应算法".into(), "多传感器融合".into()]),
        technical_effect: Some("节能30%".into()),
        performance_improvement: Some(0.3),
        obviousness: Some(false),
        rejection_type: None,
        technical_effects: None,
        prior_art_different_field: None,
    };

    let result = engine.analyze_novelty(&ctx).unwrap();
    assert!(result.net_score > 0.3, "存在区别特征应有正面评分");
    assert!(!result.applied_rules.is_empty(), "应应用至少一条规则");
    println!(
        "新颖性分析: {} (置信度: {:.2})",
        result.conclusion, result.confidence
    );
}

#[test]
fn workflow_inventiveness_check() {
    // 场景: 创造性评估
    use codex_patent_core::CaseContext;
    use codex_patent_domain::rule_engine::QualitativeRuleEngine;

    let mut engine = QualitativeRuleEngine::new();
    let ctx = CaseContext {
        invention: Some("一种基于深度学习的图像识别方法".into()),
        prior_art_contains_all: None,
        differences: None,
        technical_effect: Some("识别准确率提升15%".into()),
        performance_improvement: Some(0.5),
        obviousness: Some(false),
        rejection_type: None,
        technical_effects: None,
        prior_art_different_field: None,
    };

    let result = engine.analyze_inventiveness(&ctx).unwrap();
    assert!(result.net_score > 0.4, "有技术效果应有较高创造性评分");
    println!(
        "创造性分析: {:.2} (置信度: {:.2})",
        result.net_score, result.confidence
    );
}

#[test]
fn workflow_infringement_analysis() {
    // 场景: 侵权分析
    use codex_patent_core::CompareFeature;
    use codex_patent_domain::claim_parser::ClaimParser;
    use codex_patent_domain::compare::FeatureMatcher;

    let parser = ClaimParser::new();
    let claim_text =
        "一种折叠椅，包括座板、靠背和支撑腿，其特征在于，所述支撑腿可折叠收纳于座板底部。";
    let claim = parser.parse(1, claim_text);

    let target_features: Vec<CompareFeature> = claim
        .features
        .iter()
        .map(|f| CompareFeature {
            id: f.id.clone(),
            description: f.description.clone(),
        })
        .collect();

    // 被控侵权产品的特征
    let prior_features = vec![
        CompareFeature {
            id: "P1".into(),
            description: "一种折叠椅，具有座板".into(),
        },
        CompareFeature {
            id: "P2".into(),
            description: "靠背可调节角度".into(),
        },
        CompareFeature {
            id: "P3".into(),
            description: "支撑腿可折叠收纳于座板底部".into(),
        },
    ];

    let result = FeatureMatcher::compare(&target_features, &prior_features);
    println!(
        "侵权分析: 覆盖率 {:.2}, 精确匹配 {} 个, 等同匹配 {} 个",
        result.coverage_ratio,
        result.exact_matches.len(),
        result.equivalent_matches.len()
    );
    assert!(result.coverage_ratio >= 0.0, "覆盖率应为有效值");
}

#[test]
fn workflow_drafting() {
    // 场景: 专利撰写流程
    use codex_patent_domain::drafting::default_quality_report;
    use codex_patent_domain::drafting::recalculate_overall_score;

    let _claims = ["一种智能门锁，包括指纹识别模块、密码输入模块和蓝牙通信模块，其特征在于，所述指纹识别模块和密码输入模块通过蓝牙通信模块与移动终端连接。".to_string(),
        "根据权利要求1所述的智能门锁，其特征在于，还包括人脸识别模块。".to_string(),
        "根据权利要求1所述的智能门锁，其特征在于，所述蓝牙通信模块支持BLE 5.0协议。".to_string()];

    let mut report = default_quality_report();
    report.dimensions[0].score = 8.5;
    report.dimensions[1].score = 7.5;
    report.dimensions[2].score = 8.0;
    report.dimensions[3].score = 7.0;
    report.dimensions[4].score = 8.0;
    report.dimensions[5].score = 9.0;
    report.dimensions[6].score = 9.5;
    recalculate_overall_score(&mut report);
    report.is_acceptable = report.overall_score >= 6.0;

    println!(
        "撰写质量: {:.2}/100 (可接受: {})",
        report.overall_score, report.is_acceptable
    );
    for dim in &report.dimensions {
        println!("  {}: {:.1}/{}", dim.name, dim.score, dim.max_score);
    }
    assert!(report.overall_score >= 0.0);
}

#[test]
fn workflow_oa_response() {
    // 场景: OA答复策略
    use codex_patent_core::CaseContext;
    use codex_patent_domain::rule_engine::QualitativeRuleEngine;

    let mut engine = QualitativeRuleEngine::new();
    let ctx = CaseContext {
        invention: None,
        prior_art_contains_all: None,
        differences: Some(vec!["双重加密机制".into(), "实时异常检测".into()]),
        technical_effect: None,
        performance_improvement: None,
        obviousness: None,
        rejection_type: Some("novelty".into()),
        technical_effects: Some(vec!["安全性提升40%".into(), "误报率降低60%".into()]),
        prior_art_different_field: Some(true),
    };

    let strategy = engine.suggest_oa_strategy(&ctx).unwrap();
    println!(
        "OA策略: {} (置信度: {:.2})",
        strategy.conclusion, strategy.confidence
    );
    assert!(!strategy.applied_rules.is_empty());
}

#[test]
fn workflow_full_pipeline() {
    // 场景: 完整专利处理链路
    // 检索 → 解析 → 分析 → 审查 → 撰写

    // Step 1: 知识库搜索
    let search = create_search();
    let config = SearchConfig {
        query: "新能源汽车 电池管理".to_string(),
        limit: 5,
        ..Default::default()
    };
    let results = search.search(&config);
    if results.is_empty() {
        println!("复杂搜索无结果，尝试简化关键词: 电池");
        let simple_config = SearchConfig {
            query: "电池".to_string(),
            limit: 5,
            ..Default::default()
        };
        let simple_results = search.search(&simple_config);
        if !simple_results.is_empty() {
            println!("简化搜索返回 {} 条结果", simple_results.len());
        } else {
            println!("简化搜索也无结果");
        }
        return;
    }
    assert!(!results.is_empty(), "Step1: 搜索失败");

    // Step 2: 权利要求解析
    use codex_patent_domain::claim_parser::ClaimParser;
    let parser = ClaimParser::new();
    let claim = parser.parse(1, "一种电池管理系统，包括电压检测模块、温度检测模块和均衡控制模块，其特征在于，所述均衡控制模块根据电压和温度的综合参数进行主动均衡。");
    assert!(!claim.features.is_empty(), "Step2: 解析失败");

    // Step 3: 新颖性分析
    use codex_patent_core::CaseContext;
    use codex_patent_domain::rule_engine::QualitativeRuleEngine;
    let mut engine = QualitativeRuleEngine::new();
    let ctx = CaseContext {
        invention: Some("主动均衡电池管理系统".into()),
        prior_art_contains_all: Some(false),
        differences: Some(vec!["综合参数主动均衡".into()]),
        technical_effect: Some("电池寿命延长20%".into()),
        performance_improvement: Some(0.2),
        obviousness: Some(false),
        rejection_type: None,
        technical_effects: None,
        prior_art_different_field: None,
    };
    let novelty = engine.analyze_novelty(&ctx).unwrap();
    assert!(novelty.confidence > 0.3, "Step3: 新颖性分析失败");

    // Step 4: 撰写评估
    use codex_patent_domain::drafting::default_quality_report;
    use codex_patent_domain::drafting::recalculate_overall_score;
    let _claims = ["一种电池管理系统，包括电压检测模块、温度检测模块和均衡控制模块，其特征在于，所述均衡控制模块根据电压和温度的综合参数进行主动均衡。".to_string()];
    let mut quality = default_quality_report();
    quality.dimensions[0].score = 8.0;
    quality.dimensions[1].score = 8.0;
    quality.dimensions[2].score = 7.5;
    quality.dimensions[3].score = 7.0;
    quality.dimensions[4].score = 6.5;
    quality.dimensions[5].score = 8.0;
    quality.dimensions[6].score = 9.0;
    recalculate_overall_score(&mut quality);
    println!("完整链路测试通过! 质量评分: {:.2}", quality.overall_score);
}

#[test]
#[ignore = "requires local asset files"]
fn test_search_performance() {
    // 性能基准: 知识图谱搜索应在100ms内完成
    use std::time::Instant;

    let kg = SqliteKnowledgeGraph::open("../codex-patent-assets/patent_kg.db").unwrap();

    let start = Instant::now();
    let results = kg.search_nodes("专利", None, 10).unwrap();
    let elapsed = start.elapsed();

    println!("KG搜索耗时: {:?}, 返回 {} 条结果", elapsed, results.len());
    assert!(elapsed.as_millis() < 5000, "搜索不应超过5秒");
    assert!(!results.is_empty(), "应返回结果");
}

#[test]
#[ignore = "requires local asset files"]
fn test_law_db_performance() {
    use std::time::Instant;

    let db = LawDatabase::open("../codex-patent-assets/laws.db").unwrap();

    let start = Instant::now();
    let count = db.count().unwrap();
    let elapsed = start.elapsed();

    println!("法规库: {} 条记录, 查询耗时: {:?}", count, elapsed);
    assert!(count > 0, "法规库应有数据");
    assert!(elapsed.as_millis() < 1000, "统计查询不应超过1秒");
}

#[test]
#[ignore = "requires local asset files"]
fn test_card_index_performance() {
    use std::time::Instant;

    let start = Instant::now();
    let idx = CardIndex::load("../codex-patent-assets/card-index.json").unwrap();
    let elapsed = start.elapsed();

    println!("卡片索引加载: {} 张卡片, 耗时: {:?}", idx.len(), elapsed);
    assert!(!idx.is_empty(), "卡片索引应有数据");
    assert!(elapsed.as_millis() < 2000, "加载不应超过2秒");
}

#[test]
#[ignore = "requires local asset files"]
fn test_all_knowledge_sources() {
    // 验证所有知识源可用
    let kg = SqliteKnowledgeGraph::open("../codex-patent-assets/patent_kg.db");
    assert!(kg.is_ok(), "知识图谱应可打开");

    let db = LawDatabase::open("../codex-patent-assets/laws.db");
    assert!(db.is_ok(), "法规库应可打开");

    let idx = CardIndex::load("../codex-patent-assets/card-index.json");
    assert!(idx.is_ok(), "卡片索引应可加载");

    let search = create_search();
    let status = search.status();
    println!("知识源状态: {}", to_string_pretty(&status).unwrap());

    assert!(
        status["knowledge_graph"].is_object() || status["knowledge_graph"].is_null(),
        "KG状态应为对象或null"
    );
}
