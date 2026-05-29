use crate::cards::CardIndex;
use crate::graph::SqliteKnowledgeGraph;
use crate::law_db::LawDatabase;
use crate::synonym::SynonymDict;
use codex_patent_core::SearchResult;
use codex_patent_core::SearchSource;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Text,
    Hybrid,
}

impl Default for SearchMode {
    fn default() -> Self {
        Self::Hybrid
    }
}

pub struct SearchConfig {
    pub query: String,
    pub limit: usize,
    pub search_kg: bool,
    pub search_law: bool,
    pub search_cards: bool,
    pub min_card_quality: f64,
    pub mode: SearchMode,
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
            mode: SearchMode::Hybrid,
        }
    }
}

pub struct UnifiedSearch {
    kg: Option<SqliteKnowledgeGraph>,
    law_db: Option<LawDatabase>,
    card_index: Option<CardIndex>,
    synonym_dict: SynonymDict,
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
        }
    }

    pub fn search(&self, config: &SearchConfig) -> Vec<SearchResult> {
        let synonyms = self.synonym_dict.expand(&config.query);
        let mut all_terms: Vec<&str> = synonyms.iter().map(|s| s.as_ref()).collect();
        all_terms.push(&config.query);
        all_terms.sort();
        all_terms.dedup();

        let mut results = Vec::new();
        let mut seen_ids = HashSet::new();
        let effective_limit = config.limit.min(50);

        match config.mode {
            SearchMode::Text | SearchMode::Hybrid => {
                if config.search_kg && results.len() < effective_limit {
                    if let Some(ref kg) = self.kg {
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
                                    results.push(SearchResult {
                                        source: SearchSource::KnowledgeGraph,
                                        title: if node.name.is_empty() {
                                            node.title.clone()
                                        } else {
                                            node.name.clone()
                                        },
                                        content: node.content.clone().unwrap_or_default(),
                                        score: 0.8,
                                        id: node.id.clone(),
                                        item_type: node.node_type.clone(),
                                    });
                                }
                            }
                        }
                    }
                }

                if config.search_law && results.len() < effective_limit {
                    if let Some(ref db) = self.law_db {
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
                                    results.push(SearchResult {
                                        source: SearchSource::LawDatabase,
                                        title: law.name.clone(),
                                        content: law.content.clone(),
                                        score: 0.7,
                                        id: law.id.clone(),
                                        item_type: law.level.clone(),
                                    });
                                }
                            }
                        }
                    }
                }

                if config.search_cards && results.len() < effective_limit {
                    if let Some(ref index) = self.card_index {
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
                                results.push(SearchResult {
                                    source: SearchSource::KnowledgeCard,
                                    title: card.title.clone(),
                                    content,
                                    score: card.quality,
                                    id: card.id.clone(),
                                    item_type: card.concept.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(effective_limit);
        results
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
            "has_embedding": false,
        })
    }
}
