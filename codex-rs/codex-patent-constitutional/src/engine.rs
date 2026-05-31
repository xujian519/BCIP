use std::collections::HashMap;

use crate::model::*;

#[derive(Debug, Clone)]
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
        _tool_name: &str,
        input_text: &str,
        _output_text: Option<&str>,
    ) -> RuleCheckResult {
        let severity = RuleSeverity::parse(&rule.severity);
        let action = RuleAction::parse(&rule.action);

        let (passed, details, confidence) = match &rule.check {
            RuleCheck::KeywordBlocklist {
                keywords,
                context_ban,
                absolute_ban,
                ..
            } => {
                let all_ban: Vec<&String> = keywords
                    .iter()
                    .chain(context_ban.iter())
                    .chain(absolute_ban.iter())
                    .collect();
                let mut found = Vec::new();
                for pattern in &all_ban {
                    if input_text.contains(pattern.trim_matches('"')) {
                        found.push((*pattern).clone());
                    }
                }
                if found.is_empty() {
                    (true, vec!["未命中禁用词".into()], 0.95)
                } else {
                    (
                        false,
                        found.iter().map(|f| format!("命中禁用词: {}", f)).collect(),
                        0.9,
                    )
                }
            }
            RuleCheck::PatternAnalysis {
                pure_software_markers,
                hardware_integration_markers,
                guidance: _,
            } => {
                let pure_hits: Vec<&String> = pure_software_markers
                    .iter()
                    .filter(|p| input_text.contains(p.trim_matches('"')))
                    .collect();
                let hw_hits: Vec<&String> = hardware_integration_markers
                    .iter()
                    .filter(|p| input_text.contains(p.trim_matches('"')))
                    .collect();
                if !pure_hits.is_empty() && hw_hits.is_empty() {
                    (false, vec!["纯软件方案，需结合硬件分析".into()], 0.7)
                } else {
                    (true, vec!["通过模式分析".into()], 0.85)
                }
            }
            RuleCheck::CategoryDetection {
                categories,
                assessment: _,
            } => {
                let mut matches = Vec::new();
                for (cat_name, cat_def) in categories {
                    let cat_hits: Vec<&String> = cat_def
                        .patterns
                        .iter()
                        .filter(|p| input_text.contains(p.trim_matches('"')))
                        .collect();
                    if !cat_hits.is_empty() {
                        matches.push(format!("[{}] 命中 {} 个模式", cat_name, cat_hits.len()));
                    }
                }
                if matches.is_empty() {
                    (true, vec!["未命中排除客体类别".into()], 0.9)
                } else {
                    (false, matches, 0.8)
                }
            }
            RuleCheck::StructuralAnalysis {
                requires_all,
                min_confidence,
            } => {
                let mut missing = Vec::new();
                for elem in requires_all {
                    let has_elem = elem
                        .patterns
                        .iter()
                        .any(|p| input_text.contains(p.trim_matches('"')));
                    if !has_elem {
                        missing.push(elem.element.clone());
                    }
                }
                if missing.is_empty() {
                    (true, vec!["三要素完整".into()], *min_confidence + 0.2)
                } else {
                    (
                        false,
                        missing.iter().map(|m| format!("缺少要素: {}", m)).collect(),
                        *min_confidence,
                    )
                }
            }
            RuleCheck::SpecificationAnalysis {
                dimensions,
                assessment: _,
            } => {
                let mut dim_results = Vec::new();
                for dim in dimensions {
                    let all_checks_pass = dim
                        .checks
                        .iter()
                        .all(|c| input_text.contains(c.trim_matches('"')));
                    if !all_checks_pass {
                        dim_results.push(format!("维度 '{}' 未全部满足", dim.dimension));
                    }
                }
                if dim_results.is_empty() {
                    (true, vec!["说明书分析维度全部通过".into()], 0.85)
                } else {
                    (false, dim_results, 0.7)
                }
            }
            RuleCheck::SectionStructure {
                required_sections,
                forbidden_content: _,
            } => {
                let mut missing_sections = Vec::new();
                for section in required_sections {
                    let found = section
                        .patterns
                        .iter()
                        .any(|p| input_text.contains(p.trim_matches('"')));
                    if !found {
                        missing_sections.push(section.name.clone());
                    }
                }
                if missing_sections.is_empty() {
                    (true, vec!["章节结构完整".into()], 0.9)
                } else {
                    (
                        false,
                        missing_sections
                            .iter()
                            .map(|s| format!("缺少章节: {}", s))
                            .collect(),
                        0.75,
                    )
                }
            }
            _ => (
                true,
                vec![format!("规则 '{}' 需要深度 LLM 辅助检查", rule.name)],
                0.5,
            ),
        };

        RuleCheckResult {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            severity,
            action,
            legal_basis: rule.legal_basis.clone(),
            passed,
            details,
            confidence,
        }
    }
}

/// 单条规则的摘要信息（用于自动扫描结果）
#[derive(Debug, Clone)]
pub struct RuleSummary {
    pub rule_id: String,
    pub rule_name: String,
    pub action: RuleAction,
    pub severity: RuleSeverity,
    pub legal_basis: String,
}

/// 单个工具的自动扫描结果
#[derive(Debug, Clone)]
pub struct ScannedToolResult {
    pub tool_name: String,
    pub active_rules: Vec<RuleSummary>,
}
