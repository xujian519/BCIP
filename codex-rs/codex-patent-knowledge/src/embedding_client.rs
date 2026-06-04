//! MLX Embedding 服务客户端。
//!
//! 基于 blocking HTTP 请求调用 MLX 向量嵌入服务，支持结果缓存、重试和熔断器保护。

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::OnceLock;

const CACHE_MAX: usize = 1000;
const MAX_RETRIES: u32 = 2;
const CB_FAILURE_THRESHOLD: u32 = 5;
const CB_RESET_TIMEOUT_SECS: u64 = 30;
const CB_HALF_OPEN_MAX: u32 = 3;
const BACKOFF_BASE_MS: u64 = 200;
const BACKOFF_MAX_MS: u64 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CbState {
    Closed,
    Open,
    HalfOpen,
}

struct InlineCb {
    state: AtomicU32,
    consecutive_failures: AtomicU32,
    opened_at: AtomicU64,
    half_open_calls: AtomicU32,
}

impl InlineCb {
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

static EMBEDDING_CB: OnceLock<InlineCb> = OnceLock::new();

fn get_cb() -> &'static InlineCb {
    EMBEDDING_CB.get_or_init(InlineCb::new)
}

fn backoff_delay_ms(attempt: u32) -> u64 {
    (BACKOFF_BASE_MS * 2u64.pow(attempt)).min(BACKOFF_MAX_MS)
}

pub struct EmbeddingClient {
    base_url: String,
    api_key: String,
    model: String,
    client: reqwest::blocking::Client,
    cache: Mutex<HashMap<String, Vec<f32>>>,
}

impl EmbeddingClient {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("embedding client 构建失败"),
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn from_env() -> Option<Self> {
        let base_url =
            std::env::var("BCIP_MLX_URL").unwrap_or_else(|_| "http://localhost:8009".into());
        let api_key = std::env::var("BCIP_MLX_API_KEY").ok()?;
        let model = std::env::var("BCIP_MLX_MODEL").unwrap_or_else(|_| "bge-m3-mlx-8bit".into());
        Some(Self::new(&base_url, &api_key, &model))
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(text) {
                return Ok(cached.clone());
            }
            if cache.len() >= CACHE_MAX {
                cache.clear();
            }
        }

        let cb = get_cb();
        if !cb.allow_request() {
            return Err(format!(
                "Embedding service circuit breaker open (failures >= {CB_FAILURE_THRESHOLD}), \
                 retry after {CB_RESET_TIMEOUT_SECS}s"
            ));
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

        Err(format!("embedding failed after {MAX_RETRIES} retries: {last_error}"))
    }

    fn embed_once(&self, text: &str) -> Result<Vec<f32>, String> {
        let body = serde_json::json!({
            "model": self.model,
            "input": text,
        });

        let resp = self
            .client
            .post(format!("{}/v1/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .map_err(|e| format!("embedding 请求失败: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(format!("embedding HTTP {status}: {}", &text[..text.len().min(200)]));
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
        self.client
            .get(format!("{}/v1/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
