use codex_patent_constitutional::{
    ConstitutionalEngine, RuleAction, RuleLoader, RuleSeverity,
};
use pretty_assertions::assert_eq;

/// 构造一个包含关键词+模式分析+结构分析规则的完整 YAML 规则集，
/// 测试加载→引擎创建→各阶段检查的完整链路。
fn drafting_rules_yaml() -> &'static str {
    r#"rules:
  block_prohibited:
    id: "KW-001"
    name: "禁止主题检查"
    description: "检查是否包含专利法排除的客体"
    phase: "drafting"
    severity: "critical"
    action: "block"
    legal_basis: "专利法第25条"
    check:
      type: "keyword_blocklist"
      keywords: []
      patterns: []
      absolute_ban:
        - "赌博"
        - "色情"
      context_ban: []
      negation_context: false
      severity_if_found: "critical"
  check_technical:
    id: "PA-001"
    name: "技术方案分析"
    description: "检测是否为纯软件方案"
    phase: "drafting"
    severity: "major"
    action: "warn"
    legal_basis: ""
    check:
      type: "pattern_analysis"
      hardware_integration_markers:
        - "传感器"
        - "处理器"
      pure_software_markers:
        - "APP"
        - "SaaS"
      guidance: "需结合硬件才能获得授权"
  structural_review:
    id: "SA-001"
    name: "三要素结构检查"
    description: "检查是否同时包含技术问题、技术方案、技术效果"
    phase: "review"
    severity: "major"
    action: "warn"
    legal_basis: "审查指南第二部分第二章"
    check:
      type: "structural_analysis"
      requires_all:
        - element: "技术问题"
          description: "要解决的技术问题"
          patterns:
            - "技术问题"
            - "要解决"
        - element: "技术效果"
          description: "技术效果描述"
          patterns:
            - "有益效果"
            - "技术效果"
      min_confidence: 0.6
"#
}

// ── 加载 + 引擎创建 ──

#[test]
fn load_yaml_and_create_engine() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    assert!(!rules_map.is_empty());

    let engine = ConstitutionalEngine::new(rules_map);
    let phases = engine.known_phases();
    assert_eq!(phases, vec!["drafting", "review"]);
}

// ── 阶段过滤 ──

#[test]
fn rules_for_drafting_has_two_rules() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    let drafting_rules = engine.rules_for_phase("drafting");
    assert_eq!(drafting_rules.len(), 2);
    let ids: Vec<&str> = drafting_rules.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"KW-001"));
    assert!(ids.contains(&"PA-001"));
}

#[test]
fn rules_for_review_has_one_rule() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    let review_rules = engine.rules_for_phase("review");
    assert_eq!(review_rules.len(), 1);
    assert_eq!(review_rules[0].id, "SA-001");
}

// ── check_all 端到端 ──

#[test]
fn check_all_drafting_clean_text() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    let results = engine.check_all(
        "claim_generator",
        "本发明涉及一种基于传感器和处理器的高精度测量系统",
        None,
        "drafting",
    );
    assert_eq!(results.len(), 2);

    // 关键词检查：无禁用词 → 通过
    let kw = results.iter().find(|r| r.rule_id == "KW-001").unwrap();
    assert!(kw.passed);
    // 模式分析：有传感器（硬件标记）→ 通过
    let pa = results.iter().find(|r| r.rule_id == "PA-001").unwrap();
    assert!(pa.passed);
}

#[test]
fn check_all_drafting_blocked_keyword() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    let results = engine.check_all(
        "claim_generator",
        "本发明涉及一种赌博监控装置",
        None,
        "drafting",
    );
    let kw = results.iter().find(|r| r.rule_id == "KW-001").unwrap();
    assert!(!kw.passed);
    assert!(kw.details.iter().any(|d| d.contains("命中禁用词: 赌博")));
}

#[test]
fn check_all_review_structure_complete() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    let results = engine.check_all(
        "quality_checker",
        "本发明要解决的技术问题是提高精度，具有有益的技术效果",
        None,
        "review",
    );
    assert_eq!(results.len(), 1);

    let sa = &results[0];
    assert_eq!(sa.rule_id, "SA-001");
    assert!(sa.passed);
    assert!(sa.details.iter().any(|d| d.contains("三要素完整")));
}

#[test]
fn check_all_review_missing_elements() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    let results = engine.check_all("quality_checker", "一个装置", None, "review");
    let sa = &results[0];
    assert!(!sa.passed);
    assert!(sa.details.iter().any(|d| d.contains("缺少要素: 技术问题")));
    assert!(sa.details.iter().any(|d| d.contains("缺少要素: 技术效果")));
}

// ── rules_context_for_phase ──

#[test]
fn rules_context_includes_block_and_warn_tags() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    let ctx = engine.rules_context_for_phase("drafting");
    assert!(ctx.contains("[BLOCK]"));
    assert!(ctx.contains("[WARN]"));
    assert!(ctx.contains("专利法第25条"));
    assert!(ctx.contains("drafting 阶段"));
}

#[test]
fn rules_context_empty_for_unknown_phase() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    assert!(engine.rules_context_for_phase("nonexistent").is_empty());
}

// ── auto_scan ──

#[test]
fn auto_scan_drafting_returns_all_known_tools() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    let scanned = engine.auto_scan_for_phase("drafting");
    assert!(!scanned.is_empty());
    for tool in &scanned {
        assert!(!tool.active_rules.is_empty());
        assert!(!tool.tool_name.is_empty());
        // 所有活跃规则都应在 drafting 阶段
        for rule in &tool.active_rules {
            assert!(matches!(rule.action, RuleAction::Block | RuleAction::Warn));
        }
    }
}

#[test]
fn auto_scan_empty_for_unknown_phase() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    assert!(engine.auto_scan_for_phase("nonexistent").is_empty());
}

// ── 规则元数据正确性 ──

#[test]
fn rule_severity_and_action_parsed_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.yaml");
    std::fs::write(&path, drafting_rules_yaml()).unwrap();

    let rules_map = RuleLoader::load_rules_from(&[path]).unwrap();
    let engine = ConstitutionalEngine::new(rules_map);

    let results = engine.check_all("tool", "文本", None, "drafting");
    let kw = results.iter().find(|r| r.rule_id == "KW-001").unwrap();
    assert!(matches!(kw.severity, RuleSeverity::Critical));
    assert!(matches!(kw.action, RuleAction::Block));
    assert_eq!(kw.legal_basis, "专利法第25条");
    assert_eq!(kw.rule_name, "禁止主题检查");
}
