use serde::Deserialize;
use serde::Serialize;

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
