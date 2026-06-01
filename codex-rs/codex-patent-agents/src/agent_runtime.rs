//! 独立 Agent Runtime
//!
//! 提供不依赖 Codex core 的独立 agent 执行能力。

use crate::agent_manifest::AgentManifest;
use crate::agent_manifest::agent_store_dir;
use crate::agent_manifest::iso8601_now;
use crate::agent_manifest::list_agent_manifests;
use crate::agent_manifest::load_manifest;
use crate::agent_manifest::make_agent_id;
use crate::agent_manifest::persist_manifest;
use crate::provider_router::detect_provider;
use crate::roles::PatentAgentRole;

#[derive(Debug, Clone)]
pub struct AgentSpawnInput {
    pub description: String,
    pub prompt: String,
    pub subagent_type: Option<String>,
    pub name: Option<String>,
    pub model: Option<String>,
}

pub struct PatentAgentRuntime;

impl PatentAgentRuntime {
    pub fn spawn_agent(input: AgentSpawnInput) -> Result<AgentManifest, String> {
        if input.description.trim().is_empty() {
            return Err("description must not be empty".to_string());
        }
        if input.prompt.trim().is_empty() {
            return Err("prompt must not be empty".to_string());
        }

        let agent_id = make_agent_id();
        let output_dir = agent_store_dir()?;
        std::fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;

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

        std::fs::write(&output_file, output_contents).map_err(|error| error.to_string())?;

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

        spawn_agent_thread(&manifest, input.prompt.clone(), provider)?;

        Ok(manifest)
    }

    pub fn get_agent_status(agent_id: &str) -> Result<AgentManifest, String> {
        load_manifest(agent_id)
    }

    pub fn list_agents() -> Result<Vec<AgentManifest>, String> {
        list_agent_manifests()
    }

    pub fn cancel_agent(agent_id: &str) -> Result<(), String> {
        let mut manifest = load_manifest(agent_id)?;

        if manifest.status == "completed" || manifest.status == "failed" {
            return Err(format!("agent {agent_id} is already {}", manifest.status));
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

fn spawn_agent_thread(
    manifest: &AgentManifest,
    prompt: String,
    provider: crate::provider_router::AgentProvider,
) -> Result<(), String> {
    let thread_name = format!("bcip-agent-{}", manifest.agent_id);
    let manifest_clone = manifest.clone();

    std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_agent_job(&manifest_clone, prompt, provider)
            }));

            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    let _ =
                        persist_agent_terminal_state(&manifest_clone, "failed", None, Some(error));
                }
                Err(_) => {
                    let _ = persist_agent_terminal_state(
                        &manifest_clone,
                        "failed",
                        None,
                        Some("agent thread panicked".to_string()),
                    );
                }
            }
        })
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn run_agent_job(
    manifest: &AgentManifest,
    _prompt: String,
    _provider: crate::provider_router::AgentProvider,
) -> Result<(), String> {
    // TODO: 实现真正的 LLM streaming — 当前为 stub，仅模拟完成状态
    let _client = reqwest::Client::new();

    std::thread::sleep(std::time::Duration::from_millis(100));

    persist_agent_terminal_state(
        manifest,
        "completed",
        Some("Agent stub completed — LLM streaming not yet implemented"),
        None,
    )
}

fn persist_agent_terminal_state(
    manifest: &AgentManifest,
    status: &str,
    result: Option<&str>,
    error: Option<String>,
) -> Result<(), String> {
    append_agent_output(
        &manifest.output_file,
        &format_agent_terminal_output(status, result, error.as_deref()),
    )?;

    let mut next_manifest = manifest.clone();
    next_manifest.status = status.to_string();
    next_manifest.completed_at = Some(iso8601_now());
    next_manifest.error = error;

    persist_manifest(&next_manifest)
}

fn append_agent_output(path: &std::path::Path, suffix: &str) -> Result<(), String> {
    use std::io::Write as _;

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;

    file.write_all(suffix.as_bytes())
        .map_err(|error| error.to_string())
}

fn format_agent_terminal_output(status: &str, result: Option<&str>, error: Option<&str>) -> String {
    let mut sections = vec![format!("\n## Result\n\n- status: {status}\n")];

    if let Some(result) = result.filter(|value| !value.trim().is_empty()) {
        sections.push(format!("\n### Final response\n\n{}\n", result.trim()));
    }

    if let Some(error) = error.filter(|value| !value.trim().is_empty()) {
        sections.push(format!("\n### Error\n\n{}\n", error.trim()));
    }

    sections.join("")
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
        let output = format_agent_terminal_output("completed", Some("done"), None);
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

        std::thread::sleep(std::time::Duration::from_millis(200));

        let updated = PatentAgentRuntime::get_agent_status(&manifest.agent_id).unwrap();
        assert_eq!(updated.status, "completed");
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

        let agents = PatentAgentRuntime::list_agents().unwrap();
        assert!(agents.len() >= 2);
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
