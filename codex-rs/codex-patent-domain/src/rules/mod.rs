//! YAML 规则引擎 — 专利文档规则检查。
//!
//! 支持从 YAML 加载规则,对专利文档进行多维度检查。

pub mod checks;
pub mod engine;
pub mod schema;

pub use engine::{evaluate, load_rules};
pub use schema::{Check, PatentDocument, Rule, RuleFile, RuleViolation, Severity, Target};
