use crate::google_patents::*;
use crate::patent_search::*;
use std::collections::HashMap;

pub type ToolHandler = fn(
    serde_json::Value,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send>,
>;

pub fn register_search_tools() -> HashMap<String, ToolHandler> {
    let mut tools: HashMap<String, ToolHandler> = HashMap::new();

    tools.insert("PatentSearch".to_string(), |input| {
        Box::pin(async {
            let parsed: PatentSearchInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            patent_search(parsed).await
        })
    });

    tools.insert("GooglePatentsFetch".to_string(), |input| {
        Box::pin(async {
            let parsed: GooglePatentsInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let results = fetch_google_patents(parsed).await?;
            serde_json::to_value(results).map_err(|e| format!("{e}"))
        })
    });

    tools.insert("SearchQueryBuilder".to_string(), |input| {
        Box::pin(async {
            let parsed: SearchQueryBuilderInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            search_query_builder(parsed).await
        })
    });

    tools.insert("IterativeSearch".to_string(), |input| {
        Box::pin(async {
            let parsed: IterativeSearchInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            iterative_search(parsed).await
        })
    });

    tools.insert("PatentDownload".to_string(), |input| {
        Box::pin(async {
            let parsed: PatentDownloadInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let filename = download_patent(parsed).await?;
            Ok(serde_json::json!({"downloaded_file": filename}))
        })
    });

    tools
}
