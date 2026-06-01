use codex_web_search::anysearch::AnySearchProvider;
use codex_web_search::provider::SearchProvider;
use codex_web_search::types::{Freshness, SearchQuery, Zone};
use serde::Deserialize;
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