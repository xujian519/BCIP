//! 规则求值引擎 — 加载 YAML 规则并执行检查。

use super::checks;
use super::regex_cache::get_or_compile_regex;
use super::schema::Check;
use super::schema::PatentDocument;
use super::schema::Rule;
use super::schema::RuleFile;
use super::schema::RuleViolation;
use super::schema::Target;

/// 从 YAML 字符串加载规则列表。
pub fn load_rules(yaml_str: &str) -> Result<Vec<Rule>, String> {
    let file: RuleFile = serde_yaml::from_str(yaml_str).map_err(|e| e.to_string())?;
    Ok(file.rules)
}

/// 对专利文档执行全部规则检查,返回违反列表。
pub fn evaluate(doc: &PatentDocument, rules: &[Rule]) -> Vec<RuleViolation> {
    rules.iter().flat_map(|rule| check_one(doc, rule)).collect()
}

/// 对单条规则执行检查。
fn check_one(doc: &PatentDocument, rule: &Rule) -> Vec<RuleViolation> {
    match &rule.check {
        Check::Required { field, message } => {
            if checks::check_required(doc, rule.target, field) {
                vec![]
            } else {
                let loc = checks::location_for(rule.target, field);
                vec![RuleViolation {
                    rule_id: rule.id.clone(),
                    severity: rule.severity,
                    message: message
                        .clone()
                        .unwrap_or_else(|| format!("字段 {field} 不能为空")),
                    location: loc,
                }]
            }
        }
        Check::Pattern {
            field,
            pattern,
            message,
        } => {
            if rule.target == Target::Claims && field == "text" {
                return check_claims_pattern(doc, rule, pattern, message);
            }
            if checks::check_pattern(doc, rule.target, field, pattern) {
                return vec![];
            }
            let loc = checks::location_for(rule.target, field);
            vec![RuleViolation {
                rule_id: rule.id.clone(),
                severity: rule.severity,
                message: message
                    .clone()
                    .unwrap_or_else(|| format!("字段 {field} 不匹配模式 {pattern}")),
                location: loc,
            }]
        }
        Check::MinLength { field, value } => {
            if checks::check_min_length(doc, rule.target, field, *value) {
                vec![]
            } else {
                let loc = checks::location_for(rule.target, field);
                vec![RuleViolation {
                    rule_id: rule.id.clone(),
                    severity: rule.severity,
                    message: format!("字段 {field} 长度不足 {value}"),
                    location: loc,
                }]
            }
        }
        Check::MaxLength { field, value } => {
            if checks::check_max_length(doc, rule.target, field, *value) {
                vec![]
            } else {
                let loc = checks::location_for(rule.target, field);
                vec![RuleViolation {
                    rule_id: rule.id.clone(),
                    severity: rule.severity,
                    message: format!("字段 {field} 长度超过 {value}"),
                    location: loc,
                }]
            }
        }
        Check::Enum {
            field,
            values,
            message,
        } => {
            let values_ref: Vec<&str> = values.iter().map(String::as_str).collect();
            if checks::check_enum(doc, rule.target, field, &values_ref) {
                vec![]
            } else {
                let loc = checks::location_for(rule.target, field);
                vec![RuleViolation {
                    rule_id: rule.id.clone(),
                    severity: rule.severity,
                    message: message.clone().unwrap_or_else(|| {
                        format!("字段 {field} 必须是以下值之一: {}", values.join(", "))
                    }),
                    location: loc,
                }]
            }
        }
    }
}

/// 逐条检查 claims.text 的模式匹配。
fn check_claims_pattern(
    doc: &PatentDocument,
    rule: &Rule,
    pattern: &str,
    message: &Option<String>,
) -> Vec<RuleViolation> {
    let Some(re) = get_or_compile_regex(pattern) else {
        return vec![RuleViolation {
            rule_id: rule.id.clone(),
            severity: rule.severity,
            message: format!("无效正则模式: {pattern}"),
            location: "claims".into(),
        }];
    };

    let mut violations = Vec::new();
    for (idx, claim) in doc.claims.iter().enumerate() {
        if !re.is_match(claim) {
            violations.push(RuleViolation {
                rule_id: rule.id.clone(),
                severity: rule.severity,
                message: message
                    .clone()
                    .unwrap_or_else(|| format!("权利要求 {idx} 不匹配模式 {pattern}")),
                location: format!("claims[{idx}]"),
            });
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_rules() {
        let yaml = r#"
rules:
  - id: R001
    name: 标题必填
    target: specification
    severity: error
    check:
      type: required
      field: title
"#;
        let rules = load_rules(yaml).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "R001");
    }

    #[test]
    fn test_evaluate_required() {
        let yaml = r#"
rules:
  - id: R001
    name: 标题必填
    target: specification
    severity: error
    check:
      type: required
      field: title
"#;
        let rules = load_rules(yaml).unwrap();

        let mut doc = PatentDocument::default();
        let violations = evaluate(&doc, &rules);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "R001");

        doc.title = Some("测试标题".into());
        let violations = evaluate(&doc, &rules);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_evaluate_pattern() {
        let yaml = r#"
rules:
  - id: R002
    name: 权利要求包含特征词
    target: claims
    severity: warning
    check:
      type: pattern
      field: text
      pattern: "包括"
"#;
        let rules = load_rules(yaml).unwrap();

        let mut doc = PatentDocument {
            claims: vec!["一种方法,步骤A。".into()],
            ..Default::default()
        };
        let violations = evaluate(&doc, &rules);
        assert_eq!(violations.len(), 1);

        doc.claims = vec!["一种方法,包括步骤A。".into()];
        let violations = evaluate(&doc, &rules);
        assert!(violations.is_empty());
    }
}
