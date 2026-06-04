//! LLM 调用：重试逻辑、OpenAI 兼容接口、Anthropic 接口

use crate::provider_router::AgentProvider;
use crate::provider_router::mask_api_key;
use crate::provider_router::resolve_base_url;

pub(crate) const MAX_RETRIES: u32 = 3;
pub(crate) const REQUEST_TIMEOUT_SECS: u64 = 120;
#[allow(dead_code)]
pub(crate) const DEFAULT_TEMPERATURE: f32 = 0.7;

const CB_FAILURE_THRESHOLD: u32 = 5;
const CB_RESET_TIMEOUT_SECS: u64 = 60;
const CB_HALF_OPEN_MAX: u32 = 3;
const BACKOFF_BASE_MS: u64 = 1000;
const BACKOFF_MAX_MS: u64 = 30_000;

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CbState {
    Closed,
    Open,
    HalfOpen,
}

struct InlineCircuitBreaker {
    state: AtomicU32,
    consecutive_failures: AtomicU32,
    opened_at: AtomicU64,
    half_open_calls: AtomicU32,
}

impl InlineCircuitBreaker {
    const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
            consecutive_failures: AtomicU32::new(0),
            opened_at: AtomicU64::new(0),
            half_open_calls: AtomicU32::new(0),
        }
    }

    fn current_state(&self) -> CbState {
        let raw = self.state.load(Ordering::Relaxed);
        match raw {
            0 => CbState::Closed,
            1 => {
                let opened = self.opened_at.load(Ordering::Relaxed);
                if opened > 0 {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now.saturating_sub(opened) >= CB_RESET_TIMEOUT_SECS {
                        self.state.store(2, Ordering::Relaxed);
                        self.half_open_calls.store(1, Ordering::Relaxed);
                        return CbState::HalfOpen;
                    }
                }
                CbState::Open
            }
            2 => CbState::HalfOpen,
            _ => CbState::Closed,
        }
    }

    fn allow_request(&self) -> bool {
        match self.current_state() {
            CbState::Closed => true,
            CbState::Open => false,
            CbState::HalfOpen => {
                let calls = self.half_open_calls.fetch_add(1, Ordering::Relaxed);
                calls < CB_HALF_OPEN_MAX
            }
        }
    }

    fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        if self.current_state() == CbState::HalfOpen {
            let calls = self.half_open_calls.load(Ordering::Relaxed);
            if calls >= CB_HALF_OPEN_MAX {
                self.state.store(0, Ordering::Relaxed);
                self.opened_at.store(0, Ordering::Relaxed);
                self.half_open_calls.store(0, Ordering::Relaxed);
            }
        }
    }

    fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        match self.current_state() {
            CbState::Closed => {
                if failures >= CB_FAILURE_THRESHOLD {
                    self.trip_open();
                }
            }
            CbState::HalfOpen => {
                self.trip_open();
            }
            CbState::Open => {}
        }
    }

    fn trip_open(&self) {
        self.state.store(1, Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.opened_at.store(now, Ordering::Relaxed);
    }
}

static AGENT_LLM_BREAKER: OnceLock<InlineCircuitBreaker> = OnceLock::new();

fn get_breaker() -> &'static InlineCircuitBreaker {
    AGENT_LLM_BREAKER.get_or_init(InlineCircuitBreaker::new)
}

fn backoff_delay_ms(attempt: u32) -> u64 {
    let exp = 2u64.pow(attempt);
    let raw = BACKOFF_BASE_MS * exp;
    let jitter = 1.0 + (rand_pseudo_jitter() * 0.1);
    ((raw as f64 * jitter) as u64).min(BACKOFF_MAX_MS)
}

fn rand_pseudo_jitter() -> f64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let v = (now.wrapping_mul(1103515245).wrapping_add(12345)) as f64;
    (v / u32::MAX as f64) * 2.0 - 1.0
}

pub(crate) fn call_llm_with_retry_and_temperature(
    provider: &AgentProvider,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    api_key: &str,
    temperature: f32,
) -> Result<String, String> {
    let breaker = get_breaker();

    if !breaker.allow_request() {
        return Err(format!(
            "LLM circuit breaker open (consecutive failures >= {CB_FAILURE_THRESHOLD}), \
             retry after {CB_RESET_TIMEOUT_SECS}s"
        ));
    }

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
            Ok(response) => {
                breaker.record_success();
                return Ok(response);
            }
            Err(e) => {
                let is_auth_error = e.contains("401") || e.contains("403");
                if is_auth_error {
                    breaker.record_failure();
                    return Err(format!(
                        "Authentication failed (key={}): {e}",
                        mask_api_key(api_key)
                    ));
                }

                last_error = e;
                breaker.record_failure();

                if attempt < MAX_RETRIES {
                    let delay_ms = backoff_delay_ms(attempt);
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
