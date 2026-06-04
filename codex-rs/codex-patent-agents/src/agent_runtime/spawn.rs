//! Agent 线程调度、运行、持久化辅助

use crate::agent_manifest::AgentManifest;
use crate::agent_manifest::iso8601_now;
use crate::agent_manifest::persist_manifest;
use crate::knowledge_context::AutoKnowledgeConfig;
use crate::knowledge_context::KnowledgeContext;
use crate::learning;
use crate::provider_router::AgentProvider;
use crate::provider_router::resolve_provider_api_key;
use crate::reflection;
use crate::roles::AgentRoleConfig;
use crate::roles::PatentAgentRole;
use crate::roles::find_skills_shared_dir;
use codex_patent_knowledge::paths;

use super::llm::call_llm_with_retry_and_temperature;
use super::prompt::build_system_prompt;

const MAX_AGENT_RESTARTS: u32 = 3;
const RESTART_BASE_DELAY_MS: u64 = 2000;

pub(crate) fn spawn_agent_thread(
    manifest: &AgentManifest,
    prompt: String,
    provider: crate::provider_router::AgentProvider,
) -> Result<(), codex_patent_core::PatentError> {
    let thread_name = format!("bcip-agent-{}", manifest.agent_id);
    let manifest_clone = manifest.clone();

    std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            let mut attempt: u32 = 0;

            loop {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run_agent_job(&manifest_clone, prompt.clone(), provider.clone())
                }));

                match result {
                    Ok(Ok(())) => return,
                    Ok(Err(ref error)) => {
                        learning::record_agent_feedback(&manifest_clone, 0, false, Some(error));
                        let _ = persist_agent_terminal_state(
                            &manifest_clone,
                            "failed",
                            None,
                            Some(error.clone()),
                        );
                        if !should_restart(error) { return; }
                    }
                    Err(_) => {
                        learning::record_agent_feedback(
                            &manifest_clone,
                            0,
                            false,
                            Some("agent thread panicked"),
                        );
                        let _ = persist_agent_terminal_state(
                            &manifest_clone,
                            "failed",
                            None,
                            Some("agent thread panicked".to_string()),
                        );
                    }
                }

                attempt += 1;
                if attempt >= MAX_AGENT_RESTARTS {
                    tracing::warn!(
                        "[bcip-agent] agent {} exceeded max restarts ({MAX_AGENT_RESTARTS})",
                        manifest_clone.agent_id
                    );
                    return;
                }

                let delay_ms = RESTART_BASE_DELAY_MS * 2u64.pow(attempt - 1);
                tracing::info!(
                    "[bcip-agent] restarting agent {} (attempt {attempt}/{MAX_AGENT_RESTARTS}, delay {delay_ms}ms)",
                    manifest_clone.agent_id
                );
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            }
        })
        .map(|_| ())
        .map_err(|e| codex_patent_core::PatentError::Agent(e.to_string()))
}

fn should_restart(error: &str) -> bool {
    let fatal_patterns = [
        "API key resolution failed",
        "authentication",
        "unauthorized",
        "forbidden",
    ];
    let error_lower = error.to_lowercase();
    !fatal_patterns.iter().any(|p| error_lower.contains(p))
}

