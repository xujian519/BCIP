//! 审查员模拟器(规则层)
//!
//! 用于 OA 答复预演与答复质量评估。
//! 不调用 LLM,纯规则引擎实现。

mod response;
mod scoring;
mod simulator;
mod types;

pub use types::ArgumentationDialog;
pub use types::ArgumentationLibrary;
pub use types::ArgumentationRound;
pub use types::ArgumentationStrategy;
pub use types::ClaimObjection;
pub use types::EvaluationOutput;
pub use types::EvaluationScores;
pub use types::ExaminerSimulator;
pub use types::ObjectionTemplate;
pub use types::Rebuttal;
pub use types::RespondOutput;
pub use types::SimulateReviewOutput;
