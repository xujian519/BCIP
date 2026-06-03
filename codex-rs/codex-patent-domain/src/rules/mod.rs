//! YAML 规则引擎 — 专利文档规则检查。
//!
//! 支持从 YAML 加载规则,对专利文档进行多维度检查。

pub mod checks;
pub mod engine;
pub mod regex_cache;
pub mod schema;

pub use engine::evaluate;
pub use engine::load_rules;
pub use schema::Check;
pub use schema::PatentDocument;
pub use schema::Rule;
pub use schema::RuleFile;
pub use schema::RuleViolation;
pub use schema::Severity;
pub use schema::Target;
