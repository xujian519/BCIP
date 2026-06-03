//! 独立 Agent Runtime
//!
//! 提供不依赖 Codex core 的独立 agent 执行能力。

mod llm;
mod prompt;
mod spawn;

use crate::agent_manifest::AgentManifest;
use crate::agent_manifest::agent_store_dir;
use crate::agent_manifest::iso8601_now;
use crate::agent_manifest::list_agent_manifests;
use crate::agent_manifest::load_manifest;
use crate::agent_manifest::make_agent_id;
use crate::agent_manifest::persist_manifest;
use crate::provider_router::detect_provider;
use crate::roles::PatentAgentRole;
use codex_patent_core::PatentError;

/// Agent 生成输入参数
#[derive(Debug, Clone)]
pub struct AgentSpawnInput {
    /// 任务描述
    pub description: String,
    /// Agent 执行提示词
    pub prompt: String,
    /// 子代理类型（如 `analyzer`、`writer`）
    pub subagent_type: Option<String>,
    /// 自定义 agent 名称（可选）
    pub name: Option<String>,
    /// 指定模型（可选，默认使用 BCIP_DEFAULT_MODEL）
    pub model: Option<String>,
}

/// 独立 Agent 运行时
///
/// 提供不依赖 Codex core 的独立 agent 执行环境。
/// 支持线程调度、状态持久化和生命周期管理。
pub struct PatentAgentRuntime;

impl PatentAgentRuntime {
    /// 生成并启动一个 Agent
    ///
    /// 验证输入、创建 manifest、持久化初始状态后在新线程中启动执行。
    pub fn spawn_agent(input: AgentSpawnInput) -> Result<AgentManifest, PatentError> {
        if input.description.trim().is_empty() {
            return Err(PatentError::Validation(
                "description must not be empty".to_string(),
            ));
        }
        if input.prompt.trim().is_empty() {
            return Err(PatentError::Validation(
                "prompt must not be empty".to_string(),
            ));
        }

        let agent_id = make_agent_id();
        let output_dir = agent_store_dir()?;
        std::fs::create_dir_all(&output_dir)?;

        let output_file = output_dir.join(format!("{agent_id}.md"));
        let manifest_file = output_dir.join(format!("{agent_id}.json"));

        let normalized_subagent_type = normalize_subagent_type(input.subagent_type.as_deref());

        let model = resolve_model(input.model.as_deref(), Some(&normalized_subagent_type));

        let provider = detect_provider(&model);

        let agent_name = input
            .name
            .as_deref()
            .map(slugify_agent_name)
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| slugify_agent_name(&input.description));

        let created_at = iso8601_now();

        let output_contents = format!(
            "# Agent Task

- id: {}
- name: {}
- description: {}
- subagent_type: {}
- created_at: {}

## Prompt

{}
",
            agent_id,
            agent_name,
            input.description,
            normalized_subagent_type,
            created_at,
            input.prompt
        );

        std::fs::write(&output_file, output_contents)?;

        let manifest = AgentManifest {
            agent_id: agent_id.clone(),
            name: agent_name,
            subagent_type: normalized_subagent_type,
            model,
            status: "running".to_string(),
            output_file,
            manifest_file,
            created_at,
            completed_at: None,
            error: None,
        };

        persist_manifest(&manifest)?;

        spawn::spawn_agent_thread(&manifest, input.prompt.clone(), provider)?;

        Ok(manifest)
    }

    /// 查询 Agent 当前状态
    pub fn get_agent_status(agent_id: &str) -> Result<AgentManifest, PatentError> {
        load_manifest(agent_id)
    }

    /// 列出所有 Agent
    pub fn list_agents() -> Result<Vec<AgentManifest>, PatentError> {
        list_agent_manifests()
    }

    /// 取消正在运行的 Agent
    ///
    /// 已完成或已失败的 agent 不可取消。
    pub fn cancel_agent(agent_id: &str) -> Result<(), PatentError> {
        let mut manifest = load_manifest(agent_id)?;

        if manifest.status == "completed" || manifest.status == "failed" {
            return Err(PatentError::Agent(format!(
                "agent {agent_id} is already {}",
                manifest.status
            )));
        }

        manifest.status = "cancelled".to_string();
        manifest.completed_at = Some(iso8601_now());
        manifest.error = Some("cancelled by user".to_string());

        persist_manifest(&manifest)
    }
}

