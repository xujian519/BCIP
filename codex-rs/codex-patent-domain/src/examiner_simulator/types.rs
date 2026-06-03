//! 审查员模拟器数据类型

use serde::Deserialize;
use serde::Serialize;

use codex_patent_core::RejectionType;

/// 论证策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgumentationStrategy {
    StrictLiteral,
    BroadInterpretation,
    CombinationAnalysis,
    HindsightBias,
}

impl ArgumentationStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StrictLiteral => "strict_literal",
            Self::BroadInterpretation => "broad_interpretation",
            Self::CombinationAnalysis => "combination_analysis",
            Self::HindsightBias => "hindsight_bias",
        }
    }
}

/// 审查员模拟器(规则层)
#[derive(Debug)]
pub struct ExaminerSimulator {
    pub(crate) rejection_type: RejectionType,
    pub(crate) current_strategy: ArgumentationStrategy,
}

impl Default for ExaminerSimulator {
    fn default() -> Self {
        Self::new()
    }
}

/// 论证轮次
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentationRound {
    pub round_number: usize,
    pub examiner_objection: String,
    pub reasoning_template: String,
    pub strategy: ArgumentationStrategy,
}

/// 论证模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionTemplate {
    pub rejection_type: RejectionType,
    pub strategy: ArgumentationStrategy,
    pub templates: Vec<String>,
    pub description: String,
}

/// 论证模式库
pub struct ArgumentationLibrary;

/// 多轮论证对话
pub struct ArgumentationDialog {
    pub rounds: Vec<ArgumentationRound>,
    pub rejection_type: RejectionType,
    pub current_round: usize,
}

pub(crate) fn rejection_type_as_str(ty: &RejectionType) -> &'static str {
    match ty {
        RejectionType::Inventiveness => "inventiveness",
        RejectionType::Obviousness => "obviousness",
        RejectionType::LackOfNovelty => "lack_of_novelty",
        RejectionType::InsufficientDisclosure => "insufficient_disclosure",
        RejectionType::UnpatentableSubject => "unpatentable_subject",
    }
}
