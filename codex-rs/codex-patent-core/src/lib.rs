//! codex-patent-core — BCIP 专利核心类型与错误模型。
//!
//! 被所有其他专利 crate 依赖。提供通用领域类型（`types.rs`）、
//! 统一错误模型（`PatentError` / `ApiKeyError`），以及工具域分类
//! `ToolDomain`。

mod error;
pub mod http;
mod types;

pub use error::ApiKeyError;
pub use error::PatentError;
pub use types::*;
