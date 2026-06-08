//! codex-patent-constitutional — 专利合规范性规则引擎。
//!
//! 加载 YAML 格式合规规则，对专利撰写/审查流程进行逐条检查。
//! 核心概念：`ConstitutionalRule`（规则定义）、`ConstitutionalEngine`（规则执行引擎）。

mod checkers;
pub mod engine;
pub mod loader;
pub mod model;
pub mod types;

pub use engine::ConstitutionalEngine;
pub use loader::RuleLoader;
pub use model::ConstitutionalRule;
pub use model::ConstitutionalRules;
pub use model::RuleAction;
pub use model::RuleCheck;
pub use model::RuleSeverity;
pub use types::RuleCheckResult;
pub use types::RuleSummary;
pub use types::ScannedToolResult;
