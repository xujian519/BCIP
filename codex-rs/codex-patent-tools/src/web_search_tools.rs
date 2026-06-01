use codex_web_search::anysearch::AnySearchProvider;
use codex_web_search::exa::ExaProvider;
use codex_web_search::provider::SearchProvider;
use codex_web_search::tavily::TavilyProvider;
use codex_web_search::types::ExtractResult;
use codex_web_search::types::Freshness;
use codex_web_search::types::SearchQuery;
use codex_web_search::types::SearchResult;
use codex_web_search::types::Zone;
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

fn get_api_key() -> Option<String> {
    let key = std::env::var("BCIP_WEB_SEARCH_API_KEY")
        .or_else(|_| std::env::var("ANYSEARCH_API_KEY"))
        .ok();
    if key.as_deref() == Some("") {
        return None;
    }
    key
}

fn get_provider_name() -> String {
    std::env::var("BCIP_WEB_SEARCH_PROVIDER").unwrap_or_else(|_| "anysearch".to_string())
}

fn make_provider() -> Provider {
    match get_provider_name().as_str() {
        "tavily" => {
            let key = std::env::var("TAVILY_API_KEY").unwrap_or_default();
            Provider::Tavily(TavilyProvider::new(key))
        }
        "exa" => {
            let key = std::env::var("EXA_API_KEY").unwrap_or_default();
            Provider::Exa(ExaProvider::new(key))
        }
        _ => Provider::AnySearch(AnySearchProvider::new(get_api_key())),
    }
}

enum Provider {
    AnySearch(AnySearchProvider),
    Tavily(TavilyProvider),
    Exa(ExaProvider),
}

impl Provider {
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, String> {
        match self {
            Provider::AnySearch(p) => p.search(query).await,
            Provider::Tavily(p) => p.search(query).await,
            Provider::Exa(p) => p.search(query).await,
        }
        .map_err(|e| format!("{e}"))
    }

    async fn extract(&self, url: &str) -> Result<ExtractResult, String> {
        match self {
            Provider::AnySearch(p) => p.extract(url).await,
            Provider::Tavily(p) => p.extract(url).await,
            Provider::Exa(p) => p.extract(url).await,
        }
        .map_err(|e| format!("{e}"))
    }

    async fn batch_search(
        &self,
        queries: Vec<SearchQuery>,
    ) -> Result<Vec<Vec<SearchResult>>, String> {
        match self {
            Provider::AnySearch(p) => p.batch_search(queries).await,
            Provider::Tavily(p) => p.batch_search(queries).await,
            Provider::Exa(p) => p.batch_search(queries).await,
        }
        .map_err(|e| format!("{e}"))
    }
}

async fn web_search(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let parsed: WebSearchInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
    let provider = make_provider();
    let query = input_to_query(parsed);
    let results = provider.search(query).await?;
    let formatted = format_search_results(&results);
    Ok(serde_json::json!({
        "results": results,
        "formatted": formatted,
        "count": results.len(),
    }))
}

async fn web_extract(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let parsed: WebExtractInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
    let provider = make_provider();
    let result = provider.extract(&parsed.url).await?;
    Ok(serde_json::json!({
        "url": result.url,
        "content": result.content,
        "length": result.length,
    }))
}

async fn web_batch_search(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let parsed: WebBatchSearchInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
    let provider = make_provider();
    let queries: Vec<SearchQuery> = parsed.queries.into_iter().map(input_to_query).collect();
    let results = provider.batch_search(queries).await?;
    let formatted: Vec<String> = results.iter().map(|g| format_search_results(g)).collect();
    Ok(serde_json::json!({
        "groups": results,
        "formatted": formatted,
    }))
}

fn format_search_results(results: &[SearchResult]) -> String {
    let mut out = String::new();
    for (i, r) in results.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("{}. {} ({})", i + 1, r.title, r.url));
        if !r.content.is_empty() {
            let preview: String = r.content.chars().take(200).collect();
            out.push_str(&format!("\n   {}", preview));
        }
    }
    out
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
