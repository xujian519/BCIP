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
