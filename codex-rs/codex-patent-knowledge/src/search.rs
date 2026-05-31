use crate::cards::CardIndex;
use crate::embedding_client::EmbeddingClient;
use crate::graph::SqliteKnowledgeGraph;
use crate::keyword_search::KeywordSearch;
use crate::law_db::LawDatabase;
use crate::synonym::SynonymDict;
use crate::vector_index::VectorIndex;
use codex_patent_core::SearchResult;
use codex_patent_core::SearchSource;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    Text,
    #[default]
    KeywordEnhanced,
    Legacy,
    /// 关键词 + 语义混合搜索
    Hybrid,
}

pub struct SearchConfig {
    pub query: String,
    pub limit: usize,
    pub search_kg: bool,
    pub search_law: bool,
    pub search_cards: bool,
    pub min_card_quality: f64,
    pub mode: SearchMode,
    pub prefix_filter: Option<String>,
    /// 语义搜索在前 top_k 条结果中的权重 (0.0-1.0)
    pub semantic_weight: f64,
    pub semantic_top_k: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 10,
            search_kg: true,
            search_law: true,
            search_cards: true,
            min_card_quality: 0.0,
            mode: SearchMode::KeywordEnhanced,
            prefix_filter: None,
            semantic_weight: 0.4,
            semantic_top_k: 20,
        }
    }
}

pub struct UnifiedSearch {
    kg: Option<SqliteKnowledgeGraph>,
    law_db: Option<LawDatabase>,
    card_index: Option<CardIndex>,
    synonym_dict: SynonymDict,
    vector_index: Option<VectorIndex>,
    embedding_client: Option<EmbeddingClient>,
    vector_available: bool,
}

impl UnifiedSearch {
    pub fn new(
        kg_path: Option<&str>,
        law_db_path: Option<&str>,
        card_index_path: Option<&str>,
    ) -> Self {
        let kg = kg_path.and_then(|p| SqliteKnowledgeGraph::open(p).ok());
        let law_db = law_db_path.and_then(|p| LawDatabase::open(p).ok());
        let card_index = card_index_path.and_then(|p| CardIndex::load(p).ok());
        Self {
            kg,
            law_db,
            card_index,
            synonym_dict: SynonymDict::new(),
            vector_index: None,
            embedding_client: None,
            vector_available: false,
        }
    }

    /// 构造完整版：包含向量语义索引
    pub fn with_vector(
        kg_path: Option<&str>,
        law_db_path: Option<&str>,
        card_index_path: Option<&str>,
        semantic_index_path: Option<&str>,
        mlx_base_url: Option<&str>,
        mlx_api_key: Option<&str>,
        mlx_model: Option<&str>,
    ) -> Self {
        let kg = kg_path.and_then(|p| SqliteKnowledgeGraph::open(p).ok());
        let law_db = law_db_path.and_then(|p| LawDatabase::open(p).ok());
        let card_index = card_index_path.and_then(|p| CardIndex::load(p).ok());
        let vector_index = semantic_index_path.and_then(|p| VectorIndex::open(p).ok());
        let embedding_client = mlx_base_url.zip(mlx_api_key).map(|(url, key)| {
            EmbeddingClient::new(url, key, mlx_model.unwrap_or("bge-m3-mlx-8bit"))
        });
        let vector_available = vector_index.is_some()
            && embedding_client
                .as_ref()
                .map_or(false, |c| c.health_check());
        Self {
            kg,
            law_db,
            card_index,
            synonym_dict: SynonymDict::new(),
            vector_index,
            embedding_client,
            vector_available,
        }
    }

