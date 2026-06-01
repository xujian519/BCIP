use crate::error::WebSearchError;
use crate::provider::SearchProvider;
use crate::types::{ExtractResult, SearchQuery, SearchResult};
use serde::{Deserialize, Serialize};

pub struct TavilyProvider {
    client: reqwest::Client,
    api_key: String,
}

impl TavilyProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

#[derive(Serialize)]
struct TavilySearchRequest {
    api_key: String,
    query: String,
    max_results: Option<u32>,
    search_depth: Option<String>,
}

#[derive(Deserialize)]
struct TavilySearchResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f64,
}

impl SearchProvider for TavilyProvider {
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, WebSearchError> {
        let req_body = TavilySearchRequest {
            api_key: self.api_key.clone(),
            query: query.query,
            max_results: query.max_results,
            search_depth: Some("basic".to_string()),
        };
        let resp: TavilySearchResponse = self
            .client
            .post("https://api.tavily.com/search")
            .json(&req_body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp
            .results
            .into_iter()
            .map(|r| SearchResult {
                title: r.title,
                url: r.url,
                content: r.content,
                score: r.score,
            })
            .collect())
    }

    async fn extract(&self, url: &str) -> Result<ExtractResult, WebSearchError> {
        #[derive(Serialize)]
        struct TavilyExtractRequest {
            api_key: String,
            urls: Vec<String>,
        }
        #[derive(Deserialize)]
        struct TavilyExtractResponse {
            results: Vec<TavilyExtractResult>,
        }
        #[derive(Deserialize)]
        struct TavilyExtractResult {
            url: String,
            raw_content: Option<String>,
        }
        let resp: TavilyExtractResponse = self
            .client
            .post("https://api.tavily.com/extract")
            .json(&TavilyExtractRequest {
                api_key: self.api_key.clone(),
                urls: vec![url.to_string()],
            })
            .send()
            .await?
            .json()
            .await?;
        let result = resp.results.into_iter().next().ok_or_else(|| {
            WebSearchError::Api {
                code: -1,
                message: "No extract result".to_string(),
            }
        })?;
        let content = result.raw_content.unwrap_or_default();
        let length = content.len();
        Ok(ExtractResult {
            url: result.url,
            content,
            length,
        })
    }

    async fn batch_search(
        &self,
        queries: Vec<SearchQuery>,
    ) -> Result<Vec<Vec<SearchResult>>, WebSearchError> {
        let futures: Vec<_> = queries.into_iter().map(|q| self.search(q)).collect();
        let results = futures::future::join_all(futures).await;
        results.into_iter().collect()
    }
}