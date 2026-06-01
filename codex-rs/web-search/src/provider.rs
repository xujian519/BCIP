use crate::error::WebSearchError;
use crate::types::ExtractResult;
use crate::types::SearchQuery;
use crate::types::SearchResult;
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
