use std::collections::HashMap;

use crate::checkers;
use crate::model::*;
use crate::types::*;

/// 宪法规则执行引擎。
pub struct ConstitutionalEngine {
    rules: HashMap<String, ConstitutionalRules>,
}

impl ConstitutionalEngine {
    pub fn new(rules: HashMap<String, ConstitutionalRules>) -> Self {
        Self { rules }
    }

    pub fn rules(&self) -> &HashMap<String, ConstitutionalRules> {
        &self.rules
    }

    /// 返回指定阶段的所有规则（不执行检查，仅返回规则元信息）
    pub fn rules_for_phase(&self, phase: &str) -> Vec<&ConstitutionalRule> {
        let mut result = Vec::new();
        for ruleset in self.rules.values() {
            for rule in ruleset.rules.values() {
                if rule.phase.is_empty() || rule.phase == phase {
                    result.push(rule);
                }
            }
        }
        result
    }

    /// 生成指定阶段的合规规则上下文文本，可注入到 developer instructions 中。
    pub fn rules_context_for_phase(&self, phase: &str) -> String {
        let rules = self.rules_for_phase(phase);
        if rules.is_empty() {
            return String::new();
        }

        let mut ctx = format!("## 专利合规规则（{} 阶段）\n\n", phase);
        for rule in &rules {
            let action_tag = match rule.action.as_str() {
                "block" => "[BLOCK]",
                "warn" => "[WARN]",
                "review" => "[REVIEW]",
                "enforce" => "[ENFORCE]",
                _ => "[INFO]",
            };
            ctx.push_str(&format!(
                "- {action_tag} **{}**: {} ",
                rule.name, rule.description
            ));
            if !rule.legal_basis.is_empty() {
                ctx.push_str(&format!("（{}）", rule.legal_basis));
            }
            ctx.push('\n');
        }
        ctx
    }

    /// 执行指定阶段的所有规则检查
    pub fn check_all(
        &self,
        tool_name: &str,
        input_text: &str,
        output_text: Option<&str>,
        phase: &str,
    ) -> Vec<RuleCheckResult> {
        let mut results = Vec::new();
        for ruleset in self.rules.values() {
            for rule in ruleset.rules.values() {
                if !rule.phase.is_empty() && rule.phase != phase {
                    continue;
                }
                let result = self.evaluate_rule(rule, tool_name, input_text, output_text);
                results.push(result);
            }
        }
        results
    }

    /// 自动扫描模式：遍历所有已知的专利工具名，对每个工具运行阶段适配的规则。
    /// 用于不需要指定具体输入/输出的场景（如技能激活时的合规预检）。
    pub fn auto_scan_for_phase(&self, phase: &str) -> Vec<ScannedToolResult> {
        let known_tools = [
            "claim_generator",
            "specification_drafter",
            "patent_responder",
            "novelty_analysis",
            "inventiveness_analysis",
            "infringement_analysis",
            "validity_analysis",
            "comparison_report",
            "oa_strategist",
            "legal_qa",
            "patent_search",
            "quality_checker",
        ];

        let mut results = Vec::new();
        for tool_name in &known_tools {
            let mut active_rules = Vec::new();
            for ruleset in self.rules.values() {
                for rule in ruleset.rules.values() {
                    if !rule.phase.is_empty() && rule.phase != phase {
                        continue;
                    }
                    active_rules.push(RuleSummary {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        action: RuleAction::parse(&rule.action),
                        severity: RuleSeverity::parse(&rule.severity),
                        legal_basis: rule.legal_basis.clone(),
                    });
                }
            }
            if !active_rules.is_empty() {
                results.push(ScannedToolResult {
                    tool_name: tool_name.to_string(),
                    active_rules,
                });
            }
        }
        results
    }

    /// 返回所有规则中出现的唯一阶段列表
    pub fn known_phases(&self) -> Vec<String> {
        let mut phases: Vec<String> = self
            .rules
            .values()
            .flat_map(|rs| rs.rules.values())
            .map(|r| r.phase.clone())
            .filter(|p| !p.is_empty())
            .collect();
        phases.sort();
        phases.dedup();
        phases
    }

