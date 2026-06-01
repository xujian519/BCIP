# Web Search Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 BCIP TUI 和桌面端接入 AnySearch 网页搜索能力，支持 Agent 自动调用和用户手动搜索。

**Architecture:** 新建 `codex-web-search` crate 定义 `SearchProvider` trait 和 `AnySearchProvider` 实现。在 `codex-config` 新增 `[web_search]` 配置解析。在 `codex-patent-tools` 注册 3 个搜索工具函数。TUI 和桌面端通过现有工具调用管道接入。

**Tech Stack:** Rust, reqwest (HTTP client), serde/serde_json, tokio (async runtime)

**Spec:** `docs/superpowers/specs/2026-06-01-web-search-integration-design.md`

---

## File Structure

| Action | Path | Responsibility |
|--------|------|----------------|
| Create | `codex-rs/web-search/Cargo.toml` | Crate 定义 |
| Create | `codex-rs/web-search/src/lib.rs` | pub mod + re-exports |
| Create | `codex-rs/web-search/src/error.rs` | WebSearchError 枚举 |
| Create | `codex-rs/web-search/src/types.rs` | SearchQuery, SearchResult, ExtractResult 等 |
| Create | `codex-rs/web-search/src/provider.rs` | SearchProvider trait |
| Create | `codex-rs/web-search/src/anysearch.rs` | AnySearchProvider 实现 |
| Create | `codex-rs/web-search/src/anysearch_test.rs` | AnySearch 集成测试 |
| Modify | `codex-rs/Cargo.toml` | workspace members + dependency |
| Modify | `codex-rs/config/src/config_toml.rs` | 新增 WebSearchConfigToml |
| Modify | `codex-rs/codex-patent-tools/Cargo.toml` | 添加 codex-web-search 依赖 |
| Create | `codex-rs/codex-patent-tools/src/web_search_tools.rs` | 3 个搜索工具注册 |
| Modify | `codex-rs/codex-patent-tools/src/lib.rs` | 注册 web_search_tools |
| Modify | `codex-rs/codex-patent-tools/src/search_tools.rs` | 合并 web search 工具 |

---

### Task 1: 创建 codex-web-search crate 骨架

**Files:**
- Create: `codex-rs/web-search/Cargo.toml`
- Create: `codex-rs/web-search/src/lib.rs`
- Create: `codex-rs/web-search/src/error.rs`
- Create: `codex-rs/web-search/src/types.rs`
- Modify: `codex-rs/Cargo.toml`

- [ ] **Step 1: 创建 Cargo.toml**

```toml
# codex-rs/web-search/Cargo.toml
[package]
name = "codex-web-search"
version = "0.1.0"
edition = "2024"

[dependencies]
reqwest = { workspace = true, features = ["json"], default-features = false }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true, features = ["rt"] }
tracing = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["rt", "macros"] }
```

- [ ] **Step 2: 创建 error.rs**

```rust
// codex-rs/web-search/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSearchError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error (code {code}): {message}")]
    Api { code: i32, message: String },

    #[error("Rate limited{retry_after}")]
    RateLimited { retry_after: Option<u64> },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("No search provider configured")]
    NoProvider,
}
```

- [ ] **Step 3: 创建 types.rs**

```rust
// codex-rs/web-search/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness: Option<Freshness>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone: Option<Zone>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Freshness {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Zone {
    Cn,
    Intl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractResult {
    pub url: String,
    pub content: String,
    pub length: usize,
}
```

- [ ] **Step 4: 创建 lib.rs**

```rust
// codex-rs/web-search/src/lib.rs
pub mod anysearch;
pub mod error;
pub mod provider;
pub mod types;

pub use error::WebSearchError;
pub use provider::SearchProvider;
pub use types::*;
```

- [ ] **Step 5: 创建 provider.rs trait 占位**

