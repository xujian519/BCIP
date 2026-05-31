//! Agent 执行桥接 — 定义子 Agent 执行接口与多 Agent 委托。
//!
//! 参考 AutoGen 的 AgentTool 模式：
//! - `AgentExecutor::execute()` 直接调用 Agent
//! - `AgentExecutor::delegate_to()` 将 Agent 作为工具委托给另一个 Agent
//! - `MultiAgentExecutor` 管理多个 Agent 实例，按名称路由

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

/// Agent 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResult {
    pub agent_name: String,
    pub prompt: String,
    pub output: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Agent 执行器 trait — 负责创建并运行子 Agent。
///
/// 由 `tools` crate 实现，在工作流中注入真实的 Agent 执行能力。
pub trait AgentExecutor: Send {
    /// 执行一个子 Agent 任务。
    ///
    /// # Arguments
    /// * `agent_name` - Agent 名称（如 "patent-analysis-agent"）
    /// * `prompt` - 任务提示词
    ///
    /// # Returns
    /// Agent 执行结果，包含输出文本、成功/失败状态
    fn execute(&mut self, agent_name: &str, prompt: &str) -> Result<AgentExecutionResult, String>;

    /// 委托 Agent 作为工具执行（Agent-as-Tool 模式）。
    ///
    /// 当一个 Agent 调用另一个 Agent 作为工具时使用，参考 AutoGen AgentTool。
    fn delegate_to(
        &mut self,
        agent_name: &str,
        input: &str,
    ) -> Result<AgentExecutionResult, String> {
        let prompt = format!("[委托任务] 来自上级 Agent 的委托: {input}");
        self.execute(agent_name, &prompt)
    }

    /// Agent 执行器名称（用于日志）
    fn name(&self) -> &str;

    /// 技能 Agent 名称列表（用于 AgentTool 发现）
    fn agent_names(&self) -> &[String] {
        &[]
    }
}

/// Agent 执行回退函数类型
pub type AgentFallbackFn = Box<dyn Fn(&str, &str) -> Result<AgentExecutionResult, String> + Send>;

/// 多 Agent 执行器 — 按名称路由到不同的 Agent 实例。
///
/// 每个 Agent 通过名称注册，调用时按名称分发。
/// 支持主 Agent 将子任务委托给专业 Agent 的模式。
pub struct MultiAgentExecutor {
    label: String,
    agents: HashMap<String, Box<dyn AgentExecutor>>,
    default_fallback: Option<AgentFallbackFn>,
}

impl MultiAgentExecutor {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            agents: HashMap::new(),
            default_fallback: None,
        }
    }

    /// 注册一个 Agent 执行器。
    pub fn register(&mut self, name: impl Into<String>, executor: Box<dyn AgentExecutor>) {
        self.agents.insert(name.into(), executor);
    }

    /// 设置当请求的 Agent 未被注册时的回退策略。
    pub fn with_default_fallback(mut self, fallback: AgentFallbackFn) -> Self {
        self.default_fallback = Some(fallback);
        self
    }

    /// 获取已注册的 Agent 名称列表。
    pub fn registered_agent_names(&self) -> Vec<&str> {
        self.agents.keys().map(|k| k.as_str()).collect()
    }
}

impl AgentExecutor for MultiAgentExecutor {
    fn execute(&mut self, agent_name: &str, prompt: &str) -> Result<AgentExecutionResult, String> {
        if let Some(agent) = self.agents.get_mut(agent_name) {
            agent.execute(agent_name, prompt)
        } else if let Some(ref fallback) = self.default_fallback {
            fallback(agent_name, prompt)
        } else {
            Err(format!(
                "Agent '{}' 未在 MultiAgentExecutor '{}' 中注册。可用: {:?}",
                agent_name,
                self.label,
                self.registered_agent_names()
            ))
        }
    }

    fn delegate_to(
        &mut self,
        agent_name: &str,
        input: &str,
    ) -> Result<AgentExecutionResult, String> {
        if let Some(agent) = self.agents.get_mut(agent_name) {
            agent.delegate_to(agent_name, input)
        } else if let Some(ref fallback) = self.default_fallback {
            let prompt = format!("[委托任务] {input}");
            fallback(agent_name, &prompt)
        } else {
            Err(format!("Agent '{}' 未注册，无法委托", agent_name))
        }
    }

    fn name(&self) -> &str {
        &self.label
    }

    fn agent_names(&self) -> &[String] {
        &[]
    }
}

/// 无操作 Agent 执行器（用于测试，返回固定内容）
pub struct NoopAgentExecutor {
    pub label: String,
}

impl AgentExecutor for NoopAgentExecutor {
    fn execute(&mut self, agent_name: &str, prompt: &str) -> Result<AgentExecutionResult, String> {
        Ok(AgentExecutionResult {
            agent_name: agent_name.to_string(),
            prompt: prompt.to_string(),
            output: format!(
                "[NOOP] Agent「{agent_name}」已执行，prompt长度={}",
                prompt.len()
            ),
            success: true,
            error: None,
        })
    }

    fn name(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_agent_executor() {
        let mut executor = NoopAgentExecutor {
            label: "test".into(),
        };
        let result = executor
            .execute("patent-analysis-agent", "分析专利新颖性")
            .unwrap();
        assert!(result.success);
        assert_eq!(result.agent_name, "patent-analysis-agent");
        assert!(result.output.contains("NOOP"));
    }

    #[test]
    fn test_multi_agent_routing() {
        let mut multi = MultiAgentExecutor::new("test-router");

        let patent_agent = NoopAgentExecutor {
            label: "patent".into(),
        };
        let legal_agent = NoopAgentExecutor {
            label: "legal".into(),
        };

        multi.register("patent-expert", Box::new(patent_agent));
        multi.register("legal-expert", Box::new(legal_agent));

        let result = multi.execute("patent-expert", "分析创造性").unwrap();
        assert!(result.success);
        assert_eq!(result.agent_name, "patent-expert");

        let result = multi.execute("legal-expert", "解释专利法").unwrap();
        assert!(result.success);
        assert_eq!(result.agent_name, "legal-expert");
    }

    #[test]
    fn test_multi_agent_delegation() {
        let mut multi = MultiAgentExecutor::new("test-delegator");

        let specialist = NoopAgentExecutor {
            label: "specialist".into(),
        };
        multi.register("specialist", Box::new(specialist));

        let result = multi
            .delegate_to("specialist", "{\"task\": \"子任务分析\"}")
            .unwrap();
        assert!(result.success);
        assert_eq!(result.agent_name, "specialist");
    }

    #[test]
    fn test_unregistered_agent_error() {
        let mut multi = MultiAgentExecutor::new("test-errors");
        let result = multi.execute("nonexistent", "测试");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("未在"));
    }

    #[test]
    fn test_fallback_executor() {
        let mut multi = MultiAgentExecutor::new("test-fallback").with_default_fallback(Box::new(
            |name, prompt| {
                Ok(AgentExecutionResult {
                    agent_name: name.to_string(),
                    prompt: prompt.to_string(),
                    output: format!("[fallback] {name}: {prompt}"),
                    success: true,
                    error: None,
                })
            },
        ));

        let result = multi.execute("any-agent", "任意任务").unwrap();
        assert!(result.success);
        assert!(result.output.contains("[fallback]"));
    }
}