    fn evaluate_rule(
        &self,
        rule: &ConstitutionalRule,
        tool_name: &str,
        input_text: &str,
        output_text: Option<&str>,
    ) -> RuleCheckResult {
        checkers::evaluate_rule(rule, tool_name, input_text, output_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个包含 3 条规则（2 个 phase）的测试用 ConstitutionalEngine
    fn test_engine() -> ConstitutionalEngine {
        let rules = ConstitutionalRules {
            rules: vec![
                (
                    "keyword_rule".into(),
                    ConstitutionalRule {
                        id: "KW001".into(),
                        name: "禁用词检查".into(),
                        description: "检查禁用关键词".into(),
                        phase: "drafting".into(),
                        severity: "critical".into(),
                        action: "block".into(),
                        legal_basis: "专利法第25条".into(),
                        check: RuleCheck::KeywordBlocklist {
                            keywords: vec!["赌博".into(), "算命".into()],
                            patterns: vec![],
                            absolute_ban: vec!["色情".into()],
                            context_ban: vec![],
                            negation_context: false,
                            severity_if_found: "critical".into(),
                        },
                    },
                ),
                (
                    "pattern_rule".into(),
                    ConstitutionalRule {
                        id: "PA001".into(),
                        name: "纯软件模式分析".into(),
                        description: "检测纯软件方案".into(),
                        phase: "drafting".into(),
                        severity: "major".into(),
                        action: "warn".into(),
                        legal_basis: "".into(),
                        check: RuleCheck::PatternAnalysis {
                            hardware_integration_markers: vec!["传感器".into(), "芯片".into()],
                            pure_software_markers: vec!["APP".into(), "SaaS".into()],
                            guidance: "需结合硬件".into(),
                        },
                    },
                ),
                (
                    "structural_rule".into(),
                    ConstitutionalRule {
                        id: "SA001".into(),
                        name: "三要素结构检查".into(),
                        description: "检查技术问题/方案/效果".into(),
                        phase: "review".into(),
                        severity: "major".into(),
                        action: "warn".into(),
                        legal_basis: "审查指南第二部分第二章".into(),
                        check: RuleCheck::StructuralAnalysis {
                            requires_all: vec![
                                StructuralElement {
                                    element: "技术问题".into(),
                                    description: "要解决的技术问题".into(),
                                    patterns: vec!["技术问题".into(), "要解决".into()],
                                },
                                StructuralElement {
                                    element: "技术效果".into(),
                                    description: "技术效果描述".into(),
                                    patterns: vec!["有益效果".into(), "技术效果".into()],
                                },
                            ],
                            min_confidence: 0.6,
                        },
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        let mut map = HashMap::new();
        map.insert("test_ruleset".into(), rules);
        ConstitutionalEngine::new(map)
    }

    // ── known_phases ──

    #[test]
    fn known_phases_returns_unique_sorted_phases() {
        let engine = test_engine();
        let phases = engine.known_phases();
        // 3 条规则：2 条 phase="drafting"，1 条 phase="review"
        assert_eq!(phases, vec!["drafting", "review"]);
    }

    #[test]
    fn known_phases_empty_when_all_empty() {
        let rules = ConstitutionalRules {
            rules: vec![(
                "r1".into(),
                ConstitutionalRule {
                    id: "R1".into(),
                    name: "n".into(),
                    description: "d".into(),
                    phase: "".into(),
                    severity: "critical".into(),
                    action: "block".into(),
                    legal_basis: "".into(),
                    check: RuleCheck::KeywordBlocklist {
                        keywords: vec![],
                        patterns: vec![],
                        absolute_ban: vec![],
                        context_ban: vec![],
                        negation_context: false,
                        severity_if_found: "".into(),
                    },
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut map = HashMap::new();
        map.insert("empty".into(), rules);
        let engine = ConstitutionalEngine::new(map);

        assert!(engine.known_phases().is_empty());
    }

    // ── rules_for_phase ──

    #[test]
    fn rules_for_drafting_phase() {
        let engine = test_engine();
        let rules = engine.rules_for_phase("drafting");
        // drafting 有 2 条规则
        assert_eq!(rules.len(), 2);
        let ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"KW001"));
        assert!(ids.contains(&"PA001"));
    }

    #[test]
    fn rules_for_review_phase() {
        let engine = test_engine();
        let rules = engine.rules_for_phase("review");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "SA001");
    }

    #[test]
    fn rules_for_unknown_phase_returns_empty() {
        let engine = test_engine();
        // phase="" 的规则会匹配所有阶段，但我们没有空 phase 规则
        let rules = engine.rules_for_phase("nonexistent");
        assert!(rules.is_empty());
    }

    #[test]
    fn empty_phase_rules_match_all_phases() {
        let rules = ConstitutionalRules {
            rules: vec![(
                "universal".into(),
                ConstitutionalRule {
                    id: "UNI001".into(),
                    name: "通用规则".into(),
                    description: "适用于所有阶段".into(),
                    phase: "".into(), // 空 phase 匹配所有阶段
                    severity: "minor".into(),
                    action: "log".into(),
                    legal_basis: "".into(),
                    check: RuleCheck::KeywordBlocklist {
                        keywords: vec![],
                        patterns: vec![],
                        absolute_ban: vec![],
                        context_ban: vec![],
                        negation_context: false,
                        severity_if_found: "".into(),
                    },
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut map = HashMap::new();
        map.insert("uni".into(), rules);
        let engine = ConstitutionalEngine::new(map);

        // 空 phase 规则在任意阶段都出现
        assert_eq!(engine.rules_for_phase("anything").len(), 1);
        assert_eq!(engine.rules_for_phase("drafting").len(), 1);
    }

    // ── evaluate_rule (通过 check_all) ──

    #[test]
    fn keyword_blocklist_pass_when_no_keyword() {
        let engine = test_engine();
        let results = engine.check_all(
            "claim_generator",
            "这是一段正常的专利文本",
            None,
            "drafting",
        );
        let kw_result = results.iter().find(|r| r.rule_id == "KW001").unwrap();
        assert!(kw_result.passed);
        assert!(kw_result.details.iter().any(|d| d.contains("未命中禁用词")));
    }

    #[test]
    fn keyword_blocklist_fail_when_keyword_found() {
        let engine = test_engine();
        let results = engine.check_all(
            "claim_generator",
            "本发明涉及一种赌博装置",
            None,
            "drafting",
        );
        let kw_result = results.iter().find(|r| r.rule_id == "KW001").unwrap();
        assert!(!kw_result.passed);
        assert!(
            kw_result
                .details
                .iter()
                .any(|d| d.contains("命中禁用词: 赌博"))
        );
    }

    #[test]
    fn keyword_blocklist_detects_absolute_ban() {
        let engine = test_engine();
        let results = engine.check_all("claim_generator", "包含色情内容的检测", None, "drafting");
        let kw_result = results.iter().find(|r| r.rule_id == "KW001").unwrap();
        assert!(!kw_result.passed);
        assert!(
            kw_result
                .details
                .iter()
                .any(|d| d.contains("命中禁用词: 色情"))
        );
    }

    #[test]
    fn pattern_analysis_pass_with_hardware() {
        let engine = test_engine();
        let results = engine.check_all(
            "specification_drafter",
            "本发明使用传感器采集数据，并通过APP展示",
            None,
            "drafting",
        );
        let pa_result = results.iter().find(|r| r.rule_id == "PA001").unwrap();
        // 有硬件标记(传感器)，即使有纯软件标记(APP)也算通过
        assert!(pa_result.passed);
    }

    #[test]
    fn pattern_analysis_fail_pure_software() {
        let engine = test_engine();
        let results = engine.check_all(
            "specification_drafter",
            "本发明是一个SaaS平台，提供APP下载",
            None,
            "drafting",
        );
        let pa_result = results.iter().find(|r| r.rule_id == "PA001").unwrap();
        assert!(!pa_result.passed);
        assert!(pa_result.details.iter().any(|d| d.contains("纯软件方案")));
    }

    #[test]
    fn pattern_analysis_pass_when_no_markers() {
        let engine = test_engine();
        let results = engine.check_all(
            "specification_drafter",
            "一种机械传动装置",
            None,
            "drafting",
        );
        let pa_result = results.iter().find(|r| r.rule_id == "PA001").unwrap();
        assert!(pa_result.passed);
    }

    #[test]
    fn structural_analysis_pass_with_all_elements() {
        let engine = test_engine();
        let results = engine.check_all(
            "quality_checker",
            "本发明要解决的技术问题是X，具有有益效果Y",
            None,
            "review",
        );
        let sa_result = results.iter().find(|r| r.rule_id == "SA001").unwrap();
        assert!(sa_result.passed);
        assert!(sa_result.details.iter().any(|d| d.contains("三要素完整")));
    }

    #[test]
    fn structural_analysis_fail_missing_element() {
        let engine = test_engine();
        let results = engine.check_all("quality_checker", "本发明仅描述了一个方面", None, "review");
        let sa_result = results.iter().find(|r| r.rule_id == "SA001").unwrap();
        assert!(!sa_result.passed);
        // 两个要素都缺失
        assert!(
            sa_result
                .details
                .iter()
                .any(|d| d.contains("缺少要素: 技术问题"))
        );
        assert!(
            sa_result
                .details
                .iter()
                .any(|d| d.contains("缺少要素: 技术效果"))
        );
    }

    #[test]
    fn check_all_filters_by_phase() {
        let engine = test_engine();
        // review 阶段只有 SA001
        let results = engine.check_all("tool", "文本", None, "review");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id, "SA001");
    }

    #[test]
    fn check_result_metadata_correct() {
        let engine = test_engine();
        let results = engine.check_all("tool", "文本", None, "drafting");
        let kw = results.iter().find(|r| r.rule_id == "KW001").unwrap();
        assert!(matches!(kw.severity, RuleSeverity::Critical));
        assert!(matches!(kw.action, RuleAction::Block));
        assert_eq!(kw.legal_basis, "专利法第25条");
        assert_eq!(kw.rule_name, "禁用词检查");
    }

    #[test]
    fn rules_context_for_phase_drafting() {
        let engine = test_engine();
        let ctx = engine.rules_context_for_phase("drafting");
        assert!(ctx.contains("专利合规规则"));
        assert!(ctx.contains("drafting 阶段"));
        assert!(ctx.contains("[BLOCK]"));
        assert!(ctx.contains("[WARN]"));
        assert!(ctx.contains("专利法第25条"));
    }

    #[test]
    fn rules_context_for_phase_review() {
        let engine = test_engine();
        let ctx = engine.rules_context_for_phase("review");
        assert!(ctx.contains("审查指南第二部分第二章"));
    }

    #[test]
    fn rules_context_for_empty_phase() {
        let engine = test_engine();
        let ctx = engine.rules_context_for_phase("nonexistent");
        assert!(ctx.is_empty());
    }

    #[test]
    fn auto_scan_for_phase_drafting() {
        let engine = test_engine();
        let results = engine.auto_scan_for_phase("drafting");
        assert!(!results.is_empty());
        for scanned in &results {
            assert!(!scanned.active_rules.is_empty());
            assert!(!scanned.tool_name.is_empty());
        }
    }

    #[test]
    fn auto_scan_for_phase_review() {
        let engine = test_engine();
        let results = engine.auto_scan_for_phase("review");
        assert!(!results.is_empty());
    }

    #[test]
    fn auto_scan_for_empty_phase_returns_no_tools() {
        let engine = test_engine();
        let results = engine.auto_scan_for_phase("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn category_detection_pass_when_no_match() {
        let rules = ConstitutionalRules {
            rules: vec![(
                "cat_rule".into(),
                ConstitutionalRule {
                    id: "CD001".into(),
                    name: "排除客体检测".into(),
                    description: "检测排除客体".into(),
                    phase: "drafting".into(),
                    severity: "major".into(),
                    action: "block".into(),
                    legal_basis: "专利法第25条".into(),
                    check: RuleCheck::CategoryDetection {
                        categories: vec![(
                            "智力活动".into(),
                            CategoryDef {
                                description: "智力活动规则".into(),
                                patterns: vec!["博弈".into(), "棋类".into()],
                                guidance: "排除".into(),
                            },
                        )]
                        .into_iter()
                        .collect(),
                        assessment: "检测排除客体".into(),
                    },
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut map = HashMap::new();
        map.insert("test".into(), rules);
        let engine = ConstitutionalEngine::new(map);

        let results = engine.check_all("tool", "本发明涉及一种机械装置", None, "drafting");
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn category_detection_fail_when_match() {
        let rules = ConstitutionalRules {
            rules: vec![(
                "cat_rule".into(),
                ConstitutionalRule {
                    id: "CD001".into(),
                    name: "排除客体检测".into(),
                    description: "检测排除客体".into(),
                    phase: "drafting".into(),
                    severity: "critical".into(),
                    action: "block".into(),
                    legal_basis: "".into(),
                    check: RuleCheck::CategoryDetection {
                        categories: vec![(
                            "赌博".into(),
                            CategoryDef {
                                description: "赌博相关".into(),
                                patterns: vec!["老虎机".into()],
                                guidance: "".into(),
                            },
                        )]
                        .into_iter()
                        .collect(),
                        assessment: "".into(),
                    },
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut map = HashMap::new();
        map.insert("test".into(), rules);
        let engine = ConstitutionalEngine::new(map);

        let results = engine.check_all("tool", "本发明是一种老虎机", None, "drafting");
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[test]
    fn specification_analysis_pass() {
        let rules = ConstitutionalRules {
            rules: vec![(
                "spec_rule".into(),
                ConstitutionalRule {
                    id: "SPEC001".into(),
                    name: "说明书分析".into(),
                    description: "分析说明书维度".into(),
                    phase: "review".into(),
                    severity: "major".into(),
                    action: "warn".into(),
                    legal_basis: "".into(),
                    check: RuleCheck::SpecificationAnalysis {
                        dimensions: vec![SpecDimension {
                            dimension: "充分公开".into(),
                            description: "说明书应充分公开".into(),
                            checks: vec!["实施方式".into(), "实施例".into()],
                        }],
                        assessment: "".into(),
                    },
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut map = HashMap::new();
        map.insert("test".into(), rules);
        let engine = ConstitutionalEngine::new(map);

        let results = engine.check_all("tool", "本发明的实施方式和实施例如下", None, "review");
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn specification_analysis_fail() {
        let rules = ConstitutionalRules {
            rules: vec![(
                "spec_rule".into(),
                ConstitutionalRule {
                    id: "SPEC001".into(),
                    name: "说明书分析".into(),
                    description: "分析说明书维度".into(),
                    phase: "review".into(),
                    severity: "major".into(),
                    action: "warn".into(),
                    legal_basis: "".into(),
                    check: RuleCheck::SpecificationAnalysis {
                        dimensions: vec![SpecDimension {
                            dimension: "充分公开".into(),
                            description: "说明书应充分公开".into(),
                            checks: vec!["实施方式".into(), "实施例".into()],
                        }],
                        assessment: "".into(),
                    },
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut map = HashMap::new();
        map.insert("test".into(), rules);
        let engine = ConstitutionalEngine::new(map);

        let results = engine.check_all("tool", "本发明只有简单描述", None, "review");
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[test]
    fn section_structure_pass() {
        let rules = ConstitutionalRules {
            rules: vec![(
                "sec_rule".into(),
                ConstitutionalRule {
                    id: "SEC001".into(),
                    name: "章节结构检查".into(),
                    description: "检查必要章节".into(),
                    phase: "review".into(),
                    severity: "major".into(),
                    action: "warn".into(),
                    legal_basis: "".into(),
                    check: RuleCheck::SectionStructure {
                        required_sections: vec![SectionDef {
                            name: "技术领域".into(),
                            patterns: vec!["技术领域".into()],
                            max_length: "".into(),
                            description: "".into(),
                            subsections: vec![],
                            condition: None,
                        }],
                        forbidden_content: vec![],
                    },
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut map = HashMap::new();
        map.insert("test".into(), rules);
        let engine = ConstitutionalEngine::new(map);

        let results = engine.check_all("tool", "本发明的技术领域涉及机械", None, "review");
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn section_structure_fail_missing() {
        let rules = ConstitutionalRules {
            rules: vec![(
                "sec_rule".into(),
                ConstitutionalRule {
                    id: "SEC001".into(),
                    name: "章节结构检查".into(),
                    description: "检查必要章节".into(),
                    phase: "review".into(),
                    severity: "major".into(),
                    action: "warn".into(),
                    legal_basis: "".into(),
                    check: RuleCheck::SectionStructure {
                        required_sections: vec![SectionDef {
                            name: "技术领域".into(),
                            patterns: vec!["技术领域".into()],
                            max_length: "".into(),
                            description: "".into(),
                            subsections: vec![],
                            condition: None,
                        }],
                        forbidden_content: vec![],
                    },
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut map = HashMap::new();
        map.insert("test".into(), rules);
        let engine = ConstitutionalEngine::new(map);

        let results = engine.check_all("tool", "本发明是一种装置", None, "review");
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[test]
    fn fallback_for_unhandled_check_types() {
        let rules = ConstitutionalRules {
            rules: vec![(
                "scope_rule".into(),
                ConstitutionalRule {
                    id: "SCOPE001".into(),
                    name: "范围比较".into(),
                    description: "比较范围".into(),
                    phase: "review".into(),
                    severity: "minor".into(),
                    action: "log".into(),
                    legal_basis: "".into(),
                    check: RuleCheck::ScopeComparison {
                        direction: "narrower".into(),
                    },
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut map = HashMap::new();
        map.insert("test".into(), rules);
        let engine = ConstitutionalEngine::new(map);

        let results = engine.check_all("tool", "任意文本", None, "review");
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].confidence, 0.5);
    }

    #[test]
    fn rules_returns_reference() {
        let engine = test_engine();
        let rules = engine.rules();
        assert!(rules.contains_key("test_ruleset"));
    }
}
