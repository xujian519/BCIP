//! MLX Embedding 服务客户端。
//!
//! 基于 blocking HTTP 请求调用 MLX 向量嵌入服务，支持结果缓存和健康检查。

use std::collections::HashMap;
use std::sync::Mutex;

const CACHE_MAX: usize = 1000;

/// MLX Embedding 服务客户端（同步，blocking HTTP）
///
/// API 密钥和服务地址通过构造函数 `new()` 传入，不设默认值。
pub struct EmbeddingClient {
    base_url: String,
    api_key: String,
    model: String,
    client: reqwest::blocking::Client,
    cache: Mutex<HashMap<String, Vec<f32>>>,
}

impl EmbeddingClient {
    /// 创建 Embedding 客户端。
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

    /// 从环境变量构造客户端（`BCIP_MLX_URL`/`BCIP_MLX_API_KEY`/`BCIP_MLX_MODEL`）。
    pub fn from_env() -> Option<Self> {
        let base_url =
            std::env::var("BCIP_MLX_URL").unwrap_or_else(|_| "http://localhost:8009".into());
        let api_key = std::env::var("BCIP_MLX_API_KEY").ok()?;
        let model = std::env::var("BCIP_MLX_MODEL").unwrap_or_else(|_| "bge-m3-mlx-8bit".into());
        Some(Self::new(&base_url, &api_key, &model))
    }

    /// 获取文本的 embedding 向量
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

        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("解析 embedding 响应失败: {e}"))?;

        let embedding: Vec<f32> = json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| "embedding 响应格式异常".to_string())?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    /// 测试连接是否可用
    pub fn health_check(&self) -> bool {
        self.client
            .get(format!("{}/v1/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