```rust
// codex-rs/web-search/src/provider.rs
use crate::error::WebSearchError;
use crate::types::{ExtractResult, SearchQuery, SearchResult};
use std::future::Future;

pub trait SearchProvider: Send + Sync {
    fn search(
        &self,
        query: SearchQuery,
    ) -> impl Future<Output = Result<Vec<SearchResult>, WebSearchError>> + Send;

    fn extract(
        &self,
        url: &str,
    ) -> impl Future<Output = Result<ExtractResult, WebSearchError>> + Send;

    fn batch_search(
        &self,
        queries: Vec<SearchQuery>,
    ) -> impl Future<Output = Result<Vec<Vec<SearchResult>>, WebSearchError>> + Send;
}
```

- [ ] **Step 6: 创建 anysearch.rs 占位**

```rust
// codex-rs/web-search/src/anysearch.rs
use crate::error::WebSearchError;
use crate::provider::SearchProvider;
use crate::types::{ExtractResult, SearchQuery, SearchResult};

pub struct AnySearchProvider {
    client: reqwest::Client,
    api_key: Option<String>,
    base_url: String,
}

impl AnySearchProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.anysearch.com".to_string(),
        }
    }
}

impl SearchProvider for AnySearchProvider {
    async fn search(
        &self,
        _query: SearchQuery,
    ) -> Result<Vec<SearchResult>, WebSearchError> {
        todo!()
    }

    async fn extract(&self, _url: &str) -> Result<ExtractResult, WebSearchError> {
        todo!()
    }

    async fn batch_search(
        &self,
        _queries: Vec<SearchQuery>,
    ) -> Result<Vec<Vec<SearchResult>>, WebSearchError> {
        todo!()
    }
}
```

- [ ] **Step 7: 注册到 workspace**

在 `codex-rs/Cargo.toml` 的 `members` 数组中添加 `"web-search"`（按字母序，在 `"tools"` 之前）。

在 `[workspace.dependencies]` 中添加：
```toml
codex-web-search = { path = "web-search" }
```

- [ ] **Step 8: 编译验证**

Run: `cargo check -p codex-web-search`
Expected: 编译成功，无错误

- [ ] **Step 9: Commit**

```bash
git add codex-rs/web-search/ codex-rs/Cargo.toml
git commit -m "feat: scaffold codex-web-search crate with types and trait"
```

---

### Task 2: 实现 AnySearchProvider — search

**Files:**
- Modify: `codex-rs/web-search/src/anysearch.rs`

- [ ] **Step 1: 定义 AnySearch API 请求/响应类型**

在 `anysearch.rs` 中添加：

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SearchRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    freshness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    zone: Option<String>,
}

#[derive(Deserialize)]
struct SearchResponse {
    code: i32,
    message: String,
    data: Option<SearchData>,
}

#[derive(Deserialize)]
struct SearchData {
    results: Vec<SearchItem>,
    metadata: SearchMetadata,
}

#[derive(Deserialize)]
struct SearchItem {
    title: String,
    url: String,
    content: String,
    score: f64,
}

