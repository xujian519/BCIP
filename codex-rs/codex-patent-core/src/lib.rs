//! codex-patent-core — BCIP 专利核心类型与错误模型。
//!
//! 被所有其他专利 crate 依赖。提供通用领域类型（`types.rs`）、
//! 统一错误模型（`PatentError` / `ApiKeyError`），以及工具域分类
//! `ToolDomain`。

pub mod error;
pub mod http;
mod types;

pub use error::ApiKeyError;
pub use error::ERR_FATAL_PREFIX;
pub use error::ERR_RETRYABLE_PREFIX;
pub use error::PatentError;
pub use error::ToolErrorKind;
pub use error::classify_tool_error;
pub use error::fatal_err;
pub use error::retryable_err;
pub use types::*;
