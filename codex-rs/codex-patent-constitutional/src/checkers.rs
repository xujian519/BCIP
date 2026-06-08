use crate::model::*;
use crate::types::RuleCheckResult;

pub fn evaluate_rule(
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