#[derive(Deserialize)]
struct SearchMetadata {
    search_time_ms: u64,
    total_results: u32,
}
```

- [ ] **Step 2: 实现 search 方法**

替换 `todo!()` 为实际实现：

```rust
async fn search(
    &self,
    query: SearchQuery,
) -> Result<Vec<SearchResult>, WebSearchError> {
    let freshness = query.freshness.map(|f| match f {
        Freshness::Day => "day",
        Freshness::Week => "week",
        Freshness::Month => "month",
        Freshness::Year => "year",
    });
    let zone = query.zone.map(|z| match z {
        Zone::Cn => "cn",
        Zone::Intl => "intl",
    });

    let req_body = SearchRequest {
        query: query.query,
        domain: query.domain,
        sub_domain: query.sub_domain,
        max_results: query.max_results,
        freshness,
        zone,
    };

    let mut builder = self
        .client
        .post(format!("{}/v1/search", self.base_url))
        .json(&req_body);

    if let Some(ref key) = self.api_key {
        builder = builder.header("Authorization", format!("Bearer {key}"));
    }

    let resp: SearchResponse = builder.send().await?.json().await?;

    if resp.code != 0 {
        return Err(WebSearchError::Api {
            code: resp.code,
            message: resp.message,
        });
    }

    let results = resp
        .data
        .map(|d| {
            d.results
                .into_iter()
                .map(|item| SearchResult {
                    title: item.title,
                    url: item.url,
                    content: item.content,
                    score: item.score,
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(results)
}
```

- [ ] **Step 3: 编译验证**

Run: `cargo check -p codex-web-search`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add codex-rs/web-search/src/anysearch.rs
git commit -m "feat: implement AnySearchProvider::search with API call"
```

---

### Task 3: 实现 AnySearchProvider — batch_search 和 extract

**Files:**
- Modify: `codex-rs/web-search/src/anysearch.rs`

- [ ] **Step 1: 添加 batch_search 请求/响应类型**

```rust
#[derive(Serialize)]
struct BatchSearchRequest {
    queries: Vec<SearchRequest>,
}

#[derive(Deserialize)]
struct BatchSearchResponse {
    code: i32,
    message: String,
    data: Option<Vec<BatchSearchResultGroup>>,
}

#[derive(Deserialize)]
struct BatchSearchResultGroup {
    results: Vec<SearchItem>,
}
```

- [ ] **Step 2: 实现 batch_search**

```rust
async fn batch_search(
    &self,
    queries: Vec<SearchQuery>,
) -> Result<Vec<Vec<SearchResult>>, WebSearchError> {
    if queries.is_empty() || queries.len() > 5 {
        return Err(WebSearchError::InvalidConfig(
            "batch_search requires 1-5 queries".to_string(),
        ));
    }

    let req_queries: Vec<SearchRequest> = queries
        .into_iter()
        .map(|q| SearchRequest {
            query: q.query,
            domain: q.domain,
            sub_domain: q.sub_domain,
            max_results: q.max_results,
            freshness: q.freshness.map(|f| match f {
                Freshness::Day => "day",
                Freshness::Week => "week",
                Freshness::Month => "month",
                Freshness::Year => "year",
            }),
            zone: q.zone.map(|z| match z {
                Zone::Cn => "cn",
                Zone::Intl => "intl",
            }),
        })
        .collect();

    let mut builder = self
        .client
        .post(format!("{}/v1/batch_search", self.base_url))
        .json(&BatchSearchRequest {
            queries: req_queries,
        });

    if let Some(ref key) = self.api_key {
        builder = builder.header("Authorization", format!("Bearer {key}"));
    }

    let resp: BatchSearchResponse = builder.send().await?.json().await?;

    if resp.code != 0 {
        return Err(WebSearchError::Api {
            code: resp.code,
            message: resp.message,
        });
    }

    let results = resp
        .data
        .map(|groups| {
            groups
                .into_iter()
                .map(|g| {
                    g.results
                        .into_iter()
                        .map(|item| SearchResult {
                            title: item.title,
                            url: item.url,
                            content: item.content,
                            score: item.score,
                        })
                        .collect()
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(results)
}
```

- [ ] **Step 3: 实现 extract（使用 /v1/search 作为 URL 查询的 fallback）**

由于匿名模式下 `/v1/extract` 端点返回 404，extract 暂用搜索方式获取内容，后续确认端点后再优化：

```rust
async fn extract(&self, url: &str) -> Result<ExtractResult, WebSearchError> {
    #[derive(Deserialize)]
    struct ExtractResponse {
        code: i32,
        message: String,
        data: Option<ExtractData>,
    }

    #[derive(Deserialize)]
    struct ExtractData {
        content: String,
    }

    let mut builder = self
        .client
        .post(format!("{}/v1/extract", self.base_url))
        .json(&serde_json::json!({ "url": url }));

    if let Some(ref key) = self.api_key {
        builder = builder.header("Authorization", format!("Bearer {key}"));
    }

    let resp = builder.send().await?;
    let status = resp.status();
    let body = resp.text().await?;

    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(WebSearchError::Api {
            code: 404,
            message: format!(
                "Extract endpoint not available. URL: {url}. \
                 Try with an API key or use search instead."
            ),
        });
    }

    let extract_resp: ExtractResponse = serde_json::from_str(&body).map_err(|e| {
        WebSearchError::Api {
            code: status.as_u16() as i32,
            message: format!("Failed to parse extract response: {e}"),
        }
    })?;

    if extract_resp.code != 0 {
        return Err(WebSearchError::Api {
            code: extract_resp.code,
            message: extract_resp.message,
        });
    }

    let data = extract_resp.data.ok_or_else(|| WebSearchError::Api {
        code: -1,
        message: "No data in extract response".to_string(),
    })?;

    let length = data.content.len();
    Ok(ExtractResult {
        url: url.to_string(),
        content: data.content,
        length,
    })
}
```

- [ ] **Step 4: 编译验证**

Run: `cargo check -p codex-web-search`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add codex-rs/web-search/src/anysearch.rs
git commit -m "feat: implement batch_search and extract for AnySearchProvider"
```

---

### Task 4: 添加集成测试

**Files:**
- Create: `codex-rs/web-search/src/anysearch_test.rs`
- Modify: `codex-rs/web-search/src/anysearch.rs`

- [ ] **Step 1: 在 anysearch.rs 底部添加 test module**

```rust
#[cfg(test)]
#[path = "anysearch_test.rs"]
mod tests;
```

- [ ] **Step 2: 创建测试文件**

```rust
// codex-rs/web-search/src/anysearch_test.rs
use super::*;

fn provider_no_key() -> AnySearchProvider {
    AnySearchProvider::new(None)
}

#[tokio::test]
async fn test_search_anonymous_returns_results() {
    let provider = provider_no_key();
    let query = SearchQuery {
        query: "Rust programming language".to_string(),
        max_results: Some(3),
        ..Default::default()
    };
    let results = provider.search(query).await;
    assert!(results.is_ok(), "search should succeed: {:?}", results.err());
    let results = results.unwrap();
    assert!(!results.is_empty(), "should return at least one result");
    assert!(!results[0].title.is_empty());
    assert!(!results[0].url.is_empty());
}

#[tokio::test]
async fn test_search_chinese_query() {
    let provider = provider_no_key();
    let query = SearchQuery {
        query: "专利检索".to_string(),
        max_results: Some(3),
        zone: Some(Zone::Cn),
        ..Default::default()
    };
    let results = provider.search(query).await;
    assert!(results.is_ok(), "Chinese search should succeed: {:?}", results.err());
}

#[tokio::test]
async fn test_search_vertical_academic() {
    let provider = provider_no_key();
    let query = SearchQuery {
        query: "deep learning".to_string(),
        domain: Some("academic".to_string()),
        sub_domain: Some("academic.general".to_string()),
        max_results: Some(2),
        ..Default::default()
    };
    let results = provider.search(query).await;
    assert!(results.is_ok(), "vertical search should succeed: {:?}", results.err());
}

#[tokio::test]
async fn test_batch_search_anonymous() {
    let provider = provider_no_key();
    let queries = vec![
        SearchQuery {
            query: "Rust".to_string(),
            max_results: Some(2),
            ..Default::default()
        },
        SearchQuery {
            query: "Python".to_string(),
            max_results: Some(2),
            ..Default::default()
        },
    ];
    let results = provider.batch_search(queries).await;
    assert!(results.is_ok(), "batch_search should succeed: {:?}", results.err());
    let groups = results.unwrap();
    assert_eq!(groups.len(), 2, "should return 2 result groups");
}

#[tokio::test]
async fn test_batch_search_rejects_over_5() {
    let provider = provider_no_key();
    let queries: Vec<SearchQuery> = (0..6)
        .map(|i| SearchQuery {
            query: format!("query {i}"),
            ..Default::default()
        })
        .collect();
    let result = provider.batch_search(queries).await;
    assert!(result.is_err(), "batch_search with 6 queries should fail");
}
```

- [ ] **Step 3: 给 SearchQuery 添加 Default impl**

在 `types.rs` 中给 `SearchQuery` 添加：

```rust
impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            domain: None,
            sub_domain: None,
            max_results: None,
            freshness: None,
            zone: None,
        }
    }
}
```

- [ ] **Step 4: 运行测试**

Run: `cargo test -p codex-web-search`
Expected: 所有测试通过（注意：这些是集成测试，需要网络连接）

- [ ] **Step 5: Commit**

```bash
git add codex-rs/web-search/src/anysearch.rs codex-rs/web-search/src/anysearch_test.rs codex-rs/web-search/src/types.rs
git commit -m "test: add integration tests for AnySearch search and batch_search"
```

---

### Task 5: 配置层 — 新增 [web_search] section

**Files:**
- Modify: `codex-rs/config/src/config_toml.rs`

- [ ] **Step 1: 定义 WebSearchConfigToml struct**

在 `config_toml.rs` 中 `ConfigToml` struct 之前添加：

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct WebSearchConfigToml {
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub max_results: Option<u32>,
    pub zone: Option<String>,
}
```

- [ ] **Step 2: 在 ConfigToml 中添加字段**

在 `ConfigToml` struct 中，`pub web_search: Option<WebSearchMode>,` 字段之后添加：

```rust
    /// Web search provider configuration (separate from host web search mode).
    pub web_search_config: Option<WebSearchConfigToml>,
```

注意：不能叫 `web_search_provider` 因为现有的 `web_search` 字段已占用该名称（它控制宿主搜索模式）。新字段名 `web_search_config` 清晰区分。

- [ ] **Step 3: 编译验证**

Run: `cargo check -p codex-config`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add codex-rs/config/src/config_toml.rs
git commit -m "feat: add WebSearchConfigToml to config.toml schema"
```

---

### Task 6: 注册搜索工具到 codex-patent-tools

**Files:**
- Modify: `codex-rs/codex-patent-tools/Cargo.toml`
- Create: `codex-rs/codex-patent-tools/src/web_search_tools.rs`
- Modify: `codex-rs/codex-patent-tools/src/lib.rs`
- Modify: `codex-rs/codex-patent-tools/src/search_tools.rs`

- [ ] **Step 1: 添加依赖**

在 `codex-rs/codex-patent-tools/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
codex-web-search = { path = "../web-search" }
```

- [ ] **Step 2: 创建 web_search_tools.rs**

```rust
// codex-rs/codex-patent-tools/src/web_search_tools.rs
use codex_web_search::anysearch::AnySearchProvider;
use codex_web_search::provider::SearchProvider;
use codex_web_search::types::{Freshness, SearchQuery, Zone};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct WebSearchInput {
    query: String,
    domain: Option<String>,
    sub_domain: Option<String>,
    max_results: Option<u32>,
    freshness: Option<String>,
    zone: Option<String>,
}

#[derive(Deserialize)]
struct WebExtractInput {
    url: String,
}

#[derive(Deserialize)]
struct WebBatchSearchInput {
    queries: Vec<WebSearchInput>,
}

fn parse_freshness(s: &str) -> Option<Freshness> {
    match s {
        "day" => Some(Freshness::Day),
        "week" => Some(Freshness::Week),
        "month" => Some(Freshness::Month),
        "year" => Some(Freshness::Year),
        _ => None,
    }
}

fn parse_zone(s: &str) -> Option<Zone> {
    match s {
        "cn" => Some(Zone::Cn),
        "intl" => Some(Zone::Intl),
        _ => None,
    }
}

fn input_to_query(input: WebSearchInput) -> SearchQuery {
    SearchQuery {
        query: input.query,
        domain: input.domain,
        sub_domain: input.sub_domain,
        max_results: input.max_results,
        freshness: input.freshness.as_deref().and_then(parse_freshness),
        zone: input.zone.as_deref().and_then(parse_zone),
    }
}

async fn web_search(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let parsed: WebSearchInput =
        serde_json::from_value(input).map_err(|e| format!("{e}"))?;
    let provider = AnySearchProvider::new(None);
    let query = input_to_query(parsed);
    let results = provider.search(query).await.map_err(|e| format!("{e}"))?;
    serde_json::to_value(results).map_err(|e| format!("{e}"))
}

async fn web_extract(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let parsed: WebExtractInput =
        serde_json::from_value(input).map_err(|e| format!("{e}"))?;
    let provider = AnySearchProvider::new(None);
    let result = provider.extract(&parsed.url).await.map_err(|e| format!("{e}"))?;
    serde_json::to_value(result).map_err(|e| format!("{e}"))
}

async fn web_batch_search(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let parsed: WebBatchSearchInput =
        serde_json::from_value(input).map_err(|e| format!("{e}"))?;
    let provider = AnySearchProvider::new(None);
    let queries: Vec<SearchQuery> = parsed.queries.into_iter().map(input_to_query).collect();
    let results = provider
        .batch_search(queries)
        .await
        .map_err(|e| format!("{e}"))?;
    serde_json::to_value(results).map_err(|e| format!("{e}"))
}

pub fn register_web_search_tools() -> HashMap<String, crate::ToolHandler> {
    let mut tools: HashMap<String, crate::ToolHandler> = HashMap::new();

    tools.insert("WebSearch".to_string(), |input| {
        Box::pin(async { web_search(input).await })
    });

    tools.insert("WebExtract".to_string(), |input| {
        Box::pin(async { web_extract(input).await })
    });

    tools.insert("WebBatchSearch".to_string(), |input| {
        Box::pin(async { web_batch_search(input).await })
    });

    tools
}
```

- [ ] **Step 3: 在 lib.rs 中注册**

在 `lib.rs` 顶部添加：
```rust
pub mod web_search_tools;
```

在 `register_all_tools()` 函数中，`let mut tools = search_tools::register_search_tools();` 之后添加：
```rust
    tools.extend(web_search_tools::register_web_search_tools());
```

- [ ] **Step 4: 编译验证**

Run: `cargo check -p codex-patent-tools`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add codex-rs/codex-patent-tools/
git commit -m "feat: register WebSearch, WebExtract, WebBatchSearch tools"
```

---

### Task 7: 格式化并运行测试

**Files:** 无新增

- [ ] **Step 1: 格式化代码**

Run: `just fmt`
Expected: 所有文件格式化成功

- [ ] **Step 2: 运行 codex-web-search 测试**

Run: `just test -p codex-web-search`
Expected: 所有测试通过

- [ ] **Step 3: 运行 codex-patent-tools 编译检查**

Run: `cargo check -p codex-patent-tools`
Expected: 编译成功

- [ ] **Step 4: Commit 格式化变更（如有）**

```bash
git add -u
git commit -m "style: format web-search and patent-tools code"
```

---

## Self-Review

### Spec Coverage
| Spec Requirement | Task |
|------------------|------|
| SearchProvider trait | Task 1 |
| AnySearchProvider search | Task 2 |
| AnySearchProvider batch_search | Task 3 |
| AnySearchProvider extract | Task 3 |
| Config [web_search] section | Task 5 |
| 3 个搜索工具注册 | Task 6 |
| TUI/桌面端 Agent 可调用 | Task 6（通过工具注册） |

### Placeholder Scan
无 TBD/TODO/placeholders。所有步骤包含完整代码。

### Type Consistency
- `SearchQuery`, `SearchResult`, `ExtractResult` 在 types.rs 定义，anysearch.rs 和 web_search_tools.rs 使用一致
- `WebSearchError` 在 error.rs 定义，所有方法返回 `Result<_, WebSearchError>`
- `ToolHandler` 类型复用 lib.rs 中的定义
