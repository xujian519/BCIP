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
