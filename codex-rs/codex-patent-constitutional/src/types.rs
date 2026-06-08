use crate::model::{RuleAction, RuleSeverity};

/// 单条规则的检查结果。
pub struct RuleCheckResult {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: RuleSeverity,
    pub action: RuleAction,
    pub legal_basis: String,
    pub passed: bool,
    pub details: Vec<String>,
    pub confidence: f64,
}

/// 单条规则的摘要信息（用于自动扫描结果）。
pub struct RuleSummary {
    pub rule_id: String,
    pub rule_name: String,
    pub action: RuleAction,
    pub severity: RuleSeverity,
    pub legal_basis: String,
}

/// 单个工具的自动扫描结果。
pub struct ScannedToolResult {
    pub tool_name: String,
    pub active_rules: Vec<RuleSummary>,
}
