use crate::google_patents::GooglePatentsInput;
use crate::google_patents::fetch_google_patents;
use codex_patent_knowledge::synonym::SynonymDict;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PatentSearchInput {
    pub query: String,
    #[serde(default = "super::google_patents::default_limit")]
    pub limit: usize,
    pub patent_number: Option<String>,
    pub use_synonyms: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQueryBuilderInput {
    pub concept: String,
    pub field: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IterativeSearchInput {
    pub query: String,
    pub rounds: Option<usize>,
    #[serde(default = "super::google_patents::default_limit")]
    pub limit: usize,
}

pub fn default_limit() -> usize {
    10
}

pub async fn patent_search(input: PatentSearchInput) -> Result<serde_json::Value, String> {
    let mut query = input.query.clone();
    if input.use_synonyms.unwrap_or(true) {
        let dict = SynonymDict::new();
        let expanded = dict.expand(&input.query);
        query = expanded.join(" OR ");
    }
    let google_input = GooglePatentsInput {
        query,
        limit: input.limit,
        patent_number: input.patent_number,
    };
    let results = fetch_google_patents(google_input).await?;
    serde_json::to_value(results).map_err(|e| format!("{e}"))
}

pub async fn search_query_builder(
    input: SearchQueryBuilderInput,
) -> Result<serde_json::Value, String> {
    let dict = SynonymDict::new();
    let exact_terms = dict.expand(&input.concept);
    let mut variants = Vec::new();
    for term in &exact_terms {
        if let Some(ref field) = input.field {
            variants.push(format!("{field} {term}"));
        } else {
            variants.push(term.to_string());
        }
    }
    Ok(serde_json::json!({
        "stage1_exact": exact_terms,
        "stage2_semantic": format!("{} 相关 OR 近似 OR 类似", input.concept),
        "stage3_variants": variants,
    }))
}

pub async fn iterative_search(input: IterativeSearchInput) -> Result<serde_json::Value, String> {
    let rounds = input.rounds.unwrap_or(3);
    let mut all_results = Vec::new();
    let mut current_query = input.query.clone();
    for _ in 0..rounds {
        let google_input = GooglePatentsInput {
            query: current_query.clone(),
            limit: input.limit,
            patent_number: None,
        };
        let results = fetch_google_patents(google_input).await?;
        if results.is_empty() {
            break;
        }
        all_results.extend(results);
        let dict = SynonymDict::new();
        current_query = dict.expand(&current_query).join(" OR ");
    }
    serde_json::to_value(all_results).map_err(|e| format!("{e}"))
}
