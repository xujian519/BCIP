use crate::error::WebSearchError;
use crate::provider::SearchProvider;
use crate::types::ExtractResult;
use crate::types::SearchQuery;
use crate::types::SearchResult;

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
    async fn search(&self, _query: SearchQuery) -> Result<Vec<SearchResult>, WebSearchError> {
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
