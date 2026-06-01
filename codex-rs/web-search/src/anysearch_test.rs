use super::*;
use crate::types::Zone;

fn provider() -> AnySearchProvider {
    AnySearchProvider::new(None)
}

#[tokio::test]
async fn test_search_returns_results() {
    let provider = provider();
    let query = SearchQuery {
        query: "Rust programming language".to_string(),
        max_results: Some(3),
        ..Default::default()
    };
    let results = provider.search(query).await;
    assert!(
        results.is_ok(),
        "search should succeed: {:?}",
        results.err()
    );
    let results = results.unwrap();
    assert!(!results.is_empty(), "should return at least one result");
    assert!(!results[0].title.is_empty());
    assert!(!results[0].url.is_empty());
}

#[tokio::test]
async fn test_search_chinese_query() {
    let provider = provider();
    let query = SearchQuery {
        query: "专利检索".to_string(),
        max_results: Some(3),
        zone: Some(Zone::Cn),
        ..Default::default()
    };
    let results = provider.search(query).await;
    assert!(
        results.is_ok(),
        "Chinese search should succeed: {:?}",
        results.err()
    );
}

#[tokio::test]
async fn test_search_vertical_academic() {
    let provider = provider();
    let query = SearchQuery {
        query: "deep learning".to_string(),
        domain: Some("academic".to_string()),
        sub_domain: Some("academic.general".to_string()),
        max_results: Some(2),
        ..Default::default()
    };
    let results = provider.search(query).await;
    assert!(
        results.is_ok(),
        "vertical search should succeed: {:?}",
        results.err()
    );
}

#[tokio::test]
async fn test_batch_search() {
    let provider = provider();
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
    assert!(
        results.is_ok(),
        "batch_search should succeed: {:?}",
        results.err()
    );
    let groups = results.unwrap();
    assert_eq!(groups.len(), 2);
}

#[tokio::test]
async fn test_batch_search_rejects_over_5() {
    let provider = provider();
    let queries: Vec<SearchQuery> = (0..6)
        .map(|i| SearchQuery {
            query: format!("query {i}"),
            ..Default::default()
        })
        .collect();
    let result = provider.batch_search(queries).await;
    assert!(result.is_err(), "batch_search with 6 queries should fail");
}
