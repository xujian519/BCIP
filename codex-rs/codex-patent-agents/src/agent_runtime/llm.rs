//! LLM 调用：重试逻辑、OpenAI 兼容接口、Anthropic 接口

use crate::provider_router::AgentProvider;
use crate::provider_router::mask_api_key;
use crate::provider_router::resolve_base_url;

pub(crate) const MAX_RETRIES: u32 = 3;
pub(crate) const REQUEST_TIMEOUT_SECS: u64 = 120;
#[allow(dead_code)]
pub(crate) const DEFAULT_TEMPERATURE: f32 = 0.7;

pub(crate) fn call_llm_with_retry_and_temperature(
    provider: &AgentProvider,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    api_key: &str,
    temperature: f32,
) -> Result<String, String> {
    let mut last_error = String::new();

    for attempt in 0..=MAX_RETRIES {
        match call_llm_once(
            provider,
            model,
            system_prompt,
            user_prompt,
            api_key,
            temperature,
        ) {
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
    temperature: f32,
) -> Result<String, String> {
    match provider {
        AgentProvider::Anthropic { .. } => {
            call_anthropic(model, system_prompt, user_prompt, api_key, temperature)
        }
        AgentProvider::OpenAiCompatible { base_url, .. } => call_openai_compatible(
            base_url,
            model,
            system_prompt,
            user_prompt,
            api_key,
            temperature,
        ),
    }
}

fn call_openai_compatible(
    base_url: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    api_key: &str,
    temperature: f32,
) -> Result<String, String> {
    let url = format!("{base_url}/chat/completions");

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": temperature,
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

    log_token_usage(&json, model);

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
    temperature: f32,
) -> Result<String, String> {
    let base = resolve_base_url(model);
    let url = format!("{}/v1/messages", base.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "system": system_prompt,
        "messages": [
            {"role": "user", "content": user_prompt}
        ],
        "max_tokens": 8192,
        "temperature": temperature
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

    log_token_usage(&json, model);

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

fn log_token_usage(json: &serde_json::Value, model: &str) {
    if let Some(usage) = json.get("usage") {
        let prompt_tokens = usage
            .get("input_tokens")
            .or_else(|| usage.get("prompt_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let completion_tokens = usage
            .get("output_tokens")
            .or_else(|| usage.get("completion_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total_tokens = usage
            .get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(prompt_tokens + completion_tokens);
        tracing::info!(
            model = %model,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            "LLM token usage"
        );
    }
}

pub(crate) fn truncate_error_body(text: &str, max_len: usize) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_error_body_short() {
        assert_eq!(truncate_error_body("hello", 10), "hello");
    }

    #[test]
    fn truncate_error_body_exact() {
        assert_eq!(truncate_error_body("hello", 5), "hello");
    }

    #[test]
    fn truncate_error_body_long() {
        let text = "hello world this is long";
        let result = truncate_error_body(text, 5);
        assert!(result.starts_with("hello..."));
        assert!(result.contains(&format!("{} bytes", text.len())));
    }

    #[test]
    fn truncate_error_body_empty() {
        assert_eq!(truncate_error_body("", 5), "");
    }

    #[test]
    fn constants_are_reasonable() {
        assert_eq!(MAX_RETRIES, 3);
        assert_eq!(REQUEST_TIMEOUT_SECS, 120);
    }
}
