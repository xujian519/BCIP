//! MLX Embedding 服务客户端。
//!
//! 基于 blocking HTTP 请求调用 MLX 向量嵌入服务，支持结果缓存、重试和熔断器保护。

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

use codex_patent_core::http::{CircuitBreaker, SharedBlockingClient};

const CACHE_MAX: usize = 1000;
const CACHE_EVICT_BATCH: usize = 100;
const MAX_RETRIES: u32 = 2;
const BACKOFF_BASE_MS: u64 = 200;
const BACKOFF_MAX_MS: u64 = 5_000;

static EMBEDDING_CB: OnceLock<CircuitBreaker> = OnceLock::new();
static SHARED_BLOCKING_CLIENT: OnceLock<SharedBlockingClient> = OnceLock::new();

fn get_shared_client() -> &'static reqwest::blocking::Client {
    SHARED_BLOCKING_CLIENT
        .get_or_init(SharedBlockingClient::new)
        .client()
}

fn get_cb() -> &'static CircuitBreaker {
    EMBEDDING_CB.get_or_init(CircuitBreaker::new)
}

fn backoff_delay_ms(attempt: u32) -> u64 {
    (BACKOFF_BASE_MS * 2u64.pow(attempt)).min(BACKOFF_MAX_MS)
}

pub struct EmbeddingClient {
    base_url: String,
    api_key: Option<String>,
    model: String,
    cache: Mutex<HashMap<String, Vec<f32>>>,
}

impl EmbeddingClient {
    pub fn new(base_url: &str, api_key: Option<String>, model: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model: model.to_string(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn from_env() -> Option<Self> {
        let base_url =
            std::env::var("BCIP_MLX_URL").unwrap_or_else(|_| "http://localhost:8766".into());
        let api_key = std::env::var("BCIP_MLX_API_KEY").ok();
        let model = std::env::var("BCIP_MLX_MODEL").unwrap_or_else(|_| "bge-m3-mlx-8bit".into());
        Some(Self::new(&base_url, api_key, &model))
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(text) {
                return Ok(cached.clone());
            }
            if cache.len() >= CACHE_MAX {
                // Evict oldest batch instead of clearing everything
                let keys: Vec<String> = cache.keys().take(CACHE_EVICT_BATCH).cloned().collect();
                for key in keys {
                    cache.remove(&key);
                }
            }
        }

        let cb = get_cb();
        if !cb.allow_request() {
            return Err(
                "Embedding service circuit breaker open (failures >= 5), retry after 30s".into(),
            );
        }

        let mut last_error = String::new();
        for attempt in 0..=MAX_RETRIES {
            match self.embed_once(text) {
                Ok(embedding) => {
                    cb.record_success();
                    let mut cache = self.cache.lock().unwrap();
                    cache.insert(text.to_string(), embedding.clone());
                    return Ok(embedding);
                }
                Err(e) => {
                    last_error = e;
                    cb.record_failure();
                    if attempt < MAX_RETRIES {
                        let delay = backoff_delay_ms(attempt);
                        std::thread::sleep(std::time::Duration::from_millis(delay));
                    }
                }
            }
        }

        Err(format!(
            "embedding failed after {MAX_RETRIES} retries: {last_error}"
        ))
    }

    fn embed_once(&self, text: &str) -> Result<Vec<f32>, String> {
        let body = serde_json::json!({
            "model": self.model,
            "input": text,
        });

        let client = get_shared_client();
        let mut req = client.post(format!("{}/v1/embeddings", self.base_url));
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }
        let resp = req
            .json(&body)
            .send()
            .map_err(|e| format!("embedding 请求失败: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(format!(
                "embedding HTTP {status}: {}",
                &text[..text.len().min(200)]
            ));
        }

        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("解析 embedding 响应失败: {e}"))?;

        let embedding: Vec<f32> = json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| "embedding 响应格式异常".to_string())?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    pub fn health_check(&self) -> bool {
        let client = get_shared_client();
        let mut req = client.get(format!("{}/v1/models", self.base_url));
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }
        req.send().map(|r| r.status().is_success()).unwrap_or(false)
    }
}
