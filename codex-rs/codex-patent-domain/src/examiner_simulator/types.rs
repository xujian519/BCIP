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

// ==================== 输出类型 ====================

/// 权利要求异议
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimObjection {
    pub claim_number: usize,
    pub claim_text: String,
    pub feature_objections: Vec<String>,
    pub conclusion: &'static str,
}

/// 初次审查输出
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateReviewOutput {
    pub rejection_type: &'static str,
    pub strategy: &'static str,
    pub objections: Vec<ClaimObjection>,
    pub overall_conclusion: &'static str,
    pub integration_mode: &'static str,
}

/// 反驳内容
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rebuttal {
    pub rebuttal_points: Vec<String>,
    pub remaining_concerns: Vec<&'static str>,
    pub suggestions: Vec<&'static str>,
    pub tone: &'static str,
}

/// 审查员对申请人答复的输出
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RespondOutput {
    pub round_number: u32,
    pub response_strategy: &'static str,
    pub rebuttal: Rebuttal,
    pub applicant_points_addressed: Option<Vec<String>>,
    pub integration_mode: &'static str,
}

/// 答复质量评分明细
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationScores {
    pub completeness: f64,
    pub persuasiveness: f64,
    pub technical_depth: f64,
    pub logic_consistency: f64,
}

/// 答复评估输出
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationOutput {
    pub overall_score: f64,
    pub scores: EvaluationScores,
    pub strengths: Vec<&'static str>,
    pub weaknesses: Vec<&'static str>,
    pub recommendations: Vec<&'static str>,
    pub predicted_outcome: &'static str,
    pub integration_mode: &'static str,
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
