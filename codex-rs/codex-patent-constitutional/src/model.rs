use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalRules {
    pub rules: HashMap<String, ConstitutionalRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalRule {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub phase: String,
    pub severity: String,
    pub action: String,
    #[serde(default)]
    pub legal_basis: String,
    pub check: RuleCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RuleCheck {
    #[serde(rename = "structural_analysis")]
    StructuralAnalysis {
        requires_all: Vec<StructuralElement>,
        #[serde(default)]
        min_confidence: f64,
    },
    #[serde(rename = "keyword_blocklist")]
    KeywordBlocklist {
        #[serde(default)]
        keywords: Vec<String>,
        #[serde(default)]
        patterns: Vec<String>,
        #[serde(default)]
        absolute_ban: Vec<String>,
        #[serde(default)]
        context_ban: Vec<String>,
        #[serde(default)]
        negation_context: bool,
        #[serde(default)]
        severity_if_found: String,
    },
    #[serde(rename = "category_detection")]
    CategoryDetection {
        categories: HashMap<String, CategoryDef>,
        #[serde(default)]
        assessment: String,
    },
    #[serde(rename = "pattern_analysis")]
    PatternAnalysis {
        #[serde(default)]
        hardware_integration_markers: Vec<String>,
        #[serde(default)]
        pure_software_markers: Vec<String>,
        #[serde(default)]
        guidance: String,
    },
    #[serde(rename = "specification_analysis")]
    SpecificationAnalysis {
        dimensions: Vec<SpecDimension>,
        #[serde(default)]
        assessment: String,
    },
    #[serde(rename = "section_structure")]
    SectionStructure {
        #[serde(default)]
        required_sections: Vec<SectionDef>,
        #[serde(default)]
        forbidden_content: Vec<String>,
    },
    #[serde(rename = "claim_clarity_analysis")]
    ClaimClarityAnalysis {
        #[serde(default)]
        unclear_terms: Vec<String>,
        #[serde(default)]
        over_broad: Vec<String>,
        #[serde(default)]
        mixed_categories: MixedCategoriesDef,
        #[serde(default)]
        chained_references: ChainedRefDef,
        #[serde(default)]
        assessment: String,
    },
    #[serde(rename = "support_analysis")]
    SupportAnalysis {
        methods: Vec<SupportMethod>,
        #[serde(default)]
        severity_if_unsupported: String,
    },
    #[serde(rename = "essential_feature_analysis")]
    EssentialFeatureAnalysis {
        principles: Vec<String>,
        indicators: IndicatorsDef,
    },
    #[serde(rename = "dependency_validation")]
    DependencyValidation { rules: Vec<DepRule> },
    #[serde(rename = "novelty_analysis")]
    NoveltyAnalysis {
        #[serde(default)]
        prior_art_scope: Vec<String>,
        comparison_principles: Vec<ComparisonPrinciple>,
    },
    #[serde(rename = "grace_period_analysis")]
    GracePeriodAnalysis { conditions: Vec<GraceCondition> },
    #[serde(rename = "inventiveness_analysis")]
    InventivenessAnalysis {
        method: String,
        steps: Vec<InventivenessStep>,
        #[serde(default)]
        secondary_indicators: SecondaryIndicators,
        #[serde(default)]
        standard_lower: bool,
    },
    #[serde(rename = "utility_analysis")]
    UtilityAnalysis {
        grounds_for_rejection: Vec<RejectionGround>,
    },
    #[serde(rename = "unity_analysis")]
    UnityAnalysis {
        same_inventive_concept: UnifiedCriteria,
        allowed_combinations: Vec<String>,
        #[serde(default)]
        guidance: String,
    },
    #[serde(rename = "divisional_rules")]
    DivisionalRules {
        timing: Vec<String>,
        constraints: Vec<String>,
    },
    #[serde(rename = "amendment_analysis")]
    AmendmentAnalysis {
        principles: Vec<AmendmentPrinciple>,
        permissible: Vec<String>,
    },
    #[serde(rename = "scope_comparison")]
    ScopeComparison { direction: String },
    #[serde(rename = "timing_analysis")]
    TimingAnalysis {
        #[serde(default)]
        invention: Vec<String>,
        #[serde(default)]
        utility: Vec<String>,
        #[serde(default)]
        design: Vec<String>,
    },
    #[serde(rename = "priority_analysis")]
    PriorityAnalysis {
        priority_type: String,
        #[serde(default)]
        time_limit: HashMap<String, String>,
        #[serde(default)]
        requirements: Vec<String>,
        #[serde(default)]
        constraints: Vec<String>,
        #[serde(default)]
        special_notes: Vec<String>,
    },
    #[serde(rename = "same_subject_analysis")]
    SameSubjectAnalysis {
        criteria: Vec<String>,
        assessment: String,
    },
    #[serde(rename = "deadline_analysis")]
    DeadlineAnalysis {
        deadlines: Vec<DeadlineDef>,
        consequences: Vec<String>,
    },
    #[serde(rename = "oa_response_strategy")]
    OaResponseStrategy {
        oa_type: String,
        valid_strategies: Vec<StrategyDef>,
        #[serde(default)]
        invalid_strategies: Vec<String>,
    },
    #[serde(rename = "reexamination_rules")]
    ReexaminationRules {
        requirements: Vec<String>,
        scope: Vec<String>,
    },
    #[serde(rename = "invalidation_analysis")]
    InvalidationAnalysis {
        grounds: Vec<InvalidGround>,
        restrictions: Vec<String>,
    },
    #[serde(rename = "invalidation_amendment_rules")]
    InvalidationAmendmentRules {
        allowed: Vec<AmendmentMethod>,
        forbidden: Vec<String>,
    },
    #[serde(rename = "infringement_analysis")]
    InfringementAnalysis {
        principles: Vec<InfringementPrinciple>,
        defenses: Vec<DefenseDef>,
    },
    #[serde(rename = "damages_analysis")]
    DamagesAnalysis {
        calculation_order: Vec<DamageMethod>,
        punitive: PunitiveDef,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralElement {
    pub element: String,
    pub description: String,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDef {
    pub description: String,
    pub patterns: Vec<String>,
    #[serde(default)]
    pub guidance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDimension {
    pub dimension: String,
    pub description: String,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionDef {
    pub name: String,
    pub patterns: Vec<String>,
    #[serde(default)]
    pub max_length: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub subsections: Vec<String>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MixedCategoriesDef {
    pub description: String,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChainedRefDef {
    pub description: String,
    pub rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportMethod {
    pub method: String,
    pub description: String,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorsDef {
    #[serde(default)]
    pub too_many: IndicatorDef,
    #[serde(default)]
    pub too_few: IndicatorDef,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndicatorDef {
    pub description: String,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepRule {
    pub rule: String,
    pub description: String,
    #[serde(default)]
    pub error_pattern: String,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonPrinciple {
    pub principle: String,
    pub description: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraceCondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub description: String,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventivenessStep {
    pub step: u32,
    pub name: String,
    pub criteria: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecondaryIndicators {
    #[serde(default)]
    pub positive: Vec<String>,
    #[serde(default)]
    pub negative: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionGround {
    pub ground: String,
    pub description: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnifiedCriteria {
    pub criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendmentPrinciple {
    pub principle: String,
    pub description: String,
    #[serde(default)]
    pub detail: String,
    pub forbidden: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlineDef {
    pub scenario: String,
    pub description: String,
    pub period: String,
    #[serde(default)]
    pub extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDef {
    pub strategy: String,
    pub description: String,
    #[serde(default)]
    pub efficacy: String,
    #[serde(default)]
    pub details: Vec<String>,
    #[serde(default)]
    pub constraint: String,
    pub requirement: Option<String>,
    pub condition: Option<String>,
    pub factors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidGround {
    pub ground: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendmentMethod {
    pub method: String,
    pub description: String,
    pub constraint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfringementPrinciple {
    pub principle: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseDef {
    pub defense: String,
    pub name: String,
    pub description: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageMethod {
    pub method: String,
    pub description: String,
    pub priority: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunitiveDef {
    pub condition: String,
    pub multiplier: String,
    pub legal_basis: String,
}

#[derive(Debug, Clone)]
pub enum RuleSeverity {
    Critical,
    Major,
    Minor,
}

impl RuleSeverity {
    pub fn parse(s: &str) -> Self {
        match s {
            "critical" => Self::Critical,
            "major" => Self::Major,
            _ => Self::Minor,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    Block,
    Warn,
    Review,
    Enforce,
    Log,
}

impl RuleAction {
    pub fn parse(s: &str) -> Self {
        match s {
            "block" => Self::Block,
            "warn" => Self::Warn,
            "review" => Self::Review,
            "enforce" => Self::Enforce,
            "log" => Self::Log,
            _ => Self::Warn,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── RuleSeverity::parse ──

    #[test]
    fn severity_critical() {
        assert!(matches!(
            RuleSeverity::parse("critical"),
            RuleSeverity::Critical
        ));
    }

    #[test]
    fn severity_major() {
        assert!(matches!(RuleSeverity::parse("major"), RuleSeverity::Major));
    }

    #[test]
    fn severity_fallback_minor() {
        assert!(matches!(RuleSeverity::parse("info"), RuleSeverity::Minor));
        assert!(matches!(RuleSeverity::parse(""), RuleSeverity::Minor));
    }

    // ── RuleAction::parse ──

    #[test]
    fn action_block() {
        assert!(matches!(RuleAction::parse("block"), RuleAction::Block));
    }

    #[test]
    fn action_warn() {
        assert!(matches!(RuleAction::parse("warn"), RuleAction::Warn));
    }

    #[test]
    fn action_review() {
        assert!(matches!(RuleAction::parse("review"), RuleAction::Review));
    }

    #[test]
    fn action_enforce() {
        assert!(matches!(RuleAction::parse("enforce"), RuleAction::Enforce));
    }

    #[test]
    fn action_log() {
        assert!(matches!(RuleAction::parse("log"), RuleAction::Log));
    }

    #[test]
    fn action_fallback_warn() {
        assert!(matches!(RuleAction::parse("unknown"), RuleAction::Warn));
    }

    // ── RuleCheck 反序列化 ──

    #[test]
    fn deserialize_keyword_blocklist() {
        let json = r#"{
            "type": "keyword_blocklist",
            "keywords": ["算法", "商业方法"],
            "absolute_ban": ["赌博"],
            "context_ban": ["区块链"],
            "negation_context": false,
            "severity_if_found": "critical"
        }"#;
        let check: RuleCheck = serde_json::from_str(json).unwrap();
        match check {
            RuleCheck::KeywordBlocklist {
                keywords,
                absolute_ban,
                context_ban,
                ..
            } => {
                assert_eq!(keywords, vec!["算法", "商业方法"]);
                assert_eq!(absolute_ban, vec!["赌博"]);
                assert_eq!(context_ban, vec!["区块链"]);
            }
            other => panic!("expected KeywordBlocklist, got {:?}", other),
        }
    }

    #[test]
    fn deserialize_pattern_analysis() {
        let json = r#"{
            "type": "pattern_analysis",
            "hardware_integration_markers": ["传感器", "处理器"],
            "pure_software_markers": ["APP", "SaaS"],
            "guidance": "需结合硬件"
        }"#;
        let check: RuleCheck = serde_json::from_str(json).unwrap();
        match check {
            RuleCheck::PatternAnalysis {
                hardware_integration_markers,
                pure_software_markers,
                guidance,
            } => {
                assert_eq!(hardware_integration_markers, vec!["传感器", "处理器"]);
                assert_eq!(pure_software_markers, vec!["APP", "SaaS"]);
                assert_eq!(guidance, "需结合硬件");
            }
            other => panic!("expected PatternAnalysis, got {:?}", other),
        }
    }

    #[test]
    fn deserialize_structural_analysis() {
        let json = r#"{
            "type": "structural_analysis",
            "requires_all": [
                {"element": "技术问题", "description": "要解决的技术问题", "patterns": ["技术问题", "解决"]},
                {"element": "技术方案", "description": "技术方案描述", "patterns": ["技术方案", "实现"]}
            ],
            "min_confidence": 0.7
        }"#;
        let check: RuleCheck = serde_json::from_str(json).unwrap();
        match check {
            RuleCheck::StructuralAnalysis {
                requires_all,
                min_confidence,
            } => {
                assert_eq!(requires_all.len(), 2);
                assert_eq!(requires_all[0].element, "技术问题");
                assert_eq!(requires_all[1].patterns, vec!["技术方案", "实现"]);
                assert!((min_confidence - 0.7).abs() < f64::EPSILON);
            }
            other => panic!("expected StructuralAnalysis, got {:?}", other),
        }
    }

    // ── ConstitutionalRule 完整反序列化 ──

    #[test]
    fn deserialize_full_constitutional_rule() {
        let json = r#"{
            "id": "R001",
            "name": "禁用词检查",
            "description": "检查禁用关键词",
            "phase": "drafting",
            "severity": "critical",
            "action": "block",
            "legal_basis": "专利法第25条",
            "check": {
                "type": "keyword_blocklist",
                "keywords": ["算法"],
                "absolute_ban": [],
                "context_ban": [],
                "negation_context": false,
                "severity_if_found": "critical"
            }
        }"#;
        let rule: ConstitutionalRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.id, "R001");
        assert_eq!(rule.name, "禁用词检查");
        assert_eq!(rule.phase, "drafting");
        assert_eq!(rule.severity, "critical");
        assert_eq!(rule.action, "block");
        assert_eq!(rule.legal_basis, "专利法第25条");
        assert!(matches!(rule.check, RuleCheck::KeywordBlocklist { .. }));
    }

    #[test]
    fn deserialize_rule_with_defaults() {
        let json = r#"{
            "id": "R002",
            "name": "测试规则",
            "description": "desc",
            "severity": "major",
            "action": "warn",
            "check": {
                "type": "keyword_blocklist"
            }
        }"#;
        let rule: ConstitutionalRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.phase, ""); // serde(default)
        assert_eq!(rule.legal_basis, ""); // serde(default)
        if let RuleCheck::KeywordBlocklist {
            keywords,
            patterns,
            absolute_ban,
            context_ban,
            negation_context,
            ..
        } = &rule.check
        {
            assert!(keywords.is_empty());
            assert!(patterns.is_empty());
            assert!(absolute_ban.is_empty());
            assert!(context_ban.is_empty());
            assert!(!negation_context);
        } else {
            panic!("expected KeywordBlocklist");
        }
    }

    // ── 数据类型 serde round-trip ──

    #[test]
    fn structural_element_roundtrip() {
        let elem = StructuralElement {
            element: "技术问题".into(),
            description: "要解决的技术问题".into(),
            patterns: vec!["问题".into(), "解决".into()],
        };
        let json = serde_json::to_string(&elem).unwrap();
        let back: StructuralElement = serde_json::from_str(&json).unwrap();
        assert_eq!(back.element, "技术问题");
        assert_eq!(back.patterns, vec!["问题", "解决"]);
    }

    #[test]
    fn category_def_roundtrip() {
        let cat = CategoryDef {
            description: "智力活动".into(),
            patterns: vec!["博弈".into()],
            guidance: "注意排除".into(),
        };
        let json = serde_json::to_string(&cat).unwrap();
        let back: CategoryDef = serde_json::from_str(&json).unwrap();
        assert_eq!(back.description, "智力活动");
        assert_eq!(back.guidance, "注意排除");
    }

    #[test]
    fn spec_dimension_roundtrip() {
        let dim = SpecDimension {
            dimension: "充分公开".into(),
            description: "说明书应充分公开".into(),
            checks: vec!["实施方式".into(), "具体实施例".into()],
        };
        let json = serde_json::to_string(&dim).unwrap();
        let back: SpecDimension = serde_json::from_str(&json).unwrap();
        assert_eq!(back.dimension, "充分公开");
        assert_eq!(back.checks.len(), 2);
    }

    #[test]
    fn section_def_roundtrip() {
        let sec = SectionDef {
            name: "技术领域".into(),
            patterns: vec!["技术领域".into()],
            max_length: "500字".into(),
            description: "技术领域章节".into(),
            subsections: vec![],
            condition: Some("必须包含".into()),
        };
        let json = serde_json::to_string(&sec).unwrap();
        let back: SectionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "技术领域");
        assert_eq!(back.condition, Some("必须包含".into()));
    }
}
