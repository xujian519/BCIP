use codex_patent_core::ClaimDraft;
use codex_patent_core::ClaimType;
use codex_patent_domain::quality::QualityAssessor;
use codex_patent_domain::quality_rules;

fn sample_claims() -> Vec<ClaimDraft> {
    vec![
        ClaimDraft {
            id: "1".into(),
            claim_type: ClaimType::Independent,
            preamble: "一种装置".into(),
            transitional_phrase: "包括".into(),
            elements: vec!["底座".into(), "支架".into(), "驱动模块".into()],
            dependent_on: None,
        },
        ClaimDraft {
            id: "2".into(),
            claim_type: ClaimType::Dependent,
            preamble: "根据权利要求1所述的装置".into(),
            transitional_phrase: String::new(),
            elements: vec!["实施例中的驱动模块为伺服电机".into()],
            dependent_on: Some("1".into()),
        },
        ClaimDraft {
            id: "3".into(),
            claim_type: ClaimType::Dependent,
            preamble: "根据权利要求2所述的装置".into(),
            transitional_phrase: String::new(),
            elements: vec!["伺服电机通过联轴器连接".into()],
            dependent_on: Some("2".into()),
        },
    ]
}

#[test]
fn e2e_quality_assessment_full_pipeline() {
    let claims = sample_claims();
    let result = QualityAssessor::assess_claims(&claims);

    // 验证结果包含所有4个维度
    assert!(result.clarity_score > 0.0, "清晰性评分应为正数");
    assert!(result.support_score > 0.0, "支持性评分应为正数");
    assert!(result.scope_score > 0.0, "保护范围评分应为正数");
    assert!(result.enablement_score > 0.0, "可实施性评分应为正数");
    assert!(
        result.overall_score > 0.0 && result.overall_score <= 1.0,
        "综合评分应在(0,1]区间"
    );
}

#[test]
fn e2e_quality_assessment_detects_vague_claims() {
    let claims = vec![ClaimDraft {
        id: "1".into(),
        claim_type: ClaimType::Independent,
        preamble: "一种装置".into(),
        transitional_phrase: "其特征在于".into(),
        elements: vec!["大约合适的温度".into(), "适当的压力".into()],
        dependent_on: None,
    }];
    let result = QualityAssessor::assess_claims(&claims);
    assert!(
        result.clarity_score < 0.7,
        "含模糊词的权利要求应降低清晰性评分"
    );
}

#[test]
fn e2e_quality_assessment_broken_dependency() {
    let claims = vec![ClaimDraft {
        id: "2".into(),
        claim_type: ClaimType::Dependent,
        preamble: "根据权利要求1".into(),
        transitional_phrase: String::new(),
        elements: vec!["特征A".into()],
        dependent_on: Some("1".into()),
    }];
    let result = QualityAssessor::assess_claims(&claims);
    assert!(result.support_score < 0.6, "断链依赖应大幅降低支持性评分");
}

#[test]
fn e2e_quality_rules_loaded_from_yaml() {
    // 验证 spec-quality.yaml 规则被正确读取
    let terms = quality_rules::commercial_terms();
    assert!(!terms.is_empty(), "商业用语列表不应为空");
    assert!(terms.contains(&"最佳".to_string()), "应包含'最佳'");

    let uncertain = quality_rules::uncertain_terms();
    assert!(!uncertain.is_empty(), "不确定用语列表不应为空");
    assert!(uncertain.contains(&"厚".to_string()), "应包含'厚'");
}

#[test]
fn e2e_quality_rules_has_prohibited_reference_pattern() {
    let re_str = quality_rules::prohibited_reference_regex();
    assert!(!re_str.is_empty(), "禁用引用正则不应为空");
    let re = regex::Regex::new(&re_str).unwrap();
    assert!(re.is_match("如权利要求1所述"), "应匹配'如权利要求1所述'");
    assert!(
        re.is_match("如上述权利要求所述"),
        "应匹配'如上述权利要求所述'"
    );
}

#[test]
fn e2e_quality_assessment_good_claims_high_scores() {
    let claims = vec![
        ClaimDraft {
            id: "1".into(),
            claim_type: ClaimType::Independent,
            preamble: "一种图像处理装置".into(),
            transitional_phrase: "包括".into(),
            elements: vec!["图像采集模块".into(), "处理器".into(), "存储器".into()],
            dependent_on: None,
        },
        ClaimDraft {
            id: "2".into(),
            claim_type: ClaimType::Dependent,
            preamble: "根据权利要求1所述的装置".into(),
            transitional_phrase: String::new(),
            elements: vec!["实施例中处理器为GPU".into()],
            dependent_on: Some("1".into()),
        },
        ClaimDraft {
            id: "3".into(),
            claim_type: ClaimType::Dependent,
            preamble: "根据权利要求2所述的装置".into(),
            transitional_phrase: String::new(),
            elements: vec!["GPU为嵌入式GPU".into()],
            dependent_on: Some("2".into()),
        },
        ClaimDraft {
            id: "4".into(),
            claim_type: ClaimType::Dependent,
            preamble: "根据权利要求3所述的装置".into(),
            transitional_phrase: String::new(),
            elements: vec!["嵌入式GPU功耗小于5W".into()],
            dependent_on: Some("3".into()),
        },
    ];
    let result = QualityAssessor::assess_claims(&claims);
    // 好案例应该在各项都得合理分数
    assert!(
        result.overall_score > 0.5,
        "好案例综合评分应 > 0.5 (实际: {})",
        result.overall_score
    );
    assert!(result.clarity_score > 0.5, "清晰性评分应 > 0.5");
    assert!(result.scope_score > 0.5, "保护范围评分应 > 0.5");
}
