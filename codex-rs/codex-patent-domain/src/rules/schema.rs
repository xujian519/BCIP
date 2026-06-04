//! 规则 YAML schema 定义。

use serde::Deserialize;
use serde::Serialize;

/// 规则文件顶层结构。
#[derive(Debug, Deserialize)]
pub struct RuleFile {
    pub rules: Vec<Rule>,
}

/// 单条规则。
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub target: Target,
    pub severity: Severity,
    pub check: Check,
}

/// 检查目标:说明书 / 权利要求书 / 摘要。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Specification,
    Claims,
    #[serde(rename = "abstract")]
    Abstract,
}

/// 严重级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// 具体检查类型。
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Check {
    Required {
        field: String,
        #[serde(default)]
        message: Option<String>,
    },
    Pattern {
        field: String,
        pattern: String,
        #[serde(default)]
        message: Option<String>,
    },
    #[serde(rename = "min_length")]
    MinLength { field: String, value: usize },
    #[serde(rename = "max_length")]
    MaxLength { field: String, value: usize },
    #[serde(rename = "enum")]
    Enum {
        field: String,
        values: Vec<String>,
        #[serde(default)]
        message: Option<String>,
    },
}

/// 待检查的专利文档(简化结构)。
#[derive(Debug, Clone, Default)]
pub struct PatentDocument {
    pub title: Option<String>,
    pub abstract_text: Option<String>,
    pub claims: Vec<String>,
    pub specification: Option<String>,
    pub drawings: Vec<String>,
}

/// 规则违反记录。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuleViolation {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub location: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_file_deserialize() {
        let yaml = r#"
rules:
  - id: R001
    name: "标题必填"
    target: specification
    severity: error
    check:
      type: required
      field: title
"#;
        let rf: RuleFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(rf.rules.len(), 1);
        assert_eq!(rf.rules[0].id, "R001");
        assert_eq!(rf.rules[0].target, Target::Specification);
        assert_eq!(rf.rules[0].severity, Severity::Error);
    }

    #[test]
    fn target_deserialize_variants() {
        assert_eq!(
            serde_yaml::from_str::<Target>("specification").unwrap(),
            Target::Specification
        );
        assert_eq!(
            serde_yaml::from_str::<Target>("claims").unwrap(),
            Target::Claims
        );
        assert_eq!(
            serde_yaml::from_str::<Target>("abstract").unwrap(),
            Target::Abstract
        );
    }

    #[test]
    fn check_pattern_deserialize() {
        let yaml = r#"
type: pattern
field: title
pattern: "^\\S"
message: "标题不能以空白开头"
"#;
        let check: Check = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(check, Check::Pattern { field, .. } if field == "title"));
    }

    #[test]
    fn check_min_max_length_deserialize() {
        let yaml_min = r#"
type: min_length
field: claims
value: 1
"#;
        let check: Check = serde_yaml::from_str(yaml_min).unwrap();
        assert!(
            matches!(check, Check::MinLength { field, value } if field == "claims" && value == 1)
        );

        let yaml_max = r#"
type: max_length
field: abstract_text
value: 300
"#;
        let check: Check = serde_yaml::from_str(yaml_max).unwrap();
        assert!(
            matches!(check, Check::MaxLength { field, value } if field == "abstract_text" && value == 300)
        );
    }

    #[test]
    fn patent_document_default() {
        let doc = PatentDocument::default();
        assert!(doc.title.is_none());
        assert!(doc.abstract_text.is_none());
        assert!(doc.claims.is_empty());
        assert!(doc.specification.is_none());
        assert!(doc.drawings.is_empty());
    }

    #[test]
    fn rule_violation_equality() {
        let v1 = RuleViolation {
            rule_id: "R001".into(),
            severity: Severity::Error,
            message: "missing title".into(),
            location: "title".into(),
        };
        let v2 = RuleViolation {
            rule_id: "R001".into(),
            severity: Severity::Error,
            message: "missing title".into(),
            location: "title".into(),
        };
        assert_eq!(v1, v2);
    }
}
