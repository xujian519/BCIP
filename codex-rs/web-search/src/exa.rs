use crate::error::WebSearchError;
use crate::provider::SearchProvider;
use crate::types::ExtractResult;
use crate::types::SearchQuery;
use crate::types::SearchResult;
use serde::Deserialize;
use serde::Serialize;

pub struct ExaProvider {
    client: reqwest::Client,
    api_key: String,
}

impl ExaProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub fn new_checked(api_key: String) -> Result<Self, WebSearchError> {
        if api_key.trim().is_empty() {
            return Err(WebSearchError::InvalidConfig(
                "EXA requires a non-empty API key (set EXA_API_KEY)".to_string(),
            ));
        }
        Ok(Self::new(api_key))
    }
}

#[derive(Serialize)]
struct ExaSearchRequest {
    query: String,
    num_results: Option<u32>,
    #[serde(rename = "type")]
    search_type: String,
    contents: ExaContents,
}

#[derive(Serialize)]
struct ExaContents {
    text: bool,
}

#[derive(Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaResult>,
}

#[derive(Deserialize)]
struct ExaResult {
    title: String,
    url: String,
    text: Option<String>,
    score: Option<f64>,
}

impl SearchProvider for ExaProvider {
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, WebSearchError> {
        let req_body = ExaSearchRequest {
            query: query.query,
            num_results: query.max_results,
            search_type: "neural".to_string(),
            contents: ExaContents { text: true },
        };
        let resp = self
            .client
            .post("https://api.exa.ai/search")
            .header("x-api-key", &self.api_key)
            .json(&req_body)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WebSearchError::Api {
                code: status.as_u16() as i32,
                message: format!("EXA API error: {body}"),
            });
        }
        let search_resp: ExaSearchResponse = resp.json().await?;
        Ok(search_resp
            .results
            .into_iter()
            .map(|r| SearchResult {
                title: r.title,
                url: r.url,
                content: r.text.unwrap_or_default(),
                score: r.score.unwrap_or(0.0),
            })
            .collect())
    }

    async fn extract(&self, url: &str) -> Result<ExtractResult, WebSearchError> {
        #[derive(Serialize)]
        struct ExaContentsRequest {
            ids: Vec<String>,
            text: bool,
        }
        #[derive(Deserialize)]
        struct ExaContentsResponse {
            results: Vec<ExaContentResult>,
        }
        #[derive(Deserialize)]
        struct ExaContentResult {
            url: String,
            text: Option<String>,
        }
        let resp = self
            .client
            .post("https://api.exa.ai/contents")
            .header("x-api-key", &self.api_key)
            .json(&ExaContentsRequest {
                ids: vec![url.to_string()],
                text: true,
            })
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WebSearchError::Api {
                code: status.as_u16() as i32,
                message: format!("EXA contents error: {body}"),
            });
        }
        let contents_resp: ExaContentsResponse = resp.json().await?;
        let result =
            contents_resp
                .results
                .into_iter()
                .next()
                .ok_or_else(|| WebSearchError::Api {
                    code: -1,
                    message: "No content result".to_string(),
                })?;
        let content = result.text.unwrap_or_default();
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
