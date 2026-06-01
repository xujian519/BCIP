use crate::error::WebSearchError;
use crate::provider::SearchProvider;
use crate::types::ExtractResult;
use crate::types::Freshness;
use crate::types::SearchQuery;
use crate::types::SearchResult;
use crate::types::Zone;
use serde::Deserialize;
use serde::Serialize;

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

    fn add_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.api_key {
            Some(key) => builder.header("Authorization", format!("Bearer {key}")),
            None => builder,
        }
    }

    fn freshness_str(f: Freshness) -> &'static str {
        match f {
            Freshness::Day => "day",
            Freshness::Week => "week",
            Freshness::Month => "month",
            Freshness::Year => "year",
        }
    }

    fn zone_str(z: Zone) -> &'static str {
        match z {
            Zone::Cn => "cn",
            Zone::Intl => "intl",
        }
    }
}

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
}

#[derive(Deserialize)]
struct SearchItem {
    title: String,
    url: String,
    content: String,
    score: f64,
}

impl SearchQuery {
    fn to_request(&self) -> SearchRequest {
        SearchRequest {
            query: self.query.clone(),
            domain: self.domain.clone(),
            sub_domain: self.sub_domain.clone(),
            max_results: self.max_results,
            freshness: self
                .freshness
                .map(AnySearchProvider::freshness_str)
                .map(String::from),
            zone: self.zone.map(AnySearchProvider::zone_str).map(String::from),
        }
    }
}

fn item_to_result(item: SearchItem) -> SearchResult {
    SearchResult {
        title: item.title,
        url: item.url,
        content: item.content,
        score: item.score,
    }
}

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

impl SearchProvider for AnySearchProvider {
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, WebSearchError> {
        let builder = self
            .client
            .post(format!("{}/v1/search", self.base_url))
            .json(&query.to_request());
        let builder = self.add_auth(builder);
        let resp: SearchResponse = builder.send().await?.json().await?;

        if resp.code != 0 {
            return Err(WebSearchError::Api {
                code: resp.code,
                message: resp.message,
            });
        }

        let results = resp
            .data
            .map(|d| d.results.into_iter().map(item_to_result).collect())
            .unwrap_or_default();

        Ok(results)
    }

    async fn extract(&self, url: &str) -> Result<ExtractResult, WebSearchError> {
        let builder = self
            .client
            .post(format!("{}/v1/extract", self.base_url))
            .json(&serde_json::json!({ "url": url }));
        let builder = self.add_auth(builder);
        let resp = builder.send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(WebSearchError::Api {
                code: 404,
                message: format!("Extract endpoint not available for URL: {url}"),
            });
        }

        let extract_resp: ExtractResponse =
            serde_json::from_str(&body).map_err(|e| WebSearchError::Api {
                code: status.as_u16() as i32,
                message: format!("Failed to parse extract response: {e}"),
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

    async fn batch_search(
        &self,
        queries: Vec<SearchQuery>,
    ) -> Result<Vec<Vec<SearchResult>>, WebSearchError> {
        if queries.is_empty() || queries.len() > 5 {
            return Err(WebSearchError::InvalidConfig(
                "batch_search requires 1-5 queries".to_string(),
            ));
        }

        let futures: Vec<_> = queries.into_iter().map(|q| self.search(q)).collect();
        let results = futures::future::join_all(futures).await;
        results.into_iter().collect()
    }
}

#[cfg(test)]
#[path = "anysearch_test.rs"]
mod tests;
