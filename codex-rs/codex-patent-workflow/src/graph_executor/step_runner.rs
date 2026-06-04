use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use crate::agent_bridge::AgentExecutor;
use crate::flow::FlowStep;
use crate::flow::StepResult;

use super::CodeExecutor;
use super::GraphExecutor;
use super::ToolExecutorFn;

impl GraphExecutor {
    pub(super) fn execute_step(
        &self,
        step: &FlowStep,
        max_retries: u32,
    ) -> Result<StepResult, String> {
        execute_step_from_parts(
            step,
            &self.tool_executor,
            &self.agent_executor,
            &self.code_executor,
            max_retries,
        )
    }
}

pub(super) fn execute_step_from_parts(
    step: &FlowStep,
    tool_executor: &Option<Arc<ToolExecutorFn>>,
    agent_executor: &Option<Arc<Mutex<Box<dyn AgentExecutor>>>>,
    code_executor: &Option<Arc<Mutex<Box<dyn CodeExecutor>>>>,
    max_retries: u32,
) -> Result<StepResult, String> {
    match step {
        FlowStep::AgentCall { agent_name, prompt } => {
            if let Some(ref agent_exec) = agent_executor {
                match agent_exec
                    .lock()
                    .map_err(|e| format!("Agent executor lock error: {e}"))?
                    .execute(agent_name, prompt)
                {
                    Ok(result) => Ok(StepResult {
                        step_index: 0,
                        success: result.success,
                        output: Some(serde_json::json!({
                            "agent": result.agent_name,
                            "prompt": result.prompt,
                            "output": result.output,
                            "success": result.success,
                            "error": result.error,
                        })),
                        error: result.error,
                    }),
                    Err(e) => Ok(StepResult {
                        step_index: 0,
                        success: false,
                        output: None,
                        error: Some(format!("Agent 执行失败: {e}")),
                    }),
                }
            } else {
                Ok(StepResult {
                    step_index: 0,
                    success: false,
                    output: None,
                    error: Some(format!(
                        "未注册 Agent 执行器，无法执行 agent '{}'",
                        agent_name
                    )),
                })
            }
        }
        FlowStep::AgentTool { agent_name, input } => {
            if let Some(ref agent_exec) = agent_executor {
                let prompt = serde_json::to_string(input).unwrap_or_default();
                match agent_exec
                    .lock()
                    .map_err(|e| format!("Agent executor lock error: {e}"))?
                    .delegate_to(agent_name, &prompt)
                {
                    Ok(result) => Ok(StepResult {
                        step_index: 0,
                        success: result.success,
                        output: Some(serde_json::json!({
                            "agent": result.agent_name,
                            "output": result.output,
                        })),
                        error: result.error,
                    }),
                    Err(e) => Ok(StepResult {
                        step_index: 0,
                        success: false,
                        output: None,
                        error: Some(format!("AgentTool 委托失败: {e}")),
                    }),
                }
            } else {
                Ok(StepResult {
                    step_index: 0,
                    success: false,
                    output: None,
                    error: Some("未注册 Agent 执行器，无法委托 AgentTool".into()),
                })
            }
        }
        FlowStep::QualityCheck { criteria } => Ok(StepResult {
            step_index: 0,
            success: true,
            output: Some(serde_json::json!({
                "criteria": criteria,
                "passed": true
            })),
            error: None,
        }),
        FlowStep::HumanApproval {
            title,
            description,
            timeout_secs,
            timeout_action,
        } => Ok(StepResult {
            step_index: 0,
            success: true,
            output: Some(serde_json::json!({
                "type": "human_approval_required",
                "title": title,
                "description": description,
                "suspended": true,
                "timeout_secs": timeout_secs,
                "timeout_action": timeout_action,
            })),
            error: None,
        }),
        FlowStep::ToolCall { tool_name, input } => {
            if let Some(ref executor) = tool_executor {
                let mut last_error = String::new();

                for attempt in 0..=max_retries {
                    if attempt > 0 {
                        let delay_ms = 500u64 * 2u64.pow(attempt - 1);
                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    }

                    let start = Instant::now();
                    match executor(tool_name, input) {
                        Ok(output) => {
                            let elapsed_ms = start.elapsed().as_millis();
                            tracing::info!(
                                tool = %tool_name,
                                elapsed_ms = %elapsed_ms,
                                output_len = output.len(),
                                attempt = attempt + 1,
                                "工具调用成功"
                            );
                            return Ok(StepResult {
                                step_index: 0,
                                success: true,
                                output: Some(serde_json::json!({ "output": output })),
                                error: None,
                            });
                        }
                        Err(e) => {
                            let elapsed_ms = start.elapsed().as_millis();
                            last_error = e.clone();

                            if matches!(classify_tool_error(&e), ErrorKind::Fatal) {
                                tracing::error!(
                                    tool = %tool_name,
                                    elapsed_ms = %elapsed_ms,
                                    error = %e,
                                    "工具调用失败(致命错误，不重试)"
                                );
                                break;
                            }

                            if attempt < max_retries {
                                tracing::warn!(
                                    tool = %tool_name,
                                    attempt = attempt + 1,
                                    elapsed_ms = %elapsed_ms,
                                    error = %e,
                                    "工具调用失败，将重试"
                                );
                            } else {
                                tracing::error!(
                                    tool = %tool_name,
                                    attempt = attempt + 1,
                                    elapsed_ms = %elapsed_ms,
                                    error = %e,
                                    "工具调用失败(已达最大重试次数)"
                                );
                            }
                        }
                    }
                }

                Ok(StepResult {
                    step_index: 0,
                    success: false,
                    output: None,
                    error: Some(last_error),
                })
            } else {
                Ok(StepResult {
                    step_index: 0,
                    success: false,
                    output: None,
                    error: Some(format!("未注册 Tool 执行器: {}", tool_name)),
                })
            }
        }
        FlowStep::CodeBlock { language, code } => {
            if let Some(ref exec) = code_executor {
                match exec
                    .lock()
                    .map_err(|e| format!("Code executor lock error: {e}"))?
                    .execute(language, code)
                {
                    Ok(result) => Ok(StepResult {
                        step_index: 0,
                        success: result.success,
                        output: Some(serde_json::json!({
                            "output": result.output,
                            "language": result.language,
                        })),
                        error: result.error,
                    }),
                    Err(e) => Ok(StepResult {
                        step_index: 0,
                        success: false,
                        output: None,
                        error: Some(format!("代码执行失败: {e}")),
                    }),
                }
            } else {
                Ok(StepResult {
                    step_index: 0,
                    success: false,
                    output: None,
                    error: Some("未注册代码执行器".into()),
                })
            }
        }
    }
}

pub(super) fn node_matches_step(node_step: &FlowStep, target: &FlowStep) -> bool {
    matches!(
        (node_step, target),
        (
            FlowStep::HumanApproval { .. },
            FlowStep::HumanApproval { .. }
        )
    )
}

pub(super) enum ErrorKind {
    Retryable,
    Fatal,
}

pub(super) fn classify_tool_error(msg: &str) -> ErrorKind {
    let lower = msg.to_lowercase();
    if lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("connection")
        || lower.contains("network")
        || lower.contains("temporary")
        || lower.contains("rate limit")
        || lower.contains("429")
        || lower.contains("503")
        || lower.contains("502")
        || lower.contains("gateway")
        || lower.contains("unavailable")
        || lower.contains("eof")
        || lower.contains("reset")
        || lower.contains("refused")
        || lower.contains("broken pipe")
        || lower.contains("io error")
        || lower.contains("interrupted")
    {
        ErrorKind::Retryable
    } else {
        ErrorKind::Fatal
    }
}