fn normalize_subagent_type(subagent_type: Option<&str>) -> String {
    let trimmed = subagent_type.map(str::trim).unwrap_or_default();

    if trimmed.is_empty() {
        return "general-purpose".to_string();
    }

    let lower = trimmed.to_ascii_lowercase();

    if PatentAgentRole::from_str(&lower).is_some() {
        return lower;
    }

    match lower.as_str() {
        "general" | "generalpurpose" | "generalpurposeagent" => "general-purpose".to_string(),
        "explore" | "explorer" | "exploreagent" => "Explore".to_string(),
        "plan" | "planagent" => "Plan".to_string(),
        "verification" | "verificationagent" | "verify" | "verifier" => "Verification".to_string(),
        _ => trimmed.to_string(),
    }
}

const DEFAULT_MODEL: &str = "deepseek-v4-pro";

fn resolve_model(model: Option<&str>, subagent_type: Option<&str>) -> String {
    if let Some(m) = model.map(str::trim).filter(|m| !m.is_empty()) {
        return m.to_string();
    }

    if let Some(_role) = subagent_type.and_then(PatentAgentRole::from_str) {
        return default_model();
    }

    default_model()
}

fn default_model() -> String {
    std::env::var("BCIP_DEFAULT_MODEL")
        .ok()
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}

fn slugify_agent_name(description: &str) -> String {
    let mut out = description
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    while out.contains("--") {
        out = out.replace("--", "-");
    }

    out.trim_matches('-').chars().take(32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_normalize_subagent_type() {
        assert_eq!(normalize_subagent_type(None), "general-purpose");
        assert_eq!(normalize_subagent_type(Some("")), "general-purpose");
        assert_eq!(normalize_subagent_type(Some("analyzer")), "analyzer");
        assert_eq!(normalize_subagent_type(Some("ANALYZER")), "analyzer");
        assert_eq!(normalize_subagent_type(Some("general")), "general-purpose");
        assert_eq!(normalize_subagent_type(Some("Explore")), "Explore");
    }

    #[test]
    fn test_resolve_model() {
        assert_eq!(resolve_model(Some("gpt-4o"), None), "gpt-4o");
        assert_eq!(resolve_model(None, Some("analyzer")), "deepseek-v4-pro");
        assert_eq!(resolve_model(None, None), "deepseek-v4-pro");
    }

    #[test]
    fn test_slugify_agent_name() {
        let name = slugify_agent_name("Test Agent Name 123");
        assert_eq!(name, "test-agent-name-123");
    }

    #[test]
    fn test_format_agent_terminal_output() {
        let output = spawn::format_agent_terminal_output("completed", Some("done"), None);
        assert!(output.contains("status: completed"));
        assert!(output.contains("Final response"));
        assert!(output.contains("done"));
    }

    fn setup_store() -> TempDir {
        let dir = TempDir::new().unwrap();
        crate::agent_manifest::set_test_store_dir(dir.path().to_path_buf());
        dir
    }

    #[test]
    fn test_spawn_agent() {
        let _temp_dir = setup_store();

        let input = AgentSpawnInput {
            description: "Test agent".to_string(),
            prompt: "Test prompt".to_string(),
            subagent_type: Some("analyzer".to_string()),
            name: Some("test-agent".to_string()),
            model: None,
        };

        let manifest = PatentAgentRuntime::spawn_agent(input).unwrap();

        assert_eq!(manifest.status, "running");
        assert_eq!(manifest.subagent_type, "analyzer");
        assert!(!manifest.agent_id.is_empty());
    }

    #[test]
    fn test_list_agents() {
        let _temp_dir = setup_store();

        let input1 = AgentSpawnInput {
            description: "Agent 1".to_string(),
            prompt: "Prompt 1".to_string(),
            subagent_type: Some("analyzer".to_string()),
            name: None,
            model: None,
        };

        std::thread::sleep(std::time::Duration::from_millis(10));

        let input2 = AgentSpawnInput {
            description: "Agent 2".to_string(),
            prompt: "Prompt 2".to_string(),
            subagent_type: Some("writer".to_string()),
            name: None,
            model: None,
        };

        PatentAgentRuntime::spawn_agent(input1).unwrap();
        PatentAgentRuntime::spawn_agent(input2).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(200));

        let agents = PatentAgentRuntime::list_agents().unwrap();
        assert!(
            agents.len() >= 2,
            "expected at least 2 agents, found {}: {:?}",
            agents.len(),
            agents.iter().map(|a| &a.agent_id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_cancel_agent() {
        let _temp_dir = setup_store();

        let input = AgentSpawnInput {
            description: "Test agent".to_string(),
            prompt: "Test prompt".to_string(),
            subagent_type: Some("analyzer".to_string()),
            name: None,
            model: None,
        };

        let manifest = PatentAgentRuntime::spawn_agent(input).unwrap();

        PatentAgentRuntime::cancel_agent(&manifest.agent_id).unwrap();

        let updated = PatentAgentRuntime::get_agent_status(&manifest.agent_id).unwrap();
        assert_eq!(updated.status, "cancelled");
        assert!(updated.error.is_some());
    }
}
