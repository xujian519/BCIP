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
use crate::knowledge_context::AutoKnowledgeConfig;
use crate::knowledge_context::KnowledgeContext;
use crate::learning;
use crate::provider_router::AgentProvider;
use crate::provider_router::detect_provider;
use crate::provider_router::mask_api_key;
use crate::provider_router::resolve_base_url;
use crate::provider_router::resolve_provider_api_key;
use crate::reflection;
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
        .map_err(|error| error.to_string())
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

fn build_system_prompt(subagent_type: &str, model: &str, knowledge_prefix: &str) -> String {
    let role = PatentAgentRole::from_str(subagent_type);
    let role_name = role.map(|r| r.name()).unwrap_or("通用助手");

    let mut prompt = format!(
        "你是 BCIP 专利智能体系统的 {role_name}。\
         请基于用户提供的任务要求，给出专业、准确、完整的分析和建议。\n\n\
         ## 行为准则\n\
         - 基于事实和法律条文进行分析，不做无根据的推测\n\
         - 输出结构清晰，使用 Markdown 格式\n\
         - 如遇不确定内容，明确标注并给出建议\n"
    );

    if !knowledge_prefix.is_empty() {
        prompt.push_str("\n## 知识上下文\n");
        prompt.push_str(knowledge_prefix);
        prompt.push('\n');
    }

    if let Some(r) = role {
        let domains: Vec<&str> = match r {
            PatentAgentRole::Retriever => vec!["专利检索", "Web搜索"],
            PatentAgentRole::Analyzer => vec!["权利要求分析", "法律分析"],
            PatentAgentRole::Writer => vec!["专利撰写", "文档处理"],
            PatentAgentRole::NoveltyChecker => vec!["新颖性分析", "专利检索"],
            PatentAgentRole::CreativityChecker => vec!["创造性分析"],
            PatentAgentRole::InfringementChecker => vec!["侵权分析", "法律分析"],
            PatentAgentRole::InvalidityChecker => vec!["无效分析", "法律分析", "专利检索"],
            PatentAgentRole::Reviewer => vec!["文件审查", "质量检查"],
            PatentAgentRole::QualityChecker => vec!["质量评估", "文件审查"],
        };
        prompt.push_str(&format!("\n## 专业领域\n{}\n", domains.join("、")));
    }

    prompt.push_str(&format!("\n## 当前模型\n{model}\n"));
    prompt
}

const MAX_RETRIES: u32 = 3;
const REQUEST_TIMEOUT_SECS: u64 = 120;

fn call_llm_with_retry(
    provider: &AgentProvider,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    api_key: &str,
) -> Result<String, String> {
    let mut last_error = String::new();

    for attempt in 0..=MAX_RETRIES {
        match call_llm_once(provider, model, system_prompt, user_prompt, api_key) {
            Ok(response) => return Ok(response),
            Err(e) => {
                let is_auth_error = e.contains("401") || e.contains("403");
                if is_auth_error {
                    return Err(format!(
                        "Authentication failed (key={}): {e}",
                        mask_api_key(api_key)
                    ));
                }

                last_error = e;
                if attempt < MAX_RETRIES {
                    let delay_ms = 1000u64 * 2u64.pow(attempt);
                    eprintln!(
                        "[bcip-agent] LLM call attempt {}/{} failed, retrying in {delay_ms}ms: {last_error}",
                        attempt + 1,
                        MAX_RETRIES + 1
                    );
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                }
            }
        }
    }

    Err(format!(
        "LLM call failed after {} retries: {last_error}",
        MAX_RETRIES
    ))
}

fn call_llm_once(
    provider: &AgentProvider,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    api_key: &str,
) -> Result<String, String> {
    match provider {
        AgentProvider::Anthropic { .. } => {
            call_anthropic(model, system_prompt, user_prompt, api_key)
        }
        AgentProvider::OpenAiCompatible { base_url, .. } => {
            call_openai_compatible(base_url, model, system_prompt, user_prompt, api_key)
        }
    }
}

fn call_openai_compatible(
    base_url: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    api_key: &str,
) -> Result<String, String> {
    let url = format!("{base_url}/chat/completions");

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": 0.7,
        "max_tokens": 8192
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .no_proxy()
        .build()
        .map_err(|e| format!("build HTTP client: {e}"))?;

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("request to {url}: {e}"))?;

    let status = resp.status();
    let text = resp.text().map_err(|e| format!("read response: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "HTTP {} from {url}: {}",
            status,
            truncate_error_body(&text, 500)
        ));
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("parse JSON: {e}"))?;

    json.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            format!(
                "unexpected response structure: {}",
                truncate_error_body(&text, 300)
            )
        })
}

fn call_anthropic(
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    api_key: &str,
) -> Result<String, String> {
    let base = resolve_base_url(model);
    let url = format!("{}/v1/messages", base.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "system": system_prompt,
        "messages": [
            {"role": "user", "content": user_prompt}
        ],
        "max_tokens": 8192
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .no_proxy()
        .build()
        .map_err(|e| format!("build HTTP client: {e}"))?;

    let resp = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("request to {url}: {e}"))?;

    let status = resp.status();
    let text = resp.text().map_err(|e| format!("read response: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "HTTP {} from Anthropic: {}",
            status,
            truncate_error_body(&text, 500)
        ));
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("parse JSON: {e}"))?;

    json.get("content")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            format!(
                "unexpected Anthropic response: {}",
                truncate_error_body(&text, 300)
            )
        })
}

fn truncate_error_body(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!(
            "{}...(truncated, total {} bytes)",
            &text[..max_len],
            text.len()
        )
    }
}

fn default_kg_path() -> String {
    std::env::var("BCIP_PATENT_KG_PATH")
        .unwrap_or_else(|_| "codex-patent-assets/patent_kg.db".to_string())
}

fn default_law_db_path() -> String {
    std::env::var("BCIP_LAW_DB_PATH").unwrap_or_else(|_| "codex-patent-assets/laws.db".to_string())
}

fn default_card_index_path() -> String {
    std::env::var("BCIP_CARD_INDEX_PATH")
        .unwrap_or_else(|_| "codex-patent-assets/card-index.json".to_string())
}

fn default_semantic_index_path() -> Option<String> {
    std::env::var("BCIP_SEMANTIC_INDEX_PATH").ok()
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
