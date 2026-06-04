//! codex-patent-constitutional — 专利合规范性规则引擎。
//!
//! 加载 YAML 格式合规规则，对专利撰写/审查流程进行逐条检查。
//! 核心概念：`ConstitutionalRule`（规则定义）、`ConstitutionalEngine`（规则执行引擎）。

pub mod engine;
pub mod loader;
pub mod model;

pub use engine::ConstitutionalEngine;
pub use engine::RuleCheckResult;
pub use engine::RuleSummary;
pub use engine::ScannedToolResult;
pub use loader::RuleLoader;
pub use model::ConstitutionalRule;
pub use model::ConstitutionalRules;
pub use model::RuleAction;
pub use model::RuleCheck;
pub use model::RuleSeverity;
