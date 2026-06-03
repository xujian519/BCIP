//! codex-patent-text — 专利文本处理工具集。
//!
//! 提供 IPC 分类器、文本相似度计算、分词器等专利文本分析基础功能。

pub mod classification;
pub mod similarity;
pub mod tokenizer;

pub use classification::IpcClassifier;
pub use classification::IpcResult;
pub use similarity::*;
pub use tokenizer::*;
