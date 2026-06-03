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
            Provider::Tavily(
                TavilyProvider::new_checked(key)
                    .unwrap_or_else(|_| TavilyProvider::new(String::new())),
            )
        }
        "exa" => {
            let key = std::env::var("EXA_API_KEY").unwrap_or_default();
            Provider::Exa(
                ExaProvider::new_checked(key).unwrap_or_else(|_| ExaProvider::new(String::new())),
            )
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
    if parsed.queries.is_empty() || parsed.queries.len() > 5 {
        return Err("batch_search requires 1-5 queries".to_string());
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- Input struct deserialization tests ---

    #[test]
    fn deserialize_web_search_input_full() {
        let json = serde_json::json!({
            "query": "专利新颖性检索",
            "domain": "patent",
            "sub_domain": "novelty",
            "max_results": 10,
            "freshness": "week",
            "zone": "cn"
        });
        let input: WebSearchInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.query, "专利新颖性检索");
        assert_eq!(input.domain.as_deref(), Some("patent"));
        assert_eq!(input.sub_domain.as_deref(), Some("novelty"));
        assert_eq!(input.max_results, Some(10));
        assert_eq!(input.freshness.as_deref(), Some("week"));
        assert_eq!(input.zone.as_deref(), Some("cn"));
    }

    #[test]
    fn deserialize_web_search_input_minimal() {
        let json = serde_json::json!({
            "query": "测试查询"
        });
        let input: WebSearchInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.query, "测试查询");
        assert!(input.domain.is_none());
        assert!(input.sub_domain.is_none());
        assert!(input.max_results.is_none());
        assert!(input.freshness.is_none());
        assert!(input.zone.is_none());
    }

    #[test]
    fn deserialize_web_extract_input() {
        let json = serde_json::json!({
            "url": "https://example.com/patent/123"
        });
        let input: WebExtractInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.url, "https://example.com/patent/123");
    }

    #[test]
    fn deserialize_web_batch_search_input() {
        let json = serde_json::json!({
            "queries": [
                {"query": "检索1"},
                {"query": "检索2", "max_results": 5}
            ]
        });
        let input: WebBatchSearchInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.queries.len(), 2);
        assert_eq!(input.queries[0].query, "检索1");
        assert_eq!(input.queries[1].max_results, Some(5));
    }

    #[test]
    fn deserialize_web_batch_search_empty() {
        let json = serde_json::json!({
            "queries": []
        });
        let input: WebBatchSearchInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert!(input.queries.is_empty());
    }

    // --- parse_freshness tests ---

    #[test]
    fn parse_freshness_valid_values() {
        assert!(matches!(parse_freshness("day"), Some(Freshness::Day)));
        assert!(matches!(parse_freshness("week"), Some(Freshness::Week)));
        assert!(matches!(parse_freshness("month"), Some(Freshness::Month)));
        assert!(matches!(parse_freshness("year"), Some(Freshness::Year)));
    }

    #[test]
    fn parse_freshness_invalid_value() {
        assert!(parse_freshness("invalid").is_none());
        assert!(parse_freshness("").is_none());
        assert!(parse_freshness("DAY").is_none()); // case sensitive
    }

    // --- parse_zone tests ---

    #[test]
    fn parse_zone_valid_values() {
        assert!(matches!(parse_zone("cn"), Some(Zone::Cn)));
        assert!(matches!(parse_zone("intl"), Some(Zone::Intl)));
    }

    #[test]
    fn parse_zone_invalid_value() {
        assert!(parse_zone("us").is_none());
        assert!(parse_zone("").is_none());
        assert!(parse_zone("CN").is_none()); // case sensitive
    }

    // --- input_to_query conversion tests ---

    #[test]
    fn input_to_query_basic() {
        let input = WebSearchInput {
            query: "专利检索".into(),
            domain: None,
            sub_domain: None,
            max_results: Some(5),
            freshness: None,
            zone: None,
        };
        let query = input_to_query(input);
        assert_eq!(query.query, "专利检索");
        assert_eq!(query.max_results, Some(5));
        assert!(query.domain.is_none());
        assert!(query.freshness.is_none());
        assert!(query.zone.is_none());
    }

    #[test]
    fn input_to_query_with_all_fields() {
        let input = WebSearchInput {
            query: "测试".into(),
            domain: Some("patent".into()),
            sub_domain: Some("novelty".into()),
            max_results: Some(20),
            freshness: Some("month".into()),
            zone: Some("cn".into()),
        };
        let query = input_to_query(input);
        assert_eq!(query.domain.as_deref(), Some("patent"));
        assert_eq!(query.sub_domain.as_deref(), Some("novelty"));
        assert_eq!(query.max_results, Some(20));
        assert!(matches!(query.freshness, Some(Freshness::Month)));
        assert!(matches!(query.zone, Some(Zone::Cn)));
    }

    #[test]
    fn input_to_query_invalid_freshness_ignored() {
        let input = WebSearchInput {
            query: "测试".into(),
            domain: None,
            sub_domain: None,
            max_results: None,
            freshness: Some("invalid".into()),
            zone: None,
        };
        let query = input_to_query(input);
        assert!(query.freshness.is_none());
    }

    #[test]
    fn input_to_query_invalid_zone_ignored() {
        let input = WebSearchInput {
            query: "测试".into(),
            domain: None,
            sub_domain: None,
            max_results: None,
            freshness: None,
            zone: Some("invalid".into()),
        };
        let query = input_to_query(input);
        assert!(query.zone.is_none());
    }

    // --- format_search_results tests ---

    #[test]
    fn format_search_results_empty() {
        let results = vec![];
        let formatted = format_search_results(&results);
        assert!(formatted.is_empty());
    }

    #[test]
    fn format_search_results_single() {
        let results = vec![SearchResult {
            title: "专利标题".into(),
            url: "https://example.com".into(),
            content: "专利内容摘要".into(),
            score: 0.95,
        }];
        let formatted = format_search_results(&results);
        assert!(formatted.contains("1. 专利标题 (https://example.com)"));
        assert!(formatted.contains("专利内容摘要"));
    }

    #[test]
    fn format_search_results_multiple() {
        let results = vec![
            SearchResult {
                title: "标题A".into(),
                url: "https://a.com".into(),
                content: "内容A".into(),
                score: 0.9,
            },
            SearchResult {
                title: "标题B".into(),
                url: "https://b.com".into(),
                content: String::new(),
                score: 0.8,
            },
        ];
        let formatted = format_search_results(&results);
        assert!(formatted.contains("1. 标题A"));
        assert!(formatted.contains("2. 标题B"));
    }

    #[test]
    fn format_search_results_long_content_truncated() {
        let long_content: String = "x".repeat(300);
        let results = vec![SearchResult {
            title: "测试".into(),
            url: "https://example.com".into(),
            content: long_content.clone(),
            score: 0.9,
        }];
        let formatted = format_search_results(&results);
        // The preview in the formatted output should be at most 200 chars
        let lines: Vec<&str> = formatted.lines().collect();
        let preview_line = lines[1].trim();
        assert!(preview_line.len() <= 200);
    }

    #[test]
    fn format_search_results_empty_content_no_preview() {
        let results = vec![SearchResult {
            title: "测试".into(),
            url: "https://example.com".into(),
            content: String::new(),
            score: 0.9,
        }];
        let formatted = format_search_results(&results);
        // Should only have the title line, no preview line
        let lines: Vec<&str> = formatted.lines().collect();
        assert_eq!(lines.len(), 1);
    }

    // --- web_batch_search validation test ---

    #[tokio::test]
    async fn web_batch_search_empty_queries_rejected() {
        let input = serde_json::json!({"queries": []});
        let result = web_batch_search(input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1-5 queries"));
    }

    #[tokio::test]
    async fn web_batch_search_too_many_queries_rejected() {
        let queries: Vec<serde_json::Value> = (0..6)
            .map(|i| serde_json::json!({"query": format!("查询{i}")}))
            .collect();
        let input = serde_json::json!({"queries": queries});
        let result = web_batch_search(input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1-5 queries"));
    }

    // --- web_search input validation test ---

    #[tokio::test]
    async fn web_search_invalid_input_rejected() {
        let input = serde_json::json!({"wrong_field": "value"});
        let result = web_search(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn web_extract_invalid_input_rejected() {
        let input = serde_json::json!({"wrong_field": "value"});
        let result = web_extract(input).await;
        assert!(result.is_err());
    }

    // --- get_provider_name default test ---

    #[test]
    fn get_provider_name_default() {
        // Verifies the function returns a non-empty provider name
        let actual = get_provider_name();
        assert!(!actual.is_empty());
    }
}