    pub fn search(&self, config: &SearchConfig) -> Vec<SearchResult> {
        let synonyms = self.synonym_dict.expand(&config.query);
        let mut all_terms: Vec<&str> = synonyms.to_vec();
        all_terms.push(&config.query);
        all_terms.sort();
        all_terms.dedup();

        let effective_limit = config.limit.min(50);

        let mut results: Vec<SearchResult> = Vec::new();
        let mut seen_ids = HashSet::new();

        if config.search_kg
            && let Some(ref kg) = self.kg
        {
            for term in all_terms.iter().take(5) {
                if results.len() >= effective_limit {
                    break;
                }
                if let Ok(nodes) = kg.search_nodes(term, None, effective_limit) {
                    for node in nodes {
                        if results.len() >= effective_limit {
                            break;
                        }
                        if !seen_ids.insert(node.id.clone()) {
                            continue;
                        }
                        let content = node.content.clone().unwrap_or_default();
                        let score = self.compute_score(
                            &config.query,
                            &node.title,
                            &content,
                            &node.node_type,
                        );
                        results.push(SearchResult {
                            source: SearchSource::KnowledgeGraph,
                            title: if node.name.is_empty() {
                                node.title.clone()
                            } else {
                                node.name.clone()
                            },
                            content,
                            score,
                            id: node.id.clone(),
                            item_type: node.node_type.clone(),
                            source_path: node.full_ref.clone().unwrap_or_default(),
                            source_db: String::new(),
                        });
                    }
                }
            }
        }

        if config.search_law
            && results.len() < effective_limit
            && let Some(ref db) = self.law_db
        {
            for term in all_terms.iter().take(5) {
                if results.len() >= effective_limit {
                    break;
                }
                if let Ok(laws) = db.search_by_content(term, effective_limit) {
                    for law in laws {
                        if results.len() >= effective_limit {
                            break;
                        }
                        if !seen_ids.insert(law.id.clone()) {
                            continue;
                        }
                        let score =
                            self.compute_score(&config.query, &law.name, &law.content, &law.level);
                        let source_path = format!("{} ({})", law.name, law.level);
                        results.push(SearchResult {
                            source: SearchSource::LawDatabase,
                            title: law.name.clone(),
                            content: law.content.clone(),
                            score,
                            id: law.id.clone(),
                            item_type: law.level.clone(),
                            source_path,
                            source_db: "laws.db".into(),
                        });
                    }
                }
            }
        }

        if config.search_cards
            && results.len() < effective_limit
            && let Some(ref index) = self.card_index
        {
            let cards = index.search_by_keyword(&config.query, effective_limit.min(10));
            for card in cards {
                if results.len() >= effective_limit {
                    break;
                }
                if card.quality < config.min_card_quality {
                    continue;
                }
                if !seen_ids.insert(card.id.clone()) {
                    continue;
                }
                if let Ok(content) = index.load_content(card) {
                    let score =
                        self.compute_score(&config.query, &card.title, &content, &card.concept);
                    let source_path = format!("{} ({})", card.concept, card.domain);
                    results.push(SearchResult {
                        source: SearchSource::KnowledgeCard,
                        title: card.title.clone(),
                        content,
                        score,
                        id: card.id.clone(),
                        item_type: card.concept.clone(),
                        source_path,
                        source_db: "card-index.json".into(),
                    });
                }
            }
        }

        // Hybrid 模式：补充语义搜索结果
        if config.mode == SearchMode::Hybrid
            && self.vector_available
            && results.len() < effective_limit
            && let Some(ref client) = self.embedding_client
            && let Some(ref index) = self.vector_index
        {
            if let Ok(query_embedding) = client.embed(&config.query) {
                let semantic_results = index.search(&query_embedding, config.semantic_top_k);
                for scored in &semantic_results {
                    if results.len() >= effective_limit {
                        break;
                    }
                    let id = scored.chunk.chunk_id.clone();
                    if !seen_ids.insert(id.clone()) {
                        continue;
                    }
                    let semantic_score = scored.score;
                    let kw_score = KeywordSearch::score_text_with_query(
                        &config.query,
                        &format!("{} {}", scored.chunk.title, scored.chunk.content),
                    );
                    let combined = kw_score * (1.0 - config.semantic_weight)
                        + semantic_score * config.semantic_weight;
                    results.push(SearchResult {
                        source: SearchSource::KnowledgeGraph,
                        title: scored.chunk.title.clone(),
                        content: scored.chunk.content.clone(),
                        score: combined,
                        id,
                        item_type: "semantic_chunk".into(),
                        source_path: scored.chunk.file_path.clone(),
                        source_db: "semantic-index.sqlite".into(),
                    });
                }
            }
        }
        if let Some(ref prefix) = config.prefix_filter {
            results.retain(|r| r.title.starts_with(prefix) || r.content.starts_with(prefix));
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(effective_limit);
        results
    }

    fn compute_score(&self, query: &str, title: &str, content: &str, _item_type: &str) -> f64 {
        let title_score = KeywordSearch::score_text_with_query(query, title);
        let content_score = KeywordSearch::score_text_with_query(query, content);
        let boost = if title.contains(query) { 0.2 } else { 0.0 };
        (title_score * 0.4 + content_score * 0.6 + boost).clamp(0.0, 1.0)
    }

    pub fn status(&self) -> serde_json::Value {
        serde_json::json!({
            "knowledge_graph": self.kg.as_ref().and_then(|kg| kg.stats().ok().map(|s| serde_json::json!({
                "available": true,
                "node_count": s.node_count,
                "edge_count": s.edge_count
            }))).unwrap_or(serde_json::json!({"available": false})),
            "law_database": self.law_db.as_ref().and_then(|db| db.count().ok().map(|c| serde_json::json!({
                "available": true,
                "count": c
            }))).unwrap_or(serde_json::json!({"available": false})),
            "knowledge_cards": self.card_index.as_ref().map(|idx| serde_json::json!({
                "available": true,
                "count": idx.len()
            })).unwrap_or(serde_json::json!({"available": false})),
            "vector_index": self.vector_index.as_ref().map(|vi| serde_json::json!({
                "available": true,
                "chunk_count": vi.len(),
                "dimension": vi.dimension(),
            })).unwrap_or(serde_json::json!({"available": false})),
            "mlx_service": self.vector_available,
            "search_mode": if self.vector_available { "hybrid" } else { "keyword_enhanced" },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_score_prefers_relevant() {
        let search = UnifiedSearch::new(None, None, None);
        let s1 = search.compute_score(
            "图像识别",
            "图像识别装置",
            "一种图像识别方法和装置，包括摄像头和处理器",
            "patent",
        );
        let s2 = search.compute_score("图像识别", "化工材料", "一种化工材料的制备方法", "patent");
        assert!(s1 > s2, "relevant should score higher: {s1} vs {s2}");
    }

    #[test]
    fn test_compute_score_title_boost() {
        let search = UnifiedSearch::new(None, None, None);
        let s = search.compute_score("图像识别", "图像识别装置", "其他技术内容", "patent");
        assert!(s > 0.0, "title match should give some score");
    }
}
