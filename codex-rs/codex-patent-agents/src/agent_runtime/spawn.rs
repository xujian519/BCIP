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

use super::llm::call_llm_with_retry;
use super::prompt::build_system_prompt;

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
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_agent_job(&manifest_clone, prompt, provider)
            }));

            match result {
                Ok(Ok(())) => {}
                Ok(Err(ref error)) => {
                    learning::record_agent_feedback(&manifest_clone, 0, false, Some(error));
                    let _ = persist_agent_terminal_state(
                        &manifest_clone,
                        "failed",
                        None,
                        Some(error.clone()),
                    );
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
        })
        .map(|_| ())
        .map_err(|e| codex_patent_core::PatentError::Agent(e.to_string()))
}

fn run_agent_job(
    manifest: &AgentManifest,
    prompt: String,
    provider: AgentProvider,
) -> Result<(), String> {
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

    let system_prompt =
        build_system_prompt(&manifest.subagent_type, &manifest.model, &knowledge_prefix);

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

    let response = call_llm_with_retry(
        &provider,
        &manifest.model,
        &system_prompt,
        &prompt,
        &api_key,
    )?;

    append_agent_output(
        &manifest.output_file,
        &format!("\n## LLM Response\n\n{response}\n"),
    )?;

    learning::record_agent_feedback(manifest, 0, true, None);
    reflection::reflect_agent_result(manifest, &response);

    persist_agent_terminal_state(manifest, "completed", Some(&response), None)
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

// ---- default_*_path 辅助函数 ----

pub(crate) fn default_kg_path() -> String {
    std::env::var("BCIP_PATENT_KG_PATH")
        .unwrap_or_else(|_| "codex-patent-assets/patent_kg.db".to_string())
}

pub(crate) fn default_law_db_path() -> String {
    std::env::var("BCIP_LAW_DB_PATH").unwrap_or_else(|_| "codex-patent-assets/laws.db".to_string())
}

pub(crate) fn default_card_index_path() -> String {
    std::env::var("BCIP_CARD_INDEX_PATH")
        .unwrap_or_else(|_| "codex-patent-assets/card-index.json".to_string())
}

pub(crate) fn default_semantic_index_path() -> Option<String> {
    std::env::var("BCIP_SEMANTIC_INDEX_PATH").ok()
}
