use crate::search::SearchConfig;
use crate::search::SearchMode;
use crate::search::UnifiedSearch;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EvalQuery {
    pub query: String,
    pub expected_titles: Vec<String>,
    pub domain: String,
}

#[derive(Debug)]
pub struct EvalResult {
    pub query: String,
    pub precision_at_5: f64,
    pub precision_at_10: f64,
    pub recall: f64,
    pub mrr: f64,
    pub hits: Vec<String>,
    pub miss: Vec<String>,
}

pub struct SearchEval {
    search: UnifiedSearch,
    queries: Vec<EvalQuery>,
}

impl SearchEval {
    pub fn new(
        kg_path: Option<&str>,
        law_db_path: Option<&str>,
        card_index_path: Option<&str>,
        eval_queries_path: &str,
    ) -> Result<Self, String> {
        let content = std::fs::read_to_string(eval_queries_path)
            .map_err(|e| format!("read eval queries: {e}"))?;
        let queries: Vec<EvalQuery> =
            serde_json::from_str(&content).map_err(|e| format!("parse eval queries: {e}"))?;
        let search = UnifiedSearch::new(kg_path, law_db_path, card_index_path);
        Ok(Self { search, queries })
    }

    pub fn run(&self, mode: SearchMode) -> Vec<EvalResult> {
        self.queries
            .iter()
            .map(|eq| self.evaluate_one(eq, mode))
            .collect()
    }

    fn evaluate_one(&self, eq: &EvalQuery, mode: SearchMode) -> EvalResult {
        let config = SearchConfig {
            query: eq.query.clone(),
            limit: 10,
            mode,
            ..Default::default()
        };
        let results = self.search.search(&config);
        let titles: Vec<String> = results.iter().map(|r| r.title.clone()).collect();

        let hits_at_5 = titles
            .iter()
            .take(5)
            .filter(|t| eq.expected_titles.iter().any(|e| t.contains(e)))
            .count() as f64;
        let hits_at_10 = titles
            .iter()
            .take(10)
            .filter(|t| eq.expected_titles.iter().any(|e| t.contains(e)))
            .count() as f64;
        let total_hits = titles
            .iter()
            .filter(|t| eq.expected_titles.iter().any(|e| t.contains(e)))
            .count() as f64;

        let precision_at_5 = if 5.min(titles.len()) > 0 {
            hits_at_5 / 5.0
        } else {
            0.0
        };
        let precision_at_10 = if 10.min(titles.len()) > 0 {
            hits_at_10 / 10.0
        } else {
            0.0
        };
        let recall = if eq.expected_titles.is_empty() {
            0.0
        } else {
            total_hits.min(eq.expected_titles.len() as f64) / eq.expected_titles.len() as f64
        };

        let mrr = titles
            .iter()
            .take(10)
            .enumerate()
            .find(|(_, t)| eq.expected_titles.iter().any(|e| t.contains(e)))
            .map(|(i, _)| 1.0 / (i as f64 + 1.0))
            .unwrap_or(0.0);

        let hits: Vec<String> = eq
            .expected_titles
            .iter()
            .filter(|e| titles.iter().any(|t| t.contains(*e)))
            .cloned()
            .collect();
        let miss: Vec<String> = eq
            .expected_titles
            .iter()
            .filter(|e| !titles.iter().any(|t| t.contains(*e)))
            .cloned()
            .collect();

        EvalResult {
            query: eq.query.clone(),
            precision_at_5,
            precision_at_10,
            recall,
            mrr,
            hits,
            miss,
        }
    }

    pub fn summary(results: &[EvalResult]) -> serde_json::Value {
        let n = results.len() as f64;
        let avg_p5: f64 = results.iter().map(|r| r.precision_at_5).sum::<f64>() / n;
        let avg_p10: f64 = results.iter().map(|r| r.precision_at_10).sum::<f64>() / n;
        let avg_recall: f64 = results.iter().map(|r| r.recall).sum::<f64>() / n;
        let avg_mrr: f64 = results.iter().map(|r| r.mrr).sum::<f64>() / n;
        serde_json::json!({
            "total_queries": results.len(),
            "avg_precision_at_5": avg_p5,
            "avg_precision_at_10": avg_p10,
            "avg_recall": avg_recall,
            "avg_mrr": avg_mrr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_eval_keyword() {
        let evaluator = SearchEval::new(
            Some("../codex-patent-assets/patent_kg.db"),
            Some("../codex-patent-assets/laws.db"),
            Some("../codex-patent-assets/card-index.json"),
            "../codex-patent-assets/eval_queries.json",
        );
        if let Ok(ev) = evaluator {
            let results = ev.run(SearchMode::KeywordEnhanced);
            assert_eq!(results.len(), 10, "should evaluate all 10 queries");
            let summary = SearchEval::summary(&results);
            let p5 = summary["avg_precision_at_5"].as_f64().unwrap();
            assert!((0.0..=1.0).contains(&p5));
        }
    }
}
