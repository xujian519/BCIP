//! 工作流类型定义。

use serde::Deserialize;
use serde::Serialize;

/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FlowStep {
    AgentCall {
        agent_name: String,
        prompt: String,
    },
    AgentTool {
        agent_name: String,
        input: serde_json::Value,
    },
    QualityCheck {
        criteria: Vec<String>,
    },
    HumanApproval {
        title: String,
        description: String,
    },
    ToolCall {
        tool_name: String,
        input: serde_json::Value,
    },
    CodeBlock {
        language: String,
        code: String,
    },
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    pub id: String,
    pub name: String,
    pub steps: Vec<FlowStep>,
    pub retry_on_failure: Option<u32>,
}

/// 工作流执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowStatus {
    Pending,
    Running,
    Suspended,
    Completed,
    Failed,
}

/// 步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_index: usize,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// 工作流执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowResult {
    pub flow_id: String,
    pub status: FlowStatus,
    pub step_results: Vec<StepResult>,
    pub current_step: usize,
}