fn run_agent_job(
    manifest: &AgentManifest,
    prompt: String,
    provider: AgentProvider,
) -> Result<(), String> {
    let role = PatentAgentRole::from_str(&manifest.subagent_type);

    let (system_prompt, knowledge_enabled) = match role {
        Some(r) => {
            let config = load_bcip_role_config(r.role_id());
            let knowledge_config = config.auto_knowledge.clone().unwrap_or_default();
            let shared_dir = find_skills_shared_dir();

            let knowledge = KnowledgeContext::new(
                &default_kg_path(),
                &default_law_db_path(),
                &default_card_index_path(),
                default_semantic_index_path().as_deref(),
                knowledge_config.clone(),
            );

            let sp = r.system_prompt_with_context(
                &config,
                &prompt,
                if knowledge.is_enabled() {
                    Some(&knowledge)
                } else {
                    None
                },
                shared_dir.as_deref(),
            );
            (sp, knowledge.is_enabled())
        }
        None => {
            let knowledge = KnowledgeContext::new(
                &default_kg_path(),
                &default_law_db_path(),
                &default_card_index_path(),
                default_semantic_index_path().as_deref(),
                AutoKnowledgeConfig::default(),
            );
            let knowledge_prefix = if knowledge.is_enabled() {
                knowledge.resolve(&manifest.subagent_type, &prompt)
            } else {
                String::new()
            };
            let sp =
                build_system_prompt(&manifest.subagent_type, &manifest.model, &knowledge_prefix);
            (sp, knowledge.is_enabled())
        }
    };

    let _ = knowledge_enabled;

    let api_key_env = match &provider {
        AgentProvider::Anthropic { api_key_env } => api_key_env,
        AgentProvider::OpenAiCompatible { api_key_env, .. } => api_key_env,
    };

    let api_key = resolve_provider_api_key(api_key_env).map_err(|e| {
        format!(
            "API key resolution failed for {}: {e} (env_var={api_key_env})",
            manifest.model
        )
    })?;

    let temperature = role.map(|r| r.temperature()).unwrap_or(0.7);

    let response = call_llm_with_retry_and_temperature(
        &provider,
        &manifest.model,
        &system_prompt,
        &prompt,
        &api_key,
        temperature,
    )?;

    append_agent_output(
        &manifest.output_file,
        &format!("\n## LLM Response\n\n{response}\n"),
    )?;

    learning::record_agent_feedback(manifest, 0, true, None);
    reflection::reflect_agent_result(manifest, &response);

    persist_agent_terminal_state(manifest, "completed", Some(&response), None)
}

fn load_bcip_role_config(role_id: &str) -> AgentRoleConfig {
    let path_str = format!("patent/{role_id}.toml");
    let path = std::path::Path::new(&path_str);
    if let Some(content) = crate::bcip_roles::config_file_contents(path) {
        match toml::from_str::<AgentRoleConfig>(content) {
            Ok(config) => return config,
            Err(e) => {
                tracing::warn!(
                    "[bcip-agent] WARN: 解析角色配置 patent/{role_id}.toml 失败: {e}，使用回退配置"
                );
            }
        }
    } else {
        tracing::warn!("[bcip-agent] WARN: 角色 patent/{role_id}.toml 未找到，使用回退配置");
    }
    AgentRoleConfig {
        role_id: role_id.to_string(),
        name: role_id.to_string(),
        identity: format!("你是 BCIP 专利智能体 {role_id}。"),
        description: None,
        developer_instructions: None,
        methodology: vec![],
        output_format: String::new(),
        primary_tools: vec![],
        secondary_tools: vec![],
        constraints: vec![],
        auto_knowledge: None,
        includes: vec![],
    }
}

pub(crate) fn persist_agent_terminal_state(
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

    persist_manifest(&next_manifest).map_err(|e| e.to_string())
}

pub(crate) fn append_agent_output(path: &std::path::Path, suffix: &str) -> Result<(), String> {
    use std::io::Write as _;

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    file.write_all(suffix.as_bytes()).map_err(|e| e.to_string())
}

pub(crate) fn format_agent_terminal_output(
    status: &str,
    result: Option<&str>,
    error: Option<&str>,
) -> String {
    let mut sections = vec![format!("\n## Result\n\n- status: {status}\n")];

    if let Some(result) = result.filter(|value| !value.trim().is_empty()) {
        sections.push(format!("\n### Final response\n\n{}\n", result.trim()));
    }

    if let Some(error) = error.filter(|value| !value.trim().is_empty()) {
        sections.push(format!("\n### Error\n\n{}\n", error.trim()));
    }

    sections.join("")
}

// ---- 路径解析委托到 codex-patent-knowledge 的 paths 模块 ----
// 使用统一的路径解析（多策略回退），消除两套独立路径函数。

pub(crate) fn default_kg_path() -> String {
    paths::kg_db_path()
}

pub(crate) fn default_law_db_path() -> String {
    paths::law_db_path()
}

pub(crate) fn default_card_index_path() -> String {
    paths::card_index_path()
}

pub(crate) fn default_semantic_index_path() -> Option<String> {
    Some(paths::semantic_index_path())
}
